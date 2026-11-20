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
