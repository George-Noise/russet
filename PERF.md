# PERF — RAM and build-speed backlog

> **Status: design note. Nothing below is implemented yet.** This file is the owner's and the
> agent's shared backlog for making Russet cheaper on memory (especially in the browser) and faster
> to build. It is NOT part of the product spec: `SPEC.md` stays authoritative. §0 says how to use
> it when the owner picks a task.

## 0. How to use this backlog (read me first, agent)

This file is a **reference backlog, not an autonomous work queue.** The owner picks an item and
tells you to do it; do not start P1 just because it is first, and do not run any task on your own.
When the owner does pick one, §3 has the details, §1 the measurements, §4 the build options and §6
a suggested order.

### Rules of engagement

1. **One task per pass.** Do P1 completely (code + tests + docs + measurements), then P2. Never
   mix two tasks in one round.
2. **Owner gates are hard.** A task marked `GATE:` changes `SPEC.md` (deps in §1, undo semantics in
   §3.6, out-of-scope in §6, canvas limits) — stop and ask the owner before writing code. Tasks
   without a gate are not "free to start": the owner still has to ask for them.
3. **No new crates, no new directories, no build tooling.** Same constraints as `AGENTS.md`.
   Desired behavior must be reachable with the existing `eframe` / `tiny-skia` / `image` / `rfd`.
4. **Tests follow `AGENTS.md` "Tests":** one `<module>/tests.rs` per module, `#[cfg(test)] mod
   tests;` in the source, never inline test modules, never widen visibility for a test.
5. **Platform branches:** only `main.rs` / `file_io.rs` (+ call sites in `app.rs`), `cfg!()` macros
   rather than `#[cfg]` attributes, unless you are editing `Cargo.toml` under P4.
6. **Verify every task:** `bash scripts/verify.sh` (fmt + clippy + test + size report). For perf
   tasks also re-measure and record numbers (§1).
7. **Do not run `cargo build --release`** except for P8 (build-profile work) — the owner's rule.
8. **Update this file and `PROGRESS.md` in the same edit**: tick the checkbox, fill the task's
   `Result:` line with date + measurements, update the §1 table and the §6 status board, and
   propose a commit title (AGENTS.md "Git & progress log"). The agent does not commit; the owner
   does.
9. **Stop and ask** if a task turns out to need behavior the owner never approved, if a test would
   have to be weakened, or if the measurements show the task makes things worse.

### Definition of done for a task

- Behavior that users can see is unchanged (or exactly what the task says).
- New/updated tests in the owning module's `tests.rs`; `cargo test` green.
- Numbers: before/after written into §1 (or the task's `Result:` line if it is not a global metric).
- `PERF.md` checkbox + `PROGRESS.md` entry + one commit.
- For web-affecting tasks: list exactly what the owner must click in the browser (trunk is not
  installed in the agent sandbox; M8 is owner-verified).

### Status board

| ID | Task | Size | Gate | Status |
|---|---|---|---|---|
| P1 | Mipmaps only at zoom < 100 % | S | — | [ ] TODO |
| P2 | Open path: no duplicate image buffer | S | — | [ ] TODO |
| P3 | Region-based undo instead of full snapshots | M | **SPEC §3.6** | [ ] TODO |
| P4 | wasm-only eframe features + release log level | S/M | **SPEC §1** | [ ] TODO |
| P5 | wasm-opt for the web bundle | S | **SPEC §6** | [x] DONE (build fix, 2026-11-09) |
| P6 | Canvas/history budgets on wasm | S | **owner decision** | [ ] TODO |
| P7 | Micro-allocation cleanups | S | — (after P1–P6 profiled) | [ ] TODO |
| P8 | Release profile tuning (build speed) | S | **SPEC §1** | [ ] TODO |

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
| History after 20 strokes, 2000×1500 canvas (full snapshots) | **240 MB** (formula, §2) |
| `trunk build --release`, first build (wasm + bindgen + wasm-opt) | **144 s** (fails at wasm-opt without P5's flags); rebuild **10 s** |
| `dist/` release bundle | wasm **5,017,305 B**, JS **76,943 B**, html **6,167 B** |
| wasm `-Oz` instead of trunk's default `-O` | 4,941,128 B (−1.5 % more) |
| wasm fetch + compile in Firefox (localhost) | ~70 ms |

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

## 3. Tasks

### Which task fixes which cost

| Cost | Task |
|---|---|
| History: 240 MB after 20 strokes on 2000×1500 (full snapshots) | **P3** (biggest win) |
| Mip chain: +33 % VRAM and a full `generateMipmap` per upload while drawing | **P1** |
| Open spike: one extra full-size RGBA buffer (48 MB for a 4000×3000 photo) | **P2** |
| Bundle/code memory: 5.0 MB wasm + 77 KB JS | **P4** (features), **P5** (wasm-opt) |
| Worst case at 4096×4096: ~405 MB | **P6** (web cap) |
| Per-frame transient `Vec`/`ColorImage` allocations | **P7** |

Note: none of these is "remove the web build". The web build costs nothing on native; the RAM
numbers above are mostly the same on native and wasm — the difference is that wasm linear memory
never shrinks, so every transient peak is permanent there.

### P1 — Mipmaps only at zoom < 100 %  [no gate, S]

- **Do:** in `src/app/camera.rs`, `canvas_texture_options()` returns options from
  `CANVAS_TEXTURE_CRISP` / `CANVAS_TEXTURE_SMOOTH` (`src/app.rs`). Both set
  `mipmap_mode: Some(Linear)`. `egui_glow 0.37` (`painter.rs:632`) runs `gl.generate_mipmap` on
  **every** upload when mips are on — once per painted frame, plus every `set_partial` of a dirty
  rect. Set `mipmap_mode = (zoom < 1.0).then_some(TextureFilter::Linear)`.
- **Why:** mips are only sampled when the texture is minified. At 100–800 % they cost a full mip
  regeneration per frame and +33 % VRAM and add nothing visually. Bonus: mips on non-power-of-two
  textures are invalid on WebGL1 (a 800×600 canvas is non-POT).
- **Refactor for testability:** extract a pure `fn texture_options_for_zoom(zoom: f32) ->
  egui::TextureOptions` (used by `canvas_texture_options`) so `src/app/camera/tests.rs` can assert
  it without an `App`/`egui::Context`.
- **Tests (camera/tests.rs):** mips `None` at 1.0 / 2.0 / 1.23; mips `Some` at 0.5 / 0.1; crisp
  (nearest magnification) at integer ≥100 %, linear otherwise.
- **Verify:** tests + clippy + fmt. Owner (browser): draw at 100 % and zoom out to ~50 % — no
  shimmer or blockiness while zoomed out, edges still crisp at 200 %.
- **Result:** _not done_

### P2 — Open path: no duplicate image buffer  [no gate, S]

- **Do:** `src/file_io.rs::pixmap_from_image` uses `image.to_rgba8()` (clones/converts) and then
  copies pixel by pixel into a fresh `Pixmap`. Use `into_rgba8()` (consumes the `DynamicImage`) and
  premultiply straight into `pixmap.data_mut()` (or `pixels_mut()`), with no intermediate `Vec`.
  Take care: the PNG/JPEG decoders return `Rgba8`/`Rgb8`; `into_rgba8()` still allocates for RGB
  sources — that one conversion is unavoidable, the win is skipping the second buffer for RGBA.
- **Why:** a 4000×3000 photo costs ~48 MB extra, permanently on wasm.
- **Tests:** `file_io/tests.rs` already has `loading_an_image_premultiplies_semi_transparent_pixels`
  and the PNG round trip; keep them green (they are the correctness contract). Add a test for a
  1×1 `Rgba8` source and a 1×1 `Rgb8` source if not covered.
- **Verify:** tests + clippy + fmt; native RSS with `/usr/bin/time -v` before/after opening a large
  PNG (owner or agent on a machine with a GUI; the decode itself can be timed in a test).
- **Result:** _not done_

### P3 — Region-based undo instead of full snapshots  [GATE: SPEC §3.6, M]

- **Ask first:** "Undo becomes patch-based internally (same 20 steps, same byte cap, same visible
  behavior). May I update SPEC §3.6 and the `History` API?" Do not start without a yes.
- **Do:** `History` stops storing `Pixmap` clones. Store per-stroke patches:
  - `DirtyRect` already comes from `tools::draw_segment`; `Canvas::region_bytes` extracts the
    pixels. Add `Canvas::write_region(x, y, w, h, &[u8])` (row-wise copy, premultiplied).
  - A stroke record = ordered list of `(rect, before: Vec<u8>, after: Vec<u8>)`. Capture `before`
    immediately before each `draw_segment`, `after` immediately after, append to the open stroke.
  - `app.rs::handle_stroke`: replace `history.push_snapshot(canvas.pixmap().clone())` with
    `history.begin_stroke()` on mouse-down and `history.commit_stroke()` on mouse-up; a click that
    paints nothing must not commit an empty record.
  - `undo`/`redo` walk patches in reverse/forward order and call `write_region`; they no longer
    clone the canvas. `MAX_SNAPSHOTS` becomes `MAX_STEPS` (20 strokes), `MAX_BYTES` bounds the
    total patch bytes; `trim` drops whole strokes.
  - `app/dialogs.rs` (New/Resize) already calls `history.clear()` — keep that.
- **Tests (history/tests.rs, canvas/tests.rs):** patch apply/restore round trip; undo/redo over two
  strokes restores exact bytes; a "click = one pixel" stroke stores only that pixel; step/byte caps;
  empty history is a no-op (no panic).
- **Done when:** a 20-stroke session on a 2000×1500 canvas keeps history **≤ 8 MB** (assert it in a
  test with synthetic small rects; record the real number in §1) and no code path clones a full
  `Pixmap` for history.
- **Result:** _not done_

### P4 — wasm-only eframe features + release log level  [GATE: SPEC §1, S/M]

- **Ask first:** SPEC §1 pins the exact `Cargo.toml`; changing dependency features is a §1 edit.
- **Do:**
  - Split the `eframe` dependency by target (native keeps `accesskit`, `system_fonts`, `wayland`,
    `x11`; `wasm32` keeps `glow`, `default_fonts`, `clipboard`, `links`). Both blocks identical
    version/`default-features = false`.
  - `src/main.rs` wasm entry: `WebLogger::init(log::LevelFilter::Info)` (or `Warn`) instead of
    `Debug`.
  - Optional in the same pass: `panic = "abort"` in `[profile.release]` (panics are forbidden on
    user-data paths anyway; check the web build runs).
- **Verify:** `cargo check`, `cargo check --target wasm32-unknown-unknown` **if** the target is
  installed (it is not in the sandbox — hand this to the owner), native tests, and a size
  comparison of the native binary. Browser: the app loads, keyboard/tab focus still works.
- **Result:** _not done_

### P5 — wasm-opt for the web bundle  [DONE 2026-11-09 — build fix, was a gate]

- **What actually happened:** trunk 0.22.1 runs wasm-opt on **every release build by default**
  (`-O`), so wasm-opt is not an optional §6 optimization — a release build *fails* without a fix.
  The bundled wasm-opt 131 rejects Rust 1.99's bulk-memory output
  (`memory.copy operations require bulk memory operations`), so `web/index.html` now passes
  `data-wasm-opt-params="--all-features"` on the rust link.
- **Measured:** wasm 5,317,459 → 5,017,305 B with `-O` (5.6 %); `-Oz` would give 4,941,128 B
  (−1.5 % more) but is not worth a second config knob. First release web build 144 s, rebuild 10 s.
- **Not a §6 change:** the size gain is small and the flags are required for the default pipeline
  to succeed; §6's "no wasm size optimization" still stands (no `-Oz`, no wasm-split).
- **Result:** DONE — `web/index.html`, SPEC §2.2, verified end-to-end in Firefox (see
  `PROGRESS.md` 2026-11-09 "M8: web build verified").

### P6 — Canvas/history budgets on wasm  [GATE: owner decision, S]

- **Ask first:** "Cap web canvases at 2048 px and/or the web history at 64 MB?" This changes what
  users can do on the web, so it needs an explicit yes and a SPEC note.
- **Do:** use `cfg!(target_arch = "wasm32")` at the `History`/`MAX_CANVAS` call sites in `app.rs`
  (no new `#[cfg]` attributes outside `main.rs`/`file_io.rs`).
- **Effect:** a 4096×4096 web session is ~405 MB today; a 2048 cap makes the worst case ~16 MB
  pixmap + ~64 MB history.
- **Verify:** tests; browser smoke test at the cap.
- **Result:** _not done_

### P7 — Micro-allocation cleanups  [no gate, S, only after profiling]

Only do these if the profiler (or `/usr/bin/time -v`) still shows allocation churn:

- Build `egui::ColorImage` directly into `Vec<Color32>` from `Pixmap` rows instead of allocating
  `Vec<u8>` via `region_bytes` and copying it again.
- Reuse the region buffer across frames (keep it in `App`; `set_partial` copies).
- On open/resize, drop the old canvas before allocating the new `Pixmap` where the borrow checker
  allows, to halve the transient peak.
- **Result:** _not done_

### P8 — Release profile tuning  [GATE: SPEC §1, S]

- **Ask first:** "May I switch the release profile to `lto = "thin"` + `codegen-units = 16`?" It
  trades a few percent of runtime speed for a large cut in build time (§4).
- **Do:** edit `[profile.release]`, then **measure on a warmed cache**: cold build, `touch
  src/main.rs` + build, binary size. Fill §1 with the new row (keep the old numbers as "fat LTO").
  If the numbers disappoint or the owner says no, revert and record why.
- **Note:** do not edit the owner's global `~/.cargo/config.toml` (`jobs = 8`,
  `target-cpu=native`). Pass flags on the command line for experiments.
- **Result:** _not done_

## 4. Build-time options (reference for P8)

Today: `lto = "fat"`, `codegen-units = 1`, `strip = true`. Cold 243 s, relink 160 s (§1).

| Change | Build-time effect | Runtime/binary effect |
|---|---|---|
| `lto = "thin"` | usually −50–70 % of link time | ~1–3 % slower code, slightly bigger binary |
| `codegen-units = 16` (default) | parallel codegen, noticeable win | ~0–2 % slower code |
| `opt-level = 2` | ~10–20 % faster compile | a few % slower code, smaller binary |
| `panic = "abort"` | less code to link | no unwinding (fine: panics are forbidden) |

Notes:

- Any profile change invalidates the build cache once, then benefits every build (including
  `nix build`, which uses the same profile).
- `jobs = 8` comes from the owner's global `~/.cargo/config.toml` on a 24-thread machine — a
  release build with `-j 12` is a safe experiment here; `-j 24` needs more RAM than the ~4 GB free.
- `-C target-cpu=native` also comes from that global config. It makes the binary non-portable
  across machines — fine for local builds, must NOT leak into released artifacts or the Nix build.
- Do not enable `incremental` for release: it interacts badly with LTO and bloats `target/`.

## 5. How to measure (before/after, web + native)

```bash
# native RSS after opening a 2000x1500 PNG:
/usr/bin/time -v ./target/release/russet

# build timing (P8):
{ time cargo build --release; } 2>&1 | tail -3   # cold
touch src/main.rs && { time cargo build --release; } 2>&1 | tail -3

# web (owner; trunk is not installed in the sandbox):
trunk build --release && trunk serve
ls -l dist/*.wasm
# browser console (Chrome, cross-origin isolated):
performance.measureUserAgentSpecificMemory().then(console.log)
```

Browser DevTools → Performance while drawing: watch `generateMipmap` calls, GPU memory, and JS
heap. Repeat on a phone-sized viewport to see the real wasm ceiling.

## 6. Order and handoff

1. **P1 + P2** — small, safe, measurable; no owner gate.
2. **P3** — the big RAM/latency win; blocked on the SPEC §3.6 decision.
3. **P4 + P5** — web weight and startup; P5 blocked on the SPEC §6 exception.
4. **P6** — budgets; decide only after P3 is measured in a browser.
5. **P7** — only if the profiler still shows churn.
6. **P8** — independent, do it whenever the owner wants faster releases.

Handoff checklist for the agent: status board ticked → §1 numbers updated → task `Result:` line
filled → `PROGRESS.md` entry (date, what, files, verification, what the owner must click) → one
local commit.
