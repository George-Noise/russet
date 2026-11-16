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
> - `PERF.md` stays the separate owner-gated backlog for RAM/build-speed work;
>   the pointer below is the only cross-reference.

| # | Opened | Debt | Why it exists | Pay when |
|---|---|---|---|---|
| F3 | 2026-11-11 | CI has no wasm/trunk job. | Deliberately postponed by the owner "until the web build has something to be shown". | After M8 gets its human pass (V9) or the owner says so. |
| F5 | 2026-11-10 | PERF.md decisions pending: P3 (region-based undo — rewrites SPEC §3.6) and P6 (canvas/history budgets on wasm). | Owner-gated backlog; this line is a POINTER, the details live in PERF.md §0/§6. | Owner decision 2026-11-13: parked for the long term («долгий ящик») — during active development these are not needed; the owner picks them from PERF.md when he says so. |
| F8 | 2026-11-13 | History nodes have NO eviction/limits (owner decision 2026-11-13: "без лимита"): every node keeps its full PNG snapshot forever, so RAM grows linearly with node count. | The eviction-by-memory policy needs the action model (there is nothing to rebuild from yet — nodes are pure snapshots). | When action replay lands (the design doc's §6 policy). |
| F9 | 2026-11-13 | Resize Canvas policy is the interim "keep the nodes; old-size PNGs are crop/pad fitted on restore" (owner decision 2026-11-13, with his wording: click an old-size node shows the cropped part; continuing to draw bases the next node on the fitted piece). | Resize + history is genuinely hard; the owner asked for "хоть какое-то решение для начала". | When the design doc's open question #9 is settled for the action model. |
| F10 | 2026-11-13 | Undo/redo act on the ACTIVE layer only; the design doc's cross-layer hint ("последний штрих был на слое …", §8 / open question #3) is not implemented. | Owner's staged rollout: "реализовывать большую фичу по чуть-чуть". | When the owner picks an answer to the design doc's open question #3. |
| F12 | 2026-11-16 | Wheel-pan PLATFORM PARITY: one physical wheel notch pans different distances per platform — Windows slowest, Wayland and Web much faster (owner report after the V9 pass). Root cause: `app/camera.rs::handle_scroll` reads the RAW `Event::MouseWheel.delta` in its EVENT UNIT and ignores the `unit` field: egui 0.37.0 passes the backend delta UNCONVERTED (unit Line/Point/Page); winit 0.30.13 sends `LineDelta(±1)` per notch on Windows/X11 (→ pan = ±1 × `SCROLL_PAN_SPEED` = ±3 pt), eframe-web passes the DOM wheel PIXELS as-is with `unit: Point` (Chrome ≈ 100 px per notch → ±300 pt), winit-Wayland sends discrete `LineDelta(±1)` only when the compositor emits `wl_pointer.axis_discrete`, otherwise `PixelDelta` (physical px / ppp). egui's own `smooth_scroll_delta` is NOT a full fix: egui itself uses web=8 vs native=40 points per line (TODO(emilk) in the 0.37 sources). SPEC §3.4's «40 pt per wheel notch» described egui's INTERNAL conversion, not the raw delta russet read — §3.4 rewritten with the fix. | Owner report 2026-11-16: «непонятно где правильнее scroll спроектирован, надо думать». Owner pick 2026-11-16: variant B, INTERIM («выбираем B но с пометкой что это не окончательный вариант»; F11 учтён в расчётах — он писался после проверки на Windows) — IMPLEMENTED the same day: `app/camera.rs::wheel_notches` (pure, unit-tested) converts the delta to PRODUCT NOTCHES per its `unit` (Line pass-through; Point ÷ 100 pt; Page × 3) and every notch pans the FIXED `WHEEL_NOTCH_PAN` 15 pt on EVERY platform (`WHEEL_NOTCH_PAN` in `src/app.rs`, F11 folded: the Windows ±3 pt × 5). NOT FINAL — F12 STAYS OPEN: the web-Line divisor (3), the Point-unit divisor (100) and the Page cap (3) are interim constants, retuned from `--features diagnostics` wheel logs on the owner's machines; the final design stays owner-gated; options A (honor unit, no parity) and C (sign only) remain fallbacks. Owner pass 2026-11-16 (Windows/Firefox/Chrome): Windows + Chrome approved, Firefox overshot ~3× — Firefox emits `DOM_DELTA_LINE` with 3 lines per click and eframe passes the DOM delta as-is, so a `Line`-unit delta arrived as 3 notches; the web-Line divisor (`WEB_LINE_UNIT_LINES_PER_NOTCH` 3.0; `wheel_notches` gained a `web_backend` arg = `cfg!(target_arch = "wasm32")`, native stays pass-through) added the same day. The feature was renamed `diag` → `diagnostics` the same day (owner: the abbreviation was unclear — not everyone sees it is an abbreviation). |
| F13 | 2026-11-16 | Floating windows JITTER during main-window resize on Linux (Wayland) and Web; Windows behaves. `placement.rs::place_floating_window` used to reposition on EVERY frame whose `content_rect` changed, CHAINED off the previous frame's drawn rect (written via `set_window_pos`, applied a frame late): on Wayland/Web the resize arrives as a series of intermediate, sometimes non-monotonic content rects (double-buffered configure events; canvas resize + devicePixelRatio on the web), and the auto-sized Tools/Palette windows may change their own size in the same frames — the chain accumulated the drift and showed as «колбасит». | Owner report 2026-11-16 after the V9 pass. Owner pick 2026-11-16: variant (a), reference-based repositioning («вариант A мне нравится») — IMPLEMENTED the same day: the pure `resize_step` (3 tests) pins the PRE-resize `(window rect, content rect)` on the FIRST changed frame and recomputes EVERY resize frame from the pin (non-monotonic series cannot accumulate drift); the pin clears on the first stable frame and on title-bar drags. The step-0 diagnostic log ships behind the TEMPORARY `diagnostics` cargo feature (renamed from `diag` 2026-11-16, owner request; default build compiles none of it; native `cargo run --features diagnostics`, web `trunk serve --features diagnostics` → browser console): `[diagnostics] main resize …` + `[diagnostics] place …` lines. F13 STAYS OPEN pending the owner's Wayland feel pass (V24 — Windows, Firefox and Chrome verified clean 2026-11-16, Linux not tested yet); the `diagnostics` feature dies with the last of F12/F13. |

Owner reconfirmation 2026-11-13: F3, F8, F9, F10 stay open as is («остальное — не сейчас»).
Owner hand-check 2026-11-13: F10's behavior — undo/redo act on the ACTIVE layer only — was
verified by the owner manually («это я проверил руками»): it matches this documented
simplification, nothing to fix on the spot; the debt stays open until the design doc's
open question #3 is answered.

Paid / closed (kept until the next compaction, then dropped):

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
- F7 (the color-picker popup keeps its drop shadow) — **closed 2026-11-13, owner decision**: "с тенями пока всё" — the popup shadow stays as is; not tracked anymore.
- F2 (the History window is a read-only action log; the old "interactive undo-to-state"
  promise) — **closed 2026-11-13, owner decision**: the window stays AS IS and is not
  tracked anymore; when it is no longer needed, the owner will say himself what to do
  with it. Do not remind.
