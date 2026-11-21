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

**Verification stamp:** FULL gate re-verified 2026-11-20 (the V60-fixes
round): fmt clean, rules.sh HARD-green, clippy `-D warnings` green
(default + mcp), `cargo test --profile gates` 274 default / 287 mcp green
(+1 ignored; +33 net tests since b54a7fd — the Select-tool topics, the
V58/V59-fixes and this round's: the region clip (the rect shrink, the
fully-off None, the lasso Sutherland–Hodgman cut), the owner's
back-to-the-center scenario (the clipped shape rides along, the healed
corner is not selectable), the fully-off move deselecting, plus the
earlier rounds' topics), the visual driver ALL SCENARIOS GREEN under Xvfb
through the dev+mcp binary + the GUI probes PASS (the select probe: the
2/2/2/2 grid, the panel rows, the gaps; the blot verified on the cropped
screenshot) — Rust 1.99.1
pinned via rust-toolchain.toml, Linux. NO wasm check this round (the diff
touches no web-facing code — the economy rule). NOTE 2026-11-20 (still true):
RECYCLED sandbox — rustup/toolchain/GUI .debs reinstalled, the repo RE-CLONED
from the owner's GitHub push; heavy commands ran synchronously (see TIMING).
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes: the wasm
`cargo check` green in every hot gate; the flake's trunk apps landed 2026-11-19 (V43 =
the NixOS pass of `nix run .#web-serve` / `.#web-build` / `.#web-build-release`); the manual trunk
lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only over-guard set — the owner-deferred
`history_tree.rs` PLUS the rounds' growth: `app/preview.rs` 538,
`app/tool_icons/draw.rs` 761, `app/tool_icons.rs` 602, `selection.rs` 649,
`tools.rs` 598, `app/selection.rs` 558, and NOW `app/windows.rs` 652 over
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
> under which round's budget, 2026-11-18 → 2026-11-20) lives in
> PROGRESS-archive.md; the latest move: the Select-tool entry (during the
> V59 round's budget — the total passed ~200 after two V58-fixes cuts; all
> in the archive).

### 2026-11-20 — `Fix:` the V60 pass — the off-canvas move clips the selection to the canvas — owner bug report; AccessKit/trim audit recorded, nothing changed

- **What:** (1) the owner's V60 bug: dragging a selection PARTIALLY off the
  canvas and then moving it back toward the center RESTORED the full
  region shape over pixels that no longer exist. Root cause: the derived
  views (mask, bounds, ants) were always canvas-clamped, but the OP LIST
  was not — `contains` (the move hit test) and `translate` work on the
  ops, so the next move healed the full shape. Fix: the move commit now
  CLIPS the selection to the canvas (`Selection::clip_to_canvas` — rect
  regions intersect, lasso polygons run a Sutherland–Hodgman clip against
  the canvas rect, fully-outside regions drop, an empty result
  deselects) + 5 tests (the domain clip: the rect shrink, the fully-off
  None, the lasso cut with new crossing vertices; the app level: the
  owner's exact scenario — the clipped shape rides along AND the healed
  corner is not selectable — and a fully-off move deselecting while the
  move node still records). (2) The owner's AccessKit question answered
  READ-ONLY (measured: ≈310 of 761 normal dep edges, ~40% of the graph —
  the zbus/AT-SPI D-Bus bridge; the disable switch is a one-line feature
  removal; what breaks = only screen readers) and the whole trim-candidate
  audit recorded as DEBTS-functional F19 — NOTHING changed per the owner's
  "пока думаем".
- **Files:** `selection.rs` (+3 tests: `clipped_to`, the SH helpers,
  `clip_to_canvas`), `app/selection.rs` (the commit wiring), SPEC §3.13
  (the move row's clip rule), DEBTS-functional F19.
- **Verified:** FULL gate ALL GATES GREEN (see the stamp); the visual
  driver + the select probe PASS. No wasm check — no web-facing code (the
  economy rule).
- **Owner-checks:** V61 born (Windows + web: a move partially off-canvas
  shrinks the selection to what survived; the second move keeps the
  clipped shape; a fully-off move deselects). Open: V61, V60, V59, V58;
  parked: V30, V8. Debts open: F19 (the AccessKit/trim decision — the
  owner's), F18, F17, F16, F15.
- **Commit title:** `Fix: the move commit clips the selection to the canvas (rects intersect, lassos Sutherland–Hodgman-clipped) so a partially off-canvas drag no longer heals its old shape; the AccessKit/trim audit parked as F19`

### 2026-11-20 — `Fix:` the V59 pass — clipboard_shortcuts OFF, the overlay move drag, the pan-on-Ctrl+S guard, the real blot — owner bug report

- **What:** the owner's V59 findings: (1) the egui-winit `clipboard_shortcuts`
  conversion is DISABLED (`NativeOptions::clipboard_shortcuts = false`,
  owner: "я не согласен с разработчиками этой библиотеки") — Ctrl+C/X/V now
  arrive as plain Key events and `consume_key` sees them again; the dedicated
  `Event::Copy/Cut/Paste` stay consumed as the WEB fallback (the browser's
  copy/cut/paste DOM events are the only channel there) with a one-op guard
  (a web frame can carry the same gesture twice — 2 tests); (2) Ctrl+S /
  Ctrl+Shift+S also PANNED on S (the pan reads `key_down` — the pressed
  state — and the save's consume_key does not lift it): an accelerator
  modifier (Ctrl/⌘/Alt) now suppresses the W/A/S/D pan, Shift stays allowed
  (+ test); (3) the move drag REDESIGNED (owner: "отдельным элементом от
  холста... видно за границами холста... при отпускании"): the layer is
  written exactly TWICE — the erase at the lift, the stamp at the release —
  the drag rides a FOREGROUND overlay (the floating pixels as a GPU texture
  + the ants, whole-viewport clip, visible past the canvas bounds; the old
  per-frame restore/erase/stamp painted unclear artifacts) (+ 3 tests: the
  no-write-until-release, the live overlay texture, the past-the-bounds
  geometry); (4) the lasso icon is now a real blot: five lobes (9.3–7.2)
  interleaved with five deep waists (5.0–6.2), picked from rendered
  candidates (the second take read as an upside-down pear).
- **Files:** `main.rs`, `app/input.rs` (+ tests), `app/camera.rs` (+ tests),
  `app/selection.rs` (+ tests), `app/tool_icons/draw.rs`, SPEC §3.9 (the
  combo row + the pan row) and §3.13 (the move row).
- **Verified:** FULL gate ALL GATES GREEN (see the stamp); the visual driver
  + the select probe PASS; the blot verified on the cropped screenshot. No
  wasm check — no web-facing code (the economy rule).
- **Owner-checks:** V60 born (Windows + web: the combos fire ONCE and pan
  does not run on Ctrl+S; a clean overlay drag visible past the canvas; the
  blot reads as a blot). Open: V60, V59, V58, V57; parked: V30, V8.
  Debts open: F18, F17, F16, F15. NOTE: `app/selection.rs` (611) and
  `app/tool_icons/draw.rs` (798) grew past the 500-guard — the standing
  split proposal, at task end as the rules ask.
- **Commit title:** `Fix: clipboard_shortcuts off (the combos arrive as Key events), the move drag rides a foreground overlay (no canvas writes until release), Ctrl+S no longer pans, the lasso icon is a five-lobe blot`

### 2026-11-20 — `Fix:` the V58 pass — the move-drag ghost, the dead Ctrl+C/X/V combos, the lasso blot — owner bug report

- **What:** the owner's V58 findings: (1) dragging a selection PRINTED A
  GHOST every frame — `redraw_moved` restored only the ORIGIN rect, so the
  previous frame's stamp survived; the restore now covers origin ∪ last
  frame ∪ new position (the full-layer backup covers all three) + a
  regression test (two redraws — the intermediate stamp must vanish);
  (2) Ctrl+C/X/V did NOTHING — egui-winit's `clipboard_shortcuts` (ON by
  default) converts the combos into DEDICATED `Event::Copy/Cut/Paste` and
  swallows the Key events, so `consume_key` could never match (issue
  #3653; the conversion matches the PHYSICAL keys — the combos now also
  work on non-Latin layouts); the handler consumes the dedicated events
  (the wants-keyboard guard keeps a focused Text-tool field its OS-text
  semantics) + 2 headless tests (the events drive the ops; a bare Key
  event does NOT); (3) the lasso icon read as a CIRCLE — redrawn as an
  irregular round blot (six anchors with varying radii, C1-smoothed
  through the midpoints, the same dashes).
- **Files:** `app/input.rs` (+ tests), `app/selection.rs` (+ tests),
  `app/tool_icons/draw.rs`, SPEC §3.9 (the combo-mechanism row).
- **Verified:** FULL gate ALL GATES GREEN (see the stamp). No wasm check —
  no web-facing code (the economy rule).
- **Owner-checks:** V59 born (Windows + web: a clean drag-move, working
  combos, the blot mark). Open: V59, V58, V57, V56; parked: V30, V8.
  Debts open: F18, F17, F16, F15.
- **Commit title:** `Fix: the move drag leaves no per-frame ghosts (the restore covers the last stamp), Ctrl+C/X/V consume egui-winit's dedicated Copy/Cut/Paste events (consume_key never sees the combos), the lasso icon is an irregular blot`

