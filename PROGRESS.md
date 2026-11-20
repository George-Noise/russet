# PROGRESS — Russet

> **Why this file exists.** `.git` reaches the agent only when the owner includes it in the
> delivery, and it is gone once the agent sandbox is recycled. Git is therefore NOT a
> progress channel in this setup. **This file is the substitute for the git history**: read
> it before starting work, append to it after every finished, verified task
> (see AGENTS.md "Git & progress log" for the exact routine). Entries older than the
> newest 3–5 live in `PROGRESS-archive.md` (grep it for history).

## Status snapshot — update in the same edit as a log entry

| Milestone | Status | Notes |
|---|---|---|
| M0 Scaffold | done | |
| M1 Canvas + zoom/pan | done | pan: wheel / Shift+wheel / middle / Space+LMB / WASD (physical keys) |
| M2 Dirty-region texture updates | done | |
| M3 Tools + history | done | |
| M4 Menu + windows + §3.11 placement | done | §3.11 reworked once — see PROGRESS-archive.md, 2026-11-09 "magnetism rework" |
| M5 Files (native, Windows) | done | |
| M6 Undo/Redo + hotkeys | done | |
| M7 Resize Canvas + New | done | |
| M8 Web (WASM) | built + headless Firefox check (2026-11-09) | wasm32 check/clippy green; `trunk build --release` OK (wasm 5.0 MB + 77 KB JS; page `web/index.html` + `Trunk.toml`); `trunk serve` + Firefox: UI renders, canvas draws, WebGL2 — a headless smoke test, not a human pass. Owner must click through drawing/zoom/pan/Open/Save with a real mouse |
| M9 Acceptance + release | pending (owner) | §7 checklist is manual, on the owner's Windows machine; `cargo build --release` is run by the owner |
| Extra: Tools strip fits the window — variant A2 (owner 2026-11-19) | done — gates + probes green | the title strip's HORIZONTAL padding zeroed via `Window::title_frame` (`windows.rs::tools_title_frame` = egui's default window frame with `inner_margin.left/right = 0`): the strip paints EXACTLY the window's width instead of ~11 pt past it, the ✕ no longer hangs half outside, BOTH buttons kept, the strip's height/fill/corners unchanged (variant A `Margin::ZERO` shrank the strip vertically — not picked); +1 regression test on the `__title_click` geometry; `gui_collapse_check.sh` locate fixed (the idle crosshair cursor fooled it); V38 + V36 verified in chat, V39 born |
| Extra: Tools panel hides while collapsed (owner 2026-11-19) | done — gates + click probe green | the magnetized settings panel hides while the Tools window is collapsed to its title bar and returns on expand (`windows.rs::settings_panel_shown` on `Window::show`'s `inner.is_some()`); the header-width PROPOSAL delivered (measured options, no header code change — the owner picks); V38 born |
| Extra: tool_icons.rs file-guard split — the drawings to the child `draw` (owner 2026-11-19) | done — gates green, suite unchanged | 673 → 308 + 400, both under the 500 guard: the SEVEN icon drawings (`draw_*` fns, `TOOL_ANGLE`, `PIPETTE_TIP`, the `rasterize_tool_icon` dispatcher) moved VERBATIM to the child `app/tool_icons/draw.rs`; the parent keeps the atlas/cursor pipeline + the shared primitives and imports the dispatcher — zero external path changes (app.rs/windows.rs untouched, the tests' `super::` imports byte-identical); rules.sh's second OVER-GUARD note retired (only the owner-deferred history_tree.rs 569 remains) |
| Extra: palette alpha + a hex field in the color wells (owner 2026-11-19) | done — gates + driver green | `Alpha::OnlyBlend` (the picker's alpha slider + the A DragValue; the eyedropper's alpha no longer erased on popup open) + a "Hex" row parsing the launch-flag syntax (short form keeps the well's alpha, 8 digits sets it, `#RRGGBBAA` display, canonicalized on blur); one shared `launch.rs::parse_hex_components` feeds both consumers; windows/tests split by topic (rule 7) + 5 color-well tests; windows.rs 571 — OVER guard, the palette split PROPOSED (owner-gated); V40 born |
| Extra: lenient hex + the windows.rs palette split (owner 2026-11-19) | done — gates + driver green | the Hex field eats pasted junk (every non-hex-digit char stripped, an optional `0x` prefix dropped) and the new digit-count forms 4 `RGBA` / 5 `RGBAA` / 7 `RRGGBBA` (the 5-vs-7 asymmetry is the owner's deliberate design); a per-well SEED alpha keeps a live-typed 5-digit prefix from leaking alpha into a final 6-digit color; the launch flags share the same `parse_hex_components` syntax; PURE code-motion split: `windows.rs` 571 → 351 + the child `windows/palette_window.rs` 287 (the palette window, wells, picker popup, hex field; the color-well tests moved to `palette_window/tests.rs` per Tests rule 2) — both under the guard, the OVER-GUARD note retired; V40 verified in chat, V41 born |
| Extra: the flake drives the trunk lanes — web-serve / web-build / web-build-release (owner 2026-11-19) | done — hashes verified from the real downloads, gates green; a dup-`russet` app defect (the owner's first `.#web-serve` probe) is fixed, as is the zstd-sys build (nixpkgs' hook forces the system libzstd → pkg-config + zstd inputs — probe 2 confirmed: trunk builds and serves); probe 3 added stdenv.cc to the wrappers' PATH (host build-script linker); probe 4 added pkgs.lld (the final wasm link asks for a PATH `lld` from the owner's cargo env — honored, not overridden) | the pinned trunk 0.22.1 is BUILT by the flake from the crates.io source (nixpkgs' trunk is 0.21.14 — the pin honored by construction; both fetchurl hashes computed from the real bytes, the shipped Cargo.lock byte-identical to the v0.22.1 tag, vendored hash-free via `cargoLock.lockFile`); `nix run .#web-serve` → trunk serve, `.#web-build` → trunk build, `.#web-build-release` → trunk build --release — apps run OUTSIDE the sandbox so trunk self-manages wasm-bindgen 0.2.130 + wasm-opt (~/.cache/trunk); `nix run` KEEPS the native app (owner decision same day: the default stays native, easier on the brain — the web apps are always explicit); `nix develop` gains the pinned trunk; README / SPEC §8 / AGENTS synced; a sandboxed `nix build` of the site deliberately NOT provided (no network; a FOD dist/ would pin a hash per commit) |
| Extra: the delivery-pyc fix — .gitignore landed + deliver.sh guards (owner 2026-11-19) | done — guard tested both ways, gates green | the owner re-caught `scripts/__pycache__/mcp_smoke.cpython-312.pyc` inside russet.zip (AND the patch): the v5 round CLAIMED the .gitignore entries but they never landed — HEAD carried only /target /dist /result, so the leak reopened on the first new import; `.gitignore` now really carries `__pycache__/` + `*.pyc`; `visual_driver.py` sets `sys.dont_write_bytecode` before the mcp_smoke import (the cache is no longer written at the source); `deliver.sh` gains TWO hard guards (the staged tree + the patch's ^diff headers) that FAIL the delivery on any bytecode cache |
| Extra: the "Save Image" dialog — Save and Save As converge (owner 2026-11-19) | done — gates + GUI probes + driver green | BOTH Save and Save As open ONE in-app dialog (`app/save_dialog.rs`): the file name field, the PNG/JPEG format radio (THE format choice), the JPEG Quality 1..=100 (default 90), the live "Saving as:" preview, the native Folder row + Browse… (hidden on web; a cancelled pick keeps the dialog open); `file_io.rs` gains `SaveFormat`, the quality-parameterized JPEG encoder and `pick_save_folder`; the MCP `export_png` keeps the extension-decides path export; debt F17 born (consider other formats) |
| Extra: smooth freehand strokes — coalesced pointer moves + midpoint-quadratic pieces (owner 2026-11-19) | done — gates + driver green | NEW `tools/smooth.rs`: every coalesced `Event::PointerMoved` feeds the midpoint-quadratic window (the old one-point-per-frame stream drew the fast-stroke polygon); brush/eraser stroke ONE C1-continuous tiny-skia path per frame with a batched dirty rect; the pencil keeps its 1 px Bresenham trail along the smoothed centerline (quads flattened to ~1 px steps, translucent-safe joint dedupe); `draw_segment` stays for the press dot + MCP paths; SPEC §3.5 rewritten; V44 born — the pencil half FAILED on the owner's pass and was reworked 2026-11-20 (the next row) |
| Extra: pencil drifting pen — the 1 px staircase made stable (owner 2026-11-20) | done — gates + driver + a real-pointer Xvfb probe green | the owner's ragged-pencil screenshot diagnosed: per-run Bresenham restarts between independently rounded ~1 px endpoints stacked double-covered columns and punched holes on every diagonal, the pattern varying per computer (the pixel pattern followed the per-frame event batching); the pencil now walks a gesture-level DRIFTING PEN (`PencilTrail`: the pen + the dedupe, carried across pieces and frames) — unit steps along the chord-dominant axis, the minor coordinate an EXACT float (the same error accumulation one long Bresenham run performs), each step plotting one pixel; the release pins the raw release point; the subdivision cap 64 → 4096 (fidelity on fast strokes); V44 closed failed → V45 born |
| Extra: V45 verified (Windows + Web); the smooth-stroke X11/Wayland pass opens V46 (owner 2026-11-20) | done — docs only | DEBTS-verification bookkeeping: V45 closed verified in chat (the owner's Windows AND Web passes), the same smooth-stroke checklist on Linux (X11 AND Wayland backends) born as V46; no code touched |

**Verification stamp:** FULL gate re-verified 2026-11-20 (the pencil drifting-pen
round): fmt clean, rules.sh HARD-green, clippy `-D warnings` green (default +
mcp), `cargo test --profile gates` 211 default / 211 mcp green (+1 ignored each
— the permanent #[ignore] icon dump; +2 drifting-pen regression tests this
round), the visual driver ALL SCENARIOS GREEN under Xvfb through the gates+mcp
binary, and a real-pointer Xvfb probe (xdotool drags) drew pixel-perfect
diagonals through the real app: 0 double-covered columns, 0 holes, 8-connected
— Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes: the wasm
`cargo check` green in every hot gate; the flake's trunk apps landed 2026-11-19 (V43 =
the NixOS pass of `nix run .#web-serve` / `.#web-build` / `.#web-build-release`); the manual trunk
lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569 (the
owner-approved 2026-11-19 palette split brought `app/windows.rs` back under the guard:
571 → 351 + the child `windows/palette_window.rs` 287).

Update this stamp in the same edit as a log entry (AGENTS.md "Git & progress log").

## Warnings — open issues deliberately NOT fixed yet

- 2026-11-13 — No active warnings. Resolved items moved to PROGRESS-archive.md;
  open HUMAN checks live in DEBTS-verification.md, open functional debts in
  DEBTS-functional.md (the agent must ask about them per AGENTS.md "Debts").

## Log — newest first. Insert a new entry directly under this heading.

> Budget: ≤ ~20 lines per entry, fixed fields (What / Files / Verified / Owner-checks /
> Commit title); durable facts move to their canonical home in the same edit (AGENTS.md
> "Git & progress log"). Entries older than the newest 3–5 are moved verbatim to
> PROGRESS-archive.md. Open human checks live in DEBTS-verification.md — no entry is
> ever pinned in place by a pending check. Compacted 2026-11-18: the panels-split and
> docs rounds moved to PROGRESS-archive.md; later the same day the F14 + quotes rounds
> followed them (the verify.sh round needed the budget back), the tool-icons entry
> joined them in the evening (the icon-redraw round), the About entry followed
> it (V33's closure freed it — the tooling round's budget), the unoptimized-dev
> entry followed that (the dump round's budget), the verify.sh-hardening entry
> completed the day's compactions (the icons-v2 round's budget); on 2026-11-18/19 the
> icon-redraw, v5, Tooling, icon-dump, icons-v2, icons-v3, language-rule and
> icons-v4 entries each moved under their following rounds' budgets, and the
> collapse entry moved during the 2026-11-19 lenient-hex round's budget. The
> header-strip (variant A2) entry moved to the archive during the Save-dialog
> round's budget (2026-11-19); the Save-dialog entry followed during the
> smooth-strokes round's budget (same day); the flake entry moved during this
> docs round's budget (2026-11-20).

### 2026-11-20 — Docs: V45 verified (Windows + Web); the smooth-stroke X11/Wayland pass opens V46 — owner request

- **What:** bookkeeping only, zero code: the owner verified the pencil drifting-pen pass
  in chat in BOTH Windows and the Web build — V45 closed verified in DEBTS-verification.md;
  his remaining platform leg (the same smooth-stroke checklist on Linux, X11 AND Wayland,
  the flake's native lanes) born as V46 in the same edit. The chat also answered,
  informationally, the per-tool coverage of the smooth-stroke work: the V44 curve
  smoothing feeds ALL THREE freehand tools; the V45 drifting-pen stability fix is
  PENCIL-ONLY — `rasterize_smooth_pieces` branches on `Tool::Pencil`, while the brush and
  eraser stroke ONE continuous tiny-skia path per frame (`trail`/`pin` ignored there), so
  the per-run-restart defect class (double-covered columns, holes) cannot exist on them.
- **Files:** DEBTS-verification.md (V46 born; V45 → Resolved), PROGRESS.md (snapshot row
  + this entry; the flake entry moved to PROGRESS-archive.md — the Log budget).
- **Verified:** docs-only — the Rust tree stays byte-identical to the owner-verified V45
  delivery; `rules.sh` HARD green + `cargo fmt --check` clean before and after; the
  deliver.sh byte-compares green.
- **Owner-checks:** V45 verified in chat (Windows + Web). Open: V46; parked: V30, V8.
  Debts open: F17, F16, F15.
- **Commit title:** `Debts: V45 verified (Windows + Web); the smooth-stroke X11/Wayland pass opens V46`

### 2026-11-20 — `Fix:` the pencil's ragged 1 px trail — the drifting pen — owner request

- **What:** the owner's screenshot (a 51x44 canvas crop) showed the smoothed pencil drawing
  EXTRA pixels: double-covered columns and holes on every diagonal, the pattern differing
  between computers. Diagnosed (a Python f32 replica of the pipeline + an angle scan): the
  pencil re-ran integer Bresenham between independently rounded ~1 px endpoints of the
  flattened quads — each ~1 px sub-run restarted the line algorithm, so two consecutive
  rounded endpoints 2 columns apart plotted an extra pixel and the next run compensated with
  a hole; the subdivision step count came from the control-polygon length (over-dense on
  bent pieces) with a 64 cap (sparse on fast strokes), and the pattern followed the per-frame
  event batching (different machines coalesce differently). Replaced by the DRIFTING PEN:
  `PencilTrail` (the exact float pen + the last plotted pixel) carried across pieces AND
  frames on the gesture; the pen walks UNIT steps of the chord-dominant axis, the minor
  coordinate advances proportionally and is carried EXACTLY (never rounded into the state) —
  the same error accumulation one long Bresenham run performs, restarts impossible; one
  pixel per step, 8-connected by construction; the release pins the raw release point
  (`rasterize_smooth_pieces` gained `pin`); the subdivision cap 64 → 4096. The smoothed
  path of integer-rounded pointer events still wobbles locally past 45 degrees, so an
  isolated stacked pixel remains (honest staircase, ~2 per 32 px on the ragged input,
  was 5 + a broken look). Brush/eraser untouched; `draw_segment` unchanged (the press dot
  + MCP paths).
- **Files:** `src/tools/smooth.rs` (PencilTrail + walk_pen; plot_pencil_run removed),
  `src/tools.rs` (the PencilTrail re-export), `src/app.rs` (stroke_last_pixel → stroke_trail),
  `src/app/input.rs` (the trail plumbing + the release pin), NEW `tools/smooth/tests/trail.rs`
  (2 regression tests: the integer-event staircase + the byte-identical frame-batch
  invariance), `tools/smooth/tests/raster.rs` (the trail/pin signatures; the parity test
  rewritten to the new invariants), SPEC §3.5 + §7.2, TIMING (2 rows), DEBTS-verification
  (V44 closed failed, V45 born), PROGRESS (snapshot + entry + stamp; the pyc entry moved
  to the archive).
- **Verified:** FULL gate ALL GATES GREEN 74 s — fmt clean, rules HARD green, clippy
  `-D warnings` green (default + mcp), `cargo test --profile gates` 211 default / 211 mcp
  green (+1 ignored each; +2 tests); the visual driver ALL SCENARIOS GREEN under Xvfb;
  a real-pointer Xvfb probe (xdotool press-drag-release through the gates+mcp binary):
  the fast diagonal 379 px / 379 columns (0 double-covered, 0 holes, 8-connected), the
  slow diagonal 401/401, the steep curve 1 px per row — pixel-perfect staircases in the
  real app.
- **Owner-checks:** V45 born (the Windows pass: slow + fast diagonals and curves — an even
  staircase, the same pattern at any speed/machine; translucent joints; the click pixel;
  the release end; undo). V44 closed failed (the pencil half; the brush/eraser smoothing
  held up). Open: V45, V40 is the newest verified; ask = V45, V30 (parked), V8 (parked);
  debts F17, F16, F15.
- **Commit title:** `Pencil drifting pen: the 1 px staircase stops depending on the frame batching (exact-float minor phase carried across the gesture, one pixel per unit step, the release pins the raw release point; the per-run Bresenham restarts drew double-covered columns and holes, differently per machine)`


### 2026-11-19 — `Feat:` smooth freehand strokes — owner request

- **What:** the fast-stroke polygon look is gone on ALL THREE freehand tools (owner:
  "when you move the mouse fast you get a jagged line" — pencil, brush and eraser alike).
  The gesture now feeds EVERY coalesced `Event::PointerMoved` of the frame (the old code
  kept only `latest_pos` — one point per frame) into a midpoint-quadratic smoothing
  window — NEW `tools/smooth.rs`: the first point emits the straight filler
  `start → mid(start, p)`, every next point the quad
  `mid(prev2, prev1) → (control prev1) → mid(prev1, p)` (C1-continuous, never overshoots),
  release closes with `mid(prev1, p) → p`. Brush/eraser stroke the frame's pieces as ONE
  tiny-skia path (no per-segment cap seams; the horizontal/vertical zero-bounds epsilon
  nudge carried over from `rasterize_polyline`); the pencil keeps its 1 px Bresenham
  trail — quads flatten to ~1 px steps (≤ 64 per piece) and consecutive duplicate pixels
  are DEDUPED (`stroke_last_pixel` lives on the gesture — a translucent color must not
  double-blend at the subdivision/frame joints). Pieces rasterize in ONE batch per frame
  (one composite + one `set_partial` per frame); the gesture paints with the color
  captured at stroke start — a mid-gesture well swap no longer recolors the remainder
  (matches the single-color history node). `draw_segment` stays for the press dot + MCP
  paths; SPEC §3.5 rewritten in the same edit.
- **Files:** NEW `src/tools/smooth.rs` (301) + `smooth/tests.rs` split by Tests rule 7
  into `tests/window.rs` (7 tests) + `tests/raster.rs` (8), `src/tools.rs` (the child
  module + re-exports), `src/app/input.rs` (+ NEW `input/tests.rs`, 2 tests — the
  pointer-move stream helper), `src/app.rs` (`last_pos` → `stroke_smooth` +
  `stroke_last_pixel`), SPEC §3.5, DEBTS-verification.md (V44 born; V43's row restored
  + closed, V42/V41 closed), TIMING (2 cold rows), PROGRESS (snapshot + entry + stamp
  + compaction).
- **Verified:** FULL hot gate ALL GATES GREEN 14 s — fmt clean, rules HARD green,
  clippy `-D warnings` green (default + mcp), `cargo test --profile gates` 209 default /
  209 mcp green (+1 ignored each; +17 tests); visual driver ALL SCENARIOS GREEN under
  Xvfb through the gates+mcp binary.
- **Owner-checks:** V44 born (the Windows feel pass: fast strokes smooth on all three
  tools, the pencil stays 1 px with staircase diagonals, translucent colors behave,
  undo still one node per gesture). Closed by the owner's word in chat: V43, V42, V41
  (V43's DEBTS row had never landed — the flake round's bookkeeping gap; restored +
  closed). Open: V44, V30 (parked), V8 (parked); debts F17 (the format-set decision),
  F16, F15.
- **Commit title:** `Smooth freehand strokes: every coalesced pointer move feeds the midpoint-quadratic window (new tools/smooth.rs); brush/eraser stroke one C1 path per frame with a batched dirty rect; the pencil keeps its 1 px Bresenham trail along the smoothed centerline`
