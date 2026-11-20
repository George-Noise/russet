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
| Extra: pencil drifting pen — the 1 px staircase made stable (owner 2026-11-20) | done — gates + driver + a real-pointer Xvfb probe green | the owner's ragged-pencil screenshot diagnosed: per-run Bresenham restarts between independently rounded ~1 px endpoints stacked double-covered columns and punched holes on every diagonal, the pattern varying per computer (the pixel pattern followed the per-frame event batching); the pencil now walks a gesture-level DRIFTING PEN (`PencilTrail`: the pen + the dedupe, carried across pieces and frames) — unit steps along the chord-dominant axis, the minor coordinate an EXACT float (the same error accumulation one long Bresenham run performs), each step plotting one pixel; the release pins the raw release point; the subdivision cap 64 → 4096 (fidelity on fast strokes); V44 closed failed → V45 born |
| Extra: V45 verified (Windows + Web); the smooth-stroke X11/Wayland pass opens V46 (owner 2026-11-20) | done — docs only | DEBTS-verification bookkeeping: V45 closed verified in chat (the owner's Windows AND Web passes), the same smooth-stroke checklist on Linux (X11 AND Wayland backends) born as V46; no code touched |
| Extra: the docs-only gate exemption into AGENTS.md (owner 2026-11-20) | done — docs only | a `.md`-only diff runs `scripts/rules.sh` + the delivery byte-compares ONLY — no builds, no tests; the full gate stays for anything beyond `.md` (Rust, scripts, Cargo, flake, CI, web); comment-only `.rs` edits keep the `cargo check` quick loop |
| Extra: the web save downloads directly — rfd's fallback window gone (owner 2026-11-20) | done — gates + wasm check + driver green | the web Save dialog's confirm no longer routes through `rfd::AsyncFileDialog::save_file()` (its wasm backend pops the "click here to download your file" Ok/Cancel window): `file_io.rs::web::download_image` encodes → Blob (the format's MIME type, new `SaveFormat::mime_type`) → object URL → one synthetic click on a detached `<a download>` with the composed name → the URL revoked on a 10 s grace timer; js-sys/wasm-bindgen ride wasm-bindgen-futures' public re-exports (zero manifest additions; the web-sys line gains 8 feature gates); rfd stays only on the Open path; V47 born |
| Extra: the Save-dialog slider captions — the numbers explain themselves (owner 2026-11-20) | done — gates + wasm check + driver green | under EACH encoding knob a live caption names the CURRENT number's effect (the owner: the digits "were not obvious even to me"): PNG "<level> — stored raw / light packing / balanced packing / tight packing …" + the tail "The picture is identical at every level — the slider only trades file size against save speed."; JPEG "<quality> — near-perfect picture / high / medium / low quality …" + the tail "Lower = a smaller file, but the picture loses more detail."; pure bucket helpers exhaustively tested (every stop 0..=9 / 1..=100 names its own number), the two headless dialog render tests assert the captions; V49 born, closed verified in chat the same day |
| Extra: the Save-dialog layout pass — preview above the name, the dropdown, the distance-separated encoding group (owner 2026-11-20) | done — gates + wasm check + driver + dialog screenshots green | the owner reviewed the V49 captions and reshaped the dialog: "Saving as" ABOVE the File name field (the result reads first), the Folder row (native) DIRECTLY under it — the "what + where" cluster; the Format radios → a `egui::ComboBox` dropdown ("PNG image" / "JPEG image" — the knob and the preview re-compose on the pick); the encoding group [Lossless ABOVE the knob + the captions] separated by a 12 pt `GROUP_GAP` (vs the 3 pt row gap) so related controls read as one block; +2 layout-order tests over the REAL dialog; no encoding change — pixels/bytes as V48 saved; V50 born |
| Extra: the Save-dialog compression slider + the Lossless row (owner 2026-11-20) | done — gates + wasm check + driver green | the PNG row gains the "Compression" slider 0..=9 — the encoder's REAL stops (`CompressionType::Level`: 0 stores raw, 1..=9 the exact deflate effort; default 6 = flate2's balanced middle; file size + encode time trade, pixels NEVER, the round-trip test proves it) — and both formats gain the owner's "Lossless" row: a per-format LOCKED DISPLAY of the guarantee (PNG on — lossless by design; JPEG off — no lossless mode the pinned encoder could write nor browsers decode); JPEG keeps Quality; the dialog's choices ride a new `SaveOptions` through the native save AND the web download; V48 born |

| Extra: the Shape panel's icon buttons (owner 2026-11-20) | done — gates + driver + GUI probes green | the owner reviewed the V50 Save-dialog layout ("сейчас гораздо лучше") and reshaped the Shape tool's settings panel: the word radio and the Fill/Outline checkboxes became Tools-window-style ICON buttons — the KIND row (gray square / circle previews; tooltips carry the old words) and the MODE row after a group gap (Fill = the current kind SOLID, Outline = HOLLOW — the toggles re-render for the kind switch); the shared `icon_button` now draws both windows' buttons; the toggle invariant extracted into pure `toggled_fill`/`toggled_outline`; +7 tests (5 icons + 2 panel) over the split topics; V50 closed verified in chat, V51 born |
| Extra: the Shape panel's MODE radio over the tool's icon (owner 2026-11-20) | done — gates + driver + GUI probes green | the owner verified V51 and reshaped the MODE row: the two toggles (whose "both on" state hid behind a second click) became a THREE-way radio `ShapePaint` — Fill / Outline / the explicit Fill+Outline — whose three icons draw the Shape TOOL's emblem (solid / hollow / gray-under-the-rim), no longer the selected kind (the atlas key is pixels-per-point ALONE again); the old words stay as tooltips, the third reads "Fill + Outline"; wells/commit logic untouched (`rasterize_shape` always supported both); +4 net tests; V51 + V48 closed verified in chat, V52 born |
| Extra: the Shape tool's outline placement radio (owner 2026-11-20) | done — gates + driver + GUI probes green | the second THREE-way radio `ShapeOutline` (Outside / Centered / Inside — the outline band shifted half a stroke width outward/on/inside the drag's boundary, the clamp for tiny shapes, the dirty rect reaching the full width beyond); the panel's THIRD group row of corner icons (square's corner + gray fill quadrant + click dot + the outline bands, kind-independent, tooltips carry the words); the preview matches all three modes; V52 closed verified in chat («Отлично»), V53 born |

**Verification stamp:** FULL gate re-verified 2026-11-20 (the outline-placement
round): fmt clean, rules.sh HARD-green, clippy `-D warnings` green (default +
mcp), `cargo test --profile gates` 239 green both lanes (+1 ignored; +5 net
tests this round), the visual driver ALL SCENARIOS GREEN under Xvfb through the
dev+mcp binary + the GUI click-through screenshots (xdotool, ffmpeg: the
three-group panel, all three "Outline …" tooltips, three rectangles pixel-
measured on the screenshots — one per placement mode) — Rust 1.99.1 pinned via
rust-toolchain.toml, Linux. NO wasm check this round (the diff touches no
web-facing code — the economy rule). NOTE 2026-11-20 (still true): RECYCLED
sandbox — rustup/toolchain/GUI .debs reinstalled, the repo RE-CLONED from the
owner's GitHub push; heavy commands ran synchronously (see TIMING).
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes: the wasm
`cargo check` green in every hot gate; the flake's trunk apps landed 2026-11-19 (V43 =
the NixOS pass of `nix run .#web-serve` / `.#web-build` / `.#web-build-release`); the manual trunk
lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569 (the
owner-approved 2026-11-19 palette split brought `app/windows.rs` back under the guard:
571 → 351 + the child `windows/palette_window.rs` 287); the tool_icons tests topic-split
(Tests rule 7) kept every test file under 300.

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
> ever pinned in place by a pending check. Compaction history (which entry moved
> under which round's budget, 2026-11-18 → 2026-11-20) lives in
> PROGRESS-archive.md; the latest move: the save-dialog-layout entry (during
> the 2026-11-20 outline-placement round's budget) to PROGRESS-archive.md.

### 2026-11-20 — `Feat:` the Shape tool's outline placement radio (outside / centered / inside) — owner request

- **What:** the owner approved V52 and ordered a THIRD Shape-panel group: three
  outline placements over the drag's two mouse points — OUTSIDE (the outline
  fully beyond them: its inner edge = the shape's boundary), CENTERED (the
  classic straddle — the old look, the startup default) and INSIDE (fully
  within them: its outer edge = the boundary). New `tools.rs::ShapeOutline` +
  `ToolState.shape_outline` (default Center); `rasterize_shape` strokes a path
  SHIFTED half a stroke width outward/inward (the pure `outline_stroke_bounds`,
  the inward shift clamped for thinner-than-stroke shapes), the dirty rect
  covers the Outside reach, the fill always stays exactly the two points; the
  gesture CARRIES the placement like its colors (a mid-drag switch cannot
  change the release); the preview matches every mode (the rect: egui's
  Outside/Middle/Inside stroke kind; the ellipse: radii shifted by the same
  half screen-stroke width). The panel's THIRD row (after another group gap):
  three icon radios whose icons ALWAYS draw a SQUARE's corner — the gray fill
  quadrant (where the shape is), the click dot (white core + ink ring) and two
  ink segments along the outline's lines in the mode's position
  (`draw.rs::draw_outline_mode`); tooltips "Outline outside" / "Outline
  centered" / "Outline inside". The Shape-panel icon set is EIGHT now.
- **Files:** `tools.rs`, `app/preview.rs`, `app/windows.rs`,
  `app/tool_icons.rs` (8 icons), `app/tool_icons/draw.rs` (+draw_outline_mode);
  tests: tools/tests.rs (+5), tool_icons/{tests,shape_panel,atlas}.rs,
  windows/tests/settings_panel.rs (+1 + bounds); SPEC §3.2, TIMING (+2 rows),
  DEBTS-verification (V52 closed, V53 born), PROGRESS (stamp + entry + the
  save-dialog-layout entry and the 2026-11-19 snapshot extras moved to the
  archive).
- **Verified:** FULL gate ALL GATES GREEN 18 s hot: fmt clean, rules HARD
  green, clippy `-D warnings` green (default + mcp), `cargo test --profile
  gates` 239 green BOTH lanes (+1 ignored; +5 net tests); NO wasm check (the
  diff touches no web-facing code — the economy rule); the visual driver ALL
  SCENARIOS GREEN under Xvfb; PLUS the GUI probe through the real binary
  (xdotool + ffmpeg): the three-group panel, all three tooltips, three
  rectangles one per mode — PIXEL-MEASURED on the screenshots: Inside's outer
  edge == the drag bounds (350..550), Outside's band = the bounds ± 8
  (642..858), Center straddles ± 4 (346..554); the 256 px + 24 px icon sheets.
- **Owner-checks:** V53 born (the Windows + Web pass: the third row's three
  corner radios — always a square's corner, the gray fill, the click dot, the
  bands; the startup default Centered; a shape drawn in each mode — Outside
  grows the footprint, Inside stays within the drag; the kind switch must not
  affect the row). Open: V53, V46; parked: V30, V8. Debts open: F17, F16, F15.
- **Commit title:** `Shape tool: the outline placement becomes a second three-way radio (outside the two points / straddling them as before / inside them), the corner icons show the click dot and the outline bands against the gray fill, the panel's groups separated by distance`

### 2026-11-20 — `Feat:` the Shape panel's MODE radio over the tool's icon — owner request

- **What:** the owner verified V51 and asked to make the MODE row better: the
  two toggles should NOT repeat the selected shape — they should use the
  Shape TOOL's icon — and become THREE choices, "a radio". Done exactly so: a
  new `tools.rs::ShapePaint` (Fill / Outline / FillOutline — the former
  `shape_fill`/`shape_outline` bools retired; the hidden "both on" state a
  second click used to reach is now the explicit third radio choice), the
  panel's MODE row is three `icon_button` radios over it, and the three MODE
  icons draw the TOOL EMBLEM (`draw.rs::EMBLEM_RECT` + `emblem_circle()`, one
  geometry with `draw_shape`): solid ink (Fill), rim only (Outline;
  pixel-for-pixel the tool's button, test-asserted), the gray
  `SHAPE_PREVIEW_FILL` under the rim (Fill+Outline). Tooltips carry the old
  words + "Fill + Outline"; the icons are kind-INDEPENDENT again (the atlas
  key is pixels-per-point ALONE — `ensure` lost the kind argument); the
  wells/commit logic needed NO change (`rasterize_shape` always supported
  fill+stroke; the color rules take the radio's `fills()`/`outlines()`).
- **Files:** `src/tools.rs` (ShapePaint + ToolState.shape_paint),
  `src/app/tool_icons.rs` (ShapeIcon::FillOutline, 5 icons, the f32 key),
  `src/app/tool_icons/draw.rs` (the emblem consts + draw_paint_mode),
  `src/app/windows.rs` (the radio row; the toggled_* helpers retired),
  `src/app/preview.rs` + `colors.rs` (the call site + docs), `src/app.rs`
  (the ensure call), tests: tool_icons/{tests,atlas,shape_panel}.rs +
  windows/tests/settings_panel.rs + tools/tests.rs + preview/tests.rs
  (+4 net), SPEC §3.2, AGENTS (the module map + the texture-count quirk),
  TIMING (4 rows), DEBTS-verification (V51 + V48 closed, V52 born),
  PROGRESS (snapshot + entry + stamp; the slider-captions entry moved to
  PROGRESS-archive.md).
- **Verified:** FULL gate ALL GATES GREEN 12 s hot: fmt clean, rules HARD
  green (the OVER-GUARD note is the owner-deferred history_tree.rs 569),
  clippy `-D warnings` green (default + mcp), `cargo test --profile gates`
  218 default / 231 mcp green (+1 ignored each; +4 tests); NO wasm check
  (the diff touches no web-facing code — the economy rule); the visual
  driver ALL SCENARIOS GREEN under Xvfb; PLUS the GUI probe screenshots:
  the three-radio panel, the "Fill + Outline" tooltip, a Fill+Outline
  rectangle (red fill + black rim) drawn through the real binary, and the
  256 px design sheet of all five icons.
- **Owner-checks:** V52 born (the Windows + Web pass: the three radios on
  the tool's icon, exactly one active, the tooltips, a Fill+Outline shape
  painting both wells; the kind switch must NOT touch the MODE row). V51
  closed verified in chat ("Проверил V51") and V48 too ("V48 проверил").
  Open: V52, V46; parked: V30, V8. Debts open: F17, F16, F15.
- **Commit title:** `Shape panel: the Fill/Outline toggles become a three-way radio over the Shape tool's icon (solid Fill, hollow Outline, gray-under-the-rim Fill+Outline, the old words as tooltips, the atlas kind-independent again)`

### 2026-11-20 — `Feat:` the Shape panel's icon buttons — owner request

- **What:** the owner reviewed V50 ("сейчас гораздо лучше") and reshaped the Shape
  tool's settings panel: the word radio and the Fill/Outline checkboxes became ICON
  BUTTONS in the Tools-window button style. The KIND row = two buttons — square /
  circle drawn as neutral previews (mid-gray fill + the fixed ink rim) selecting
  Rectangle / Ellipse; after a group gap (`SHAPE_GROUP_GAP`, the
  distance-separates-meaning principle) the MODE row = two toggles — Fill (the
  current kind SOLID in the fixed ink) and Outline (HOLLOW) — whose icons preview
  the CURRENT kind: the atlas key is (pixels-per-point, kind) and a kind switch
  re-rasterizes (egui's `load_texture_cached` dedupes identical content — only the
  two toggles' pixels re-upload). Hovering shows EXACTLY the old words; the
  at-least-one-mode invariant lives in pure `toggled_fill`/`toggled_outline`; the
  shared `icon_button` fn now draws the Tools grid's buttons AND the panel's.
- **Files:** `src/app/tool_icons.rs` (ShapeIcon + shape_icon_label +
  shape_texture_id; the Atlas key + `shape_icons`), `src/app/tool_icons/draw.rs`
  (rasterize_shape_icon + the two drawings + SHAPE_PREVIEW_FILL),
  `src/app/windows.rs` (icon_button extracted; shape_settings_ui + the toggles),
  `src/app.rs` (the ensure call passes the kind), `src/tools.rs` (ShapeKind +Debug),
  `src/app/tool_icons/tests.rs` split by topic per Tests rule 7 (94 + drawings 199
  + atlas 244 + shape_panel 104; +5 tests), `src/app/windows/tests/settings_panel.rs`
  (+2), SPEC §3.2 (the panel bullets), TIMING (2 rows), DEBTS-verification (V50
  closed verified, V51 born), PROGRESS (snapshot + entry + stamp).
- **Verified:** FULL gate ALL GATES GREEN 16 s hot: fmt clean, rules HARD green
  (the OVER-GUARD note is the owner-deferred history_tree.rs 569 again), clippy
  `-D warnings` green (default + mcp), `cargo test --profile gates` 227 / 227 green
  (+1 ignored each; +5 tests); NO wasm check (the diff touches no web-facing code —
  the economy rule); the visual driver ALL SCENARIOS GREEN under Xvfb + the GUI
  click-through with screenshots: the rectangle state, the Ellipse click turning
  the toggles round, the "Rectangle" tooltip, an ellipse drawn on canvas.
- **Owner-checks:** V51 born (the Windows + Web pass: the four buttons, the
  tooltips, the kind switch re-rounding the toggles, drawing from the panel state).
  V50 closed verified in chat (the owner: "Проверил как выглядит окошко Save,
  сейчас гораздо лучше"). Open: V51, V48, V46; parked: V30, V8. Debts open: F17,
  F16, F15.
- **Commit title:** `Shape panel: the Kind radio and the Fill/Outline checkboxes become Tools-style icon buttons (gray square/circle previews, solid/hollow toggles previewing the current kind, the old words as tooltips)`
