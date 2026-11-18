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
| 2026-11-18 | `bash scripts/verify.sh` FULL gate on a COLD lane (first run of the hardened gate — `exec </dev/null` shield + stage timers): rules 0 s / fmt 0 s / clippy 84 s / test 156 green 98 s / mcp clippy 3 s / mcp test 168 green 6 s — the mcp stage exited instantly, no stdin hang | **191 s total** | the script's own stage timers |
| 2026-11-18 | `cargo check` (dev, FIRST run under the FULLY UNOPTIMIZED policy — owner decision 2026-11-18: fresh sandbox, all deps compiled unoptimized; toolchain fetched before the timer) | **1m31s** (was 3m51s the same morning under optimized deps) | time'd exactly |
| 2026-11-18 | `cargo check` (dev, fresh sandbox: rustup 1.99.1 installed first, then ALL deps compiled; docs/comment-only tree) | 3m51s | time'd exactly |
| 2026-11-17 | rustup install + fetch pinned 1.99.1 toolchain (fresh sandbox, no rustup at all) | ≈ 3–4 min | ≈ (background-log mtimes) |
| 2026-11-17 | `cargo test --profile gates` (gates profile fresh: all test deps compiled) | ≈ 4–6 min | ≈ (observed via background poll) |
| 2026-11-18 | `bash scripts/verify.sh` FIRST run in a cold gates lane (all gate deps compiled; the run itself FAILED at a new test — the 3 min is the compile cost, the fix re-ran hot) | ≈ 3 min | time'd exactly |
| 2026-11-17 | `cargo check --target wasm32-unknown-unknown` (target triple fresh; deps recompiled for wasm) | ≈ 1.5–2.5 min + one-time `rustup target add` download | ≈ (background-log mtimes; the finishing pass measured 1m38s) |

## Hot

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-18 | `bash scripts/verify.sh` after a small code change (About window: 2 files + 2 tests, deps hot, gates profile) | **15.1 s** | time'd exactly |
| 2026-11-17 | `bash scripts/verify.sh` after docs/script-only changes (no src/ touched — no rebuild; same after a bare `touch` of an unchanged file: cargo detects identical content) | **9 s** | time'd exactly |
| 2026-11-17 | `cargo clippy --profile gates --all-targets` after real code changes (the magnet round) | 59 s | cargo's own "Finished in" |
| 2026-11-17 | `cargo test --profile gates` after real code changes (the magnet round) | ≈ 3–4 min | ≈ (background poll) |
| 2026-11-17 | `cargo build --features mcp` (dev GUI binary, nothing changed) | **2 s** | time'd exactly |

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
