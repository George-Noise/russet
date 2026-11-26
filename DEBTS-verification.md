# DEBTS-verification — checks awaiting a HUMAN

> **Why this file exists.** Verification debts are checks only the OWNER can do:
> Windows `cargo run` click-throughs, browser passes, `nix build` on a Nix machine,
> doc wording sign-offs. They used to live only as `Owner-checks` / `Owner must check`
> lines inside `PROGRESS.md` log entries, where they silently accumulated — nobody
> was reminded, and the owner forgot what is still unverified (owner request
> 2026-11-13: "ask whether things are hand-verified so they don't pile up").
>
> **Rules (AGENTS.md "Debts"):**
> - The agent reads this file at session start and at task end and MUST ask the
>   owner, in ONE batched list, which open items are now human-verified.
> - A verified item gets `Status: verified <date>` (and how: chat / commit check)
>   and is dropped at the next compaction; the historical record stays in the
>   `PROGRESS.md` log entry — this file only tracks what is STILL open.
> - Every new verification debt lands here in the SAME edit as the task that
>   creates it, with the date it was born.
> - Open items are sorted: V checks first, then F debts (functional), then
>   anything else — NEWEST birth date first within each group (owner request
>   2026-11-16, AGENTS.md "Debts" rule 6). A closed row leaves the table for
>   the Resolved list in the same edit.
> - Presenting (owner request 2026-11-18): the batched ask lists ONLY the 4
>   newest open items, newest first — the table keeps every open item (AGENTS.md
>   "Debts" rule 7); nothing is dropped, only the ask stays scannable.
> - `scripts/rules.sh` reports PROGRESS.md size and live owner-checks as a
>   report-only reminder, so nothing hides.
> - Milestones (M8/M9) keep their Status-snapshot rows; only their still-open
>   human checks are mirrored here.

| # | Opened | What the human must check | Source (PROGRESS.md log) | Status |
|---|---|---|---|---|
| V85 | 2026-11-26 | The BMP + pinch-round pass (Windows `cargo run`, plus the WEB build `trunk serve`): File ▸ Open lists and opens a `.bmp` (a 32-bit one with alpha keeps its transparency); the Save Image dialog's format dropdown carries "BMP image" — picking it shows the locked-ON Lossless row and NO knob (the format is uncompressed), Save writes a `.bmp` that re-opens in russet AND in a system viewer (a viewer that predates the V4 header may show the alpha channel as opaque — the format's own quirk); the typed-name rules carry `.bmp` (a `photo.bmp` prefill keeps `photo`, a conflicting extension is replaced); the phone web legs of the amended rules: the pinch zoom anchors at the fingers (the midpoint content stays put), and a phone ROTATION neither shows nor hides any window (the Tool Log toggle stays exactly as the user left it). | 2026-11-26 owner request in chat ("Добавь формат BMP в open и save" + the pinch-zoom/Tool Log-startup amendments) | open |
| V84 | 2026-11-25 | The web touch + smartphone round (the web build, `trunk serve`/`trunk build --release`, ideally a REAL phone; plus one native `cargo run`): (a) two-finger drag pans smoothly (the content follows the fingers), AND a two-finger pinch SPREAD/PINCH zooms continuously anchored at the pinch midpoint — the content between the fingers stays under them (the zoom and the pan ride the SAME gesture; amended 2026-11-26), one finger still paints, a second finger landing mid-stroke ends the stroke as ONE undoable node (the painted part stays) and nothing paints for the rest of the touch session — the leftover finger pans only, painting resumes after every finger lifts; (b) on a phone-width viewport (~360-430 pt wide): the Tool Log window starts HIDDEN but the View ▸ Tool Log item IS there — turning it on shows the window (amended 2026-11-26: the hiding is STARTUP-only), every floating window first-open FLUSH against its edges (Tools/Palette/Layers/History Tree), and the compact rules hold after a phone rotation too — rotating must NOT show, hide or re-hide any window (the startup latch never re-runs; whatever the user toggled stays); (c) a desktop-width web session is unchanged (8 pt gaps, the Tool Log shows), and the native legs: the MIDDLE-BUTTON drag and Space+LMB pans MOVE the view again (the drag-delta line lost in the 2026-11-13 split, restored 2026-11-25 — on the previously delivered build they did nothing). | 2026-11-25 owner request in chat (multitouch pan in web, web-only for now, no native Android build planned; the Tool Log hidden on smartphone web; every window flush to the edges at 0 on smartphones) | open |
| V81 | 2026-11-23 | The fill/wand flood is span-scanline and fast: the SAME pixel results as the old BFS (the random-canvas oracle test kept the naive algorithm verbatim), translucent and transparent colors behave identically (the dense sample_bytes_of sweep + the transparent-hole and alpha-tolerance tests), and the fresh benchmark logs show the speedup (dev FullHD 1.72 s → 0.34 s, release 238 ms → 13.1 ms). The owner feels it: big fills snap in both modes. | 2026-11-23 owner request in chat ("we do 1+2+3+writing from the spans") | open |
| V80 | 2026-11-23 | The fill/wand flood benchmark exists and is permission-locked: it compiles only under the cargo feature `fill-bench` (the default build has none of it), triggers on RUSSET_BENCH_FILL=1, REFUSES to run without RUSSET_BENCH_ALLOWED=1 (exit 2 with a pointer to the rule), then writes `benchmarks/fill_bench_<dev\|release>_<YYYYMMDDHHMMSS>.log` — the UTC timestamp digits-only down to the second, the mode IN the name — carrying the machine specs (CPU model, cores, RAM, kernel, rustc, git HEAD) and per-case nanosecond timings (min/median/mean/p95 of the flood read and the fill write) over 10 deterministic canvases. The agent must not run it without the owner's explicit chat instruction. | 2026-11-23 owner request in chat (the bench + the logging + the agent-lock) | open |
| V75 | 2026-11-23 | The settings round (Windows `cargo run`, a REAL 5-button mouse): the FIRST Mouse4 press still opens the modal; **Yes**/**No** now PERSIST — quit and relaunch: no question, Mouse4/Mouse5 act per the choice (or stay inert after No); the menu bar's new **Settings** menu (after Theme, before Help) opens the Settings window — the dropdown shows the LOADED choice, and switching it (Ask every session / Use for Undo / Redo / Don't use) works LIVE: the next side press obeys immediately, Ask brings the question back, switching while the modal is open closes it, the Tool Log records every settled choice ("ask every session" / "Undo/Redo on" / "off"); the storage file `%APPDATA%\Russet\data\app.ron` after a choice holds EXACTLY ONE key (`settings/mouse_side_buttons`) and is ABSENT for a user who never decided (or reset to Ask); the dropdown's popup could NOT be driven by the headless probe (a synthetic-click gap — the modal's buttons and the palette's popup click fine on the same binary), so the dropdown's own click-through is the core of this check. | 2026-11-23 owner request in chat ("only the Mouse4/Mouse5 choice should persist" + the Settings window with the three-choice dropdown) | verified 2026-11-23 (chat) |
| V74 | 2026-11-23 | The persistence opt-out pass (Windows `cargo run`, run TWICE): the app data folder `%APPDATA%\Russet\data` EXISTS after a session (eframe creates it at startup — the feature's one unavoidable side effect) but stays EMPTY — no `app.ron` appears even after drawing (the 30-s autosave and the exit save both write nothing); a restart opens everything at the DEFAULT positions (the OS window at the 1200×800 builder default, floating windows at their §3.11 spots, dialogs centered) — nothing is remembered or restored; the web build writes nothing to the browser's localStorage (devtools: no `egui_memory_ron` key appears). | 2026-11-23 owner request in chat ("enable the persistence feature and make it save nothing at all for now or as little as possible, and check whether it saves anything and if so what") | verified 2026-11-23 (chat) |
| V73 | 2026-11-23 | The Mouse4/Mouse5 undo/redo opt-in (Windows `cargo run`, a REAL 5-button mouse): the FIRST Mouse4 press opens the centered "Mouse Side Buttons" modal and does NOT undo/redo; **Yes** enables for the session — Mouse4 undoes, Mouse5 redoes, from anywhere in the window (an exhausted stack is a silent no-op), a press mid-stroke/mid-drag does nothing, the answer shows in the Tool Log; **No** leaves both buttons inert; the ✕ closes WITHOUT deciding and the NEXT press asks again. AMENDED the same day (the settings round, V75): the answer now PERSISTS — the restart does NOT re-ask; the Settings window's dropdown re-opens the question. | 2026-11-23 owner request in chat ("It was suggested to me to add undo and redo on Mouse4 and Mouse5…" + "ask the user through a modal") | verified 2026-11-23 (chat) |
| V72 | 2026-11-23 | The translucent-stroke pass (Windows `cargo run`): with a semi-transparent brush color a stroke paints UNIFORM — the joint circles (where one frame's pieces hand over to the next) and the press dot are the SAME color as the stroke body, however fast or slow the drag; opaque colors draw exactly as before; the eraser erases as before (a self-overlapping eraser stroke no longer over-erases at its AA-edge overlaps — hard to see, just confirm nothing regressed); the pencil is pixel-unchanged; undo still records ONE node per gesture; a selection still confines every stroke. | 2026-11-23 owner bug report in chat ("There is currently a bug when using brush with a semi-transparent drawing color…") | verified 2026-11-23 (chat) |
| V71 | 2026-11-21 | The History Tree's icon rows now cover the three PAINTING tools (Windows `cargo run`): pencil, brush AND eraser rows draw their tool's emblem instead of the word (the eraser's is the classic pink block); the eraser row shows NO color chip (it erases — it does not paint), while pencil/brush keep theirs pulled CLOSER to the emblem (a 6 pt gap instead of 10 — the chip groups with the icon); the start row stays word-only and chipless; the rails, dots, hover/click areas and scroll/pin behave as before; both themes look right. | 2026-11-21 owner requests in chat ("like pencil, let's have brush and eraser in the history tree too", "so there is a bit less space between the icon and the color", "for some reason the eraser has a color … that must not be") | verified 2026-11-23 (chat) |
| V70 | 2026-11-21 | The History Tree row-icon pass, final geometry (Windows `cargo run`): the pencil rows' icon is clearly BIGGER (16 pt instead of 12 — the emblem fills most of its picture), two consecutive icons sit with EXACTLY a hair's width between them vertically (0.0625rem = 1 pt at standard scale — the rows sit back-to-back; measure two rings under `--atlas-test`: a 1 px gap), and the pencil emblem rests a touch LEFT-DOWN of the icon's center (visible against the ring); the rails, dots, color chips and hover/click areas stay aligned, the tree still scrolls/pins as before; both themes look right. | 2026-11-21 owner requests in chat ("the pictures in the history tree are too small…", "between two icons … 0.0625rem", "move the pencil a touch left-down") | verified 2026-11-23 (chat) |
| V69 | 2026-11-21 | The `--atlas-test` diagnostic (Windows `cargo run -- --atlas-test`): every icon picture — the Tools grid buttons, the Shape/Line/Select panel buttons, the History Tree's pencil row icon — carries a 1 px BLACK-AND-WHITE SPECKLE ring on its very edge (exactly one screen pixel, not scaled by zoom/DPI), static (no flicker between frames), painted over the emblem where the drawing reaches the edge; the pipette cursor over the canvas shows the ring too; WITHOUT the flag nothing changes. The point: judge the empty margin between the ring and the emblem (the pencil leaves ~4 px at the button size) and decide what to do about the insets. | 2026-11-21 owner request in chat ("let's add a new flag that means atlas testing… a black-and-white speckle… exactly 1 px") | verified 2026-11-23 (chat) |
| V68 | 2026-11-21 | The History Tree row pass (Windows `cargo run`): draw a few pencil strokes and open the History Tree — every pencil row shows the tool's ICON (the same yellow-pencil emblem the Tools button uses, at the ROW size — about the capital letter of the row text or slightly bigger) instead of the word "pencil"; NO row shows a node id digit anymore ("pencil 7" is just "pencil", "start" has no number); a crop row reads "crop 800×600→300×200 @(100,50)" (no doubled "crop crop"); the color chip, the dot click-to-jump and every other row kind are unchanged; the icon stays legible on BOTH the dark and the light theme. | 2026-11-21 owner request in chat ("in the history tree we use an icon instead of the pencil word… drop the digit in every row") | verified 2026-11-23 (chat) |
| V67 | 2026-11-21 | The F21 async-dialog pass (Windows `cargo run` AND a GNOME/Linux session): File ▸ Open no longer freezes the window — GNOME never dims it or offers "Not Responding"/Force Quit while the chooser is open, the window keeps painting and the menu answers; the picked image opens exactly as before; the Save dialog's Browse… fills the folder row; a Save click with NO folder opens the picker and the save COMPLETES BY ITSELF once the folder is chosen (a cancelled pick keeps the dialog open); while any chooser is open the Open… item and Browse… are GRAYED and no second chooser stacks; the web build behaves exactly as before. | 2026-11-21 owner pick in chat ("We go with Variant A") | verified 2026-11-23 (chat) |
| V66 | 2026-11-21 | The selection-clip pass (Windows `cargo run` AND the web build): with a selection active, EVERY painting tool paints only INSIDE it — make a rect (and a lasso) selection and draw across its boundary with the pencil, the brush, the eraser, the Line/Curve, the Shape and the Text: nothing outside the selection changes, everything inside works normally (the eraser erases only inside; the live previews also stop at the selection's bounds); with NO selection the behavior is exactly as before; the selection's own ops (move/copy/cut/paste/crop) are unchanged. | 2026-11-21 owner request in chat ("the selection confines the canvas changes to its own area") | open |
| V46 | 2026-11-20 | The smooth-stroke Linux pass (NixOS, BOTH backends — X11 AND Wayland), the flake's native lanes (`nix run` release / `nix run .#russet-dev`): draw SLOW and FAST diagonals and curves with the pencil — the 1 px trail is an EVEN staircase, the SAME pattern for the same gesture regardless of speed or machine; fast strokes stay smooth on the pencil, brush and eraser (no polygon runs); translucent colors do not darken at the joints; a single click sets exactly one pixel; the stroke ends exactly on the release pixel; undo still records one node per gesture. Run both backends: the Wayland session AND X11 (an X11 session or `WINIT_UNIX_BACKEND=x11`). | 2026-11-20 owner request in chat (V45's pass covered Windows + Web; the Linux legs stayed unchecked) | open — the Wayland leg verified 2026-11-21 via chat (owner: "checked V46 Wayland on GNOME"); the X11 leg remains |
| V40 | 2026-11-19 | The palette alpha + hex pass (Windows `cargo run`): clicking a color well opens the picker with an ALPHA slider under the hue slider and an A DragValue in the top R/G/B/A row; dragging the alpha paints TRANSLUCENT strokes on the canvas; a "Hex" row sits under the picker showing `#RRGGBBAA` — typing `#FF7F00` (or `ff7f00`, or `#f70`) keeps the current alpha, typing 8 digits (`#FF7F0032`) sets it, invalid text applies nothing and reverts on Enter; the field re-formats on Enter; the preset grid and Swap still work, and both wells keep independent fields. | 2026-11-19 palette alpha + hex entry | **verified 2026-11-19 via chat** (owner checked the pass; his hex-input leniency follow-up became V41) |
| V37 | 2026-11-18 | The theme-switcher pass (Windows `cargo run`): the menu-bar button sits BETWEEN View and Help (Help always LAST), reads just "Theme" (no ": Dark" suffix), the radio options are plainly Dark / Light / System; picking one re-styles the WHOLE UI immediately; System follows the OS preference and is the startup default; the Tool Log records "Theme: …"; the MCP tool `set_theme` (dark/light/system) switches the theme the same way. | 2026-11-18 theme-switcher entry | **verified 2026-11-18 via chat** (owner approved the look; no further asks) |
| V35 | 2026-11-18 | The redrawn-icons look pass (Windows `cargo run`, Tools window): the BRUSH reads at a glance as a classic round brush (round volumetric bristle belly, crimped metal ferrule, wooden handle, dark paint-loaded tip — no longer "a second pencil") and the EYEDROPPER as a classic pipette (round red rubber bulb, wider metal collar, tapering glass with the blue liquid — no longer "a pen"); the two-column grid gaps are EQUAL horizontally and vertically; hover tooltips still name each tool. | 2026-11-18 icon-redraw entry | **verified 2026-11-18 via chat** (owner checked the icons; his improvement list became V36) |
| V34 | 2026-11-18 | The fully-unoptimized-dev pass (Windows `cargo run` on the delivered tree): the dev profile now compiles EVERYTHING unoptimized (`opt-level 0`, no debuginfo, no per-dependency overrides) — the pass only confirms the build completes and the app runs with every feature still visually checkable (draw, zoom/pan, floating windows, dialogs, history). UI responsiveness MAY lag — expected and accepted (owner: "it's dev — it's fine if it lags"); the owner judges whether the faster build is enough for quick visual feature checks. | 2026-11-18 unoptimized-dev entry (PROGRESS-archive.md) | **verified 2026-11-18 via chat** (owner: the build is fast "and nothing even lags") |
| V33 | 2026-11-18 | The About-window pass (Windows `cargo run`; optionally the browser build): Help → About opens the window with the description now naming the language ("A tiny bitmap paint program **written in Rust** with egui/eframe."), the new "GitHub: George-Noise/russet" hyperlink in egui's link color that OPENS `https://github.com/George-Noise/russet` in the system browser (on web — a new tab), and the old lines unchanged (Version / Platform / Renderer / Close). | 2026-11-18 About-window entry (PROGRESS-archive.md) | **verified 2026-11-18 via chat** (owner checked it on NixOS AND Windows) |
| V32 | 2026-11-17 | The first-open placement fix (Windows `cargo run`; FRESH launch, no drags): resize the main window — every window KEEPS its first-open gaps (Tools: 8 pt from the left edge + the 40/60 vertical ratio; Palette: 8 pt left+bottom; Tool Log: bottom edge flush; Layers: 8 pt right and 36 pt above the bottom — the zoom % label stays clear; History Tree: 8 pt right, 8 pt below the menu bar). Before the fix the never-dragged windows drifted off these boundaries on the first resize and only a manual re-drag re-anchored them. Dragging a window to a boundary afterwards still anchors the dropped gap, as before. | 2026-11-17 first-open anchors entry (PROGRESS-archive.md) | verified in chat by the owner 2026-11-17 ("checked the windows") — kept as an anchor row |
| V31 | 2026-11-17 | The magnetized tool-settings panel pass (Windows `cargo run`): the Tools window holds ONLY the tool list; the settings (Size, Line Mode + curve hints, Shape Kind + Fill/Outline, Text hints) live in a separate title-bar-less panel glued to it at a 0.25rem (4 pt) gap that CANNOT be detached (drag the Tools window — the panel follows same-frame); parked near the bottom edge the panel flips ABOVE the Tools window; moving the Tools window across the screen center switches left/right edge alignment; the Size slider shows only for Brush/Eraser/Line/Shape (never a disabled one), and Pencil/Eyedropper show NO panel at all. | 2026-11-17 magnet-panel entry (PROGRESS-archive.md) | **verified 2026-11-17 via chat** (owner: "checked the Tools window and its panel, everything works") |
| V30 | 2026-11-17 | Approve the 8 whole-app scenarios ONE BY ONE (in chat): every scenario in the OWNER-LOCKED suite `app/scenarios/tests.rs` carries an `OWNER STATUS: NOT YET CONFIRMED` marker and the header holds the confirmation ledger (0 of 8 approved) — the owner's word flips each marker to CONFIRMED <date>. Before approving, note the three code-corrected steps from the chat round: the BR crop lands the dab at (100,100); scenario 5 needs THREE undos to reach 800×600; scenario 3's fork hangs off the mark's node (see the 2026-11-17 scenario-suite entry in PROGRESS-archive.md). | 2026-11-17 launch-args + markers entry (PROGRESS-archive.md) | **parked by the owner 2026-11-17** (owner: "V30 is on pause, I'm tired of checking it, we're on lighter tasks/edits now") — re-ask only when he returns to it |
| V29 | 2026-11-17 | The launch-arguments pass on the owner's machine (Windows `cargo run`): `cargo run -- --tool brush --primary FF0000 --canvas-size 1024x768` opens with the BRUSH active, the red primary well and a 1024×768 canvas (the Resize dialog prefills 1024×768; the New dialog still offers 800×600 — by design); `russet --help` prints the list and exits WITHOUT opening a window; an invalid value (e.g. `--tool marker`) prints a `russet: ...` warning to stderr and the default stays; `--primary #FF0000AA` sets alpha. | 2026-11-17 launch-args entry | **verified 2026-11-17 via chat** (owner: "checked V29") |
| V28 | 2026-11-17 | The Resize-dialog pass on the owner's machine (Windows `cargo run`; optionally the browser build): (1) Canvas mode — the 3×3 anchor moves the crop/pad correctly (grow AND shrink from every corner/edge/center); (2) Image mode — Nearest/Bilinear/Bicubic/Lanczos3 differ as expected (nearest = hard pixels), the aspect lock links W/H; (3) every layer's History Tree gains ONE `resize` row printing the choices (e.g. "resize 5 canvas 800×600→1000×800 (TL)"), and jumping a layer back beyond it restores that node's NATIVE size — the bottom layer's jump resizes the canvas with it, a non-bottom one just gets clipped; (4) deleting the bottom layer over a DIFFERENT-size successor opens "Canvas Size Source" and BOTH branches behave (Keep = top-left adaptation + its own resize node; Use = the canvas takes the layer's size); a same-size successor switches silently. | 2026-11-17 `Feat:` resize modes entry | **verified 2026-11-17 via chat** (owner: "checked V28 — I tried the Resize dialog, it suits me for now") |
| V26 | 2026-11-16 | The flake DEV build: `nix build .#russet-dev` on the Nix machine compiles in the fast `gates` profile (the repo's gate lane: opt-level 0, no LTO — minutes faster than the release lane) AND still runs the pure unit tests (doCheck), and the produced binary installs and runs: `nix run .#russet-dev` starts the app (UI UNOPTIMIZED — functional checks, not feel); plain `nix build` / `nix run` must stay release, unchanged. | 2026-11-16 `Feat:` flake dev build entry | **verified 2026-11-17 via chat** (owner: "checked nix build .#russet-dev") |
| V8 | 2026-11-12 | MCP on Windows: `cargo build --release --features mcp`, run with `--mcp`, check the "MCP on (N)" badge. | 2026-11-12 `Feat:` MCP entry | open — parked by the owner 2026-11-13: no Windows MCP pass for now; the agent-side MCP run is the working state; reconfirmed 2026-11-13 |

Resolved (kept until the next compaction, then dropped):
- V83, V82, V79, V78, V77, V76 — verified 2026-11-25 via chat (owner batch:
  the Ctrl+A/Ctrl+S pan-tail latch (V83), the Ctrl+A Select All (V82), the
  fill-only-inside-selection gate (V79) and the cut-hole crash guard (V78),
  the fill/wand Tolerance slider (V77), and the wand/fill/Esc round (V76)
  all clicked through on Windows).

- V65 — verified 2026-11-21 via chat (owner: "checked V65, V64, ..." in one batch): the redrawn outline-placement icons read as the outline — thicker segments running past the icon's edge, the click dot untouched.

- V64 — verified 2026-11-21 via chat (the same batch): the Tools window's two-column order reads Pencil+Brush / Eraser+Eyedropper / Select+Text / Shape+Line; hotkeys and tooltips match.

- V63 — verified 2026-11-21 via chat (the same batch): the diameter guides — the dashed chord + the center dot on a Shift drag with the Ellipse kind and the box ON, gone at release, never for Rectangle/Line/no-box.

- V62 — verified 2026-11-21 via chat (owner: "we keep the checkbox, checked") — the verdict is KEEP: the `Diameter circle (Shift)` checkbox STAYS as the Ellipse panel's toggle, overriding the probe design's "when the verdict lands, the checkbox goes away" clause; the TESTING framing is closed.

- V61 — verified 2026-11-21 via chat (the same batch): a partially off-canvas selection stays clipped (the healed shape does not return), fully off-canvas deselects.

- V60 — verified 2026-11-21 via chat (the same batch): the clipboard_shortcuts-off pass — the combos arrive as plain Key events, one op each; Ctrl+S no longer pans; the move drag rides the overlay; the lasso mark is the five-lobe blot.

- V59 — verified 2026-11-21 via chat (the same batch): no per-frame move ghosts; Ctrl+C/X/V copy/cut/paste; the paste lands centered and selects.

- V58 — verified 2026-11-21 via chat (the same batch): the whole Select package — the mode-aware icon, the cycle clicks, the MODE/OPS rows, add/subtract, the move drag, the in-app clipboard, crop + the history rows.

- V57 — verified 2026-11-21 via chat (the same batch): a manually entered size above 100 reaches the drawing on every tool; the bracket keys step from the typed value; the slider stays 1..=100.

- V56 — verified 2026-11-21 via chat (the same batch): the Size block — the label above, the narrowed slider, the number box right of it, the 12-pt gap.

- V53 — verified 2026-11-21 via chat (the same batch): the three outline placements — the corner icons and the Outside/Centered/Inside behavior on rectangles and ellipses.

- V55 — verified 2026-11-20 via chat (the owner opened his next message with
  "Excellent" — no defect reported). His three follow-ups the same message — the
  Size block's rows gap, the label above the slider and the manual entry
  above 100 — opened V56.
- V54 — verified 2026-11-20 via chat (the owner opened his next message with
  "checked V54"). His follow-up the same message — the Line panel's Mode
  radio as icon buttons in the Shape panel's style — opened V55.
- V52 — verified 2026-11-20 via chat (the owner opened his next message with
  "Excellent — now there is a feature-addition task" — an approval of the
  delivered MODE radio, no defect reported). His follow-up the same message —
  the three outline placements — opened V53.
- V51 — verified 2026-11-20 via chat (the owner: "Checked V51"). His follow-up
  the same message — the MODE row as a THREE-way radio over the Shape tool's
  icon — opened V52 (and his "checked V48" the same message closed V48).
- V48 — verified 2026-11-20 via chat (the owner: "checked V48") — the
  compression checklist (file-size-vs-pixels, the locked Lossless rows, the
  reopen persistence) re-run on the newest build, which also covers the V50
  layout note.
- V50 — verified 2026-11-20 via chat (the owner: "checked how the Save window looks —
  far better now" — the layout pass approved). His next request the
  same message — the Shape panel's icon buttons — opened V51.
- V49 — verified 2026-11-20 via chat (the owner: "looked at the captions" — he reviewed
  the captions; no wording complaints). His follow-up on the same dialog — the
  layout/grouping pass — opened V50 the same day.
- V47 — verified 2026-11-20 via chat (the owner: "I checked V47"): the web
  direct-download pass — the Save dialog downloads the image immediately, no
  fallback window, both formats land with the composed names.
- V45 — verified 2026-11-20 via chat (owner checked the pass in BOTH Windows and the
  Web build): the pencil's 1 px trail is an even staircase at any speed, the pattern
  machine-independent, brush/eraser smooth. His remaining platform leg — the same
  checklist on Linux, X11 AND Wayland — opened as V46 the same day.
- V44 — **failed 2026-11-20** (the owner's ragged-pencil report with a screenshot:
  the smoothed pencil drew extra pixels — double-covered columns and holes — and
  the pattern differed between computers): the brush/eraser smoothing itself held
  up; the pencil's per-run Bresenham restarts were the defect, replaced by the
  drifting pen the same day. Superseded by V45 (which re-covers the whole
  smooth-stroke surface on the fixed pencil).
- V43 — verified 2026-11-19 via chat (owner: "checked V43, V42, V41"): the flake's
  trunk lanes re-passed on NixOS after the four same-day fixes. NOTE: V43's row had
  never landed in the open table (the flake round's bookkeeping gap — its log entry
  said "Open: V43" but no row existed here); closed by the owner's word this session.
- V42 — verified 2026-11-19 via chat (owner: "checked V43, V42, V41"): the Save
  Image dialog pass on Windows.
- V41 — verified 2026-11-19 via chat (owner: "checked V43, V42, V41"): the lenient
  hex pass on Windows.
- V39 — verified 2026-11-19 via chat (owner: "checked V39"): the header pass
  clicked through on Windows — the title strip fits the window (variant A2),
  both the ▼ and the ✕ kept.
- V38 — verified 2026-11-19 via chat (owner: "checked V38"): the collapse-hide
  pass clicked through on Windows — the panel vanishes with the collapsed
  window and returns on expand.
- V36 — verified 2026-11-19 via chat (owner: "checked the V36 icons and the
  text-less header too"): the icons pass AND the v5 label-less header both
  checked; the owner's follow-up (the strip still overflows the window) became
  the header task and V39.
- V1, V2, V3 — verified 2026-11-13 via chat (owner: "verified V1 V2 V3"): the six
  tools, the six UX-fix items, the §3.11 magnetize/shadow pass — all clicked through
  on Windows.
- V5, V6 — verified 2026-11-13 via chat: the owner tried the History Tree by hand and
  is satisfied for now ("tried it by hand, it suits me for now"). Improvement
  ideas may come later — those will be NEW tasks, not reopened checks.
- V10 — closed 2026-11-13, owner decision: he builds/runs on Nix periodically himself
  ("almost everything is fine") — not a debt, stop reminding.
- V11 — verified 2026-11-13 via chat (About window checked on Windows).
- V12 — verified 2026-11-13 via chat, partial read: the owner skimmed the docs;
  development is stable and good, no wording fixes requested.
- V4, V7 — verified 2026-11-13 via chat (owner: "checked V4, V7"): one `cargo run`
  each — after the toolchain pin + `gates` profile and after the file splits; behaves
  exactly as before.
- V13, V14, V17, V18, V19, V20, V21, V22 — verified 2026-11-16 via chat (owner:
  "checked V22, V21, V20, V19, V18, V17, V14, V13"): the real-layers / History-tree
  click-through, the Layers-window width + white initial layer, the Tool Log restyle,
  the dialog button rows, the centered content-hugging dialogs, Shape RMB's inverted
  wells without a swap, the two-color history chips, the Shift-proportional shapes.
- V15 — checked 2026-11-16 via chat: the ×2 bump works, but the owner wants wheel pan
  ANOTHER ×5 faster — the follow-up is born as F11 in DEBTS-functional.md (owner:
  "record it as something to do later"); the check itself is closed.
- V9 — verified 2026-11-16 via chat (owner: "checked V9"): the owner clicked through
  the web build in a browser with a real mouse — draw, zoom, pan, Open/Save; the pass
  surfaced two platform gaps, born as F12 (wheel-pan parity) and F13 (floating-window
  jitter on main-window resize) in DEBTS-functional.md — the M8 human check itself is
  done.
- V16 — closed 2026-11-15 (superseded by the owner's own request: Shape RMB no longer
  swaps the wells at all; the replacement behavior was born as V19 and verified
  2026-11-16). The closed row sat in the open table; it moved here on 2026-11-16 when
  the debt-ordering rule (AGENTS.md "Debts" rule 6) landed — bookkeeping only.
- V23, V24 — verified 2026-11-16 via chat (owner: "checked V23, V24"): the F12-B
  wheel-pan feel (one notch = the same fixed 15 pt on Windows, Firefox after the
  Line÷3 retune, Chrome, and the Linux/Wayland leg — testable only once the winit
  Adwaita CSD frame made the Linux window usable) and the F13-A resize pass (no
  floating-window jitter on Wayland, Windows unchanged). V24's pass satisfied F13's
  documented last pending condition → F13 paid in DEBTS-functional.md the same edit;
  F12 stays open as the final-design decision.
- V27, V25 — verified 2026-11-17 via chat (owner: "checked V27, V25"): V27 — the release
  web build after the `strip` fix: the owner's local `trunk build --release` logs NO
  wasm-bindgen `target_features` warning and the release page passed the browser smoke
  (page loads, a stroke draws, zoom/pan); V25 — the GNOME Wayland pass of the winit
  Adwaita CSD frame (title text follows New/Open/Save/dirty, double-click
  maximize/restore, close/minimize buttons, light and dark shell themes). Nothing else
  rode on these two — V23/V24 and F13 were already closed 2026-11-16.
