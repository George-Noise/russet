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
| F1 | 2026-11-10 | The Layers window is a placeholder: exactly one "Background" row with a live 28×28 thumbnail, nothing interactive. | Owner ordered the six tools "простенько" for a first iteration; real layers are a feature task of their own. | The owner asks for real layers. |
| F2 | 2026-11-10 | The History window is a read-only action log; "interactive undo-to-state will replace it later" was the original promise. | Same first-iteration batch; the visual History Tree windows (2026-11-12) may have superseded the idea. | Owner decides: interactive History log, or remove/keep read-only. |
| F3 | 2026-11-11 | CI has no wasm/trunk job. | Deliberately postponed by the owner "until the web build has something to be shown". | After M8 gets its human pass (V9) or the owner says so. |
| F4 | 2026-11-12 | History Tree pseudo-layers: interleaved brush/eraser strokes across layers are a known prototype caveat (pencil-only sessions stay exact; commented at `App::undo`). | The tree mirrors the real History only where the prototype's assumptions hold. | When layers become real (F1) or the owner hits the caveat. |
| F5 | 2026-11-10 | PERF.md decisions pending: P3 (region-based undo — rewrites SPEC §3.6) and P6 (canvas/history budgets on wasm). | Owner-gated backlog; this line is a POINTER, the details live in PERF.md §0/§6. | Owner picks them from PERF.md. |
| F6 | 2026-11-10 | Round line caps in Straight/Curve previews are drawn as endpoint circles — a stopgap "until the owner gets a cap choice". | egui 0.37 has no line-cap option (verified in epaint). | Owner wants a cap choice; otherwise the circles stay. |
| F7 | 2026-11-11 | The color-picker popup still has a drop shadow while every window lost theirs. | Shadows were switched off app-wide; popups/menus were deliberately untouched. | Owner decides: drop it too, or keep popups shaded. |

Paid (kept until the next compaction, then dropped):

- 2026-11-13 — CI did not run the `mcp` feature gates (the code compiled only in manual builds and could rot unseen) — **paid 2026-11-13**: clippy+test with `--features mcp` added to the CI `native` job (owner OK 2026-11-13) and already gated in `verify.sh` since 2026-11-13.
- 2026-11-13 — keep-or-drop question for the two python helpers in `scripts/` — **paid 2026-11-13**: kept; `visual_driver.py` is now the reusable scenario runner (owner request 2026-11-13).
