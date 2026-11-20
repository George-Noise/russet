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
| Extra: the docs-only gate exemption into AGENTS.md (owner 2026-11-20) | done — docs only | a `.md`-only diff runs `scripts/rules.sh` + the delivery byte-compares ONLY — no builds, no tests; the full gate stays for anything beyond `.md` (Rust, scripts, Cargo, flake, CI, web); comment-only `.rs` edits keep the `cargo check` quick loop |
| Extra: the web save downloads directly — rfd's fallback window gone (owner 2026-11-20) | done — gates + wasm check + driver green | the web Save dialog's confirm no longer routes through `rfd::AsyncFileDialog::save_file()` (its wasm backend pops the "click here to download your file" Ok/Cancel window): `file_io.rs::web::download_image` encodes → Blob (the format's MIME type, new `SaveFormat::mime_type`) → object URL → one synthetic click on a detached `<a download>` with the composed name → the URL revoked on a 10 s grace timer; js-sys/wasm-bindgen ride wasm-bindgen-futures' public re-exports (zero manifest additions; the web-sys line gains 8 feature gates); rfd stays only on the Open path; V47 born |
| Extra: the Save-dialog slider captions — the numbers explain themselves (owner 2026-11-20) | done — gates + wasm check + driver green | under EACH encoding knob a live caption names the CURRENT number's effect (the owner: the digits "were not obvious even to me"): PNG "<level> — stored raw / light packing / balanced packing / tight packing …" + the tail "The picture is identical at every level — the slider only trades file size against save speed."; JPEG "<quality> — near-perfect picture / high / medium / low quality …" + the tail "Lower = a smaller file, but the picture loses more detail."; pure bucket helpers exhaustively tested (every stop 0..=9 / 1..=100 names its own number), the two headless dialog render tests assert the captions; V49 born, closed verified in chat the same day |
| Extra: the Save-dialog layout pass — preview above the name, the dropdown, the distance-separated encoding group (owner 2026-11-20) | done — gates + wasm check + driver + dialog screenshots green | the owner reviewed the V49 captions and reshaped the dialog: "Saving as" ABOVE the File name field (the result reads first), the Folder row (native) DIRECTLY under it — the "what + where" cluster; the Format radios → a `egui::ComboBox` dropdown ("PNG image" / "JPEG image" — the knob and the preview re-compose on the pick); the encoding group [Lossless ABOVE the knob + the captions] separated by a 12 pt `GROUP_GAP` (vs the 3 pt row gap) so related controls read as one block; +2 layout-order tests over the REAL dialog; no encoding change — pixels/bytes as V48 saved; V50 born |
| Extra: the Save-dialog compression slider + the Lossless row (owner 2026-11-20) | done — gates + wasm check + driver green | the PNG row gains the "Compression" slider 0..=9 — the encoder's REAL stops (`CompressionType::Level`: 0 stores raw, 1..=9 the exact deflate effort; default 6 = flate2's balanced middle; file size + encode time trade, pixels NEVER, the round-trip test proves it) — and both formats gain the owner's "Lossless" row: a per-format LOCKED DISPLAY of the guarantee (PNG on — lossless by design; JPEG off — no lossless mode the pinned encoder could write nor browsers decode); JPEG keeps Quality; the dialog's choices ride a new `SaveOptions` through the native save AND the web download; V48 born |

**Verification stamp:** FULL gate re-verified 2026-11-20 (the save-dialog layout
round): fmt clean, rules.sh HARD-green, clippy `-D warnings` green (default +
mcp), `cargo test --profile gates` 222 default / 222 mcp green (+1 ignored each
— the permanent #[ignore] icon dump; +2 tests this round), the wasm
`cargo check --target wasm32-unknown-unknown` green (the dialog compiles into
the web build), the visual driver ALL SCENARIOS GREEN under Xvfb through the
gates+mcp binary — Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
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
> docs round's budget (2026-11-20); the smooth-strokes entry followed during
> this round's budget (2026-11-20); the pencil-drifting-pen entry followed
> during the save-compression round's budget (2026-11-20); the docs-only +
> V45/V46 bookkeeping entries moved to PROGRESS-archive.md during this
> round's budget (2026-11-20); the web-download entry followed during the
> layout round's budget (2026-11-20).

### 2026-11-20 — `Feat:` the Save-dialog layout pass — owner request

- **What:** the owner reviewed the V49 captions ("надписи посмотрел") and reshaped
  the dialog: (1) the composed-name preview "Saving as" moved ABOVE the File name
  field — the result reads first; (2) the Folder row (native) sits DIRECTLY under
  the name field — "what is it called" and "where does it go" read as one cluster;
  (3) the Format row's two radios became a `egui::ComboBox` dropdown ("PNG image" /
  "JPEG image"; picking one swaps the knob and re-composes the preview live);
  (4) the encoding GROUP [Lossless + knob + captions] is separated by vertical
  distance (`GROUP_GAP` 12 pt above and below vs the 3 pt row gap) and the Lossless
  row sits ABOVE the Compression/Quality slider — the guarantee is the headline,
  the effort slider the detail under it. No encoding change — pixels and bytes are
  exactly what V48 saved; the captions are unchanged.
- **Files:** `src/app/save_dialog.rs` (the reordered body + the ComboBox + the gap
  constant + the module/method docs), `src/app/save_dialog/tests.rs` (+2
  layout-order tests over the REAL dialog), `src/file_io.rs` (3 doc comments:
  radio → dropdown), SPEC §3.7 (the rows, top to bottom), TIMING (1 row),
  DEBTS-verification (V49 closed verified; V48 + the re-run note; V50 born),
  PROGRESS (snapshot + entry + stamp; the web-download entry moved to
  PROGRESS-archive.md).
- **Verified:** FULL gate ALL GATES GREEN 16 s hot: fmt clean, rules HARD green,
  clippy `-D warnings` green (default + mcp), `cargo test --profile gates` 222
  default / 222 mcp green (+1 ignored each; +2 tests); `cargo check --target
  wasm32-unknown-unknown` green (the dialog compiles into the web build — the
  owner's V50 pass covers the browser too); the visual driver ALL SCENARIOS GREEN
  under Xvfb (the gates+mcp binary); PLUS an Xvfb probe of the REAL dialog —
  Ctrl+S screenshots of BOTH branches (the PNG and the JPEG picked through the
  real dropdown): the order, the dropdown and the group gap all visible.
- **Owner-checks:** V50 born (the Windows + Web layout pass: the preview above the
  name, the folder under it, the dropdown picks both formats, the group reads as
  one block). V49 closed verified in chat ("надписи посмотрел"). Open: V50, V48
  (+ the re-run-on-V50-build note), V46; parked: V30, V8. Debts open: F17, F16,
  F15.
- **Commit title:** `Save dialog layout: the Saving-as preview above the file name, the folder row under it, the format dropdown instead of the radio, and the encoding group (Lossless above the knob) separated by distance`

### 2026-11-20 — `Feat:` the Save dialog's slider meaning captions — owner request

- **What:** the owner looked at the V48 dialog and asked to tell the user what
  the slider's numbers MEAN ("even I wasn't quite sure"). Under EACH knob the
  dialog now shows a live caption + a fixed trade-off line: PNG —
  "<level> — stored raw / light packing / balanced packing / tight packing: …"
  + "The picture is identical at every level — the slider only trades file
  size against save speed."; JPEG — "<quality> — near-perfect picture / high
  quality / medium quality / low quality: …" + "Lower = a smaller file, but
  the picture loses more detail." The captions are pure bucket helpers
  (`png_compression_hint` / `jpeg_quality_hint`), loop-tested over EVERY stop
  (each of 0..=9 / 1..=100 names its own number); the two headless render
  tests over the REAL dialog assert the default captions + tails. No encoding
  change — pixels and bytes are exactly what V48 saved.
- **Files:** `src/app/save_dialog.rs` (the two helpers + the two tails + the
  caption rows), `src/app/save_dialog/tests.rs` (+2 tests, 2 extended),
  SPEC §3.7, DEBTS-verification (V49 born), PROGRESS (snapshot + entry +
  stamp; the docs-only + V45/V46 entries moved to PROGRESS-archive.md).
- **Verified:** FULL gate ALL GATES GREEN 16 s hot: fmt clean, rules HARD
  green, clippy `-D warnings` green (default + mcp), `cargo test --profile
  gates` 220 default / 220 mcp green (+1 ignored each; +2 tests); `cargo
  check --target wasm32-unknown-unknown` green (the dialog compiles into the
  web build); the visual driver ALL SCENARIOS GREEN under Xvfb (the gates+mcp
  binary). No TIMING rows — every command ran under a minute.
- **Owner-checks:** V49 born (the Windows + Web pass: drag each slider — the
  caption tracks the number and names its effect; the wording must be clear
  to a fresh user). Open: V49, V48, V46; parked: V30, V8. Debts open: F17,
  F16, F15.
- **Commit title:** `Save dialog: a live caption under each slider explains the current number (PNG level = packing effort: file size vs save speed, pixels identical at every level; JPEG quality = look vs file size) + the fixed trade-off tails`

### 2026-11-20 — `Feat:` the Save dialog's Compression slider + the Lossless row — owner request

- **What:** the owner asked for a save-time compression slider and THE most
  important checkbox — the lossless guarantee (PNG first; JPEG "if possible").
  The pinned image 0.25.10 PNG encoder exposes the REAL deflate scale
  (`CompressionType::Level`: 0 = store, 1..=9 = exact effort) — the PNG row's
  "Compression" slider 0..=9 (default 6 = flate2's balanced middle) has NO
  placebo stops: every stop is a distinct setting, file size + encode time
  trade, pixels never (the round-trip test decodes 0/6/9 byte-identical; the
  pre-slider fdeflate "Fast" settings live on only in the MCP/path export).
  The "Lossless" checkbox is the owner's guarantee — deliberately a per-format
  LOCKED DISPLAY, not a toggle (a toggle would be a placebo on both formats):
  PNG locks it ON ("always — PNG is lossless"), JPEG locks it OFF ("never —
  JPEG is always lossy" — the standard's lossless mode is neither writable by
  the pinned encoder nor decodable by mainstream browsers); JPEG keeps its
  Quality slider. A new `SaveOptions` (format + jpeg_quality + png_compression,
  both clamped in `commit_save_dialog`) carries the choices through the native
  save AND the web download unchanged.
- **Files:** `src/file_io.rs` (SaveOptions + DEFAULT_PNG_COMPRESSION +
  png_compression_type + encode_png_with_compression; encode_png keeps the
  legacy settings), `src/file_io/tests.rs` (+4), `src/app/save_dialog.rs` (the
  per-format knob match + the Lossless row), `src/app/save_dialog/tests.rs`
  (+2 headless render tests over the REAL `App::save_dialog`), `src/app.rs`
  (save_png_compression field), `src/app/document.rs` (the SaveOptions glue),
  SPEC §3.7, TIMING (3 rows), DEBTS-verification (V47 closed verified, V48
  born), PROGRESS (snapshot + entry + stamp; the pencil entry moved to
  PROGRESS-archive.md).
- **Verified:** FULL gate ALL GATES GREEN 186 s cold (the fresh sandbox — the
  gates lane's dep graph compiled inside it), then 13 s hot after the test
  additions: fmt clean, rules HARD green, clippy `-D warnings` green (default
  + mcp), `cargo test --profile gates` 218 default / 218 mcp green (+1 ignored
  each; +6 tests); `cargo check --target wasm32-unknown-unknown` green
  (web-facing); the visual driver ALL SCENARIOS GREEN under Xvfb.
- **Owner-checks:** V48 born (the Windows + Web pass: the slider changes the
  file size but not the pixels; the Lossless row's two locked states; both
  formats land with the composed names). V47 closed verified in chat (the
  direct-download pass). Open: V48, V46; parked: V30, V8. Debts open: F17,
  F16, F15.
- **Commit title:** `Save dialog: a PNG Compression slider 0..=9 (the encoder's real deflate stops, pixels never change) + the per-format Lossless row (locked on for PNG, locked off for JPEG) carried by SaveOptions through native and web`

