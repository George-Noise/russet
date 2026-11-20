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
| 2026-11-20 | `cargo check` (the save-dialog layout round's quick loop — COLD dep graph on the dev lane, fresh sandbox recycle: rustup + the pinned toolchain installed first) | **1m 35s** | time'd exactly |
| 2026-11-20 | `cargo check` (the save-compression round's quick loop, COLD dep graph on a fresh sandbox — rustup installed first, the whole dep graph compiled) | **1m 17s** | time'd exactly |
| 2026-11-20 | `bash scripts/verify.sh` (the save-compression round's first gate run — the gates lane's dep graph was cold; only the dev lane was warm from cargo check) | **ALL GATES GREEN 186 s** | the gate's own print |
| 2026-11-20 | `bash scripts/verify.sh` (the web direct-download round) on a COLD clone — the sandbox was recycled: rustup + the pinned toolchain installed first, the whole dep graph + the crate per stage | **ALL GATES GREEN 114 s** (the gate's own print; the first attempt failed fast on the dead-code lint, fixed, re-run) | the gate's own print |
| 2026-11-20 | `cargo check --target wasm32-unknown-unknown` (the web-facing round's wasm economy check) — COLD dep graph on the fresh wasm triple | **≈ a few minutes** (inside one bounded call; the exact seconds were not captured — the HOT re-run finishes in 0.3 s) | observed ≈, not precisely timed |

## Hot

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-20 | `bash scripts/verify.sh` after the test additions (the save-compression round's second gate run; gates deps hot, the crate recompiled per stage) | **13 s** | the gate's own print |
| 2026-11-20 | `bash scripts/verify.sh` after the pencil-drifting-pen change (4 src files + 3 test files; hot deps, the crate recompiled per stage) | **74 s** | the gate's own print |
| 2026-11-19 | `cargo build --features mcp` (dev GUI binary for the visual driver; deps cold after the sandbox recycle, the crate rebuilt after the header change) | **≈ 5 min** (observed 07:44→07:49 from the binary's mtime; the re-run overwrote its log) | the binary's timestamps ≈ |
| 2026-11-19 | `cargo build` (dev lane, hot deps, `windows.rs` + `windows/tests.rs` changed) | **1m 39s** | time'd exactly |
| 2026-11-18 | `bash scripts/verify.sh` after a small code change (About window: 2 files + 2 tests, deps hot, gates profile) | **15.1 s** | time'd exactly |

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
