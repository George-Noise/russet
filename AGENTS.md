# AGENTS.md

The agent rulebook — read in FULL at every session start. Budget and maintenance
rules: see "Maintenance" at the end.

## Source of truth

- `SPEC.md` is the full product spec and the authoritative task list. Follow its milestones (M0–M9) in order. Do not add features from §6 (out of scope).
- `PROGRESS.md` is the progress log and the substitute for git history (see "Git & progress log"). Read it before starting any task and update it after every finished, verified one.
- Reading is routed by task type (see "Reading routes" below) — do not read the whole SPEC when a task touches one §.
- `PERF.md` is a **reference backlog** for memory/build-speed work, not an autonomous queue: the owner picks an item, and the agent executes it only when asked. Its §0 has the task details and the owner gates.
- `DEBTS-verification.md` and `DEBTS-functional.md` are the live debt lists (see "Debts" below). Read both at session start and at task end — and ASK about them.
- `PROBLEMS.md` is the agent-friction log — a pure entry log the agent never reads and never edits by hand (see "Agent-friction log" below).

## Language — the repo's written language is a WHITELIST with one entry: English

NOTHING may be written into any repo file in any other language — all documentation (`.md` files), source code and its comments, scripts and their text output, `PROGRESS.md` (entries, snapshot, stamp), both `DEBTS-*.md` files, `PROBLEMS.md` entries, proposed commit titles, and the app's user-facing strings (already hardcoded English). Whatever must reach the owner in another language goes in the chat reply ONLY, never into a repo file. It is a language whitelist (English is simply the only written language here), not a script blacklist.

## Reading routes — task type → what to read

Always read first (cheap): this file, plus `PROGRESS.md` — status snapshot, warnings, verification stamp, the newest log entries. Then, by task type, read NOTHING beyond the listed material:

| Task type | Read |
|---|---|
| Behavior change in an existing feature | that `SPEC.md` §3.x + the implementing module (the module map in `app.rs`'s doc comment) + its tests (`<module>/tests.rs`) |
| New tool / feature | its §3.x + `SPEC.md` §6 (scope check) + the closest existing module as the pattern (preview-then-commit tool → `app/preview.rs`; new floating window → `app/layers_window.rs`) |
| Window placement / resize behavior | `SPEC.md` §3.11 + `app/placement.rs` (everything about §3.11 lives there, incl. its `placement/tests.rs`) |
| Dialogs / file I/O | `SPEC.md` §3.7–3.8 + `app/dialogs.rs` / `file_io.rs` |
| Hotkeys | `SPEC.md` §3.9 (that list is exhaustive) |
| Build / toolchain / flake | `SPEC.md` §1 + §8 + `flake.nix`; update `README.md` in the same edit if a human-facing command changed |
| UI / windows / egui work | `docs/agents/egui-quirks.md` (the eframe/egui 0.37 API quirks — each quirk keeps its fact + fix) |
| Unfamiliar area, or the § is unclear | additionally `SPEC.md` §4–§7 (milestones, pitfalls, scope, acceptance) before coding |

Never change behavior based on a § you have not read this session. `SPEC.md` §4–§7 in full is required reading for milestone (M<N>) work only, not for one-line fixes.

## Project layout — the filesystem is the source of truth

No file tree is kept in the docs: agents list the actual files with their own tools when needed, and docs only carry the rules that survive file churn.

- A module's tests live in a sibling `tests.rs` file (see "Tests"): `src/canvas.rs` ↔ `src/canvas/tests.rs`.
- New subsystem → a child module of the module that owns the state/behavior (for `App` state that means `src/app/<name>.rs`). Never grow `app.rs` with a new subsystem.
- A new or removed file is reported in the `PROGRESS.md` entry of the same task (what and why); no tree or layout list needs updating anywhere.
- Do not add top-level directories or build tooling (no `xtask`, `just`, scripts) without asking the owner. The owner-approved exceptions: CI (see "Repo facts") and the friction-log writer `scripts/log_problem.sh` (see "Agent-friction log"). The build is `cargo` + `scripts/verify.sh` + `trunk` + the flake.

## Tests — separate files, always

Hard rules; they exist so production files stay readable and an agent can add tests without touching behavior code.

1. **No inline test modules.** Never write `#[cfg(test)] mod tests { ... }` inside a source file.
   The module file ends with exactly:
   ```rust
   #[cfg(test)]
   mod tests;
   ```
   and the tests live in `<module>/tests.rs` next to it (`src/canvas.rs` → `src/canvas/tests.rs`,
   `src/app/placement.rs` → `src/app/placement/tests.rs`, `src/text.rs` → `src/text/tests.rs`).
   The file is a CHILD module: start it with `use super::*;` (or an explicit `use super::{a, b};`)
   and test private items directly.
2. **Never widen visibility for a test.** No `pub(crate)`/`pub` just to reach code from a test.
   If a test cannot reach it, it is in the wrong module — move the test, not the visibility.
3. **No crate-root `tests/` directory.** The crate is a single binary (no lib target), so
   integration tests cannot import it. Keep tests as in-crate unit tests. Adding a lib target is a
   structural change that needs the owner's OK first.
4. **Every fix/feature ships with a test**, in the same task, in the module that owns the behavior.
   A bug fix starts from a regression test that fails before the fix. Never delete a test to get
   green; update it in the same edit when behavior intentionally changes. Docs-only rounds
   (a diff touching only `.md` files) ship no tests — see "Commands".
5. **Test what the spec promises**, not implementation details. Prefer small pure helpers over a
   full egui frame; if behavior is only reachable through `App`, extract the math into a testable
   helper in the same task.
6. **Style.** `#[test]` only (no async, no new dev-dependencies). Names are behavior sentences in
   `snake_case`, no `test_` prefix. Arrange–Act–Assert with blank lines between blocks.
   `expect("what must hold")` for invariants; `unwrap()` forbidden everywhere. No filesystem,
   network, sleeps or wall-clock in tests.
7. **Size guard.** Keep `tests.rs` under ~300 lines. Past that, split by topic: `tests.rs` keeps
   only `mod <topic>;` declarations and the files move to `src/<module>/tests/<topic>.rs`.
   **Owner-locked exception (2026-11-17):** `src/app/scenarios/tests.rs` — the whole-app scenario
   suite — is exempt from BOTH this guard and the 500-line file guard: it grows scenario by
   scenario on the owner's request, splitting it is NOT a debt and is never proposed, and editing
   it without the owner's explicit permission is forbidden (the file's own header says the same;
   `scripts/rules.sh` skips it in the size-guard loop).
8. `cargo test` must pass before a task counts as done, and `cargo clippy --all-targets` lints the
   test files too.

## Git & progress log

- `.git` reaches the agent only when the owner includes it in the delivery, and it is gone once the agent sandbox is recycled — never count on it being there. The agent never commits; the owner commits locally using the titles proposed below. **Git is not the progress channel here — `PROGRESS.md` (repo root) is.**
- **The owner sent a `russet.zip`? Use it immediately:** do NOT run `git pull` or `git clone` first — unpack the .zip into the workspace and start working from it right away; a delivered .zip is the freshest state and takes priority over any sandbox copy or remote. Snapshot the unpacked tree as the patch base BEFORE the first edit (`git add -A; git write-tree > /tmp/base_tree; git reset`), even when the .zip ships without `.git` — a throwaway `git init` keeps the delivery patch possible.
- Before starting work: read `PROGRESS.md` (status snapshot, warnings, latest log entries) to learn where the project actually stands.
- After finishing AND verifying a task or milestone: insert one entry at the top of the "Log" section of `PROGRESS.md` (newest first) — date, `M<N>:`-style title (or a short title for non-milestone work), what changed and in which files, how it was verified (commands + result), and what the owner still must check manually. Update the status snapshot and warnings in the same edit.
- **Commit titles:** whenever a task leaves changes that will later go into a commit, ALWAYS propose a commit title for them — one short imperative line in the repo's existing `git log` style (multi-part changes joined with `;`). State it in TWO places: the `PROGRESS.md` log entry (a `Commit title:` line) AND as the **very last line of the chat reply to the owner**. The title's canonical form is the whole line wrapped in a pair of backticks — in BOTH places (a backticked string is click-to-copy in the chat UI; `rules.sh` rule 6 prints the log field with the backticks kept, and warns NOT WRAPPED when the newest entry's title lacks them). The agent never commits itself.
- **Log-entry budget:** one entry = at most ~20 lines with the fixed fields (What / Files / Verified / Owner-checks / Commit title). Anything durable learned during a task MUST move to its canonical home in the SAME edit — a new egui quirk → `docs/agents/egui-quirks.md`, an owner decision or behavioral law → `SPEC.md` §3.x, a module fact → that module's docstring — and the log entry only references it.
- **Log compaction:** when the "Log" section of `PROGRESS.md` grows past ~150 lines, move all but the newest 3–5 entries verbatim into `PROGRESS-archive.md` (read it only via grep when investigating history) and note that in the Log heading. The status snapshot carries a one-line verification stamp (`fmt/clippy clean, cargo test: N green — date`), updated in the same edit.
- Never record work as done in `PROGRESS.md` before its verification commands actually pass.

## Debts — pending human checks & functional debts

Two files carry what would otherwise rot silently inside PROGRESS.md log entries:

- **`DEBTS-verification.md`** — checks only the OWNER can do: Windows `cargo run` click-throughs, browser passes, `nix build`, doc wording sign-offs. A log entry's `Owner-checks` line is history; the LIVE tracking of what is still unverified lives in the file.
- **`DEBTS-functional.md`** — deliberate simplifications, postponed work and open decisions (placeholders, postponed CI jobs, "later" items).

Hard rules:

1. A debt is born in the SAME edit as the task that creates it: every new owner-check item and every deliberate simplification lands in the matching file immediately, with its birth date. Never leave a debt only in the log entry.
2. **Ask, don't accumulate.** At session start and at task end, surface the open items of BOTH files to the owner in ONE batched list and ask which are human-verified / which decisions are made. Never assume a check happened; never remove an item silently. Items unchecked for over a week are flagged explicitly as stale.
3. A verified item gets `Status: verified <date>`; a paid functional debt gets `Status: paid <date>`; both are dropped at the next compaction. The historical record stays in the PROGRESS.md entries.
4. `scripts/rules.sh` is the automated reminder: it reports PROGRESS.md size, its Log-section size, RESOLVED-but-unarchived warnings and the count of live owner-checks (report-only).
5. PROGRESS.md `Warnings` holds ACTIVE blockers only; a warning moves to `PROGRESS-archive.md` in the SAME edit that resolves it. The Log keeps only the newest 3–5 entries (the compaction rule above); `rules.sh` warns when the budgets (~200 file lines, ~150 log lines) are exceeded.
6. **Ordering.** Open items in BOTH debt files are written sorted: verification checks (V) first, then functional debts (F), then anything else; within each group, NEWEST birth date first (the same newest-first convention as the PROGRESS.md Log). A closed item leaves the open table for the Resolved/Paid section in the SAME edit — it never lingers there.
7. **The ask lists only the 4 newest.** When surfacing open debts to the owner — the batched session-start/task-end list and every chat reply — write out ONLY the 4 newest open items, newest first (rule 6's order applies inside the four). The FILES keep every open item; nothing is dropped, only the ask stays scannable.

## Agent-friction log — PROBLEMS.md

`PROBLEMS.md` is LITERALLY a log: it holds entries and nothing else — no header, no template; it starts straight with its newest entry. The agent NEVER reads a single line of it (not for context, not for dedup, not to check the format) and never edits it by hand: entries go in ONLY through `scripts/log_problem.sh <entry-file>`, which PREPENDS the entry's bytes verbatim at the very top (newest first) and BYTE-CHECKS after the write that the previous content survived intact below the new entry — a mismatch restores the backup and fails without touching the log. Nothing in the log can be deleted or rewritten by an agent (the owner's explicit permission is the only door). Keep the entry file OUTSIDE the repo (e.g. `/tmp/entry.md`) — a file inside the worktree would leak into the delivery artifacts.

The ENTRY TEMPLATE (abstract — fields are guidance, not a form; style may vary entry to entry): a `### <date> — <one-line what>` heading, then a few bullets — what happened (expected vs actual), how it showed up, what it cost (minutes / rebuilds / retries / "nothing material"), and, when visible, what could improve ("none, one-off" is a valid answer). Anything qualifies. Writing an entry is bookkeeping, never a task; a durable fact learned along the way still moves to its canonical home in the same edit — the log entry stays raw. The owner reads the file and reviews what to improve only when HE decides to.

## Delivery ritual — artifacts sent to the owner

The owner applies the work locally, so every finished round ships the project as files, and it is not known in advance which form will fit the owner's local state. Always prepare BOTH, under FIXED names so every session delivers the same files:

- **`russet.zip`** — the full project tree, NEVER containing `.git`, `target/` or other build junk. It applies on any owner state: unpack and replace the tree.
- **`russet.patch`** — `git diff <base-commit>` with new files intent-to-add (`git add -N` first, so they are included), base commit stated in the reply. Prepare it ONLY when `.git` is present in the sandbox copy; without git there is no patch — say so plainly in the reply and `russet.zip` is the only artifact. When the owner has already committed the previous delivery locally (the normal state — his commits never reach the sandbox), the base is NOT an old sandbox SHA: snapshot the PRE-EDIT tree first (`git add -A; git write-tree > /tmp/base_tree; git reset` — a tree object, no commit or ref is created) and diff against it; the reply states that the patch applies on top of the owner's latest local commit, which must byte-match the previous delivery.

Before calling the delivery ready: with git, apply the patch onto a clean copy of the base commit and byte-compare the result with the working tree AND with the unpacked zip — all three must match; without git, byte-compare the unpacked zip with the working tree. Always regenerate the artifacts in the same session as the changes — never re-send an artifact from an earlier round. Print the sizes and sha256 checksums of the delivered files so the owner can verify the download. No target artifact is ever part of this ritual (the warm-target store is retired — the one canonical statement lives in "Commands").

The mechanics are automated: `bash scripts/deliver.sh <base-ref> <out-dir>` stages the tree, builds both artifacts, runs BOTH byte-compares and prints the checksums itself — use it for every delivery. The agent still writes the `PROGRESS.md` entry and states the base commit + checksums in the chat reply (when the out-dir is the chat's download directory, the reply links the files).

The final chat reply's last line is the suggested commit title, verbatim and backtick-wrapped, copied from the newest `PROGRESS.md` log entry's `Commit title:` field (see "Commit titles" above; `rules.sh` rule 6 prints it as a reminder at every done-gate). The reply lists ONLY the newest delivery's title — ONE title per reply; the owner keeps the older titles' list himself. If a round is docs-only and adds no log entry, say that no commit title is proposed.

## Commands

- Session start, BEFORE anything else: run `bash scripts/sandbox_check.sh --warm` (report-only probe of the agent sandbox: base tools, pinned toolchain + wasm target, network, `target/` warmth per lane, the Xvfb GUI fix) and read its verdict before launching any heavy command — never start a build/test lane whose missing prerequisites are not accounted for. The `--warm` flag additionally starts a DETACHED gates-lane warm-up — clippy + `cargo test --no-run`, exactly the compile work verify.sh does and nothing executed — so the done-gate is hot by the time the task ends. **The warm-target STORE idea (a hosted `russet-target.zip`) is RETIRED for now — the warm-up is in-sandbox compilation; a cold build on a fresh or recycled sandbox is the accepted cost** (reviving the store would be a NEW owner decision, never a routine delivery artifact). `sandbox_check.sh` still reports the per-lane warmth verdict; the cold-vs-hot numbers stay in TIMING.md ("Reading the numbers").
- Heavy-command timing: every command that ran longer than ~1 minute gets a row in `TIMING.md` (cold vs hot tables, 5 newest per table, same edit). The newest row is what the agent picks its `sleep` chunks from.
- Baseline at session start, BEFORE any edit: `bash scripts/rules.sh` and `cargo fmt --check` — nothing else. Both are instant and never compile anything. The tree always arrives green (byte-compared delivery + CI), so pre-edit `cargo check`/clippy/test runs are wasted minutes: NO pre-edit test/compile runs; the only exception is a toolchain/target warm-up in a fresh sandbox, which is environment work, not a test baseline. If the baseline fails, STOP and tell the owner — a red baseline means a broken delivery, not a task.
- Quick loop after edits: `cargo check`.
- Wasm-check economy: run `cargo check --target wasm32-unknown-unknown` ONLY when the change touches web-facing code (`web/`, `main.rs`'s wasm init, `#[cfg(target_arch = "wasm32")]` paths, trunk config) — native gate green is enough otherwise; neither verify.sh nor CI runs wasm (the CI wasm job is deliberately postponed), so an unneeded ad-hoc wasm run is pure lost minutes.
- Done gate: `bash scripts/verify.sh` — first `scripts/rules.sh` (machine rules, no cargo needed: inline test modules and `unwrap()` in `src/` are HARD failures; `expect()` outside tests, the 500-line file-size guard and the PROGRESS.md budget/stale check are report-only notes), then `cargo fmt --check`, `cargo clippy --profile gates --all-targets -- -D warnings`, `cargo test --profile gates` (the unoptimized `gates` lane — see "Repo facts"; identical checks). Run it before recording a task as done in `PROGRESS.md` — docs-only rounds excepted (the bullet below); relay any report lines to the owner (warn + propose a split at task end per "Repo facts"). CI (`.github/workflows/ci.yml`) runs the same gate on GitHub — keep it green (see "Repo facts").
- Tests: `cargo test --profile gates` (all — the lane verify.sh uses) / `cargo test app::placement --profile gates` (one module by path fragment). Unit tests live in `<module>/tests.rs` (see "Tests").
- **Docs-only rounds skip the cargo gate entirely:** when a task's diff touches ONLY the repo's documentation (the `.md` files), the done-gate is `scripts/rules.sh` ALONE (grep-level, no cargo — it still validates the PROGRESS.md budgets) plus the usual delivery byte-compares; NO `verify.sh`, NO `cargo check`/fmt/clippy/test, NO wasm check, and NO new tests (Tests rule 4 covers code fixes/features only). State the shortcut explicitly in the `PROGRESS.md` entry. Anything beyond `.md` in the diff — Rust, `scripts/`, `Cargo.toml`/lock, flake, CI, `web/` — is NOT docs-only: the full gate runs as usual. Comment-only edits INSIDE `.rs` files keep the quick loop: `cargo check` is enough; say so explicitly in the `PROGRESS.md` entry.
- Daily run: `cargo run` (Windows). Do not run `cargo build --release` during development; release validation belongs to M9.
- Agent sandbox / headless GUI: russet CAN run and be scenario-driven in the container — Xvfb plus two user-space .debs, no root needed. The exact recipe, pitfalls and the executable proof (`scripts/gui_recipe_test.sh`) live in `SANDBOX.md`; follow it instead of re-diagnosing, and include a green `visual_driver.py` run in the done-gate for GUI-visible features. For features whose behavior is only visible through WINDOW DRAGS and pixel gaps, the drag-probe template lives at `scripts/gui_magnet_check.sh`: copy it and adapt per feature — Xvfb + ffmpeg x11grab frames + xdotool drags.
- Web (M8): `rustup target add wasm32-unknown-unknown`, `cargo install trunk --locked --version 0.22.1` (pinned — see "Repo facts"), then from the repo root `trunk serve` / `trunk build --release` (config `Trunk.toml`, page `web/index.html`, output `dist/`). Trunk 0.22.1 is installed at `~/.cargo/bin/trunk` (that directory is NOT on `PATH`): `export PATH="$HOME/.cargo/bin:$PATH"`. Release builds run wasm-opt — the page passes the required flags (see "Repo facts").

## Dependencies — approval flow

The former hard ban ("do not add dependencies") is replaced by an explicit approval flow, so a worthy crate is used instead of being re-invented as an in-house wheel:

1. The agent never adds a dependency silently. When a task needs one, it stops and asks the owner FIRST — this includes bumping an existing dependency beyond its pinned version.
2. The proposal must state: (a) the problem and why it is real; (b) the candidate crates with trade-offs (maintenance, MSRV, weight, transitive dependencies, runtime libs for `flake.nix`, wasm impact); (c) the no-dependency alternative and whether skipping it means re-inventing the wheel; (d) the agent's recommendation.
3. The owner picks an option or rejects the request. Only then does the code land: the version pinned in `Cargo.toml` (SPEC §1 style), `Cargo.lock` updated in the same task, runtime libs added to `flake.nix` `runtimeLibs`, and README/SPEC updated in the same edit when build/run docs change.
4. An opt-in capability ships as a feature-gated optional dependency (`dep:name` under a `[features]` flag) so the default build stays light.

## Repo facts

Cross-cutting facts with no single module home. Per-module facts live in the module doc comments — the module map at the top of `app.rs` is the entry point.

- Single Rust binary crate `russet`: edition 2024, pinned deps from `SPEC.md` §1. Dependencies follow the approval flow above: never added silently, never hand-rolled into an in-house wheel just to avoid asking.
- Toolchain and gate lane: `rust-toolchain.toml` pins channel 1.99.1 (minimal profile + clippy/rustfmt) — rustup installs that exact compiler automatically on the first cargo command inside the repo; `[profile.gates]` in `Cargo.toml` (inherits dev; opt-level 0, no debuginfo) builds gate runs unoptimized. INCREMENTAL is ON in the lane (the cold cost is the dependency graph + the test-binary link, which incremental never touches); the original no-incremental config lives on as the OWNER's personal `[profile.gates-strict]` lane, not part of the agent gate. The GUI dev lane (`[profile.dev]`) is FULLY unoptimized too — dev exists to compile fast and to run the app for visual checks; UI lag is accepted. The dev build stays SLOW on purpose: the owner keeps dev as the early-warning sensor for algorithmic symptoms — never propose or apply dev-profile optimizations (e.g. `[profile.dev] opt-level`) to mask algorithmic slowness. The lanes keep separate `target/` fingerprints — switching between them rebuilds.
- Required module split (monolith forbidden): `main`, `app`, `canvas`, `tools`, `history` (the read-only `ActionLog` convenience view — the real history lives in `layers`), `layers`, `file_io`, `palette`, `text`, `selection`. `app.rs` is the orchestrator (all `App` state, frame flow, undo/redo, the canvas widget, texture sync); cohesive subsystems are its CHILD modules, which can access `App`'s private fields without any visibility churn. Every module's doc comment states what it owns and what its neighbours own — the module map at the top of `app.rs` is the entry point.
- File-size guard (soft): when a source file exceeds 500 lines, do not split it right away and never split mid-task. Instead, warn the owner that the file has outgrown the limit and propose a split — at the END of the current task, after the owner has verified that everything works. Split only with the owner's approval. Test files have their own 300-line guard (see "Tests" rule 7). Owner-approved exceptions: `app.rs` stays the single orchestrator and may remain over 500 lines — agent readability beats the line count there; do not propose splitting it again unless it roughly doubles. `src/app/scenarios/tests.rs` — the OWNER-LOCKED whole-app scenario suite has NO line limit, its split is NOT a debt and is never proposed, and it must not be edited without the owner's explicit permission (see "Tests" rule 7). Owner DEFERRAL (2026-11-17): `history_tree.rs` is NOT to be split and its over-guard warning is NOT to be re-proposed — the owner has an unfinished task of his own in that file; revisit only when he finishes it and approves a split.
- Platform `cfg` branches may appear only in `main.rs` and `file_io.rs` (plus call sites in `app.rs` and the document glue in `app/document.rs`). The About window's platform/renderer strings are `crate::platform_label()` / `crate::renderer_label()` in `main.rs` — plain `cfg!()` macros (no `#[cfg]` attributes), mirroring winit's Linux backend choice: `WINIT_UNIX_BACKEND` override, then `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` → Wayland, else `DISPLAY` → X11. Keep them in sync with the actual backend selection if winit's behavior changes.
- Canvas is a premultiplied `tiny-skia::Pixmap`; feed egui with `ColorImage::from_rgba_premultiplied`. Use `TextureHandle::set_partial(pos, image, options)` for dirty rectangles.
- File I/O: the file dialogs NEVER run on the UI thread: native spawns rfd's blocking call (the pinned 0.15.4 xdg-portal backend) plus the Open-path decode on a `file_io::FileTask` worker thread; WASM must use `rfd::AsyncFileDialog` only and rides the SAME `FileTask` slot. One chooser at a time — the slot is the single-dialog guard; the per-frame poll is `app/document.rs::poll_file_task`.
- Web page: `web/index.html` is the trunk entry. It declares `<link data-trunk rel="rust" href="../Cargo.toml" data-wasm-opt-params="--all-features" />` on purpose — trunk resolves the manifest relative to the HTML file, so the explicit `href` is what makes `web/` work, and trunk 0.22 runs wasm-opt on release builds where the bundled wasm-opt needs `--all-features` for Rust 1.99's bulk-memory output. `#russet_canvas` is hardcoded in `main.rs`: rename in both or neither. `Trunk.toml` pins `target`/`dist`.
- Trunk is PINNED to 0.22.1: `cargo install trunk --locked --version 0.22.1` (README, SPEC §8). The page's `data-wasm-opt-params` is tuned to what trunk 0.22 bundles, so a different trunk can break the release web build. After any trunk upgrade, re-evaluate the flags under PERF.md P9 before trusting the build.
- CI (the single exception to the no-build-tooling rule): `.github/workflows/ci.yml` runs the done-gate on GitHub for every push to `master` and every PR — two jobs, `rules` (grep-level `scripts/rules.sh`, no toolchain) and `native` (`cargo fmt --check` → `cargo clippy --all-targets -- -D warnings` → `cargo test` → the same clippy+test pair with `--features mcp`), with Rust pinned to the owner's local toolchain. Native only — the wasm/trunk job is deliberately postponed. `scripts/rules.sh` is the machine-checkable subset of this file, single source of truth shared with `verify.sh`: HARD = no inline test modules, no `unwrap()` anywhere in `src/` (tests included); SOFT report-only = `expect()` outside test files (each must stay off user-data paths) and the 500-line guard. CI keeps the dev profile on purpose — it stays the independent full-lane verifier on GitHub's machines (the dev profile is fully unoptimized, so CI compiles unoptimized as well), and it is the independent verifier of the `PROGRESS.md` stamps. Keep it green: a red CI on `master` is a stop-everything signal. Editing the workflow (or adding the postponed wasm job) is a build-tooling change and needs the owner's OK.
- Floating windows ("Tools"/"Palette"/"Tool Log"/"Layers"): §3.11 placement (drop-time edge anchoring + per-axis repositioning on main-window resize) is specified in `SPEC.md` §3.11 and implemented in `app/placement.rs` with unit tests in `app/placement/tests.rs`; it covers ALL FOUR floating windows. Anchors are decided only when a drag ends — never during drags, never on resizes. First-open positions are §3.2/§3.3 facts, set per window via `Window::pivot` + `default_pos` and documented at each window builder (the Tools window's 40/60 first-open split is `app/placement.rs::TOOLS_FIRST_OPEN_RATIO`); once a window has a stored area position, `default_pos` is dead and the user's placement wins. The Tools window additionally carries a magnetized settings panel (SPEC §3.2, NOT §3.11 — no edge anchoring, no user placement): the position math is `app/placement.rs::magnet_settings_pos`, and the panel hides while the Tools window is collapsed. Window drop shadows are off (`visuals.window_shadow = Shadow::NONE` in `App::new`; popups/menus untouched).
- All user-facing strings are hardcoded English. No i18n/localization.
- `README.md` is written for HUMANS — people who want to build and run the app: build & run instructions (Windows, NixOS/Linux via `flake.nix`, Web/WASM) and one pointer to `CONTRIBUTING.md` — no feature lists, no changelog, no agent guidance. Agents must read project rules here in AGENTS.md and in SPEC.md, never from README.md. When build/run commands change (SPEC §8, `flake.nix`, Cargo.toml toolchain requirements), update README.md in the same edit.
- `CONTRIBUTING.md` is for OUTSIDERS — humans and foreign agents, both readable. It states how the repo is developed (LLM agents; the owner pushes to `master` directly and does not use PRs), the PR rules for non-owners (ONE logical change per PR so an LLM reviewer can review the diff), the verification gates, and a document map. It is linked from README.md.
- `flake.nix` builds/ runs the app on Linux (Wayland/X11) via `nix build` / `nix run` / `nix develop` (an exception to SPEC §6's "Linux builds/CI"); it also drives the web lane (`nix run .#web-serve` / `.#web-build` / `.#web-build-release` — the web apps are always explicit; plain `nix run` keeps its old meaning: build and start the native release app). **When `Cargo.toml` gains deps with runtime libs, update `runtimeLibs` in the flake.**
- Never `unwrap()`/`expect()` on user-data paths; report errors via `error_popup`. Panics are forbidden.

## eframe/egui 0.37 API quirks

Moved verbatim to `docs/agents/egui-quirks.md` — that file is the canonical home; each quirk keeps its fact + fix, and new quirks are appended THERE, never here. Read it for UI / windows / egui work (see "Reading routes").

## Maintenance

AGENTS.md budget: ~150–200 lines. New durable facts go to their canonical home (module docstring, SPEC §, docs/agents/egui-quirks.md), never as new narrative bullets here; compact an over-budget section in the same edit — mirroring the PROGRESS.md log-entry budget.
