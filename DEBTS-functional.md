# DEBTS-functional — deliberate simplifications, postponed work, open decisions

> **Why this file exists.** Functional debts are things the code does NOT do (yet)
> on purpose: first-iteration placeholders, postponed jobs, decisions the owner
> still owes. They used to be scattered across `PROGRESS.md` entries and SPEC
> notes; this file is the single live list (owner decision 2026-11-13).
>
> **Rules (AGENTS.md "Debts"):**
> - Paying a debt is the OWNER's decision: this is a reference list, not an
>   autonomous work queue (same engagement rules as `PERF.md` §0).
> - The agent surfaces open items at session start and at task end, in ONE
>   batched list together with `DEBTS-verification.md`.
> - A debt is born in the SAME edit as the task that creates it. A paid debt
>   gets `Status: paid <date>` and is dropped at the next compaction.
> - Open items are sorted: V checks first, then F debts (this file), then
>   anything else — NEWEST birth date first within a group (owner request
>   2026-11-16, AGENTS.md "Debts" rule 6). A paid item moves to the Paid list
>   in the same edit.
> - `PERF.md` stays the separate owner-gated backlog for RAM/build-speed work;
>   the pointer below is the only cross-reference.

| # | Opened | Debt | Why it exists | Pay when |
|---|---|---|---|---|
| F22 | 2026-11-23 | The wand/fill flood TOLERANCE is a FIXED shared const (`selection.rs::FLOOD_TOLERANCE` = 32, per-channel against the SEED's color) with NO user-facing control — Paint.NET's wand and bucket carry a tolerance slider (0–100%), russet's algorithm was ordered simple ("a simple algorithm is needed") and shipped without a knob. A future knob = a settings row (the Select panel's wand cell or the Fill panel — currently the Fill panel shows NOTHING), a ToolState field, both tools reading it; the seed-reference rule and the 4-neighborhood stay unchanged. | The owner asked for the SIMPLE algorithm first; the slider is a scope choice recorded at birth, not an omission. | Status: paid 2026-11-23 (the owner asked: the tolerance slider in the magnetized panel, in %) |
| F20 | 2026-11-21 | INPUT DEVICES BEYOND THE DESKTOP MOUSE: the WEB slice is now PAID (owner request 2026-11-25, implemented the same day): multi-touch PAN in the web build — two or more fingers pan by the average movement (egui `multi_touch().translation_delta`, `app/camera.rs::TouchPan`; an in-flight stroke commits as its node when the gesture begins; the session stays pan-only until the last finger lifts) — plus the SMARTPHONE compact rules for the phone-sized web viewport (§3.11: every floating window first-open FLUSH 0, the Tool Log window and its View item hidden; the gate `main.rs::is_smartphone_viewport`, 600 pt). NO native Android build is planned (owner 2026-11-25). STILL OPEN: trackpads (two-finger pan, pinch zoom), pen/graphics tablets (pressure? eraser end? per-app mapping), and native touchscreens beyond what egui reports for free — the phone/tablet WEB surface and the native touchscreen finger-pan now ride the same code, the rest is unhandled input. | The owner's own parking note at birth; the 2026-11-25 slice was his explicit unpick. | When the owner returns to input devices, or a mobile/touch build makes the surfaces real. |
| F18 | 2026-11-20 | The selection's clipboard is IN-APP (not the OS image clipboard — Paint.NET can paste into other apps, russet cannot yet) AND paste stamps immediately (centered in the viewport) instead of leaving a FLOATING selection the user drags before committing. The selection itself is not part of history (undo restores the pixels of a move/cut/paste/crop, but the selection BOUNDS are plain tool state). All three are deliberate v1 simplifications of §3.13, recorded at birth. | The round's scope was the full select/move/copy/cut/paste/crop loop on the app's own canvas; the OS clipboard interop and the floating-paste state machine are separate pieces of machinery (platform APIs / an overlay pipeline) that would have doubled the round. | When the owner asks for OS-clipboard interop, drag-after-paste, or undoable selection bounds — each is an owner-gated follow-up. |
| F17 | 2026-11-19 | CONSIDER other file formats for Open and Save beyond PNG/JPEG (owner request 2026-11-19). OPTIONS NOTE (the owner asked for it 2026-11-25; facts verified against the PINNED image 0.25.10 source): **BMP** — the `bmp` feature flag on the pinned crate alone, ZERO new dependencies (the codec is in-crate pure Rust), full decode AND encode (Rgb8/Rgba8, no options row needed — a quality knob does not apply); **WebP** — the `webp` feature adds exactly ONE pure-Rust crate (`image-webp` 0.2): full DECODE (lossy + lossless) and LOSSLESS-ONLY ENCODE (no quality slider possible — lossy saving needs the C-libwebp `webp` crate, which is wasm-hostile for the web build and would bring native libs + flake runtime); a Save-dialog radio for either also touches `save_dialog.rs::compose_file_name`'s extension rules, the Open filter list in `file_io.rs::open_image`, and the MCP export's path-extension mapping. Which formats make the cut for OPEN, which for SAVE, and any per-save-format options row is an owner decision. | The owner asked to RECORD the consideration; the Save dialog shipped with PNG/JPEG as its scope. | When the owner names the formats he wants (or says "evaluate and propose" — done, see the note; the code decision is his). |
| F16 | 2026-11-18 | CI rust-cache is NOT installed (owner decision 2026-11-18: not now — recorded as a POSSIBLE IMPROVEMENT for when CI speed matters): when the time comes, add ONE step `uses: Swatinem/rust-cache@v2` to the `native` job right after `actions/checkout` in `.github/workflows/ci.yml`. How it works: the runner is a CLEAN VM per push today, so cargo downloads and compiles every dependency each time (~3–4 min); the action restores `~/.cargo` (downloaded dependency sources) and `target/` keyed on `Cargo.lock` + `Cargo.toml` + the toolchain version — unchanged deps are instant, only the crate itself rebuilds (the full `native` job drops from ~4 min to ~1 min). Facts to know: the first run after adding it stays cold (the cache builds up after), cache ≤ 10 GB per repo (ours fits easily), an occasional eviction costs one cold run, no config beyond the one line (the action locates `~/.cargo` and `target/` itself). | Nothing is broken and CI latency does not hurt yet; the improvement only pays when it does. | When the owner says CI is too slow and approves the edit (a ci.yml change is build tooling — needs the owner's OK per AGENTS.md "Repo facts"; one-line diff, owner-reviewed). |
| F15 | 2026-11-18 | The Tools window's TWO-COLUMN icon grid is INTERIM (owner decision 2026-11-18, owner: "two columns for now — it will be reworked later, because I have a better idea, but I'm not up to it now"): the grid stays the configurable `TOOL_COLUMNS = 2` (`app/windows.rs`) until the owner redesigns the tools layout. The icon buttons themselves (1.5rem, cached tiny-skia atlas — SPEC §3.2) are NOT part of the debt. | The owner explicitly deferred the better layout; the interim grid is growth-ready (a configurable column count, the original §3.2 rule). | When the owner describes the better idea and says "do it now" (owner-gated, like F14). |
| F14 | 2026-11-18 | Replace `egui::Window` with an IN-HOUSE window solution (owner decision 2026-11-18, owner: "record it in the debts — replace egui::Window with our own solution, we'll do it later"): a hand-rolled frame over `egui::Area` with its own title bar, dragging, z-order and close buttons, replacing ALL floating windows and dialogs. The decision's pros/cons summary (chat review 2026-11-18): pros — full look/behavior control, freedom from the recorded `egui::Window` quirks (340×420 default outer size, first-pass `available_width`, non-shrinking `desired_size`, `default_pos` consumption rules, the private `"__title_click"` id), trivial re-centering without the `forget_dialog_position` reset; cons — reimplementing by hand what the window gives for free (drag, z-order/focus, input occlusion — clicks must not paint the canvas, resize, AccessKit nodes), ~9 `Window::new` call sites in 6 files, the 1024-line §3.11 placement glue plus `dialogs/tests.rs` / `windows/tests.rs` / scenarios coupled to `egui::Window` behavior — a wide regression surface for zero user-facing pain today. The ALTERNATIVE (a frameless MAIN window with an egui-drawn title bar) was considered and REJECTED for now: the native WM integration loss (edge resize, half-screen snap, tiling, Alt+Tab) is not worth it, and the Wayland decoration pain is already solved by the `wayland-csd-adwaita` AdwaitaFrame. | Nothing is broken today: the helper layer (`dialog_window` / `dialog_button_row` / `forget_dialog_position` / `place_floating_window`) already absorbs the recorded quirks; the job is pure control/polish, deliberately deferred by the owner. | Owner-gated: when the owner says "do it now", or a window need appears that `egui::Window` cannot express (docking/tabs/snap-grid), or a visual redesign pass. |
| F8 | 2026-11-13 | History nodes have NO eviction/limits (owner decision 2026-11-13: "no limits"): every node keeps its full PNG snapshot forever, so RAM grows linearly with node count. | The eviction-by-memory policy needs the action model (there is nothing to rebuild from yet — nodes are pure snapshots). | When action replay lands (the design doc's §6 policy). |
| F9 | 2026-11-13 | Resize Canvas policy is the interim "keep the nodes; old-size PNGs are crop/pad fitted on restore" (owner decision 2026-11-13, with his wording: click an old-size node shows the cropped part; continuing to draw bases the next node on the fitted piece). UPDATE 2026-11-17 (owner's resize-feature request): the crop/pad-fit restore is SUPERSEDED — restores keep the node's NATIVE size now (SPEC §3.6), the bottom layer owns the canvas size, and resize commits one node per layer with the full dialog choices; the underlying open question #9 (the action model) is untouched. | Resize + history is genuinely hard; the owner asked for "at least some solution to start with". | When the design doc's open question #9 is settled for the action model. |
| F3 | 2026-11-11 | CI has no wasm/trunk job. | Deliberately postponed by the owner "until the web build has something to be shown". | After M8 gets its human pass (V9) or the owner says so. |
| F5 | 2026-11-10 | PERF.md decisions pending: P3 (region-based undo — rewrites SPEC §3.6) and P6 (canvas/history budgets on wasm). | Owner-gated backlog; this line is a POINTER, the details live in PERF.md §0/§6. | Owner decision 2026-11-13: parked for the long term ("long-term parking") — during active development these are not needed; the owner picks them from PERF.md when he says so. |

Owner reconfirmation 2026-11-21: F5 reconfirmed parked ("parked for later — all the features need implementing first").
Owner reconfirmation 2026-11-13: F3, F8, F9, F10 stay open as is ("the rest — not now").
Owner hand-check 2026-11-13: F10's behavior — undo/redo act on the ACTIVE layer only — was
verified by the owner manually ("I verified this by hand"): it matches this documented
simplification, nothing to fix on the spot; the debt stays open until the design doc's
open question #3 is answered.

Paid / closed (kept until the next compaction, then dropped):

- F21 (the native dialogs froze the window — the owner's GNOME screenshot
  «"Russet" Is Not Responding») — **paid 2026-11-21**: the owner picked
  variant A in chat ("We go with Variant A"). One correction to the debt's birth
  record: the pinned rfd 0.15.4 uses the xdg-portal backend on Linux (NOT a
  nested GTK loop) — the freeze was `pollster::block_on` running the portal
  round-trip ON the UI thread. The fix: both native pickers (File ▸ Open
  with its decode; the Save dialog's Browse… AND its Save-click fallback)
  run on a worker thread, the frame polls the shared `FileTask` slot
  (`file_io.rs::FileTask` + `app/document.rs::poll_file_task`; the web
  `AsyncFileDialog` rides the same slot, `WebTask`/`WebOutcome` folded in);
  the single-dialog guard rejects a second chooser (Open…/Browse… gray);
  the Save-click fallback continues automatically when its pick answers.
  No task in flight = zero machinery. +4 tests; V67 born (the human pass).

- F19 (the AccessKit trim decision) — **paid 2026-11-21**: the owner approved
  and applied the trim ("the AccessKit removal went smoothly") — the `accesskit`
  feature is OFF the eframe feature list (the owner's local commit "Build:
  AccessKit removed from the eframe feature list"; the sandbox tree already
  carries it). The audit's other candidates (`system_fonts` ≈126 edges ~16%,
  `links`) stay recorded in the archived entry for a future size round.
- F12 (the wheel-pan parity final design) — **paid 2026-11-21**: the owner
  closed the design ("there are differences across systems but they are minor —
  we keep it as is") — the interim constants ARE final, no retune owed.
  The `diagnostics` feature's last consumer is gone; dropping it is now a
  possible cleanup on the owner's word (nothing scheduled). The input
  surfaces the owner asked to record during the closure (touch, tablets,
  touchpad, graphics tablet) opened as F20 below.
- F10 (undo/redo act on the ACTIVE layer only) — **paid 2026-11-21**: the
  owner accepts the active-layer behavior as the answer for now ("it is
  already implemented and good enough for now"); the design doc's open
  question #3 stays a design note, not a tracked work item.

- F13 (floating windows JITTER during main-window resize on Linux/Wayland and Web) —
  **paid 2026-11-16**: the F13-A reference-based repositioning (`resize_step`) was
  implemented the day the debt was born; the ONLY pending condition in the entry was the
  owner's Wayland feel pass (V24), and the owner verified V23 + V24 via chat 2026-11-16
  ("checked V23, V24") — no jitter on Wayland, Windows unchanged. The temporary
  `diagnostics` feature stays until F12 closes (its last consumer).
- F11 (wheel pan ANOTHER ×5 faster, plain wheel = vertical pan, Shift+wheel = horizontal) —
  **paid 2026-11-16**: folded into the F12 variant-B fix — the fixed notch is
  `WHEEL_NOTCH_PAN` 15 pt (`src/app.rs`) = the pre-fix Windows notch ±3 pt (`SCROLL_PAN_SPEED`
  3.0, the speed the owner calibrated the ×5 on) × 5. The literal how-to (bump
  `SCROLL_PAN_SPEED` to 15.0) is superseded by the notch conversion
  (`app/camera.rs::wheel_notches`); the feel verification rides with V23 in
  DEBTS-verification.md.
- 2026-11-13 — CI did not run the `mcp` feature gates (the code compiled only in manual builds and could rot unseen) — **paid 2026-11-13**: clippy+test with `--features mcp` added to the CI `native` job (owner OK 2026-11-13) and already gated in `verify.sh` since 2026-11-13.
- 2026-11-13 — keep-or-drop question for the two python helpers in `scripts/` — **paid 2026-11-13**: kept; `visual_driver.py` is now the reusable scenario runner (owner request 2026-11-13).
- F1 (the Layers window is a placeholder) — **paid 2026-11-13**: REAL layers + the interactive Layers window (select/rename/visibility/reorder/delete-with-confirm) landed together with the per-layer PNG-snapshot history trees (owner request 2026-11-13); SPEC §3.6 rewritten.
- F4 (History Tree pseudo-layers caveat: interleaved brush/eraser strokes shift the mirror) — **paid 2026-11-13**: the tree IS the history now (one PNG canvas per node, every tool records), and each layer owns its tree, so the caveat cannot occur.
- F6 (round line caps in Straight/Curve previews drawn as endpoint circles) — **closed 2026-11-13, owner decision**: the instrument works well enough for the current project state; the circles stay and the item is no longer tracked. Revisit only if the owner asks for a cap choice.
- F7 (the color-picker popup keeps its drop shadow) — **closed 2026-11-13, owner decision**: "shadows are fine for now" — the popup shadow stays as is; not tracked anymore.
- F2 (the History window is a read-only action log; the old "interactive undo-to-state"
  promise) — **closed 2026-11-13, owner decision**: the window stays AS IS and is not
  tracked anymore; when it is no longer needed, the owner will say himself what to do
  with it. Do not remind.
