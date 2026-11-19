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
| Extra: CLI launch args + owner-unconfirmed scenario markers (owner 2026-11-17) | done — gates green | native `--primary/--secondary/--tool/--canvas-size/--help` (`app/launch.rs`, parsed before the window opens; invalid values warn, defaults stay; wasm takes none); the 8 suite scenarios carry "NOT YET CONFIRMED" markers + a header ledger (the owner approves one by one); V29/V30 born |
| Extra: repo-wide English-only owner quotes (owner 2026-11-18) | done — docs only | every verbatim Russian owner quote across docs/scripts/comments translated to English (meaning kept); PROGRESS Log compacted — 2 entries moved to PROGRESS-archive.md |
| Extra: debt F14 — in-house windows later (owner 2026-11-18) | done — docs only | egui::Window → an in-house `egui::Area` frame recorded in DEBTS-functional.md from the owner's pros/cons chat review; owner-gated, no code touched |
| Extra: tool icon buttons + two-column grid (owner 2026-11-18) | done — gates green + visual driver | 1.5rem icon buttons via a cached tiny-skia atlas (`app/tool_icons.rs`), colored pencil/brush/eraser/pipette + mono line/shape/text through ONE pipeline, name+key hover tooltips, the pipette as the Eyedropper's canvas cursor; the two-column grid is debt F15; the driver screenshot caught the dead-texture trap (handles kept in the Atlas now — AGENTS.md quirks) |
| Extra: fully unoptimized dev profile (owner 2026-11-18) | done — cargo check green on the new lane | `[profile.dev]` opt-level 0 + no debuginfo, the `[profile.dev.package."*"] opt-level 2` dep override REMOVED (the "deps optimized once for responsive `cargo run`" policy retired — development is sandbox-only now, UI lag accepted); flake `russet-dev` already ran the unoptimized gates lane (untouched); SPEC §1 + AGENTS.md synced; owner visual pass = V34 |
| Extra: verify.sh hardening (owner 2026-11-18) | done — FULL gate green on a cold lane (191 s) | `exec </dev/null` closes stdin for the whole run (SANDBOX.md trap 10 fixed AT THE SOURCE — the mcp test stage exits in seconds instead of hanging); per-stage seconds + total printed (TIMING.md rows are copied now, not re-measured); cold-lane hint when `target/gates` is absent; the mcp pair stays UNCONDITIONAL (hot gate ≈ 15 s, zip sandboxes have no git, the gate never guesses) |
| Extra: brush + pipette icons redrawn + equal grid gaps (owner 2026-11-18) | done — hot gate 14 s + driver green | round volumetric bristle belly + crimped ferrule (the wedge read as a second pencil); tapering glass cone + round rubber bulb via a new `arc_pts` helper (the rectangle stick read as a pen); Tools-grid H-gap 18 pt → 6 pt = V-gap (egui Grid floors columns at `interact_size.x` 40 pt — `.min_col_width` fix, AGENTS.md quirk); tool_icons.rs 610 lines — the draw/atlas split proposed (owner-gated) |
| Extra: --warm warm-up + session rules; debts ask = 4 newest; V33 closed (owner 2026-11-18) | done — live cold-lane test + hot gate | `sandbox_check.sh --warm` = probe + detached gates-lane compile (clippy + test --no-run, nothing executed — the mcp stdin trap impossible) with a double-start pid guard; session start = `sandbox_check.sh --warm` FIRST; wasm ad-hoc checks only for web-facing changes; the batched debts ask lists only the 4 newest open items (AGENTS.md Debts rule 7) |
| Extra: permanent #[ignore] icon-dump + F16 rust-cache debt (owner 2026-11-18) | done — dump live-tested, gate skips it | `cargo test --profile gates -- --ignored icons_dump` = 14 PNGs (7 tools × 256/24 px) into /tmp/icon_dump, BLACK raster keeps the monochrome icons legible; plain gate = 156/168 passed + 1 ignored; rust-cache (`Swatinem/rust-cache@v2` after checkout) recorded as postponed F16 with the mechanism + limits written out, ci.yml untouched |
| Extra: icons v3 + Theme-button v2 (owner pass 2026-11-18) | done — contact sheet + FULL gate + driver | tool buttons keep the LIGHT-theme fills in both themes (`egui::Visuals::light()` in `windows.rs::tool_button`) → the outline is the ONE dark rim everywhere (v2 light rim reverted, tested); pipette glass = cylinder + cone only at the tip, wider collar ring; menu-bar button just "Theme" before Help (Help always last), System radio plain; MCP `set_theme` unchanged |
| Extra: icons v4 polish + title-bar-less Tools (owner pass 2026-11-18) | done — contact sheet + FULL gate | eraser rim dark plum (in the set's style); buttons' rest/hover fills dimmed −14/channel (selection stays bright); mono ink FIXED `TOOL_INK` gray 64 → icons fully theme-independent → atlas key = ppp only (the theme re-raster gone; the atlas test reworked: a theme switch uploads only egui's font atlas); Tools window `title_bar(false)` — id kept via `tools_window_id()`, drag falls back to Anywhere, close via View ▸ Tools; V37 verified, V36 → v4 re-check pending |
| Extra: Tools window v5 — the bar back, only the label hidden (owner pass 2026-11-18) | done — gate + driver + magnet check | `Window::new("")` + the explicit `tools_window_id()` renders the title bar WITHOUT the text (egui derives the Id from the title even when empty); `.open` plumbing restored (the ✕ is back); AGENTS quirk rewritten (the empty-title recipe); .gitignore keeps `__pycache__`/`*.pyc` out of the artifacts; V36 → v5 re-check pending |
| Extra: language rule → English-only whitelist (owner request 2026-11-18) | done — gate + deliver | the stamp's "Zero Cyrillic repo-wide" reworded: a language WHITELIST (English is the only written language), not a script blacklist — friendlier to readers of any writing system |
| Extra: Tools strip fits the window — variant A2 (owner 2026-11-19) | done — gates + probes green | the title strip's HORIZONTAL padding zeroed via `Window::title_frame` (`windows.rs::tools_title_frame` = egui's default window frame with `inner_margin.left/right = 0`): the strip paints EXACTLY the window's width instead of ~11 pt past it, the ✕ no longer hangs half outside, BOTH buttons kept, the strip's height/fill/corners unchanged (variant A `Margin::ZERO` shrank the strip vertically — not picked); +1 regression test on the `__title_click` geometry; `gui_collapse_check.sh` locate fixed (the idle crosshair cursor fooled it); V38 + V36 verified in chat, V39 born |
| Extra: Tools panel hides while collapsed (owner 2026-11-19) | done — gates + click probe green | the magnetized settings panel hides while the Tools window is collapsed to its title bar and returns on expand (`windows.rs::settings_panel_shown` on `Window::show`'s `inner.is_some()`); the header-width PROPOSAL delivered (measured options, no header code change — the owner picks); V38 born |
| Extra: tool_icons.rs file-guard split — the drawings to the child `draw` (owner 2026-11-19) | done — gates green, suite unchanged | 673 → 308 + 400, both under the 500 guard: the SEVEN icon drawings (`draw_*` fns, `TOOL_ANGLE`, `PIPETTE_TIP`, the `rasterize_tool_icon` dispatcher) moved VERBATIM to the child `app/tool_icons/draw.rs`; the parent keeps the atlas/cursor pipeline + the shared primitives and imports the dispatcher — zero external path changes (app.rs/windows.rs untouched, the tests' `super::` imports byte-identical); rules.sh's second OVER-GUARD note retired (only the owner-deferred history_tree.rs 569 remains) |
| Extra: palette alpha + a hex field in the color wells (owner 2026-11-19) | done — gates + driver green | `Alpha::OnlyBlend` (the picker's alpha slider + the A DragValue; the eyedropper's alpha no longer erased on popup open) + a "Hex" row parsing the launch-flag syntax (short form keeps the well's alpha, 8 digits sets it, `#RRGGBBAA` display, canonicalized on blur); one shared `launch.rs::parse_hex_components` feeds both consumers; windows/tests split by topic (rule 7) + 5 color-well tests; windows.rs 571 — OVER guard, the palette split PROPOSED (owner-gated); V40 born |
| Extra: lenient hex + the windows.rs palette split (owner 2026-11-19) | done — gates + driver green | the Hex field eats pasted junk (every non-hex-digit char stripped, an optional `0x` prefix dropped) and the new digit-count forms 4 `RGBA` / 5 `RGBAA` / 7 `RRGGBBA` (the 5-vs-7 asymmetry is the owner's deliberate design); a per-well SEED alpha keeps a live-typed 5-digit prefix from leaking alpha into a final 6-digit color; the launch flags share the same `parse_hex_components` syntax; PURE code-motion split: `windows.rs` 571 → 351 + the child `windows/palette_window.rs` 287 (the palette window, wells, picker popup, hex field; the color-well tests moved to `palette_window/tests.rs` per Tests rule 2) — both under the guard, the OVER-GUARD note retired; V40 verified in chat, V41 born |
| Extra: the "Save Image" dialog — Save and Save As converge (owner 2026-11-19) | done — gates + GUI probes + driver green | BOTH Save and Save As open ONE in-app dialog (`app/save_dialog.rs`): the file name field, the PNG/JPEG format radio (THE format choice), the JPEG Quality 1..=100 (default 90), the live "Saving as:" preview, the native Folder row + Browse… (hidden on web; a cancelled pick keeps the dialog open); `file_io.rs` gains `SaveFormat`, the quality-parameterized JPEG encoder and `pick_save_folder`; the MCP `export_png` keeps the extension-decides path export; debt F17 born (consider other formats) |

**Verification stamp:** FULL gate re-verified 2026-11-19 (hot lane, 14 s): fmt clean,
rules.sh HARD-green, clippy `-D warnings` green (default + mcp), `cargo test --profile gates`
179 default / 192 mcp green (+1 ignored each — the permanent #[ignore] icon dump; +9
save-dialog tests this round), the wasm `cargo check --target wasm32-unknown-unknown` green
(file_io.rs touched) and the visual driver ALL SCENARIOS GREEN under Xvfb through
the gates+mcp binary — Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
English-only repo-wide — a language WHITELIST (English is simply the only written language
here: comments, docs, commit titles), not a script blacklist. Web/trunk lanes last verified
2026-11-16 (see the archived log history);
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
> round's budget (2026-11-19).

### 2026-11-19 — `Feat:` the "Save Image" dialog — Save and Save As converge — owner request

- **What:** BOTH File ▸ Save (Ctrl+S) and File ▸ Save As (Ctrl+Shift+S) now open ONE in-app
  "Save Image" dialog (owner request 2026-11-19: the png/jpg settings + the file name in the
  window, and THE FORMAT CHOICE must live there) — the OS save dialog is gone, Save no longer
  writes silently to the current path. Contents: the file name field (prefilled from the
  display name minus its image extension), the PNG image / JPEG image radio (prefilled from
  the current path's extension, else the display name's, else PNG), a JPEG Quality slider
  1..=100 (default 90 — the old hardcoded value; kept across opens; JPEG only), the live
  "Saving as: name.ext" preview, and — NATIVE only — a Folder row (prefilled from the
  document's own folder; Browse… = the native folder picker; hidden on web — the browser owns
  the download location). With no folder known the Save click opens the picker and a
  cancelled pick KEEPS the dialog open (observed live in the sandbox probe). Name rules
  (`save_dialog.rs::compose_file_name`): separators + the Windows-forbidden characters
  stripped, a typed extension matching the format survives (canonicalized .jpeg → .jpg), a
  conflicting image extension is replaced, a non-image one stays, empty → "Untitled".
  Success: `dirty = false`, `display_name` = the saved name, (native) `current_path` updated;
  an error is the Error popup and the dialog STAYS OPEN. file_io.rs: pub `SaveFormat`
  (+extension/label/from_extension), `encode_jpeg_with_quality` (clamped 1..=100),
  `pick_save_folder`, the dialog save `save_image_as(pixmap, dir, name, format, quality)`;
  the MCP `export_png` keeps `save_image_to` (extension decides — now feature-mcp-gated, its
  only consumer). NEW debt F17 (owner ask): consider other file formats for open and save.
- **Files:** NEW `src/app/save_dialog.rs` + `save_dialog/tests.rs` (6 tests);
  `src/file_io.rs` (+3 tests), `src/app.rs` (fields + frame hook + module map),
  `src/app/document.rs` (the save glue: `prefill_save_from_path` + `commit_save_dialog`),
  `src/app/menu.rs` / `input.rs` (no-arg call sites), SPEC §3.7/§3.10, AGENTS.md (module
  map), DEBTS-functional.md (F17 born), DEBTS-verification.md (V42 born), PROGRESS
  (snapshot + entry + stamp + compaction), TIMING (2 cold rows).
- **Verified:** FULL hot gate ALL GATES GREEN 14 s — fmt clean, rules HARD green, clippy
  `-D warnings` green (default + mcp), `cargo test --profile gates` 179 default / 192 mcp
  green (+1 ignored each; +9 tests); `cargo check --target wasm32-unknown-unknown` green
  (file_io.rs is web-facing); GUI probes under Xvfb: the dialog opens from BOTH menu items,
  radio/quality/preview behave live, the cancel-keeps-open path observed, the write core
  proven through the gates+mcp binary (MCP export_png → PNG + JPEG files PIL-decoded),
  visual driver ALL SCENARIOS GREEN.
- **Owner-checks:** V42 born (the Windows save pass: the dialog on Ctrl+S / Ctrl+Shift+S,
  the radio + quality, Browse… = the native folder dialog, both formats saved). Open: V42,
  V41, V30 (parked), V8 (parked); NEW debt F17 (which other formats).
- **Commit title:** `Save Image dialog: Save and Save As converge on the in-app window (PNG/JPEG radio, quality, folder row); F17 debt: consider more file formats`

### 2026-11-19 — `Feat:` the hex field eats pasted junk + the windows.rs palette split — owner request

- **What:** two asks, one round. (1) LENIENT HEX (V41): `launch.rs::parse_hex_components`
  strips EVERY non-hex-digit character (an optional `0x`/`0X` prefix first — the owner: "keep
  only what means color") and reads the DIGIT COUNT: 3 `RGB` (doubled), 4 `RGBA` (doubled),
  5 `RGBAA` (shorthand RGB + a FULL alpha byte), 6 `RRGGBB`, 7 `RRGGBBA` (full RGB + a DOUBLED
  alpha digit), 8 `RRGGBBAA`; <3 or >8 digits is invalid. The 5-vs-7 RGB-count asymmetry is
  the owner's deliberate design (SPEC §3.3). A live-typed longer entry passes THROUGH
  alpha-carrying prefixes (`FF7F0` inside `FF7F00`) — a NEW per-well SEED alpha (snapshotted
  with the text on popup open / outside changes) holds the short forms' fallback at the
  session-start alpha, so a prefix cannot leak its alpha (+1 regression test;
  `hex_edited_color(fallback_alpha, text)` carries the seed). The launch flags inherit the
  same leniency (one primitive, no drift; a missing alpha stays opaque; the old "12345 →
  invalid" garbage case became a valid `RGBAA` form). (2) THE SPLIT (owner-approved):
  `app/windows.rs` was 571, over the 500 guard — the palette half (the window, the wells with
  the picker popup, the hex helpers) moved VERBATIM to the child `windows/palette_window.rs`;
  the color-well tests moved with the code to `palette_window/tests.rs` (Tests rule 2 — the
  parent cannot re-import items nothing uses). Same shape as the `tool_icons`/`draw` split.
- **Files:** `src/app/launch.rs` (+ tests: +1, the garbage set re-based), `src/app/windows.rs`
  (571 → 351; `pub(super)` → `pub(in crate::app)` — the only non-verbatim line), NEW
  `windows/palette_window.rs` (287) + `palette_window/tests.rs` (6 tests), `windows/tests.rs`
  (color_well → the child), `src/app.rs` (module map), `SPEC.md` (§3.3 + §3.12; the §3.3
  "§5.1" mis-link fixed — no such section), `AGENTS.md` (module map + hex quirk),
  `DEBTS-verification.md` (V40 verified, V41 born), PROGRESS (snapshot + entry + stamp +
  compaction). TIMING: no rows — every command ran under a minute.
- **Verified:** FULL hot gate ALL GATES GREEN 9 s — 170 default / 183 mcp green (+1 ignored
  each; +2 tests, the moved 5 rewritten in place); the visual driver ALL SCENARIOS GREEN under
  Xvfb through the gates binary. No wasm lane: not a web-facing change (the session rule).
- **Owner-checks:** V41 born (the lenient hex pass on Windows). V40 verified in chat. Open:
  V41, V30 (parked), V8 (parked).
- **Commit title:** `Palette hex field: junk-tolerant parsing + the digit-count forms (seed alpha); Split app/windows.rs: the Palette window to the child palette_window.rs (pure code motion)`

### 2026-11-19 — `Feat:` the color wells edit alpha + a hex field under the picker — owner request

- **What:** the owner asked for alpha in the custom-color picker and, bonus, a hex field
  ("hex is needed too often"). `windows.rs::color_swell_button`: `Alpha::Opaque` →
  `Alpha::OnlyBlend` — egui's alpha slider + the A DragValue appear, and egui stops forcing
  the stored alpha to 1.0 (which silently erased an Eyedropper-picked translucent pixel on
  the first popup open); the pipeline composites alpha end-to-end already. NEW: a "Hex" row
  under the picker — displays `#RRGGBBAA`, parses the launch-flag syntax (`#RGB` doubled,
  `#RRGGBB`, `#RRGGBBAA`, `#`/spaces optional): 8 digits SET the alpha, a short form KEEPS
  the well's current alpha; the raw text lives in egui's per-well `IdTypeMap` (a frame-rebuilt
  string would fight the caret), re-seeded on popup open / outside changes, canonicalized on
  blur; invalid text applies nothing. One shared primitive `launch.rs::parse_hex_components`
  (the old `parse_color` core extracted) feeds BOTH the flags and the field — the syntax
  cannot drift; launch behavior unchanged (a missing alpha stays opaque). egui's premultiplied
  `Color32` storage can nudge a translucent channel ±1 — an app-wide property, now noted in
  the tests + the AGENTS quirk.
- **Files:** `src/app/windows.rs` (the popup + `hex_edited_color`/`format_color_hex`; 571
  lines — OVER the 500 guard: a palette-window split PROPOSED at task end, owner-gated),
  `src/app/launch.rs` (the extraction + docstring), NEW `src/app/windows/tests/{common,
  settings_panel,tools_strip,color_well}.rs` (the suite split by topic, Tests rule 7 — the
  old tests.rs sat at 292 lines) + 5 color-well tests, `SPEC.md` §3.3, `AGENTS.md` (+1 quirk:
  egui 0.37 has NO hex entry in the picker anywhere; the split's test path fixed),
  `DEBTS-verification.md` (V40 born), PROGRESS (snapshot + entry + stamp + compaction),
  TIMING (2 rows).
- **Verified:** FULL hot gate ALL GATES GREEN 8 s — 168 default / 181 mcp green (+1 ignored
  each; +5 color-well tests, the moved ones byte-identical); the visual driver ALL SCENARIOS
  GREEN (15 checks) under Xvfb through the gates binary. No wasm lane: not a web-facing
  change (the session rule).
- **Owner-checks:** V40 born (the palette alpha + hex pass on Windows). Open: V40, V30
  (parked), V8 (parked).
- **Commit title:** `Palette: the color picker edits alpha (OnlyBlend) + a #RRGGBBAA hex field under it`

