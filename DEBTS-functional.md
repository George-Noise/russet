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

Owner reconfirmation 2026-11-13: F3, F8, F9, F10 stay open as is («остальное — не сейчас»).

Paid / closed (kept until the next compaction, then dropped):

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
