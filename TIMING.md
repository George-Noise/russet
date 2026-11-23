# TIMING — durations of the heavy agent commands in the agent sandbox

> **Why this file exists** (owner request 2026-11-17): the agent waits on long
> commands in bounded `sleep` chunks (AGENTS.md "Long-running and background
> commands"); sleeps much shorter than the real duration waste a step per poll,
> sleeps much longer stall the task. This log gives the agent the LAST MEASURED
> duration to pick the sleep chunk from. **Keep 5 newest rows per table** (drop
> the oldest at the next compaction, like PROGRESS.md).
>
> **Cold** = the command had no usable `target/` for its lane (fresh clone,
> recycled sandbox without `target/`, first run of a new profile/target triple)
> or the toolchain itself had to be fetched. **Hot** = `target/` carried over.
> `target/` normally persists with the project directory. The warm-target store
> (a hosted `russet-target.zip`) was RETIRED by the owner 2026-11-17 ("drop the
> warm-target idea for now — dev builds fast enough for tests"); the cold
> numbers below are the accepted cost of a fresh sandbox.
>
> Rule: **every** heavy command (> ~1 min) gets a row in the same edit that ran
> it, with HOW it was measured (`time`d exactly vs observed from a background
> log ≈). One command per row; never merge rows.

## Cold

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-23 | detached gates-lane warm-up (`cargo clippy --profile gates --all-targets -- -D warnings` + `cargo test --profile gates --no-run`) on the FRESH GitHub clone — rustup + the pinned 1.99.1 toolchain installed first; the check's own first warm-up attempt silently no-op'd on the missing rustup (PROBLEMS.md 2026-11-23) and was restarted manually | **clippy 1m 32s ≈, test --no-run ≈ 1 min** | the warm-up log ≈ |
| 2026-11-21 | detached gates-lane warm-up (`cargo clippy --profile gates --all-targets --features mcp` + `cargo test --profile gates --features mcp --no-run`): the whole lane COLD on the fresh GitHub clone — rustup + the pinned 1.99.1 toolchain installed first | **≈ 3 min total (clippy 1m 30s)** | the warm-up log's timestamps (started 07:29:25, finished 07:32:21) |
| 2026-11-21 | `cargo check` (this round's quick loop — COLD dev-lane dep graph on the fresh clone) | **1m 20s** | cargo's print |
| 2026-11-21 | targeted `cargo test app::preview --profile gates` + `cargo test app::windows::tests::settings_panel --profile gates` (the first compiled the COLD gates test graph, the second ran hot) | **≈ 2–3 min total** | the tool-call wall clock ≈ (not time'd) |
| 2026-11-20 | targeted `cargo test app::windows --profile gates` (the panel-spacing round: the gates test graph COLD in the fresh sandbox — first compile died on a fmt-broken signature, the hot re-run was instant) | **1m 59s** | time'd exactly |

## Hot

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-23 | `cargo build` (the dev GUI binary for the mouse-side probe; warm deps, the crate itself recompiled unoptimized) | **1m 32s** | `time`d exactly |
| 2026-11-21 | `cargo build --features mcp` (the dev MCP binary for the driver; warm deps, the crate itself recompiled unoptimized) | **1m 35s** | cargo's print |
| 2026-11-21 | `bash scripts/verify.sh` (the diameter-circle round's full gate — the gates lane warm from the detached warm-up) | **19.7 s** | time'd exactly |
| 2026-11-21 | NOTE: THIS recycled sandbox KEPT the detached warm-up alive across tool calls (started in one call, finished ~3 min later, verified hot by the next call's targeted tests) — the 2026-11-20 NOTE below is per-sandbox; re-probe detached survival each session | — | the warm-up log |
| 2026-11-20 | `bash scripts/verify.sh` (the V60-fixes round's full gate — rules + fmt + clippy ×2 + test ×2; warm deps) | **20 s** | the gate's own print |
| 2026-11-20 | `cargo build --features mcp` (the dev MCP binary for the driver; warm deps) | **2 s** | cargo's print |
| 2026-11-20 | NOTE: DETACHED `setsid` background processes do NOT survive between tool calls in THIS recycled sandbox — both the gates warm-up and a detached verify.sh died silently between commands; heavy commands must run SYNCHRONOUSLY here (the compiled fingerprints persist, so the sync re-run is hot) | — | observed twice |

## Reading the numbers

- The done-gate (`verify.sh`) after an ACTUAL code change recompiles the
  `russet` crate itself per stage (clippy, test, mcp-clippy, mcp-test — deps
  stay cached): a small change (a couple of files) costs ~20 s total; bigger
  change sets cost proportionally more (the magnet round: clippy 59 s, tests
  ≈ 3–4 min). ~10 s means cargo saw nothing new.
- A COLD session (fresh sandbox, no `target/`) costs ~15–20 min of dep
  compilation before the first useful command; the store that used to skip it
  is retired (owner decision 2026-11-17) — a warm `target/` carried over from
  the previous session gives the hot numbers above. Since owner decision
  2026-11-18 the dev profile is FULLY unoptimized, so the dep-compile share
  shrank (first measured point: cold `cargo check` 1m31s vs 3m51s optimized,
  same morning — see the cold rows).
- Sleep chunk guidance: start with ~1/3 of the last measured duration, then
  poll in ~60–120 s chunks. Never poll more often than needed (every poll is a
  step).
