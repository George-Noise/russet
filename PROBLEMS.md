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
