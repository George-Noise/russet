# PROBLEMS — the agent-friction log (WRITE-ONLY for now)

> **Why this file exists** (owner request 2026-11-20): every problem, snag,
> dead end or wrong assumption the LLM agent runs into — tooling, sandbox,
> docs, spec wording, egui quirks, process, whatever — is recorded here the
> moment it happens. Entries ACCUMULATE; when the owner decides enough has
> piled up, HE reads the file and the review happens: what can be improved
> (a doc rule, a script, the sandbox, the spec — or nothing, one-off).
>
> **Rules (AGENTS.md "Agent-friction log — PROBLEMS.md"):**
> - The agent WRITES here and does NOT read it: not for context, not for
>   decisions, not for dedup ("did I already log this?"), never to fix
>   anything — until the owner EXPLICITLY says the file is to be read. The
>   one mechanical exception: inserting a new entry needs locating the Log
>   heading below (a one-line peek at the structure, not a content read).
> - Every entry goes at the VERY TOP of the Log section (newest first) —
>   the same convention as the PROGRESS.md Log.
> - Anything qualifies: a command that lied or hung, a quirk that cost a
>   rebuild, a doc that was missing the needed fact, a wrong assumption,
>   wasted minutes, a silent failure — and also "worked as intended but
>   felt wrong or fragile".
> - The template below is ABSTRACT on purpose: entries may vary in style
>   from one to the next; what matters is the CONTENT and the conveyed
>   meaning — what went wrong, how it showed up, what it cost, and (when
>   visible) what could improve. One entry = one problem.
> - Writing an entry is bookkeeping, never a task: nothing is fixed because
>   an entry exists, and no entry is ever deleted or rewritten (the owner
>   reviews the history as-is). A durable fact learned along the way still
>   moves to its canonical home (SPEC §, AGENTS.md, SANDBOX.md, the module
>   docstring) in the same edit — this file stays raw.

## Template (abstract — fields are guidance, not a form)

Each entry carries, in any shape the writer finds natural:

- **When** — the date, plus the task it happened in (a few words).
- **What happened** — the problem itself: what was expected vs what
  actually occurred.
- **How it showed up** — the symptom the agent actually saw (an error, a
  silent no-op, missing info, a hang, wasted time).
- **What it cost** — minutes, rebuilds, retries, a wrong turn — or
  "nothing material, just noting it".
- **Possible improvement** (optional) — what would have prevented it;
  "none, one-off" is a valid answer and so is "needs the owner's call".

## Log — newest first. Insert a new entry directly under this heading.

---

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

---

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

---
