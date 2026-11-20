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

**Verification stamp:** FULL gate re-verified 2026-11-20 (the Size-block round):
fmt clean, rules.sh HARD-green, clippy `-D warnings` green (default + mcp),
`cargo test --profile gates` 236 default / 249 mcp green (+1 ignored; +3 net
tests this round — the NEW size_block topic: the label above + the 96-pt rail
+ the block width, the 12-pt rows-gap arithmetic for BOTH panels, the
above-100 persistence), the visual driver ALL SCENARIOS GREEN under Xvfb
through the dev+mcp binary + the GUI probe `scripts/gui_line_panel_check.sh`
PASS (now ALSO measuring the SIZE BLOCK on the real binary's screenshots: the
rail 96 px wide, the "Size" label above it, the click-to-type number box right
of it (w 40, gap 6), the rail center 22.5 px above the first button row; the
Line/Shape rows and gaps as before) — Rust 1.99.1
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
`history_tree.rs` 569 PLUS the rounds' growth: `app/preview.rs` 538,
`app/tool_icons/draw.rs` 616, `tools.rs` 538, and NOW `app/windows.rs` 557
over the guard (warn + propose splits at task end; no split mid-task — the
proposed split: the settings-panel half → an `app/windows/settings_panel.rs`
child, the palette-split pattern); the tool_icons tests
topic-split (Tests rule 7) kept every test file under 470.

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
> PROGRESS-archive.md; the latest move: the PROBLEMS.md docs entry + the
> panel-spacing entry (during the 2026-11-20 Line-panel round's budget; the
> MODE-radio entry went the round before — all in the archive).

### 2026-11-20 — `Feat:` the Size block — label above, narrowed slider, manual entry above 100 — owner request

- **What:** the owner verified V55 («Отлично») with three follow-ups the same
  message: (1) "в Shape и в Line сделать gap между слайдером size и кнопками,
  такой же высоты что и между кнопками в Shape"; (2) "элемент size слишком
  широкий, нужно саму надпись Size разместить над слайдером"; (3) "дать
  возможность пользователю вручную ввести Size больше 100, а сам слайдер пусть
  и дальше будет ограничен 100". Done: the shared Size block extracted as
  `size_block_ui` (the `shape_settings_ui`/`line_settings_ui` sibling): the
  label ABOVE, the slider pinned to `SIZE_SLIDER_W` = 3 × 28 + 2 × 6 = 96 pt
  via `spacing.slider_width` (egui's `Slider` reads its width from the spacing
  ALONE — the first `add_sized` attempt measured 100), a click-to-type
  `DragValue` (1..=10 000, a sanity guard — the owner set no ceiling) right of
  it; the slider binds to a CLAMP COPY and writes back only when dragged
  (egui's default `SliderClamping::Always` clamps the bound value EVERY frame
  — a typed 250 would snap back to 100; with the copy the thumb just pins);
  for Line/Shape the block ends with `SHAPE_GROUP_GAP` → the SAME 12-pt rows
  step; Brush/Eraser get the block without the gap.
- **Files:** `src/app/windows.rs` (the block + `SIZE_SLIDER_W`/`SIZE_ENTRY_MAX`
  + the caller), `src/app/windows/tests.rs` + NEW
  `src/app/windows/tests/size_block.rs` (3 tests: the label above + the 96-pt
  rail + the narrow block; the 12-pt gap arithmetic for BOTH panels; the
  above-100 persistence), `scripts/gui_line_panel_check.sh` (the size-block
  scan: the rail/label/number box/gap on the real screenshots), SPEC §3.2,
  DEBTS-verification (V55 closed verified in chat, V56 born), PROGRESS
  (snapshot + entry + stamp), tests +3.
- **Verified:** FULL gate ALL GATES GREEN (see the stamp): fmt, rules HARD
  green, clippy `-D warnings` both lanes, `cargo test --profile gates` 236
  default / 249 mcp (+3: the size_block topic); the visual driver ALL
  SCENARIOS GREEN; the pixel probe PASS: rail 96 px, label above, the number
  box right (w 40, gap 6), rail-center→buttons 22.5 px on BOTH panels' shots.
- **Owner-checks:** V56 born (the Windows `cargo run` AND the web build pass
  of the Size block in the Shape AND Line panels). Open: V56, V53, V46;
  parked: V30, V8. Debts open: F17, F16, F15.
- **Commit title:** `Size block: the label above the slider, the slider narrowed to the button-grid row width, a click-to-type entry for sizes above 100 (the slider stays 1..=100), the same 12-pt rows gap in the Shape and Line panels`

### 2026-11-20 — `Feat:` the Line panel's icon buttons in the Shape panel's style — owner request

- **What:** the owner verified V54 ("V54 проверил") and asked for the LINE
  tool's panel the same pass: "сделаем также кнопочки как в Shape, то есть
  такие же по стилю и отступам, вместо текущих радио Mode". Done: the word
  "Mode" radio became ONE row of TWO icon buttons — Straight / Curve —
  extracted as `line_settings_ui` (the `shape_settings_ui` sibling): the
  shared `icon_button` (28-pt squares, the light-theme states), the row laid
  out with `item_spacing.x = TOOL_GRID_GAP` (the panel has a single button
  row — no group gap), the Curve hint lines stay UNDER the row. The buttons
  draw the LINE TOOL's emblem (`tool_icons::LineIcon`, the Shape MODE trio's
  one-geometry principle): Curve = the tool button's mark pixel-for-pixel
  (`draw_line`'s curve extracted into the shared `line_emblem_curve`),
  Straight = the same stroke UNBENT between the emblem's endpoints. Tooltips
  carry the old words. The atlas gained the `line_icons` pair (the same
  retain-count handling as `shape_icons`); `LineMode` gained Debug.
- **Files:** `src/app/windows.rs` (line_settings_ui + the match arm),
  `src/app/tool_icons.rs` (LineIcon/LINE_ICONS/line_texture_id + the atlas
  pair), `src/app/tool_icons/draw.rs` (the emblem extraction +
  draw_line_straight + rasterize_line_icon), `src/tools.rs` (Debug),
  SPEC §3.2, DEBTS-verification (V54 closed verified in chat, V55 born),
  PROGRESS (snapshot + entry + stamp), `scripts/gui_line_panel_check.sh` (the
  new pixel probe), tests +6.
- **Verified:** FULL gate ALL GATES GREEN hot (see the stamp): fmt, rules HARD
  green, clippy `-D warnings` both lanes, `cargo test --profile gates` 233
  default / 246 mcp (+6 net: the line_panel topic + the panel's
  radio/compact/gap tests); the visual driver ALL SCENARIOS GREEN under Xvfb
  through the dev+mcp binary; the NEW `scripts/gui_line_panel_check.sh` pixel
  probe PASS on the same binary: the Line panel's in-row gap == the Tools
  grid's == every Shape row's (7 px on screen = the 6-pt gap + the 1-px
  stroke rounding), the Shape rows 12/12 px.
- **Owner-checks:** V55 born (the Windows + Web pass of the Line panel).
  V54 closed verified in chat. Open: V55, V53, V46; parked: V30, V8. Debts
  open: F17, F16, F15.
- **Commit title:** `Line panel: the word Mode radio becomes two icon buttons in the Shape panel's style (the tool's emblem per mode — Curve pixel-for-pixel the tool icon, Straight unbent; the same grid-gap row)`

### 2026-11-20 — `Tooling:` PROBLEMS.md becomes a pure log written only by scripts/log_problem.sh — owner request

- **What:** the owner refined the friction-log round: PROBLEMS.md is LITERALLY
  a log — the agent reads NO line of it, so the rules + entry template moved
  OUT of the file into the AGENTS.md section that explains the log (compact;
  the template is guidance, not a form), and entries go in ONLY through the
  new `scripts/log_problem.sh <entry-file>`: it PREPENDS the entry verbatim at
  the very top (newest first; the entry's trailing newlines normalize to one
  blank line) and BYTE-CHECKS after the write that the previous content
  survived below it — a mismatch restores the backup and fails without
  touching the log, so an agent cannot delete or rewrite anything in it (the
  owner's explicit permission is the only door). The two existing entries
  stayed byte-identical; the file now starts straight with its newest entry.
  Second owner-approved exception to the "no scripts without asking" rule;
  entry temp files stay OUT of the repo (the script warns — a worktree file
  would leak into the delivery artifacts).
- **Files:** `scripts/log_problem.sh` (new), `PROBLEMS.md` (header/template
  stripped, entries kept verbatim), `AGENTS.md` (the section now carries the
  template + the script rule; the Source-of-truth bullet + the layout
  exception updated), PROGRESS (entry + stamp).
- **Verified:** the script exercised on a /tmp COPY only — prepend + the tail
  byte-identical to the real log, blank-only entry refused, entry==log
  refused, an in-repo path warned, a double prepend keeps newest-first; the
  real PROBLEMS.md untouched this round; FULL gate ALL GATES GREEN (the
  stamp).
- **Owner-checks:** none new (docs + a shell script — nothing to click).
  Open: V54, V53, V46; parked: V30, V8. Debts open: F17, F16, F15.
- **Commit title:** `PROBLEMS.md: the file becomes a pure log written only by scripts/log_problem.sh (prepend-only, integrity-checked, nothing deletable without the owner); the template moves to AGENTS.md`
