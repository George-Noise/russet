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
| 2026-11-20 | targeted `cargo test app::windows --profile gates` (the panel-spacing round: the gates test graph COLD in the fresh sandbox — first compile died on a fmt-broken signature, the hot re-run was instant) | **1m 59s** | time'd exactly |
| 2026-11-20 | `cargo check` (this round's quick loop — COLD dep graph on the fresh clone; rustup + the pinned toolchain installed first) | **1m 25s** | time'd exactly |
| 2026-11-20 | bare `cargo test --profile gates --features mcp` AGAIN (trap 10 recidive No. 2: no stdin shield → the MCP stdin-reader teardown hung; two 300 s tool timeouts died mid-run) AND the mcp-feature switch rebuilt the gates dep graph (~4 min); with the shield (`> /tmp/log`, no pipe) and the cache the suite ran in 4 s | **≈ 10 min wasted across 2 retries; 4 s final** | the tool timeouts + the log |
| 2026-11-20 | bare `cargo test --profile gates --features mcp` (no `< /dev/null` shield) — SANDBOX.md trap 10 recidive in the recycled sandbox: tests themselves green, the cargo↔test-binary teardown hung on the MCP stdin reader until the 600 s tool timeout killed it | **600 s (timeout)** | the tool timeout |
| 2026-11-20 | `cargo check` (this round's quick loop — COLD dep graph on the dev lane, fresh sandbox: rustup + the pinned toolchain installed first) | **1m 23s** | time'd exactly |

## Hot

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-20 | `bash scripts/verify.sh` (the V59-fixes round's full gate — rules + fmt + clippy ×2 + test ×2; warm deps) | **21 s** | the gate's own print |
| 2026-11-20 | `cargo build --features mcp` (the dev MCP binary for the driver; warm deps) | **2 s** | cargo's print |
| 2026-11-20 | `cargo test --profile gates` (the V59-fixes round's suite; warm deps, the crate recompile per iteration) | **≈ 5 s** × 6 runs | the gate's own print |
| 2026-11-20 | `bash scripts/verify.sh` (the panel-spacing round's gate; the dev `cargo build` for the GUI probe ran just before — 1m 43s, the dev-lane codegen cold from a check-only session) | **73 s** | the gate's own print |
| 2026-11-20 | `cargo build` (the dev GUI-probe binary; check-artifacts existed but codegen was cold) | **1m 43s** | cargo's print |
| 2026-11-20 | `cargo build` (the dev GUI-probe binary; deps hot from this round's cargo check) | **1m 40s** | cargo's print |
| 2026-11-20 | `bash scripts/verify.sh` (this round's gate — the gates dep graph warm from a first run that the sandbox killed mid-compile) | **12 s** | the gate's own print |
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
