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
> `target/` normally persists with the project directory; the owner also hosts
> a prebuilt `russet-target.zip` on GitHub to skip the cold dep build entirely
> (AGENTS.md "Session start").
>
> Rule: **every** heavy command (> ~1 min) gets a row in the same edit that ran
> it, with HOW it was measured (`time`d exactly vs observed from a background
> log ≈). One command per row; never merge rows.

## Cold

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-17 | rustup install + fetch pinned 1.99.1 toolchain (fresh sandbox, no rustup at all) | ≈ 3–4 min | ≈ (background-log mtimes) |
| 2026-11-17 | `cargo check` (dev, fresh clone: ALL deps compiled) | ≈ 8–12 min | ≈ (background-log mtimes) |
| 2026-11-17 | `cargo test --profile gates` (gates profile fresh: all test deps compiled) | ≈ 4–6 min | ≈ (observed via background poll) |
| 2026-11-17 | `cargo check --target wasm32-unknown-unknown` (target triple fresh; deps recompiled for wasm) | ≈ 1.5–2.5 min + one-time `rustup target add` download | ≈ (background-log mtimes; the finishing pass measured 1m38s) |

## Hot

| Date | Command | Duration | Measured |
|---|---|---|---|
| 2026-11-17 | `bash scripts/verify.sh` after docs/script-only changes (no src/ touched — no rebuild; same after a bare `touch` of an unchanged file: cargo detects identical content) | **9 s** | time'd exactly |
| 2026-11-17 | `bash scripts/verify.sh` (nothing changed since the last green run) | **14 s** | time'd exactly |
| 2026-11-17 | `cargo clippy --profile gates --all-targets` after real code changes (the magnet round) | 59 s | cargo's own "Finished in" |
| 2026-11-17 | `cargo test --profile gates` after real code changes (the magnet round) | ≈ 3–4 min | ≈ (background poll) |
| 2026-11-17 | `cargo build --features mcp` (dev GUI binary, nothing changed) | **2 s** | time'd exactly |

## Reading the numbers

- The done-gate (`verify.sh`) after an ACTUAL code change is dominated by the
  gates-lane rebuild of the `russet` crate itself (deps stay cached): expect
  ~1–3 min total; 14 s means cargo saw nothing new.
- A cold session WITHOUT the hosted `russet-target.zip` costs ~15–20 min of
  dep compilation before the first useful command; with the zip restored it is
  minutes of unzip + the hot numbers above.
- Sleep chunk guidance: start with ~1/3 of the last measured duration, then
  poll in ~60–120 s chunks. Never poll more often than needed (every poll is a
  step).
