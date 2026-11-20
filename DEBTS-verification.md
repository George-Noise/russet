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
| V53 | 2026-11-20 | The Shape tool's outline-PLACEMENT pass (Windows `cargo run` AND the web build): with the Shape tool active the panel shows THREE groups separated by gaps — the KIND pair, the paint MODE radio and the new PLACEMENT radio of THREE corner icons that ALWAYS draw a square's corner (whatever kind is selected): the gray fill quadrant (where the shape is), the click dot (a white core with the ink ring) at the corner and two ink segments along the outline's lines; the bands' position tells the modes apart — Outline outside (the bands fully beyond the dot), Outline centered (straddling the dot — the old look, the startup default) and Outline inside (the bands within the fill); drawing matches: Outside's band lies beyond the two mouse points (the footprint grows by the stroke width), Inside's outer edge IS the two points, Centered straddles half-half; the on-canvas preview matches the commit for rectangles AND ellipses in all three modes; the Fill-only paint keeps the row clickable (the choice is remembered for the next outlined shape). | 2026-11-20 outline-placement entry | open |
| V46 | 2026-11-20 | The smooth-stroke Linux pass (NixOS, BOTH backends — X11 AND Wayland), the flake's native lanes (`nix run` release / `nix run .#russet-dev`): draw SLOW and FAST diagonals and curves with the pencil — the 1 px trail is an EVEN staircase, the SAME pattern for the same gesture regardless of speed or machine; fast strokes stay smooth on the pencil, brush and eraser (no polygon runs); translucent colors do not darken at the joints; a single click sets exactly one pixel; the stroke ends exactly on the release pixel; undo still records one node per gesture. Run both backends: the Wayland session AND X11 (an X11 session or `WINIT_UNIX_BACKEND=x11`). | 2026-11-20 owner request in chat (V45's pass covered Windows + Web; the Linux legs stayed unchecked) | open |
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

- V52 — verified 2026-11-20 via chat (the owner opened his next message with
  «Отлично, теперь есть задачка на добавление функционала» — an approval of the
  delivered MODE radio, no defect reported). His follow-up the same message —
  the three outline placements — opened V53.
- V51 — verified 2026-11-20 via chat (the owner: "Проверил V51"). His follow-up
  the same message — the MODE row as a THREE-way radio over the Shape tool's
  icon — opened V52 (and his "V48 проверил" the same message closed V48).
- V48 — verified 2026-11-20 via chat (the owner: "V48 проверил") — the
  compression checklist (file-size-vs-pixels, the locked Lossless rows, the
  reopen persistence) re-run on the newest build, which also covers the V50
  layout note.
- V50 — verified 2026-11-20 via chat (the owner: "Проверил как выглядит окошко
  Save, сейчас гораздо лучше" — the layout pass approved). His next request the
  same message — the Shape panel's icon buttons — opened V51.
- V49 — verified 2026-11-20 via chat (the owner: "надписи посмотрел" — he reviewed
  the captions; no wording complaints). His follow-up on the same dialog — the
  layout/grouping pass — opened V50 the same day.
- V47 — verified 2026-11-20 via chat (the owner: "я проверил V47"): the web
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
