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
| 2026-11-24 | `cargo test --profile gates --features mcp camera` — the gates-lane dep-graph CODEGEN cold on the wiped-sandbox fresh clone (`~/.cargo` AND `target/` were both wiped; rustup + the pinned 1.99.1 toolchain reinstalled first): the unoptimized codegen exceeded FOUR 10-min tool-call caps, cargo resuming each time, the final chunk linked the 61 MB test binary and ran the suite | **≈ 40 min wall across chunks** | the tool-call caps ≈ |
| 2026-11-24 | `cargo check --profile gates --features mcp` — the dep-graph check COLD (the same wiped sandbox; the check artifacts do NOT feed the test codegen — the test build recompiled every dep) | **1m 26s** | cargo's print |
| 2026-11-24 | `cargo build` (the dev GUI binary for the visual-driver run — the dev lane COLD on the fresh GitHub clone: rustup + the pinned 1.99.1 toolchain installed first, then the whole dep graph) | **1m 35s** | `time`d exactly |
| 2026-11-24 | `cargo check` (this round's quick loop after the Ctrl+A edits — the dev-lane dep graph COLD, the crate itself compiled after) | **1m 16s** | cargo's print |
| 2026-11-23 | detached gates-lane warm-up on the RECYCLED sandbox — rustup itself missing and reinstalled first (minimal, no default), then `cargo fetch` + `cargo check --profile gates` cold (1m 13s); the fetch also pulled the wasm toolchain pieces later | **≈ 2 min total** | the background log |

## Hot
| 2026-11-24 | `cargo test --profile gates --features mcp` run DIRECTLY in the agent shell — HANGS on the shell's open pipe stdin (killed at 320 s and at 100 s caps); with `< /dev/null` the same run finishes in ~50 s wall (the compile was cached by verify.sh, 377 tests in 7.5 s). verify.sh itself never hangs — run direct mcp-lane tests with stdin closed | **320 s + 100 s killed; ~50 s OK** | the tool-call wall clock ≈ |
| 2026-11-24 | `visual_driver.py ./target/debug/russet` against the WRONG build (no `--features mcp`) — the driver speaks MCP over piped stdin, a plain binary never handshakes: hung until the 240 s cap killed it (the mcp build ran the same probe in 3 s; SANDBOX.md now says the binary must be mcp-enabled) | **4m 0s (killed)** | `time`d exactly |
| 2026-11-23 | `bash scripts/gui_settings_check.sh` (the settings round's storage probe — six headless sessions dominated by the two 30-s autosave waits and the launches; the gates binary) | **≈ 5 min** | the tool-call wall clock ≈ |
| 2026-11-23 | `bash scripts/verify.sh` (the persistence round's full gate — the gates lane rebuilt with the NEW eframe `persistence` feature: eframe/egui/egui-winit/serde + ron recompiled, then the whole gate) | **2m 28s** | `time`d exactly |
| 2026-11-23 | `cargo check --target wasm32-unknown-unknown` (after the same feature change; wasm target had to be re-added to the fresh sandbox's toolchain first — the env-less `rustup target add` needs `RUSTUP_TOOLCHAIN=1.99.1` when no default toolchain exists) | **37 s** | `time`d exactly |
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

## 2026-11-23 — the wand/fill/Esc round (fresh sandbox: cold toolchain + cold target)

| Command | Duration | Note |
|---|---|---|
| `cargo build` (cold target/, 9th tool + wand icons) | 1 m 41 s | the first full build after the clone |
| `cargo test --profile gates` | 6.5 s | 346 tests |
| `verify.sh` (clippy + tests + mcp + wasm + driver) | 20 s | ALL GATES GREEN |
| `gui_wand_fill_check.sh` (3 runs: calibration + fixes) | ~45 s each | the wand/fill/Esc probe |

Sandbox notes: the toolchain re-installed (rustup 1.99.1), `/tmp/xkbfix` rebuilt,
`/tmp/xdotool` rebuilt — needs THREE lib debs now (libxtst6 + libxi6 joined libxdo3,
SANDBOX.md trap 13: with only libxdo3 every xdotool call dies on libXtst and the
probe's pointer never moves).

## 2026-11-23 — the tolerance-slider round (second recycle in one day: toolchain + /tmp rebuilt, tree restored from the delivery zip)

| Command | Duration | Note |
|---|---|---|
| `cargo build` (cold target/) | 1 m 39 s | the slider round's first full build |
| `verify.sh` | 82 s first / 15 s warm | ALL GATES GREEN (351 default / 364 mcp) |
| `gui_wand_fill_check.sh` | ~45 s ×2 | the calibration run + the 100 % pour-over assert |

## 2026-11-23 — the fill-confinement round (fourth recycle: full restore + rebuild)

| Command | Duration | Note |
|---|---|---|
| `cargo build` (cold target/) | 1 m 59 s | after the restore |
| `verify.sh` | (see the round log) | the full gate before the delivery |
| `gui_cut_fill_check.sh` | ~65 s | the extended probe: the outside-click step added, PASS |

## 2026-11-23 — the fill-bench round (fifth recycle: full restore + rebuild)

| Command | Duration | Note |
|---|---|---|
| restore + rustup + warm-up | ~6 min | background |
| `cargo build --release --features fill-bench` | 5 m 45 s | cold release |
| the dev bench suite | 21 s | 10 cases, the log in benchmarks/ |
| the release bench suite | 2.9 s | the gap measurement |
| `verify.sh` | (the round log) | the new fill-bench clippy stage |

## 2026-11-23 — the span-rewrite round (sixth recycle: full restore + rebuild)

| Command | Duration | Note |
|---|---|---|
| restore + rustup + warm-up | ~7 min | background |
| `cargo build --release --features fill-bench` | 5 m 47 s | cold release |
| the dev bench after the rewrite | 4.3 s | was 21 s pre-rewrite |
| the release bench after the rewrite | ~1 s | was 2.9 s |
| `verify.sh` | 18 s | 359 tests green |
