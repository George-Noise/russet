# PROGRESS — Russet

> **Why this file exists.** `.git` reaches the agent only when the owner includes it in the
> delivery, and it is gone once the agent sandbox is recycled. Git is therefore NOT a
> progress channel in this setup. **This file is the substitute for the git history**: read
> it before starting work, append to it after every finished, verified task
> (see AGENTS.md "Git & progress log" for the exact routine). Entries older than the
> newest 3–5 live in `PROGRESS-archive.md` (grep it for history).

## Status snapshot — update in the same edit as a log entry

| Milestone | Status | Notes |
|---|---|---|
| M0 Scaffold | done | |
| M1 Canvas + zoom/pan | done | pan: wheel / Shift+wheel / middle / Space+LMB / WASD (physical keys) |
| M2 Dirty-region texture updates | done | |
| M3 Tools + history | done | |
| M4 Menu + windows + §3.11 placement | done | §3.11 reworked once — see PROGRESS-archive.md, 2026-11-09 "magnetism rework" |
| M5 Files (native, Windows) | done | |
| M6 Undo/Redo + hotkeys | done | |
| M7 Resize Canvas + New | done | |
| M8 Web (WASM) | built + headless Firefox check (2026-11-09) | wasm32 check/clippy green; `trunk build --release` OK (wasm 5.0 MB + 77 KB JS; page `web/index.html` + `Trunk.toml`); `trunk serve` + Firefox: UI renders, canvas draws, WebGL2 — a headless smoke test, not a human pass. Owner must click through drawing/zoom/pan/Open/Save with a real mouse |
| M9 Acceptance + release | pending (owner) | §7 checklist is manual, on the owner's Windows machine; `cargo build --release` is run by the owner |
| Extra: About window + Nix | code written | Help → "About Russet" with Platform/Renderer lines; 2026-11-18: + "written in Rust" description + GitHub repo hyperlink; `flake.nix` for Linux (Wayland/X11) — `nix build` itself must be verified by the owner on a Nix machine |
| Extra: `tests.rs` layout + `PERF.md` (PR) | done (2026-11-10) | every module's tests live in `<module>/tests.rs`; `PERF.md` is a reference RAM/build-speed backlog, not an autonomous queue |
| Extra: README.md | done | English, build & run only (Windows / NixOS / Web); declared the human-facing doc in AGENTS.md + SPEC §8 |
| Extra: 6 tools/panels (owner 2026-11-10) | code written, tests green | Eyedropper (I), Line/Curve (L), Shape (U), Text (T), History log window, Layers placeholder window — see the log entries below; deliberately simple first iterations, followed by a UX-polish patch the same day (window positions, Shape outline+fill, preview fidelity) |
| Extra: CI + CONTRIBUTING.md + doc-tree removal (owner 2026-11-11) | done — first CI run GREEN (success for `0188c52`, GitHub Actions) | `.github/workflows/ci.yml` (native gate: fmt/clippy/test, Rust pinned 1.99.1; wasm job postponed by the owner); `CONTRIBUTING.md` + PR template for outsiders (one logical change per non-owner PR; the owner pushes directly); file trees removed from AGENTS.md and SPEC §2 — the filesystem is the source of truth; trunk pinned 0.22.1 everywhere, PERF P9 added |
| Extra: machine rules job (owner 2026-11-11) | code written, rules.sh tested in sandbox | new `scripts/rules.sh` — grep-level subset of AGENTS.md, single source of truth shared by verify.sh and CI: HARD (exit 1) = inline test modules + `unwrap()` anywhere in `src/`; SOFT report-only = `expect()` outside tests (5 known sites listed) + 500-line guard; CI gained a fast `rules` job (no Rust toolchain) |
| Extra: dependency approval flow (owner 2026-11-12) | done | AGENTS.md "Dependencies — approval flow" + SPEC §6 exception + CONTRIBUTING.md — the hard no-deps ban became propose → discuss → owner decides; no code touched |
| Extra: opt-in MCP server (owner 2026-11-12) | done — 10 tools behind `mcp` | default build byte-identical graph (no serde_json/threads); `get_tree_layout` + `add_layer` added 2026-11-13; verify.sh gates the feature |
| Extra: History Tree windows (owner 2026-11-12) | done — lanes reworked 2026-11-13 | divergence-only forks, recency lanes, shared connector exits, per-row labels; MCP-driven scenarios + Xvfb screenshots green |
| Extra: 500-guard splits (owner 2026-11-13) | done — pure code motion | `history_tree/layout.rs`, `tree_window/paint.rs`, `app/document.rs` + `menu.rs` + `input.rs`; app.rs 936 → ~440, every src file ≤ 500 |
| Extra: agent workflow v2 (owner 2026-11-13) | done | DEBTS-verification.md + DEBTS-functional.md debt lists; rules.sh PROGRESS budget check; baseline-before-edits rule; CI runs the mcp gates; visual_driver.py rewritten as the scenario runner |
| Extra: REAL layers + per-node PNG history (owner 2026-11-13) | done — gates green | pays F1+F4: interactive Layers window, per-layer git-style trees whose nodes carry full fast-PNG canvases, node-dot LEFT-click switching; SPEC §3.6 rewritten; GUI click-through = V13 (owner) |
| Extra: headless-GUI sandbox recipe (owner 2026-11-13) | done — driver all green headless | `SANDBOX.md` + `scripts/gui_recipe_test.sh`: raw Xvfb + two user-space .debs, no root; MCP `add_layer` added (10 tools); visual_driver.py 3/3 scenarios green in-sandbox; V13 narrowed to pointer feel |
| Extra: MCP tools split + commit-title ritual (owner 2026-11-13) | done — pure code motion | root `mcp/tools.rs` = the tool map (dispatch + declarations, 157); themed `tools/queries.rs` (200) + `tools/edits.rs` (177); rules.sh rule 6 prints the newest commit title (backticks kept) at every done-gate; AGENTS.md Delivery ritual requires it verbatim, backtick-wrapped, in the reply |
| Extra: UX round (owner 2026-11-13) | done — gates green | History window → Tool Log (list max height halved to 140 pt, 0.5rem entries, zero row spacing, bottom edge flush), Help item → About, dialog button rows per IBM/Apple/Google via shared `dialog_button_row`, Shape RMB-drag swaps the color wells, wheel pan ×2 (`SCROLL_PAN_SPEED` 3.0) |
| Extra: dialog epoch + Shape RMB round (owner 2026-11-15) | done — gates green | dialogs arise centered on EVERY open and hug their content (`dialog_window` + `forget_dialog_position`, NOT §3.11); Shape RMB uses the wells inverted WITHOUT swapping; Palette "Swap (X)" → "Swap"; 3 headless-egui regression tests + a scripted 9-check real-binary pass |
| Extra: layers split + two-color history chips (owner 2026-11-16) | done — gates green | pure code motion into `layers/snapshot.rs` + `layers/composite.rs` (tests split the same way); `TreeNode.color2` + the outline\|fill chip halves for two-color gestures; MCP `get_history_tree` dumps `color2`; preview's pure color rules → `app/preview/colors.rs` |
| Extra: Shift-proportional shapes (owner 2026-11-16) | done — gates green | Shape (U) drag with held Shift = square/circle, Paint.NET-style, LIVE (mid-drag snap/release); pure `proportional_current` in `app/preview.rs` + 5 unit tests; SPEC §3.2 bullet + §3.9 hold-gesture row; V22 born |
| Extra: third debt triage (owner 2026-11-16) | done — docs only | V13/V14/V17–V22 verified via chat; V15 closed into the NEW F11 (wheel pan ANOTHER ×5 — `SCROLL_PAN_SPEED` 3.0 → 15.0, how-to recorded in DEBTS-functional.md); AGENTS.md gained the zip-priority rule (a delivered .zip is used immediately — no git pull/clone); open after this edit: V8, V9, F3, F5, F8–F11 |
| Extra: V9 pass + platform-gap analysis (owner 2026-11-16) | done — docs only | V9 verified via chat (web build clicked through in a browser); the pass surfaced F12 (wheel-pan platform parity — `camera.rs` reads the RAW wheel delta in EVENT UNITS: Line ±1/notch on Windows/X11 vs DOM px as-is on the web) and F13 (window jitter on Wayland/Web resize — per-frame repositioning meets non-monotonic content rects); both analysis-only, fixes postponed by the owner |
| Extra: wheel-pan parity F12-B + resize-jitter fix F13-A + diag feature (owner 2026-11-16) | done — gates green | owner picked F12 variant B (INTERIM, not final; F11 folded into the math) and F13 variant A, approved the diagnostic patch behind a build flag; `wheel_notches` (unit → notches) + fixed `WHEEL_NOTCH_PAN` 15 pt/notch on every platform; `resize_step` pins the pre-resize rects and recomputes from the pin; temporary `diag` cargo feature ([diag] wheel/resize/reposition logs, compiled out of the default build); SPEC §3.4 + §7 rewritten; V23/V24 born |
| Extra: web-Line notch retune + `diag` → `diagnostics` rename (owner 2026-11-16) | done — gates green | Firefox scroll-pan ~3× overshoot fixed inside F12-B: Firefox emits `DOM_DELTA_LINE` ×3 lines/notch, so web `Line` deltas now divide by `WEB_LINE_UNIT_LINES_PER_NOTCH` 3.0 (native pass-through); the approved Windows/Chrome feel UNCHANGED (15 pt/notch); the temporary feature renamed `diag` → `diagnostics` (owner: the abbreviation was unclear) — feature, cfg sites, fn, `[diagnostics]` prefix; trunk-serve how-to: `trunk serve --features diagnostics` (logs → browser console); V23/V24 narrowed, not closed |
| Extra: Resize modes + per-layer resize nodes (owner 2026-11-17) | done — gates + headless driver green | the dialog's Canvas mode (3×3 anchor) / Image mode (nearest/bilinear/bicubic/Lanczos3 + aspect lock); ONE `resize` node per layer with the full `ResizeInfo`; the bottom layer owns the canvas size; the delete-bottom "Canvas Size Source" choice; native-size restores (F9 updated); V28 born |
| Extra: CLI launch args + owner-unconfirmed scenario markers (owner 2026-11-17) | done — gates green | native `--primary/--secondary/--tool/--canvas-size/--help` (`app/launch.rs`, parsed before the window opens; invalid values warn, defaults stay; wasm takes none); the 8 suite scenarios carry "NOT YET CONFIRMED" markers + a header ledger (the owner approves one by one); V29/V30 born |
| Extra: repo-wide English-only owner quotes (owner 2026-11-18) | done — docs only | every verbatim Russian owner quote across docs/scripts/comments translated to English (meaning kept); PROGRESS Log compacted — 2 entries moved to PROGRESS-archive.md |
| Extra: debt F14 — in-house windows later (owner 2026-11-18) | done — docs only | egui::Window → an in-house `egui::Area` frame recorded in DEBTS-functional.md from the owner's pros/cons chat review; owner-gated, no code touched |
| Extra: tool icon buttons + two-column grid (owner 2026-11-18) | done — gates green + visual driver | 1.5rem icon buttons via a cached tiny-skia atlas (`app/tool_icons.rs`), colored pencil/brush/eraser/pipette + mono line/shape/text through ONE pipeline, name+key hover tooltips, the pipette as the Eyedropper's canvas cursor; the two-column grid is debt F15; the driver screenshot caught the dead-texture trap (handles kept in the Atlas now — AGENTS.md quirks) |
| Extra: fully unoptimized dev profile (owner 2026-11-18) | done — cargo check green on the new lane | `[profile.dev]` opt-level 0 + no debuginfo, the `[profile.dev.package."*"] opt-level 2` dep override REMOVED (the "deps optimized once for responsive `cargo run`" policy retired — development is sandbox-only now, UI lag accepted); flake `russet-dev` already ran the unoptimized gates lane (untouched); SPEC §1 + AGENTS.md synced; owner visual pass = V34 |
| Extra: verify.sh hardening (owner 2026-11-18) | done — FULL gate green on a cold lane (191 s) | `exec </dev/null` closes stdin for the whole run (SANDBOX.md trap 10 fixed AT THE SOURCE — the mcp test stage exits in seconds instead of hanging); per-stage seconds + total printed (TIMING.md rows are copied now, not re-measured); cold-lane hint when `target/gates` is absent; the mcp pair stays UNCONDITIONAL (hot gate ≈ 15 s, zip sandboxes have no git, the gate never guesses) |
| Extra: brush + pipette icons redrawn + equal grid gaps (owner 2026-11-18) | done — hot gate 14 s + driver green | round volumetric bristle belly + crimped ferrule (the wedge read as a second pencil); tapering glass cone + round rubber bulb via a new `arc_pts` helper (the rectangle stick read as a pen); Tools-grid H-gap 18 pt → 6 pt = V-gap (egui Grid floors columns at `interact_size.x` 40 pt — `.min_col_width` fix, AGENTS.md quirk); tool_icons.rs 610 lines — the draw/atlas split proposed (owner-gated) |
| Extra: --warm warm-up + session rules; debts ask = 4 newest; V33 closed (owner 2026-11-18) | done — live cold-lane test + hot gate | `sandbox_check.sh --warm` = probe + detached gates-lane compile (clippy + test --no-run, nothing executed — the mcp stdin trap impossible) with a double-start pid guard; session start = `sandbox_check.sh --warm` FIRST; wasm ad-hoc checks only for web-facing changes; the batched debts ask lists only the 4 newest open items (AGENTS.md Debts rule 7) |

**Verification stamp:** FULL gate re-verified 2026-11-18 (fresh sandbox): fmt clean, rules.sh
HARD-green, clippy `-D warnings` green (default + mcp), `cargo test --profile gates` 156 default /
168 mcp green (+11 new tool-icon tests), `cargo check --target wasm32-unknown-unknown` green,
visual driver ALL SCENARIOS GREEN + `--shots` — Rust 1.99.1 pinned via rust-toolchain.toml, Linux.
The hardened gate (stdin shield + stage timers, same day) then re-ran EVERYTHING on a cold lane
after the dev-profile de-optimization: 191 s total, 156/168 green again — the profile change is
fully gate-verified. The icon-redraw round (same day) exercised the hardened gate on a REAL
change: hot run 14 s (156/168 green), the glass test caught a real drawing bug pre-delivery,
driver ALL SCENARIOS GREEN, the grid gaps pixel-measured 6 pt = 6 pt. The tooling round (same
day) proved `sandbox_check.sh --warm` on a truly cold lane (0.6 s return, the double-start
guard fired, a hot gate after the log's "finished") and closed V33 (NixOS + Windows).
Zero Cyrillic repo-wide. Web/trunk lanes last verified 2026-11-16 (see the archived log history);
every src file ≤ 500 EXCEPT the report-only owner-deferred `history_tree.rs` 569 AND
`app/tool_icons.rs` 610 (grew in the 2026-11-18 icon redraw — the natural split is the icon
draw fns vs the atlas/cursor pipeline; proposed to the owner at delivery, not done without
his approval).

Update this stamp in the same edit as a log entry (AGENTS.md "Git & progress log").

## Warnings — open issues deliberately NOT fixed yet

- 2026-11-13 — No active warnings. Resolved items moved to PROGRESS-archive.md;
  open HUMAN checks live in DEBTS-verification.md, open functional debts in
  DEBTS-functional.md (the agent must ask about them per AGENTS.md "Debts").

## Log — newest first. Insert a new entry directly under this heading.

> Budget: ≤ ~20 lines per entry, fixed fields (What / Files / Verified / Owner-checks /
> Commit title); durable facts move to their canonical home in the same edit (AGENTS.md
> "Git & progress log"). Entries older than the newest 3–5 are moved verbatim to
> PROGRESS-archive.md. Open human checks live in DEBTS-verification.md — no entry is
> ever pinned in place by a pending check. Compacted 2026-11-18: the panels-split and
> docs rounds moved to PROGRESS-archive.md; later the same day the F14 + quotes rounds
> followed them (the verify.sh round needed the budget back), the tool-icons entry
> joined them in the evening (the icon-redraw round), and the About entry followed
> it (V33's closure freed it — the tooling round's budget).

### 2026-11-18 — `Tooling:` warm-up flag + session rules; debts-ask trimmed; V33 closed — owner request

- **What:** four owner orders (2026-11-18), one docs/tooling round, NO src/ change:
  (1) `sandbox_check.sh --warm` — after the usual probe it starts a DETACHED gates-lane warm-up
  (clippy + `cargo test --no-run --features mcp`: exactly the compile work verify.sh does, nothing
  EXECUTED — no test binary spawns, so the mcp stdin trap is impossible here even without the
  shield), a pid-file guard blocks double starts; the session-start rule now runs
  `sandbox_check.sh --warm` FIRST — the done-gate is hot when the task ends (in-sandbox
  compilation; the retired warm-target STORE stays retired); (2) AGENTS.md wasm-check economy:
  ad-hoc `cargo check --target wasm32-unknown-unknown` ONLY for web-facing changes — neither
  verify.sh nor CI runs wasm, so an unneeded run is pure lost minutes; (3) debts rule 7
  (AGENTS.md + DEBTS-verification.md): the batched ask lists ONLY the 4 newest open items,
  newest first — the files keep every open item; (4) V33 verified (owner: About checked on
  NixOS AND Windows).
- **Files:** `scripts/sandbox_check.sh` (--warm), `AGENTS.md` (session-start bullet, wasm
  bullet, Debts rule 7), `DEBTS-verification.md` (presenting bullet, V33 → verified, V33's
  source ref → archive), `TIMING.md` (cold warm-up row), `PROGRESS.md` (compaction: the About
  entry moved; snapshot row + this entry + stamp).
- **Verified:** live test on a truly cold lane (`rm -rf target/gates`): `--warm` returned in
  0.6 s with the background build compiling (log /tmp/russet_warm_build.log), a second `--warm`
  hit the guard ("already running, pid N"), and after the log's "finished" the same-session
  verify.sh ran HOT (this round's gate run below); rules.sh HARD-green; no src/ change — no
  wasm check (the new economy rule's first application).
- **Owner-checks:** V33 closed. Open after this edit: V35 (the icon look), V34 (the
  unoptimized-dev pass), V30 (parked), V8 (parked).
- **Commit title:** `Add sandbox_check --warm warm-up; AGENTS wasm-economy + debts-ask rules; close V33`

### 2026-11-18 — `Feat:` brush + pipette icons redrawn; equal Tools-grid gaps — owner request

- **What:** the owner tested the hardened gate on a real task (2026-11-18): (1) the BRUSH
  icon's chisel wedge read as a second pencil — redrawn as a classic round brush: a ROUND
  volumetric bristle belly (peak half-height 2.7 pt past the ferrule) split into light/dark
  facets for volume, a metal ferrule with two crimp ridges, the loaded paint tip kept;
  (2) the PIPETTE's all-rectangle stick read as a pen — redrawn as a classic eyedropper:
  a tapering glass cone with a blue liquid stripe (the tip full of the sample), a WIDER metal
  collar band, and a ROUND rubber bulb (a new `arc_pts` helper tessellates the round parts);
  `PIPETTE_TIP` and the cursor hotspot unchanged; (3) the Tools grid's horizontal gap was
  18 pt vs 6 pt vertical — ROOT CAUSE inside egui's own `grid.rs`: every column is floored
  at `interact_size.x` (40 pt), padding the 28 pt buttons with dead width; fixed with
  `.min_col_width(TOOL_BUTTON_PT)` — pixel-measured 6 pt = 6 pt after the fix; quirk
  recorded in AGENTS.md.
- **Files:** `src/app/tool_icons.rs` (draw_brush + draw_pipette redrawn, `arc_pts`, the glass
  widened so the outline doesn't eat it), `src/app/windows.rs` (`min_col_width`),
  `src/app/tool_icons/tests.rs` (the glass assert follows the new palette), `AGENTS.md`
  (Grid quirk), `PROGRESS.md` (compaction: the tool-icons entry moved to PROGRESS-archive.md;
  snapshot row + this entry + stamp).
- **Verified:** the design was iterated on rendered PNGs (256 px + real 24 px contact sheets),
  then the FULL hot gate: ALL GATES GREEN 14 s (156 default / 168 mcp, fmt + clippy
  `-D warnings` clean) — the FIRST gate run caught a real bug: the glass band was thinner
  than two outline half-widths, so zero opaque glass pixels survived and the glass test
  failed; fixed by widening the cone; headless visual driver ALL SCENARIOS GREEN + `--shots`,
  the Tools window re-measured pixel-exact after the fix (H 6 / V 6).
- **Owner-checks:** V35 born (the redrawn-icons look pass on Windows). Open: V35 (new), V34
  (the unoptimized-dev pass), V33 (About), V30 (parked), V8 (parked).
- **Commit title:** `Redraw brush + pipette icons; equal Tools-grid gaps (egui Grid interact_size quirk)`

### 2026-11-18 — `Tooling:` verify.sh hardened — stdin shield, stage timers, cold-lane hint — owner request

- **What:** the owner (2026-11-18: "make verify.sh good — you work with this file") ordered the
  done-gate reworked. Three changes, NO lane/semantics change (the rules → fmt → clippy → test →
  mcp-clippy → mcp-test chain is identical): (1) `exec </dev/null` closes stdin for the WHOLE run —
  SANDBOX.md trap 10 (mcp test binaries' stdin reader hangs the cargo ↔ test-binary teardown on a
  recycled sandbox's open pipe/pty even after green tests) is fixed AT THE SOURCE; the
  `verify.sh < /dev/null` habit is dead (a BARE `cargo test --features mcp` outside the gate still
  needs the manual pipe). (2) Every stage prints its own seconds and the total — TIMING.md rows are
  copied from the output now, not re-measured. (3) A cold-lane hint prints when `target/gates` is
  absent. The mcp pair stays UNCONDITIONAL on purpose: the whole hot gate is ~15 s, zip-delivered
  sandboxes usually have no git, and the done-gate must never depend on change-detection guesses.
- **Files:** `scripts/verify.sh`, `SANDBOX.md` (trap 10 rewritten: fixed at the source),
  `TIMING.md` (cold row), `PROGRESS.md` (compaction: the F14 + quotes rounds moved to
  PROGRESS-archive.md; snapshot row + this entry + stamp).
- **Verified:** the NEW script's first live run = the FULL gate on a fully COLD lane, end to end:
  rules 0 s, fmt 0 s, clippy 84 s, test 156 green (98 s), mcp clippy 3 s, mcp test 168 green
  (6 s — the stdin reader EOF'd instantly, no hang: first live proof of the shield), ALL GATES
  GREEN 191 s total. This also gate-verifies the 2026-11-18 dev-profile change end to end (the
  morning round had skipped the full gate as a config-only change). The PROGRESS compaction and
  this entry are md-only edits AFTER the gate run — rules.sh re-run at delivery covers them.
- **Owner-checks:** none born. Open: V34 (the unoptimized-dev pass), V33 (About), V30 (parked), V8 (parked).
- **Commit title:** `Harden verify.sh: exec </dev/null stdin shield, per-stage timers, cold-lane hint`

### 2026-11-18 — `Build:` dev profile fully unoptimized (opt-level 0, no debuginfo, no dep overrides) — owner request

- **What:** the owner (2026-11-18) retired the PARTIAL dev optimization: development now happens
  entirely inside the agent sandbox, so `[profile.dev]` exists only to compile fast and to run the
  app for visual checks — UI lag is accepted (owner: "it's dev — it's fine if it lags"; he judges
  the build-speed-vs-checking trade-off himself → V34). `[profile.dev]` is now `opt-level = 0` +
  `debug = false`, and the `[profile.dev.package."*"] opt-level = 2` dependency override is GONE —
  the crate AND every dependency compile unoptimized. The former policy ("dependencies optimized
  ONCE and taken from cache — the UI stays responsive") dated from the owner's-machine era and made
  the one-time optimized dep compile the dominant cold cost on 2-core sandboxes. `[profile.gates]`
  is functionally unchanged — its explicit opt-level/debug/incremental overrides simply mirror dev's
  new defaults now. The flake's `russet-dev` ALREADY built in the unoptimized `gates` lane (owner
  request 2026-11-16) — nothing to remove there. CI's native job keeps the (now unoptimized) dev
  profile untouched and gets faster automatically.
- **Files:** `Cargo.toml`, `SPEC.md` (§1 profile block + the "About build speed" note), `AGENTS.md`
  (Repo facts toolchain bullet; done-gate + CI bullets wording), `TIMING.md` (first cold row under
  the new policy), `DEBTS-verification.md` (V34 born), `PROGRESS.md` (snapshot row + entry + stamp).
- **Verified:** config-only round, no src/ change — `scripts/rules.sh` HARD-green, `cargo fmt
  --check` clean, `cargo check` green on the NEW dev profile (fresh sandbox, cold, time'd exactly:
  1m31s vs 3m51s the same morning under the old optimized deps — TIMING.md). The gates test lane is
  untouched by construction (every changed dev key is explicitly overridden in `[profile.gates]`),
  so clippy/test results are unaffected; the full verify.sh was not re-run for this round.
- **Owner-checks:** V34 born (the unoptimized-dev visual pass on Windows + the build-speed judgment).
- **Commit title:** `Make the dev lane fully unoptimized (opt-level 0, no debuginfo, no dep overrides); sync SPEC §1 and AGENTS.md`
