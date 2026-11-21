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
| Extra: the docs-only gate exemption into AGENTS.md (owner 2026-11-20) | done — docs only | a `.md`-only diff runs `scripts/rules.sh` + the delivery byte-compares ONLY — no builds, no tests; the full gate stays for anything beyond `.md` (Rust, scripts, Cargo, flake, CI, web); comment-only `.rs` edits keep the `cargo check` quick loop |
| Extra: the web save downloads directly — rfd's fallback window gone (owner 2026-11-20) | done — gates + wasm check + driver green | the web Save dialog's confirm no longer routes through `rfd::AsyncFileDialog::save_file()` (its wasm backend pops the "click here to download your file" Ok/Cancel window): `file_io.rs::web::download_image` encodes → Blob (the format's MIME type, new `SaveFormat::mime_type`) → object URL → one synthetic click on a detached `<a download>` with the composed name → the URL revoked on a 10 s grace timer; js-sys/wasm-bindgen ride wasm-bindgen-futures' public re-exports (zero manifest additions; the web-sys line gains 8 feature gates); rfd stays only on the Open path; V47 born |
| Extra: the Save-dialog slider captions — the numbers explain themselves (owner 2026-11-20) | done — gates + wasm check + driver green | under EACH encoding knob a live caption names the CURRENT number's effect (the owner: the digits "were not obvious even to me"): PNG "<level> — stored raw / light packing / balanced packing / tight packing …" + the tail "The picture is identical at every level — the slider only trades file size against save speed."; JPEG "<quality> — near-perfect picture / high / medium / low quality …" + the tail "Lower = a smaller file, but the picture loses more detail."; pure bucket helpers exhaustively tested (every stop 0..=9 / 1..=100 names its own number), the two headless dialog render tests assert the captions; V49 born, closed verified in chat the same day |
| Extra: the Save-dialog layout pass — preview above the name, the dropdown, the distance-separated encoding group (owner 2026-11-20) | done — gates + wasm check + driver + dialog screenshots green | the owner reviewed the V49 captions and reshaped the dialog: "Saving as" ABOVE the File name field (the result reads first), the Folder row (native) DIRECTLY under it — the "what + where" cluster; the Format radios → a `egui::ComboBox` dropdown ("PNG image" / "JPEG image" — the knob and the preview re-compose on the pick); the encoding group [Lossless ABOVE the knob + the captions] separated by a 12 pt `GROUP_GAP` (vs the 3 pt row gap) so related controls read as one block; +2 layout-order tests over the REAL dialog; no encoding change — pixels/bytes as V48 saved; V50 born |
| Extra: the Save-dialog compression slider + the Lossless row (owner 2026-11-20) | done — gates + wasm check + driver green | the PNG row gains the "Compression" slider 0..=9 — the encoder's REAL stops (`CompressionType::Level`: 0 stores raw, 1..=9 the exact deflate effort; default 6 = flate2's balanced middle; file size + encode time trade, pixels NEVER, the round-trip test proves it) — and both formats gain the owner's "Lossless" row: a per-format LOCKED DISPLAY of the guarantee (PNG on — lossless by design; JPEG off — no lossless mode the pinned encoder could write nor browsers decode); JPEG keeps Quality; the dialog's choices ride a new `SaveOptions` through the native save AND the web download; V48 born |
| Extra: the Shape panel's icon buttons (owner 2026-11-20) | done — gates + driver + GUI probes green | the owner reviewed the V50 Save-dialog layout ("сейчас гораздо лучше") and reshaped the Shape tool's settings panel: the word radio and the Fill/Outline checkboxes became Tools-window-style ICON buttons — the KIND row (gray square / circle previews; tooltips carry the old words) and the MODE row after a group gap (Fill = the current kind SOLID, Outline = HOLLOW — the toggles re-render for the kind switch); the shared `icon_button` now draws both windows' buttons; the toggle invariant extracted into pure `toggled_fill`/`toggled_outline`; +7 tests (5 icons + 2 panel) over the split topics; V50 closed verified in chat, V51 born |
| Extra: the Shape panel's MODE radio over the tool's icon (owner 2026-11-20) | done — gates + driver + GUI probes green | the owner verified V51 and reshaped the MODE row: the two toggles (whose "both on" state hid behind a second click) became a THREE-way radio `ShapePaint` — Fill / Outline / the explicit Fill+Outline — whose three icons draw the Shape TOOL's emblem (solid / hollow / gray-under-the-rim), no longer the selected kind (the atlas key is pixels-per-point ALONE again); the old words stay as tooltips, the third reads "Fill + Outline"; wells/commit logic untouched (`rasterize_shape` always supported both); +4 net tests; V51 + V48 closed verified in chat, V52 born |
| Extra: the Shape tool's outline placement radio (owner 2026-11-20) | done — gates + driver + GUI probes green | the second THREE-way radio `ShapeOutline` (Outside / Centered / Inside — the outline band shifted half a stroke width outward/on/inside the drag's boundary, the clamp for tiny shapes, the dirty rect reaching the full width beyond); the panel's THIRD group row of corner icons (square's corner + gray fill quadrant + click dot + the outline bands, kind-independent, tooltips carry the words); the preview matches all three modes; V52 closed verified in chat («Отлично»), V53 born |
| Extra: PROBLEMS.md (the write-only agent-friction log) + the Shape panel's spacing pass (owner 2026-11-20) | done — gates + driver + GUI pixel probe green | two owner requests in one session: (1) PROBLEMS.md born — every agent problem logged raw at the top, the file is NOT read until the owner says so (the AGENTS.md rule + abstract template + the first entry: the false warm-up verdict); (2) the Shape panel's buttons sit EXACTLY the Tools window's grid gap apart horizontally (`TOOL_GRID_GAP` 6 pt via `item_spacing.x`) and the rows a step further apart vertically (12 pt = 3 + `SHAPE_GROUP_GAP` 9); pixel-measured on the Xvfb screenshots: in-row 6 px == the Tools grid's own 6 px on the SAME shot, rows 12 px apart; V54 born |
| Extra: PROBLEMS.md v2 (the pure log written only by scripts/log_problem.sh) + the Line panel's icon-button pass (owner 2026-11-20) | done — gates + driver + GUI pixel probe green | two owner requests in one session: (1) PROBLEMS.md v2 — the file is LITERALLY a log (no header/template — they moved into the AGENTS.md section), entries go in ONLY through the new prepend-only `scripts/log_problem.sh` (byte-checks the old content survives; nothing deletable without the owner); (2) the owner verified V54 and reshaped the LINE panel the same way («кнопочки как в Shape, такие же по стилю и отступам, вместо радио Mode»): ONE row of TWO icon buttons — Straight / Curve — drawing the Line TOOL's emblem (Curve pixel-for-pixel the tool's icon, Straight the same stroke unbent), the row exactly the grid gap apart (the pixel probe: the same 7-px-on-screen gap as the Tools grid and every Shape row on the SAME binary); the Curve hints stay; V54 closed verified in chat, V55 born |
| Extra: the Size block — label above, the narrowed slider, manual entry above 100 (owner 2026-11-20) | done — gates + driver + GUI pixel probe green | the owner verified V55 («Отлично») with three follow-ups: the Shape AND Line panels (Brush/Eraser share the block, no gap — nothing below) get the "Size" label ABOVE the slider; the slider narrowed to the button grid's row width (`SIZE_SLIDER_W` 96 pt — egui's `Slider` takes its width ONLY from `spacing.slider_width` (`SliderCore::desired_size`; a container's max width cannot narrow it — the first `add_sized` attempt measured 100), so the block's scope pins it); right of the slider a click-to-type `DragValue` (1..=10 000 — a sanity guard, the owner set no ceiling) accepts sizes ABOVE 100 while the slider stays 1..=100, binding to a clamp COPY and writing back only when dragged (egui's default `SliderClamping::Always` clamps the bound value EVERY frame — a typed 250 would snap back; with the copy the thumb just pins at the right end); after the block a `SHAPE_GROUP_GAP` — the SAME 12-pt step as between the Shape panel's own rows («такой же высоты что и между кнопками в Shape»); the probe measures the real screen: rail 96 px, the number box right of it (w 40, gap 6), rail center 22.5 px above the buttons, the label above; +3 tests (the new `windows/tests/size_block.rs` topic); V55 closed verified in chat, V56 born |
| Extra: the hand-over lists only the newest commit title (owner 2026-11-20) | done — docs only | the owner traced the AGENTS.md delivery ritual's "list every round's title" clause to a ONE-TIME request the agent mis-recorded as a standing rule ("это была единичная просьба и агент решил записать это как правило") and had it fixed: the reply lists ONLY the newest delivery's title — one per reply (he records the older ones himself — "я и так их записываю"); the title mechanism (the PROGRESS entry's `Commit title:` field + the reply's last line, backticks, rules.sh rule 6) unchanged; docs-only round (rules.sh + byte-compares only) |
| Extra: the manual-entry size reaches the drawing — bug fix (owner 2026-11-20) | done — gates + driver + probe green | the owner's report ("задаю вручную size 200 всё ещё рисуется с радиусом 100") landed in the DRAW paths: all FOUR rasterizers (draw_segment, smooth, polyline, shape) still clamped the stroke width to the slider-era 1..=100, and the [ / ] hotkeys clamped tools.size itself — the UI accepted 200, every path drew 100; the ceiling is now `tools::MAX_STROKE_SIZE` = 10 000 (the entry box's own sanity bound, "not a design limit"), all clamps + the DragValue range use it, a typed 200 survives the bracket keys (→ 199); +5 pixel probes (`tools/tests/stroke_size.rs`) proving a 200-wide stroke covers what a 100-clamped one cannot; sizes ≤ 100 draw pixel-identical; V57 born |
| Extra: the Select tool — regions, copy/cut/paste/move, crop (owner 2026-11-20) | done — gates + driver + GUI probes green | ONE "Select (V)" button (the grid is 8 tools), the icon = the mode's mark (dashed square/blob), while active L-click cycles the mode forward / R-click backward (over `SelectMode::ALL` — ready for more kinds); the panel: the MODE row (Rect/Lasso) + the OPS row (Copy/Cut/Paste/Crop, grayed without a selection/clipboard; the Edit menu + Ctrl+C/X/V run the same ops); a plain drag replaces, Shift-drag adds, Alt-drag subtracts (rect AND lasso — the mask derives from the op list, so subtract cuts true holes); the ants = the crack-boundary loops of the mask (the true contour, holes, checkerboard corners) drawn under every tool; move = press inside the selection lifts the pixels (a full-layer backup) + the drag re-draws at the integer offset + ONE `move` node, Esc/pan/tool-switch restores; crop = `layers::crop_all` reusing the resize machinery's `fit_copy_at` (one `crop` node per layer, `CropInfo` in the tree rows) — the owner's "связать с resize"; the pure domain in `selection.rs`, the gestures/ops in `app/selection.rs`, the panel in `app/windows/select_panel.rs`; the clipboard is in-app and paste stamps immediately (F18); V58 born |
| Extra: the V58 fixes — the move ghost, the dead combos, the blot (owner 2026-11-20) | done — gates + driver + probes green | the owner's three findings fixed: (1) the move drag printed a GHOST every frame (the restore covered only the origin — the previous frame's stamp survived); now the restore covers origin ∪ last frame ∪ new position + the regression test (two redraws, the intermediate stamp vanishes); (2) Ctrl+C/X/V never fired — egui-winit's `clipboard_shortcuts` (ON by default) converts the combos into DEDICATED `Event::Copy/Cut/Paste` and swallows the Key events (issue #3653; matches PHYSICAL keys — non-Latin layouts work too); the handler consumes the dedicated events, the wants-keyboard guard keeps the Text-tool field its OS-text semantics + 2 tests (the events drive the ops; a bare Key event does not); (3) the lasso icon was circle-like — redrawn as an irregular round blot (6 anchors, varying radii, C1 midpoints, the same dashes); V59 born |
| Extra: the V59 fixes — clipboard_shortcuts OFF, the overlay move, the pan guard, the real blot (owner 2026-11-20) | done — gates + driver + probes green | (1) the egui-winit conversion is DISABLED (`NativeOptions::clipboard_shortcuts = false`, owner: "я не согласен с разработчиками") — Ctrl+C/X/V arrive as plain Key events (logical-or-physical `key`, so non-Latin layouts still work); the dedicated events stay consumed as the WEB fallback (the browser's DOM copy/cut/paste — not flag-controlled) with a one-op-per-frame twin guard + 2 tests; (2) Ctrl+S / Ctrl+Shift+S no longer PAN on S: an accelerator modifier (Ctrl/⌘/Alt) suppresses the W/A/S/D pan (Shift stays allowed) + a test; (3) the move drag is an OVERLAY ELEMENT: the layer is written exactly TWICE (the erase at the lift, the stamp at the release), the drag rides a Foreground painter (the floating texture + the ants, whole-viewport clip, visible past the canvas bounds) + 3 tests (no-write-until-release, the live texture, the past-the-bounds geometry); (4) the lasso icon = five lobes + five deep waists, picked from rendered candidates (the pear is gone) |
| Extra: the V60 fixes — the off-canvas move clip + the AccessKit audit (owner 2026-11-20) | done — gates + driver + probes green | (1) a move that lands PARTIALLY off-canvas now CLIPS the selection to the canvas (`Selection::clip_to_canvas`: rects intersect, lassos Sutherland–Hodgman-clipped, fully-outside regions drop, an empty result deselects) — the op list was the only unclamped thing (the views always clamped), so the next move used to heal the full shape over destroyed pixels + 5 tests (the domain clip ×3, the owner's back-to-the-center scenario, the fully-off deselect); (2) the AccessKit/trim audit measured and parked (F19: accesskit ≈310/761 dep edges ~40%, the one-line disable switch, what breaks = only screen readers; system_fonts ≈126 edges ~16%; clipboard/wayland/x11/default_fonts stay) — NOTHING changed per the owner's "пока думаем" |
| Extra: the TESTING diameter-circle Shift rule for the Shape tool's Ellipse kind (owner 2026-11-21) | done — gates + driver + GUI probe green | with the Ellipse kind a Shift drag makes the two drag points the endpoints of the circle's DIAMETER (center = the segment's midpoint, the pure `app/preview.rs::diameter_rect`) instead of the corner-anchored proportional circle; behind the Shape panel's FOURTH group — the plain "Diameter circle (Shift)" checkbox, startup ON, shown ONLY for the Ellipse kind (the rectangle's row does not exist — no dead controls); the flag rides the gesture refreshed live (a mid-drag Shift/checkbox flip re-shapes the next frame's preview); the rectangle's Shift rule and the plain Shift-less ellipse untouched; +6 tests; the new probe `scripts/gui_diameter_check.sh` PROBE PASS (ON: ring center = the drag midpoint, both points on the ring, the center unpainted; OFF: the old square; the checkbox appears only after the Ellipse click); SANDBOX.md pitfall 11 (raw-Xvfb key events need `xdotool windowfocus` — no WM → no input focus); V62 born |
| Extra: the diameter drag's measurement guides (owner picked idea #3, 2026-11-21) | done — gates + driver + GUI probe green | while a diameter gesture is LIVE the overlay draws WHAT IS MEASURED on top of the shape: a thin DASHED chord between the two drag points (the gesture's outline color, dashes 4 / gaps 3) and a small white dot with a dark ring at the segment's midpoint (the center; the polyline vertex-handle marker style — visible on any canvas color); the pure `app/preview.rs::diameter_guides`, drawn only while the flag rides (Line/Rectangle never guide) and GONE at release (the commit rasterizes just the ellipse); +2 tests (every dash endpoint lies on the measured segment, the dot = the midpoint, a zero-length drag draws no dashes); the probe gained MID-DRAG screenshots (chord dashes at t=0.25/0.75 + the dot's ring at the midpoint with the box ON; no chord red and no dot leak with the box OFF) — PROBE PASS; V63 born |
| Extra: the Tools window's button order pass (owner 2026-11-21) | done — gates green; the visual probe/driver skipped per the owner's waiver | the canonical `app/tool_icons.rs::TOOLS` array reordered by two swaps — the Eyedropper ↔ the Select and the Text ↔ the Line — so the two-column grid (row-major over the array) now reads Pencil+Brush, Eraser+Eyedropper, Select+Text, Shape+Line; hotkeys, tooltips, the atlas and the panels untouched (every consumer is order-agnostic — the atlas looks icons up by position, the grid chunks the array); pinned by the new `tool_icons/tests/order.rs` test; SPEC §3.8's Select row updated (its "right after the paint trio" phrase is now historical); V64 born |

**Verification stamp:** FULL gate re-verified 2026-11-21 (the
tools-order round): fmt clean, rules.sh HARD-green, clippy `-D warnings` green
(default + mcp), `cargo test --profile gates` 282 default / 295 mcp green
(+1 ignored; +1 net — the `tool_icons/tests/order.rs` layout-pinning test),
the visual probe and the MCP driver SKIPPED this round — the owner waived
them for the reorder ("простенькая задачка… визуально не проверять и mcp
не использовать"); the on-screen layout is pinned by the order test (the
grid fills row-major over `TOOLS`) — Rust 1.99.1 pinned via rust-toolchain.toml,
Linux. NO wasm check
this round (the diff touches no web-facing code — the economy rule). NOTE
2026-11-21: FRESH GitHub clone (the repo re-cloned from the owner's push) —
rustup + the pinned toolchain + the GUI .debs installed per SANDBOX.md; the
detached warm-up SURVIVED between tool calls in this sandbox (see TIMING).
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes: the wasm
`cargo check` green in every hot gate; the flake's trunk apps landed 2026-11-19 (V43 =
the NixOS pass of `nix run .#web-serve` / `.#web-build` / `.#web-build-release`); the manual trunk
lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only over-guard set — the owner-deferred
`history_tree.rs` PLUS the rounds' growth: `app/preview.rs` 641,
`app/tool_icons/draw.rs` 786, `app/tool_icons.rs` 602, `selection.rs` 752,
`tools.rs` 608, `app/selection.rs` 639, and NOW `app/windows.rs` 672 over
the guard (warn + propose splits at task end; no split mid-task — the
proposed splits: the settings-panel half → an `app/windows/settings_panel.rs`
child, the select icons → an `app/tool_icons/select.rs` child, both the
palette-split pattern); the
tool_icons + tools tests topic-splits (Tests rule 7) kept the topic files
small (tools/tests.rs itself rides at 476 — the root's shared helpers + the
topic mod decls).

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
> under which round's budget, 2026-11-18 → 2026-11-21) lives in
> PROGRESS-archive.md; the latest move: the V60-pass entry (during the
> tools-order round's budget — the file rode over the ~200-line budget; all
> in the archive).

### 2026-11-21 — `Fix:` the Tools window's button order — the Eyedropper ↔ Select, the Text ↔ Line (owner request in chat)

- **What:** the owner's reorder: the grid's row 2 becomes Eraser +
  Eyedropper (the color-picking pipette reads paint-adjacent), row 3
  Select + Text, row 4 Shape + Line. One array (`TOOLS`) drives the grid,
  the atlas and the icon lookup — the two swaps move the buttons without
  touching anything else; the hotkeys and the tooltips travel with their
  tools.
- **Files:** `src/app/tool_icons.rs` (the `TOOLS` array + its doc), the new
  `src/app/tool_icons/tests/order.rs` (+1 test pinning the layout), SPEC
  §3.8 (the Select row's placement sentence).
- **Verified:** FULL gate ALL GATES GREEN (see the stamp — 282 default /
  295 mcp). The visual probe and the MCP driver were SKIPPED — the owner
  waived them for this round ("простенькая задачка… визуально не
  проверять и mcp не использовать"); the on-screen layout is pinned by the
  order test (the grid fills row-major over `TOOLS`). No wasm check — no
  web-facing code (the economy rule).
- **Owner-checks:** V64 born (Windows + web: the grid reads
  Eraser+Eyedropper, Select+Text, Shape+Line; every hotkey still matches
  its button). Open: V64, V63, V62, V61; parked: V30, V8.
  Debts open: F19, F18, F17, F16.
- **Commit title:** `Fix: the Tools window's button order — the Eyedropper swapped with the Select, the Text with the Line (the TOOLS array drives the grid row-major; hotkeys and tooltips unchanged)`

### 2026-11-21 — `Feat:` the diameter drag previews WHAT IT MEASURES — a dashed chord and the center dot (owner picked idea #3 from the chat)

- **What:** while a diameter-circle gesture is LIVE (Ellipse kind, Shift
  held, the panel's checkbox ON), the overlay draws on top of the shape: a
  thin DASHED chord between the two drag points (the gesture's outline
  color — falling back to the fill for Fill-only gestures) and a small
  white dot with a dark ring at the segment's midpoint (the circle's
  center; the polyline vertex-handle marker style — visible on any canvas
  color). Overlay-only: the commit rasterizes just the ellipse and the
  guides die with the gesture; the flag rides the gesture (Line and the
  Rectangle kind never guide).
- **Files:** `src/app/preview.rs` (the pure `diameter_guides` + the live
  overlay call), `src/app/preview/tests.rs` (+2 tests), SPEC §3.2 (the
  guides sentence), the probe `scripts/gui_diameter_check.sh` (+ the
  MID-DRAG screenshots).
- **Verified:** FULL gate ALL GATES GREEN (see the stamp — 281 default /
  294 mcp); the visual driver ALL SCENARIOS GREEN; the probe PROBE PASS
  with the new mid-drag checks (chord dashes at t=0.25/0.75 of the drag
  segment and the dot's ring at the midpoint with the box ON; no chord red
  and no dot leak with the box OFF; the final-ring measurements
  unchanged). No wasm check — no web-facing code (the economy rule).
- **Owner-checks:** V63 born (Windows + web: the guides visible during a
  diameter drag, gone at release, absent when the checkbox is off or the
  kind is Rectangle/Line). Open: V63, V62, V61, V60; parked: V30, V8.
  Debts open: F19, F18, F17, F16.
- **Commit title:** `Feat: the diameter-circle Shift drag previews WHAT IT MEASURES — a thin dashed chord between the two drag points and a center dot at the segment's midpoint (overlay-only; the commit still rasterizes just the ellipse)`

### 2026-11-21 — `Feat:` the TESTING diameter-circle Shift rule for the Shape tool's Ellipse kind (owner request in chat)

- **What:** with the Ellipse kind selected, a Shift drag no longer pulls the
  free corner onto the proportional circle — the press and release points lie
  ON the circle and the segment between them is its DIAMETER (the center is
  the segment's midpoint). Behind a TESTING checkbox: the Shape panel's FOURTH
  group — the plain `Diameter circle (Shift)` checkbox (startup ON) — shown
  for the ELLIPSE kind only (the owner: nobody should have to read the toggle
  through the rectangle; no dead controls — the row does not exist for the
  Rectangle kind). The flag rides the gesture refreshed live (a mid-drag
  Shift/checkbox flip re-shapes the next frame's preview); the overlay preview
  and the commit rasterize the same bounding square (the pure
  `app/preview.rs::diameter_rect`). The rectangle's Shift rule and the plain
  Shift-less ellipse are untouched (the ellipse itself stays as it was).
- **Files:** `src/tools.rs` (the `shape_diameter_circle` state), `src/app
  /preview.rs` (the rule + the live flag + the commit/overlay geometry),
  `src/app/windows.rs` (the Ellipse-only checkbox row), `src/app/preview
  /tests.rs` (+5 tests), `src/app/windows/tests/settings_panel.rs` (+1 test),
  SPEC §3.2 + §3.9, the new probe `scripts/gui_diameter_check.sh`, SANDBOX.md
  pitfall 11 (raw-Xvfb key events need `xdotool windowfocus` — no WM → no
  input focus; found by this round's probe), PROBLEMS.md (the raw entry).
- **Verified:** FULL gate ALL GATES GREEN (see the stamp — 279 default / 292
  mcp); the visual driver ALL SCENARIOS GREEN under Xvfb through the gates
  binary; the diameter probe PROBE PASS (the checkbox appears only for the
  Ellipse kind; ON: ring center = the drag midpoint, both points on the ring,
  the center unpainted; OFF: the old corner-anchored square). No wasm check —
  no web-facing code (the economy rule).
- **Owner-checks:** V62 born (Windows + web: the checkbox shows only with the
  Ellipse kind; Shift-drag = the circle through the two points; unchecking
  restores the old corner-anchored circle; the rectangle keeps its Shift
  rule; when the verdict lands the checkbox goes away). Open: V62, V61, V60,
  V59; parked: V30, V8. Debts open: F19, F18, F17, F16.
- **Commit title:** `Feat: the Shape tool's Ellipse kind draws the Shift circle THROUGH the two drag points (the segment is its diameter) behind a testing Diameter-circle checkbox shown only for the Ellipse kind`

