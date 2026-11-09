# PERF — RAM and build-speed backlog

> **Status: design note, 2026-11-09. Nothing in §3–§4 is implemented.** This file is the
> owner's and the agent's shared backlog for making Russet cheaper on memory (especially in the
> browser) and faster to build. It is NOT part of the product spec: `SPEC.md` stays authoritative,
> and every item that changes behavior needs a `SPEC.md` update, tests, an entry in `PROGRESS.md`
> and the owner's OK first (see `AGENTS.md`). Nothing here adds a dependency.

## 1. Measured facts

Machine used for the numbers below: Intel Xeon E5-2680 v3, 24 threads, 15 GB RAM (Linux, Rust
1.99.1). Caveat: this box builds with the owner's global `~/.cargo/config.toml`, which sets
`jobs = 8` and `-C target-cpu=native` for all builds.

| What | Value |
|---|---|
| `cargo build --release`, cold (all deps + fat LTO) | **243 s (4 min 03 s)** |
| `cargo build --release`, after `touch src/main.rs` (one file + relink) | **160 s (2 min 40 s)** |
| `cargo build --release`, no-op (nothing changed) | **6 s** |
| `target/release/russet` size (stripped) | **14.1 MiB (14,784,656 bytes)** |

The relink number is the painful one: any source edit costs ~2.7 minutes in release because
rewriting one crate re-runs the whole `lto = "fat"` + `codegen-units = 1` pipeline.

Release profile today (`Cargo.toml`): `lto = "fat"`, `codegen-units = 1`, `strip = true` — the
slowest combination possible. The web build has never been produced (`trunk` is not installed
here), so every browser number below is an estimate, not a measurement.

## 2. Where the memory goes

A canvas is kept twice by design: a premultiplied `tiny-skia::Pixmap` in RAM (editing) and an
egui texture in VRAM (display). On top of that, every stroke stores a **full-canvas** history
snapshot (`history.push_snapshot(self.canvas.pixmap().clone())`), and every undo/redo clones the
canvas again (`History::undo/redo(current)`).

Sizes are exact (`w × h × 4` bytes per copy; mip chain = 1/3 of the base texture; history is
bounded by `MAX_SNAPSHOTS = 20` and `MAX_BYTES = 256 MiB`):

| Canvas | Pixmap (RAM) | Texture + mips (VRAM) | History (full snapshots) | Total |
|---|---|---|---|---|
| 800×600 (default) | 1.9 MB | 1.9 + 0.6 MB | 20 × 1.9 MB = **38 MB** | **~43 MB** |
| 2000×1500 | 12 MB | 12 + 4 MB | 20 × 12 MB = **240 MB** | **~268 MB** |
| 4096×4096 (max) | 64 MB | 64 + 21 MB | 4 × 64 MB = **256 MB** (byte cap) | **~405 MB** |

Plus transient spikes the browser cannot reclaim:

- **Wasm heap never shrinks.** Rust's wasm allocator grows linear memory and does not return
  pages to the OS. Every peak (an opened photo, a canvas resize, an undo) permanently raises the
  tab's RSS until the page reloads.
- `Canvas::resize_canvas` and `set_pixmap` hold the old and the new pixmap at the same time →
  **2× canvas spike** (128 MB for 4096×4096).
- Opening an image keeps the decoded `image` buffer **and** the new pixmap alive
  (`file_io::pixmap_from_image` → `to_rgba8()` allocates a full RGBA copy even when the source is
  already RGBA8).
- A single click on the canvas snapshots the whole canvas even though one pixel changed.

## 3. Optimization backlog

### A. Big win — region-based (tile/patch) undo instead of full snapshots

- **What:** store the pixels that actually changed, not the whole canvas.
- **Design sketch:** `History` keeps `Vec<StrokeRecord>`; a record is a list of
  `(DirtyRect, before: Vec<u8>, after: Vec<u8>)` patches. `draw_segment` already returns the
  `DirtyRect` it painted (`tools::DirtyRect`) and `Canvas::region_bytes` already extracts exactly
  that region, so the data needed is available with no new code paths. `undo` restores the
  `before` patches in reverse order, `redo` re-applies `after`; limits (`MAX_SNAPSHOTS`,
  `MAX_BYTES`) become patch-budget limits.
- **Effect:** a 32 KB patch instead of a 12 MB snapshot for a typical stroke — history shrinks
  by roughly **10–100×**, stroke start loses the full-canvas memcpy (lower input latency on the
  web), undo/redo stop cloning the whole pixmap. The 4096×4096 case drops from ~256 MB of history
  to whatever the stroke area adds up to.
- **Cost:** medium. `History` is used from `app.rs` (stroke start, undo, redo), `app/dialogs.rs`
  (New/Resize) and has 3 unit tests; all of them move to the patch API.
- **Needs:** SPEC §3.6 rewrite ("snapshot-based undo/redo" → region-based with the same visible
  behavior: 20 steps / byte cap), owner approval.
- **No new dependencies.**

### B. Mipmaps only where they are used (zoom < 100 %)

- **What:** `CANVAS_TEXTURE_CRISP` / `CANVAS_TEXTURE_SMOOTH` (`src/app.rs`, `canvas_texture_options`)
  both set `mipmap_mode: Some(Linear)`. `egui_glow 0.37` (`painter.rs:632`) calls
  `gl.generate_mipmap` on **every** texture upload when mipmaps are on — i.e. once per frame while
  a stroke is being drawn, plus in `set_partial` for every dirty rectangle.
- **Fix:** return options with `mipmap_mode: None` while `zoom >= 1.0` (magnification only — mips
  are not sampled), keep mipmaps when `zoom < 1.0` where minification actually needs them. Visual
  result at 100–800 % is identical; zoomed-out filtering (§3.4) stays mipmapped.
- **Effect:** removes a full mip-chain regeneration per painted frame on the web, and frees the mip
  chain (~33 % on top of the base texture) in VRAM at normal zoom levels. Texture options only
  change when crossing 100 %, which costs one full re-upload (rare). Bonus: on a WebGL1 context
  mipmaps on non-power-of-two textures are invalid — a 800×600 canvas is non-POT, so keeping mips
  off at ≥100 % also removes that failure mode (modern browsers give WebGL2, but eframe still
  carries the WebGL1 path).
- **Cost:** one function in `app/camera.rs`; camera zoom tests still pass. Verify in the browser
  that zoom-out still looks smooth.
- **Needs:** nothing from §6; still deserves a `PROGRESS.md` note and an owner smoke test.

### C. Stop duplicating the decoded image on open

- **What:** `file_io::pixmap_from_image` uses `image.to_rgba8()` (clones/converts) and then copies
  pixel by pixel into a fresh `Pixmap` — two full-size buffers plus the source.
- **Fix:** `into_rgba8()` (consumes the `DynamicImage`), then premultiply directly into
  `pixmap.data_mut()` (no intermediate `Vec` copy). Same result, one allocation less.
- **Effect:** for a 4000×3000 photo, one 48 MB peak less — and that peak is permanent on wasm.
- **Cost:** small, `file_io/tests.rs` already covers premultiplication.

### D. Trim what the web build carries

- **eframe features per target:** `accesskit` keeps a browser accessibility tree in sync every
  frame (JS calls + allocations) and the app has no a11y requirement; `system_fonts` is meaningless
  in a browser (no system fonts to read). Move eframe to target-specific dependency blocks and drop
  those two features for `wasm32` (keep `glow`, `default_fonts`, `clipboard`).
- **Logging:** `WebLogger::init(log::LevelFilter::Debug)` in `main.rs` — use `Info`/`Warn` in
  release builds; the per-zoom `log::info!` then stops formatting.
- **Bundle:** §6 currently forbids wasm size optimization. If the owner lifts that exception for
  `web/` only, `<link data-trunk rel="rust" data-wasm-opt="2" />` (or `"z"`) typically cuts 30–50 %
  of the `.wasm`, which directly reduces load time, compile time and code memory in the browser.
- **`panic = "abort"`** in `[profile.release]`: smaller binary, faster link; the project already
  forbids panics on user-data paths.
- **Effect:** faster first paint and lower steady-state RAM on the web; no UX change.

### E. Budgets and caps (owner decisions)

- Cap the canvas size on `wasm32` (e.g. 2048 → 16 MB pixmap) or lower the history byte budget on
  wasm (e.g. 64 MB) — a full 4096×4096 history is a mobile OOM.
- Consider `MAX_BYTES = 256 MiB` as native-only and a smaller wasm value; the constant currently
  lives in `history.rs` without a platform split (allowed only via `cfg!()` at a call site, per
  `AGENTS.md` rules for `cfg` branches).

### F. Micro (do only after the above are measured)

- Build `egui::ColorImage` straight into `Vec<Color32>` from `Pixmap` rows instead of allocating a
  `Vec<u8>` via `region_bytes` and copying it again — saves one transient allocation per update.
- Reuse the region buffer across frames (`set_partial` only copies, so the buffer can live in
  `App`).
- `Pixmap` allocation order on resize/open: drop the old canvas before allocating the new one
  where possible to halve the transient peak.

## 4. Build-time options (release profile)

Today: `lto = "fat"`, `codegen-units = 1`, `strip = true`. The final LTO pass is the long pole:
cold build 243 s, a one-file relink 160 s (`jobs = 8`, §1). Options, cheapest-first in effort:

| Change | Build-time effect | Runtime/binary effect |
|---|---|---|
| `lto = "thin"` | usually −50–70 % of link time | ~1–3 % slower code, slightly bigger binary |
| `codegen-units = 16` (default) | parallel codegen, noticeable win | ~0–2 % slower code |
| `opt-level = 2` | ~10–20 % faster compile | a few % slower code, smaller binary |
| `panic = "abort"` | less code to link | no unwinding (fine: panics are forbidden) |

Notes:

- Any profile change invalidates the build cache once, then benefits every build (including
  `nix build`, which uses the same profile).
- `jobs = 8` comes from the owner's global `~/.cargo/config.toml` on a 24-thread machine — for
  release builds `cargo build --release -j 24` is faster here, but only if RAM allows it
  (15 GB total, ~4 GB currently free → 24 parallel `rustc` can swap/OOM; `jobs = 12` is a safer
  experiment).
- `-C target-cpu=native` also comes from that global config. It makes the binary non-portable
  across machines — fine for local builds, must NOT leak into released artifacts or the Nix build.
- Do not enable `incremental` for release: it interacts badly with LTO and bloats `target/`.

## 5. How to measure (before/after, web + native)

```bash
# native RSS after opening a 2000x1500 PNG:
/usr/bin/time -v ./target/release/russet

# web: build, open the page, draw a long stroke, then in the browser console:
trunk build --release && trunk serve
performance.measureUserAgentSpecificMemory().then(console.log)  # Chrome, cross-origin isolated
```

Browser DevTools → Performance while drawing: watch `generateMipmap` calls, GPU memory, and JS
heap. Do the same on a phone-sized viewport to see the real wasm ceiling.

## 6. Suggested order

1. **B** (mipmaps) + **C** (open path) — small, safe, measurable.
2. **A** (region undo) — the big RAM/latency win; needs a SPEC §3.6 decision.
3. **D** (wasm features / logging / wasm-opt) — needs the §6 exception for wasm-opt.
4. **E** (budgets) — decide after measuring A on the web.
5. **F** — only if the profiler still shows allocation churn.
