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
| Extra: the Outline-placement radios' icon redraw (owner 2026-11-21) | done — gates + driver green, the icon dump looked at | the Shape panel's three OUTLINE-PLACEMENT icons redrawn per the owner's recipe: the dot untouched ("точечку оставить как есть"), the horizontal/vertical segments ~2.5× thicker (`OUTLINE_BAND_W` 2.6 → 6.5) and each segment's FAR end running past the icon's 24-pt edge (`OUTLINE_BAND_LEN` 11 → 17, the bitmap crop = the outline continues beyond the corner window); the modes still differ only by where the bands sit (outside / straddling / inside); the placement test's probe strips/box recomputed for the new band edges (the Outside-only strip left of Center's 6.75, the Inside-only right of 13.25, the straddle box clear of Outside's ≤10); verified visually on the icon dump at 256 px AND the real 24 px button size (all three distinct, the dot legible on every background); V65 born |
| Extra: the selection CLIPS every painting tool (owner 2026-11-21) | done — gates + driver + new probe green + wasm lane green | with an active selection every canvas-changing tool paints ONLY inside it, like the classic paint apps: the mask snapshots at each gesture's start (`App::selection_clip` — freehand keeps `stroke_clip`, the Line/Shape drags carry it in `PreviewState`, the Text builds at commit; the MCP pencil stroke clips the same way), the pencil filters its plots against the mask, the thick strokes rasterize onto a transparent bbox scratch through an integer translation (`tools.rs::paint_clipped`) and transfer via `selection.rs::stamp_masked` / `erase_masked_by` (the eraser marks its footprint; the scratch's alpha IS the erase strength — AA edges fade like the direct path within ±1 rounding), the text rasterizer skips masked glyph pixels; outside pixels stay byte-identical, the dirty rect shrinks to region ∩ mask bounds, no selection = the unchanged direct path; the live vector previews clip to the selection's cached bounds; +13 tests (`tools/tests/clip.rs` ×5 — the full-mask pixel-identity, the outside-untouched trio, the shape; `selection/tests.rs` ×3 — the transfer primitives; text ×1, app preview ×2, input ×2); the new probe `scripts/gui_selection_clip.sh` PROBE PASS (a rect selection + the `b` hotkey + two red strokes on one self-located canvas row: the crossing stroke's red STOPS at the edge x=728/edge 730, the drag endpoint 850 px); SANDBOX.md pitfall 12 (the git-checkout test-loss incident — restored from the delivery zip); V66 born |

| Extra: the five over-guard files split (owner 2026-11-21) | done — gates green, pure code motion | the owner ordered the splits for the five files past the ~500-line guard: tools.rs → tools/shapes.rs (241), tool_icons.rs → tool_icons/primitives.rs (180), tool_icons/draw.rs → draw/tools.rs (350) + draw/panels.rs (315), app/windows.rs → windows/settings_panel.rs (~460), app/preview.rs → preview/geometry.rs (63) + commit.rs (123) + overlay.rs (191) — every item moved verbatim by scripts/split_russet.py, roots re-export what bin code consumes, tests import from the owning module; selection.rs + app/selection.rs deferred by the owner, history_tree.rs stays owner-deferred; the post-split sizes: selection.rs 890 and app/selection.rs 647 are the ONLY remaining over-guard files (+ tools/tests.rs 503, report-only) |

| Extra: the native dialogs stop freezing the window — F21, variant A (owner pick 2026-11-21) | done — gates + wasm check + driver green | File ▸ Open (with its decode) and the Save dialog's folder pick (Browse… AND the Save-click fallback) run on a WORKER THREAD — the portal round-trip + `pollster::block_on` left the UI thread, GNOME no longer dims the window ("Not Responding" gone); ONE shared `file_io::FileTask` slot (both platforms, the web `AsyncFileDialog` rides it too) polled per frame (`document::poll_file_task`); the single-dialog guard (Open…/Browse… gray, no second chooser stacks); the Save-click fallback continues by itself when its pick answers (`save_after_folder_pick`); no task in flight = zero machinery; +4 tests; V67 born |
| Extra: file_io.rs split under the ~500-line guard (owner 2026-11-21) | done — gates + wasm green, pure code motion | the pixel/byte codecs (`pixmap_from_image`, `png_compression_type`, the two PNG encoders, the JPEG encoder) + the two encoder defaults moved VERBATIM into the new `src/file_io/codecs.rs` (the F21 round's approved proposal); the root keeps the task/types/dialog glue and re-exports what bin code consumes (the MCP `encode_png` gated by the feature like `save_image_to`); the tests import from the owning module; file_io.rs 606 → 495, codecs.rs 134; +1 stale comment fixed (`FOLDER_PICKING_SUPPORTED`'s doc claimed the web stub "returns `None`") |
| Extra: the History Tree rows go wordless for pencil, digitless for all (owner 2026-11-21) | done — gates + wasm + driver + new probe green, the icon dump and the tree screenshots looked at | the tree rows drop the node id EVERYWHERE ("pencil 7" → "pencil") and the PENCIL row draws the TOOL'S ICON — the same emblem the Tools button shows, from a SECOND, row-sized atlas (`tool_icons::TREE_ICON_PT` 12 pt, "по размеру заглавной буквы или чуть больше", the same `ensure` pass/key — a native small raster, not a downscaled 24 pt texture; the headless texture count 25 → 33); new tools join one row at a time; the digit removal EXPOSED a doubled kind word on crop rows ("crop crop …" — `CropInfo::summary` opens with "crop" itself, the id used to separate them) — the row now prints the summary alone; +4 tests (3 paint row-label/icon helpers in the new `paint/tests.rs`, 1 atlas row-size set); the new probe `scripts/gui_tree_icon_check.sh` PROBE PASS (two pencil drags: the rails' blue + the icon's warm-gold pixels on the screen's right side); V68 born |
| Extra: the `--atlas-test` diagnostic — a 1 px speckle edge on every atlas picture (owner 2026-11-21) | done — gates + wasm + driver green, the flagged screenshot looked at | the owner sees a gap between the icon pictures' edges and their emblems and wants to SEE it before deciding: the new boolean flag `--atlas-test` (§3.12, no value; `apply_to` → `App::atlas_test` → `ensure(…, debug_border)`) repaints every raster's outermost pixel ring with a STATIC black-and-white speckle (`tool_icons::paint_debug_border` — a per-pixel coordinate hash, replayed byte-identically on every rebuild, so no animation) — exactly ONE PHYSICAL pixel thick at any scale (the ring lives in the raster: ppp and the icon size cannot stretch it), applied to all five icon sets AND the pipette cursor's source raster, painted OVER the emblem on purpose; +1 test (the ring is opaque black/white only, BOTH colors present, the interior byte-identical, the repaint deterministic); the new shot script `scripts/gui_atlas_test_shot.sh` PASS (364 adjacent pure-black/pure-white pairs on screen); the measured insets it exists to expose: the 24 pt emblems leave 2–7 px of empty edge per side (the pencil 4/1/1/4 — the diagonal drawing), the 12 pt tree icons ≤3 px; V69 born |
| Extra: the History Tree row icons grow 12 → 16 pt, the row grows to fit (owner 2026-11-21) | done — gates + wasm + driver + probe green, the flagged before/after screenshots looked at | the rings made it visible: the first 12 pt row icons read «слишком маленькие — там ещё довольно много свободного места». The measured reality (ppp 1): row 15 px, backdrop 13, icon 12 — a mere +1 pt would fit WITHOUT growing the row, so the row grows too: `TREE_ICON_PT` 12 → 16 and `row_height = max(text line, TREE_ICON_PT + 4)` (tree_window.rs) — one height for every row (~15 → 20 pt; pitch 18 → 23 px measured), so the emblem becomes the row's tallest element and the backdrop/swatch/hit/scroll math all follow the one value; the probe's warm-gold count 18 → 42 (2.3× the emblem pixels on screen); the 16 px dump's insets: pencil 2/1/1/2, Text 2/4/2/3 (~80% filled); the dump test's third size now rides `TREE_ICON_PT` (it was pinned 12); the tree probe passes extra launch args through (`… russet --atlas-test` — how this round's evidence was shot); +0 new tests (the row-size test and the counts ride the constant); docs: SPEC §3.6 + §3.12 example, AGENTS tool_icons clause, paint.rs/tool_icons.rs docs, the probe's header; V70 born |

**Verification stamp:** FULL gate re-verified 2026-11-23 (the
wet-stroke round): fmt clean, rules.sh HARD-green, clippy `-D warnings`
green on --all-targets (default + mcp), `cargo test --profile gates` 311
default / 324 mcp green (+1 ignored), ALL GATES GREEN in 23 s; no wasm
check per the economy rule (no web-facing paths touched); the visual
driver exit 0; the new wet-joint GUI probe PASS.

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
> PROGRESS-archive.md; the latest move: the `--atlas-test` Feat entry
> (during the brush/eraser round's budget — the file rode past ~200 lines;
> three entries stay in the Log, all in the archive).

### 2026-11-23 — `Fix:` translucent brush strokes no longer darken at the joints — brush/eraser paint through a wet stroke layer (owner bug report in chat)

- **What:** the owner's report: with a semi-transparent brush color the
  joint circles turn less transparent («полупрозрачный цвет накладывает
  отрезок на отрезок и в стыках отрезков кружочек становится менее
  прозрачным»). Root cause: each frame stroked its smooth pieces onto the
  layer directly — the next frame's start cap (and the press dot) sat on
  the previous frame's end cap, and premultiplied source-over is not
  idempotent, so the alpha stacked at every overlap (measured: 61/255 per
  channel darker at the joint). Fix: brush/eraser gestures paint through
  the WET STROKE LAYER `tools/wet.rs::StrokeWet` — the whole gesture's
  coverage accumulates in a canvas-sized full-alpha scratch (opaque
  source-over = a coverage union), and the layer is recomposited per
  changed region from the gesture-start base with the stroke's alpha
  applied ONCE (the eraser destination-outs by the coverage once — no
  more over-erase at self-overlaps either); idempotent, history/undo
  untouched, the selection clip rides the composite. The tool + color
  freeze at stroke start (one gesture = one node = one tool; a mid-drag
  hotkey switch no longer re-rasterizes with the new tool). Pencil
  unchanged (its trail dedupe stays the translucency guard). SPEC §3.5
  rewritten for all of it.
- **Files:** `src/tools/wet.rs` (+`wet/tests.rs`, 5 tests), `src/tools.rs`,
  `src/app.rs` (`stroke_wet`/`stroke_tool`), `src/app/input.rs` (the wet
  paths, the freezes, the resets), `src/selection.rs`
  (`tight_bounds` → `pub(crate)`), `src/tools/smooth.rs` (docs),
  `scripts/gui_wet_joint_check.sh` (new probe).
- **Verified:** FULL gate ALL GATES GREEN (the stamp — 311 default / 324
  mcp, +6 tests; the app-level regression ran RED on the old path first —
  joint (71,78,86) vs body (132,137,142) — then GREEN); the visual driver
  exit 0; the wet-joint probe PASS (a translucent-red drag: 276 row
  pixels ALL one color, deviation 0); no wasm check (no web-facing paths).
- **Owner-checks:** V72 born (Windows `cargo run`: a translucent brush
  stroke is uniform — joints and the press dot match the body; the eraser
  as before; one node per gesture; a selection still confines). Open:
  V72, V71, V70, V69 (the 4 newest); parked: V30, V8. Debts: F20, F18,
  F17, F16 newest-first in the files.
- **Commit title:** `Fix: translucent brush strokes no longer darken at the joints - brush/eraser paint through a wet stroke layer (StrokeWet)`

### 2026-11-21 — `Feat:` the History Tree's brush and eraser rows draw their icons like the pencil — the eraser row drops its color chip and the icon rows pull the chip closer (owner request in chat)

- **What:** three follow-ups on the tree's row format. (1) The icon-row
  set grows to the three PAINTING tools: `row_icon` maps Brush and
  Eraser too («как pencil давай в дереве истории сделаем и brush и
  eraser») — the tree atlas already rasters one icon per `Tool`, so no
  pipeline change. (2) The eraser row paints NO color chip — it erases,
  it does not paint («у eraser почему-то есть цвет в дереве истории, так
  не должно быть»): the swatch condition moved into a pure
  `row_swatch` (Start, resize and now Eraser are chipless; the painting
  tools keep theirs), and chipless rows stop reserving chip room in the
  content width. (3) Icon rows pull the chip in to `ICON_CHIP_GAP` = 6 pt
  after the emblem («чтобы между иконкой и цветом … стало чуть меньше
  места» — word rows keep `LABEL_GAP` 10). Measured on the probe shot:
  the gap 10 → 6 px, the eraser row chipless, the pitch still 17 px.
- **Files:** `src/app/tree_window/paint.rs` (`row_icon` + the new
  `row_swatch` + `ICON_CHIP_GAP`, the content-width reservation, the
  swatch block, the doc comments), `src/app/tree_window.rs` (the module
  doc), `src/app/tree_window/paint/tests.rs` (the icon-set test renamed
  and extended + the new `the_eraser_row_paints_no_swatch`), the probe
  script (brush + eraser strokes via the B/E hotkeys, the eraser stroke
  crossing the first pencil line, the pink-emblem assertion, the header).
- **Verified:** FULL gate ALL GATES GREEN (the stamp — 305 default / 318
  mcp, +1 test: the chip policy is a code-level fact; clippy `-D
  warnings` clean); the wasm `cargo check` green; the visual driver
  exit 0; the tree probe PASS (gold 40 = pencil+brush emblems, pink 9 =
  the eraser's pink body — a hue nothing else in the scan region has);
  the flagged screenshot LOOKED AT (brush row: icon + chip in tight; the
  eraser row: the pink emblem and NO chip; the chip border measured at
  exactly 6 columns after the icon picture in all three chip rows).
- **Owner-checks:** V71 born (brush/eraser icons, the 6 pt chip gap, the
  chipless eraser row). Open: V71, V70, V69, V68, V67, V66 (X11);
  parked: V30, V8. Debts: F20, F18, F17, F16 newest-first in the files.
- **Commit title:** `Feat: the History Tree's brush and eraser rows draw their icons like the pencil - the eraser row drops its color chip and the icon rows pull the chip closer`

### 2026-11-21 — `Fix:` the History Tree rows sit back-to-back — the icon gap is exactly 0.0625rem; the pencil emblem nudges left-down (owner request in chat)

- **What:** two refinements on top of the size pass. (1) The vertical gap
  between two consecutive row icons is EXACTLY 0.0625rem (= 1 pt; the
  repo's rem convention, `ICON_PT`/`FIRST_OPEN_GAP`): the rows now sit
  BACK-TO-BACK — `row_height = max(text line, TREE_ICON_PT +
  TREE_ICON_GAP)` in `tree_window.rs` (new `TREE_ICON_GAP` const) and
  `paint.rs` cancels egui's default inter-row spacing after every row
  allocation (`ui.add_space(-item_spacing.y)`) — the row height IS the
  pitch, measured 23 → 17 px at ppp 1. (2) The pencil emblem moves a
  touch «влево-вниз» relative to the icon's center: `PENCIL_NUDGE`
  (-1, +1 design pt of the 24 pt emblem space) applied AFTER
  `icon_transform` through the new `primitives.rs::nudged_icon_transform`
  (the shift lands in the transform's translation column) — the direction
  is screen-FIXED (the -45° rotation cannot turn it) and scales with the
  raster; the dump confirms: 24 px insets 4/1/1/4 → 3/2/2/3.
- **Files:** `src/app/tree_window.rs` (the `TREE_ICON_GAP` const + the
  row-height rule), `src/app/tree_window/paint.rs` (the spacing cancel),
  `src/app/tool_icons/draw.rs` (`PENCIL_NUDGE`),
  `src/app/tool_icons/primitives.rs` (`nudged_icon_transform`),
  `src/app/tool_icons/draw/tools.rs` (the pencil dispatcher), the
  tool_icons re-export; SPEC §3.6, AGENTS.md (the tool_icons clause),
  the `TREE_ICON_PT` doc.
- **Verified:** FULL gate ALL GATES GREEN (the stamp — 304 default / 317
  mcp, clippy `-D warnings` clean, no new tests: the geometry rides the
  consts and the probe); the wasm `cargo check` green; the visual driver
  exit 0; the tree probe PASS with `--atlas-test`; the flagged screenshot
  LOOKED AT (the two rings 1 px apart, nothing clipped, the pencil
  off-center as ordered); the measured pitch 17 px and the 24/16 px dump
  insets confirm both changes.
- **Owner-checks:** V70 AMENDED (the refinements fold into the same
  open check). Open: V70, V69, V68, V67, V66, V46 (X11); parked: V30,
  V8. Debts: F20, F18, F17, F16 newest-first in the files.
- **Commit title:** `Fix: the History Tree rows sit back-to-back - the icon gap is exactly 0.0625rem; the pencil emblem nudges a touch left-down`
