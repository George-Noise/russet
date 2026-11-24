### 2026-11-24 — the `--warm` warm-up verdict lies when cargo is missing
- Expected: `sandbox_check.sh --warm` starts a detached gates-lane warm-up; "verify.sh is hot when the log says finished".
- Actual: on the wiped sandbox rustup was MISSING; the warm-up's cargo calls died instantly ("cargo: command not found" twice) and the log STILL ended with "warm-up finished (exit 0) — verify.sh is now hot" — a green verdict with nothing compiled. The tell was the log's lifetime: 0 seconds.
- Cost: the session-start probe read as "warm-up started"; the lie surfaced only when the first real cargo call failed. One wasted probe plus a SANDBOX.md re-read; the toolchain install itself was the accepted cold cost.
- Could improve: the --warm branch could gate on the cargo presence it already reports ("[NO] rustup MISSING") and print a clear refusal instead of writing "finished (exit 0)". Same shape as the 2026-11-23 entry.

### 2026-11-24 — MCP stdio hangs twice in one session: the visual driver on a non-mcp binary, and direct `cargo test --features mcp` on the agent shell's open stdin

- What happened (expected vs actual): (1) the done-gate's GUI probe —
  `visual_driver.py ./target/debug/russet` per SANDBOX.md's verified block —
  never produced a single scenario line and sat until the 240 s cap killed
  it; the run needs the `--features mcp` build (the driver speaks MCP over
  piped stdin; the SANDBOX.md recipe omits that, only the driver header's
  "feature `mcp`" hints it). (2) Re-running `cargo test --profile gates
  --features mcp` directly (to read the mcp test count for the PROGRESS
  entry — verify.sh's tail had already shown the default count) hung twice:
  killed at a 320 s cap and again at 100 s; the identical run with
  `< /dev/null` finished in ~50 s (7.5 s of tests, the compile cached).
  verify.sh runs the same command fine, so the blocker is the agent shell's
  open pipe stdin reaching the suite.
- How it showed up: a silent hang, no error output to grep; the exit came
  only from the tool-call timeout, which looked like a stalled sandbox the
  first time.
- What it cost: ~10 min of wall clock across the two hangs plus the
  rebuild-then-rerun cycle (~3 min).
- What could improve: done — SANDBOX.md's driver block now states the
  mcp-build requirement and the `< /dev/null` rule for direct mcp-lane test
  runs (durable fix landed in the same edit as the round); TIMING.md rows
  recorded. Nothing further — one-off doc gap.

### 2026-11-23 — `rustup target add` on the recycled sandbox failed twice before the wasm check could run

- What happened: the persistence round's wasm check (`cargo check --target
  wasm32-unknown-unknown`, needed because Cargo.toml gained the eframe
  `persistence` feature) failed on a fresh recycled sandbox twice: first
  E0463 "can't find crate for `std` — the wasm32 target may not be
  installed" (the sandbox_check output had already shown rustup MISSING,
  so the target was gone too), then the natural fix
  `rustup target add wasm32-unknown-unknown` itself died with
  "rustup could not choose a version of rustup to run" — with no default
  toolchain, rustup cannot even run its own subcommands.
- Expected vs actual: `sandbox_check.sh` flags the missing rustup, but the
  restore recipe for the WASM TARGET was not written anywhere — the plain
  `rustup target add` form (in every Rust doc on earth) is the wrong one
  for this sandbox; `RUSTUP_TOOLCHAIN=1.99.1 rustup target add
  wasm32-unknown-unknown` is the working form.
- What it cost: one failed wasm-check run (~3 s, cheap) + one failed
  target-add (~1 s) + the target download (~40 s). Minor, but the fix is
  now in SANDBOX.md ("Rust:" bullet) so the next recycled sandbox skips it.
- What could improve: the durable fact moved to its canonical home
  (SANDBOX.md) in the same edit; nothing else — the trap is sandbox-local,
  not repo-local.

### 2026-11-23 — Bare `cargo test --profile gates --features mcp` hung the tool call: the SANDBOX.md trap-10 stdin pipe, again

- What happened: ran `cargo test --profile gates --features mcp` directly
  (not through verify.sh) to read off the mcp lane's test count; the tests
  themselves passed (the same lane ran green inside verify.sh in 9 s), but
  the cargo↔test-binary teardown hung and the 5-minute tool-call timeout
  killed the command.
- Expected vs actual: verify.sh's comment and SANDBOX.md trap 10 document
  that every mcp test binary spawns an MCP stdin reader and the recycled
  sandbox's session stdin is an open pipe/pty whose EOF never comes —
  the run needs `</dev/null`. The agent ran the bare command anyway and
  paid the timeout.
- What it cost: one wasted 5-minute tool call + one re-run (hot, seconds).
- What could improve: none beyond following the documented trap — the fix
  is `cargo test --profile gates --features mcp </dev/null` (or reading
  verify.sh's staging first). One-off.

### 2026-11-23 — sandbox_check's detached warm-up fired without rustup and its verdict still said "verify.sh is now hot"

- What: fresh recycled sandbox — `scripts/sandbox_check.sh --warm` printed
  `[NO] rustup MISSING` but STARTED the detached gates warm-up anyway; the
  background script died instantly on `cargo: command not found` while its
  log still ended with "warm-up finished (exit 0) — verify.sh is now hot".
  Expected vs actual: the warm-up should be skipped (and the verdict say
  NOT hot / skipped) when rustup is absent; instead the exit-0 line lied.
- How it showed up: the next warm-build-log check showed two
  `cargo: command not found` lines; the warm-up had to be restarted
  manually after `rustup` + the pinned toolchain were installed.
- Cost: one wasted round-trip and a false "hot" belief for a minute; the
  restarted warm-up ran fine (clippy 1m 32s cold — TIMING.md).
- Could improve: gate the warm-up on rustup being present and print
  "warm-up SKIPPED (no rustup)" in the verdict instead.

2026-11-21 — the file-split round (5 files) — re-exports consumed ONLY by tests: clippy `-D warnings` flags them unused even with `--all-targets` (the bin view of the same lint also fires), so a re-export whose only consumers are test files cannot carry the round. The convention that actually passes the gate: the tests import the moved item from its OWNING module (`tools/tests.rs` → `super::shapes::outline_stroke_bounds`, `windows/tests/*` → `super::super::settings_panel::…`) and the parent keeps re-exports for BIN consumers only. AGENTS.md "Tests" rule 2 ("move the test, not the visibility", learned in the palette split) is the same law seen from the other side — worth a line in the split playbook.

2026-11-21 — the file-split round (5 files) — impl-method visibility after a move into a CHILD module: `pub(super)` there means "visible in the child's parent" (= the old module's scope), NOT visible to `app.rs` — `app.rs:736 draw_preview_overlay` failed with E0624 twice until the method was widened to `pub(in crate::app)`. Rule of thumb that worked: match the visibility to the CALLER's module path, not to the old declaration.

### 2026-11-21 — raw-Xvfb key/modifier events silently dropped (no WM → no input focus); cost three probe iterations

- Expected vs actual: the diameter-circle GUI probe's `xdotool keydown shift` +
  drag should draw a Shift-constrained shape; every drag drew the UNCONSTRAINED
  form (a flat 204×44 ellipse instead of a circle). Clicks and pointer moves
  landed fine the whole time, so nothing looked broken from the outside.
- How it showed up: only through measured pixels — the drawn ring was 208×48
  where 208×208 was expected. No error, no log line; the screenshots even looked
  plausible at a glance (a red ring WAS there).
- Cost: ~3 probe iterations / ~10 minutes, plus one extra app relaunch to
  isolate the rectangle-vs-ellipse confound.
- Root cause + fix: raw Xvfb has no window manager → no window ever gets the
  X input focus → winit drops key/modifier events entirely; mouse events still
  work because X's initial focus is PointerRoot (they follow the pointer). One
  `xdotool windowfocus <id>` per launch fixes it. The durable fact moved to
  SANDBOX.md pitfall 11 in the same edit; the drag-probe template
  (`gui_magnet_check.sh`) never hit this because its drags carry no modifier
  keys.
- Could improve: `scripts/sandbox_check.sh`'s GUI-lane verdict could probe the
  `windowfocus` path alongside its xdotool row — a one-line addition, not done
  now (this entry is bookkeeping, not a task).

### 2026-11-20 — visual_driver.py exits 0 SILENTLY against a non-MCP binary

- Expected: the driver prints its per-scenario lines (or fails loudly when the binary cannot speak MCP).
- Actual: `python3 scripts/visual_driver.py target/debug/russet` built WITHOUT `--features mcp` printed NOTHING and exited 0; piped output stayed empty for the full 600s Bash timeout (stdout buffering + no scenario attempts), and a rerun with `-u` also produced a 0-byte log and exit 0.
- Cost: one 600s hung Bash call + a wasted rerun (~12 min) before the cause was spotted (the driver needs the MCP stdio server; the plain dev build does not speak it).
- What could improve: a startup probe in visual_driver.py — if the binary does not answer the first MCP handshake within ~5 s, print one loud line ("binary has no MCP server? build with --features mcp") and exit non-zero. None of the docs state the MCP-build requirement in the driver's own usage line.

### 2026-11-20 — egui `Rect` has no `min_x`/`max_x` fields (a wrong guess that cost one compile)

- **When:** 2026-11-20, the Shape-panel spacing round — writing the new
  gap-measuring test (`windows/tests/settings_panel.rs`).
- **What happened:** wrote the gap math against `rect.min_x`, `rect.max_y`
  and friends the way several other Rust rect types (and egui's own
  `Rect::min/max` field names) suggest — those shorthand FIELDS do not
  exist on `eframe::egui::Rect` 0.37.
- **How it showed up:** six `E0609: no field min_x on type Rect` compile
  errors on the first `cargo test --profile gates` run ("available fields
  are: `min`, `max`") — the compile of the whole gates test graph
  (~2 min cold) wasted on an API guess.
- **What it cost:** one failed compile ≈ 2 min on the cold lane + a fix
  round; nothing else (the correct accessors are the `left()/right()/
  top()/bottom()` methods, and they were already used elsewhere in this
  repo's tests).
- **Possible improvement:** done in the same edit — a one-line entry in the
  AGENTS.md "eframe/egui 0.37 API quirks" list (`Rect` accessors are
  METHODS, the only fields are `min`/`max`), so the guess is not repeated.
  Nothing else needed.

### 2026-11-20 — `sandbox_check.sh --warm` reports a warm-up that never ran (the file's first entry, logged by the session that created this file)

- **When:** 2026-11-20, session start (fresh recycled sandbox; the repo
  re-cloned from GitHub; the task round = the Shape-panel spacing + this
  file itself).
- **What happened:** the session-start probe starts its DETACHED gates
  warm-up (clippy + test compile) unconditionally — but rustup did not
  exist in the fresh sandbox yet, so the warm-up died instantly with
  `bash: cargo: command not found` while the probe still printed
  "[yes] background warm-up started (log /tmp/russet_warm_build.log) —
  verify.sh is hot when the log says finished" and "warm-up finished
  (exit 0)".
- **How it showed up:** the warm-up log contained two
  `cargo: command not found` lines and zero compile work, yet the verdict
  implied the done-gate was already being warmed. Nothing in the probe's
  output flagged that the warm-up was a no-op.
- **What it cost:** nothing material this once — rustup was being
  installed in parallel and the gate ended up running synchronously anyway
  (in this recycled sandbox detached jobs die between tool calls
  regardless, TIMING.md). But on a normal session an agent could trust the
  false "hot" verdict and skip its own toolchain setup until verify.sh
  fails much later.
- **Possible improvement:** `sandbox_check.sh --warm` could check for
  cargo/rustup BEFORE starting the warm-up and print
  "[NO] warm-up skipped: no toolchain" instead of a silent zero-work
  exit 0 (or start the warm-up AFTER the toolchain check passes).
