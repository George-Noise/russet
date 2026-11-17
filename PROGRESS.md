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
| Extra: About window + Nix | code written | Help → "About Russet" with Platform/Renderer lines; `flake.nix` for Linux (Wayland/X11) — `nix build` itself must be verified by the owner on a Nix machine |
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

**Verification stamp:** fmt clean, `cargo clippy --profile gates --all-targets -- -D warnings` clean,
`cargo test --profile gates` — 99 green; with `--features mcp` — clippy clean, 111 green;
with `--features diagnostics` — clippy clean, 99 green; `trunk serve --features diagnostics`
smoke-tested earlier 2026-11-16 (build + serve OK, wasm checks green with and without the
feature); the winit-CSD dependency landed (manifest-only, no new tests — re-gated 2026-11-16)
(2026-11-16, Rust 1.99.1 pinned via rust-toolchain.toml, Linux; every src file ≤ 500;
rules.sh reports the PROGRESS.md/Log budgets — green; the later flake/docs round the same
day touched no Rust — cargo gates not re-run).
2026-11-17: the release-strip fix round (manifest-only, no Rust touched) — rules.sh
HARD-green + fmt clean before and after; `trunk build --release` run twice in-sandbox
(`strip = true` → the warning reproduces; `strip = "debuginfo"` → clean; sizes in PERF §1);
clippy/test lanes not re-run — no Rust change and the `gates` profile is untouched.
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
> ever pinned in place by a pending check.

### 2026-11-17 — `Fix:` release `strip = true` deleted the wasm `target_features` section (the wasm-bindgen warning) — owner request

- **What:** every `trunk build --release` warned "the `target_features` custom section is
  missing, likely due to `strip = true`": symbols-stripping is `--strip-all`, which deletes
  ALL wasm custom sections — including the `target_features` list rustc embeds so wasm-bindgen
  can detect the enabled Wasm features (here: bulk-memory, reference-types, multivalue,
  sign-ext). Harmless today (nothing non-default is enabled), but bindgen silently mis-glues
  the day simd128/atomics/threads land, and it already served a guessed empty-default section.
  Fix per wasm-bindgen's own advice: `strip = true` → `strip = "debuginfo"` — keeps the custom
  sections (release `debug` is 0, nothing extra to strip; Windows/MSVC unaffected), the size
  cut stays with wasm-opt AFTER wasm-bindgen (trunk 0.22 runs it on every release build —
  PERF P5). No Rust code touched — manifest + docs only.
- **Files:** `Cargo.toml`, `SPEC.md` §1, `PERF.md` §1 + §4, `DEBTS-verification.md` (V27 born),
  `PROGRESS.md` (+ `PROGRESS-archive.md`: the 2026-11-16 wheel-pan-parity entry compacted
  verbatim — the file hit the ~200-line budget).
- **Verified:** baseline rules.sh HARD-green + fmt clean before the edit; three in-sandbox
  `trunk build --release` runs (trunk 0.22.1, bindgen 0.2.130, wasm-opt 131, toolchain 1.99.1):
  the warning reproduces with `strip = true` and is gone with `"debuginfo"`; dist/ wasm
  5,155,501 → 5,156,370 B (+869 B — the real feature section), JS 76,952 → 78,750 B raw
  (+112 B gzipped) — measured pairs in PERF §1. Manifest-only change → clippy/test lanes not
  re-run (no Rust touched, the `gates` profile untouched); rules.sh + fmt re-run after.
- **Owner-checks:** V27 born — rebuild `trunk build --release` + a browser smoke (load, draw,
  zoom/pan) on the owner's machine.
- **Commit title:** `Fix the wasm-bindgen target_features warning: release strip=true → debuginfo`

### 2026-11-16 — `Feat:` fast dev Nix build (`russet-dev`); V23/V24 verified → F13 paid; the debt-ordering rule — owner request

- **What:** (1) the owner verified V23 + V24 via chat («проверил V23, V24») → both moved to the
  Resolved list; that satisfied F13's documented LAST pending condition (the owner's Wayland
  feel pass), so F13 is PAID (the `diagnostics` feature lives until F12 closes); F12 stays open
  as the final-design decision only. (2) The flake gained `russet-dev` (owner request: a fast
  non-release build to check that everything builds AND to poke the functionality; plain
  `nix build` / `nix run` stay release): the SAME source/lock/tests via buildRustPackage's
  supported profile knobs `buildType = checkType = "gates"` — the repo's gate lane (opt-level 0,
  no LTO; the nixos-unstable hook sources confirm `--profile $cargoBuildType` for the build,
  `cargo test --profile $cargoCheckType` for the check and `target/<triple>/$cargoBuildType`
  for the install) + a `.#russet-dev` app entry; README + SPEC §8 updated in the same edit.
  No nix in the sandbox → NOT machine-verified → V26 born. (3) AGENTS.md "Debts" gained rule 6
  (owner request): open items sorted V → F → rest, newest birth date first within a group,
  closed items leave the open tables the same edit; both DEBTS headers echo it and both files
  were reordered (V16's closed row → Resolved — bookkeeping, announced in chat).
- **Files:** `flake.nix`, `README.md`, `SPEC.md` §8, `AGENTS.md`, `DEBTS-verification.md`,
  `DEBTS-functional.md`, `SANDBOX.md` (pitfall 9: bare-dir `git apply` is a silent no-op),
  `PROGRESS.md`. NO Rust touched → cargo gates not re-run this round.
- **Verified:** rules.sh HARD-green before and after; the `--profile` mechanism quoted from the
  nixos-unstable sources (sparse clone of `pkgs/build-support/rust/`); the delivery
  byte-compared as usual.
- **Owner-checks:** V26 born — `nix build .#russet-dev` / `nix run .#russet-dev` on the Nix
  machine; plain `nix build` unchanged.
- **Commit title:** `Add the fast russet-dev Nix build (gates profile); docs: verify V23 V24, pay F13, add the debt-ordering rule`

### 2026-11-16 — `Fix:` GNOME Wayland title bar — winit Adwaita CSD frame (approved dep) — owner request

- **What:** on the owner's GNOME (Wayland) the window had NO title text and a double-click on
  the title bar did not maximize/restore. Root cause, verified against the pinned sources:
  eframe 0.37 enables only `winit/wayland` → winit falls back to sctk 0.19.2's `FallbackFrame`,
  whose `set_title` is a NO-OP and whose title-bar click emits only `Move` (no double-click
  logic at all); Mutter draws no server-side decorations, so nothing masked it. Fix per the
  dependency approval flow (the owner picked the option in chat): a Linux-only direct `winit`
  dependency with `wayland-csd-adwaita` — same 0.30.13 that eframe resolves, cargo unifies
  them, and winit switches to `AdwaitaFrame` (title text, Adwaita buttons, GTK-like 400 ms
  double-click maximize). Manifest + lock only, no Rust code touched (a unit test cannot reach
  winit's internal frame choice — the fix is verified by the gates + `cargo tree` + V25);
  pure-Rust deps → flake.nix `runtimeLibs` untouched; no commands changed → README untouched;
  the durable quirk moved to AGENTS.md quirks in the same edit.
- **Files:** `Cargo.toml`, `Cargo.lock` (+5 packages, 0 version bumps), `SPEC.md` §1,
  `AGENTS.md` (quirks), `DEBTS-verification.md` (V25 born), `PROGRESS.md`.
- **Verified:** verify.sh ALL GATES GREEN in-sandbox (toolchain 1.99.1 via rust-toolchain.toml):
  rules.sh HARD-green before and after, fmt clean, clippy `-D warnings` clean default + mcp +
  diagnostics, 99 default / 111 mcp tests green; `cargo tree -i sctk-adwaita` shows the feature
  unified into eframe's winit; the delivery byte-compared as usual.
- **Owner-checks:** V25 born — the GNOME Wayland pass on the owner's Linux machine (it also
  unblocks the Linux legs of V23/V24, which had no usable window bar to test against).
- **Commit title:** `Fix GNOME Wayland title bar: enable the winit Adwaita CSD frame (approved dep)`

### 2026-11-16 — `Fix:` Firefox scroll-pan notch retune + the diag→diagnostics rename — owner request

- **What:** the owner's platform pass (Windows/Firefox/Chrome; Linux not yet possible)
  narrowed V23/V24 and surfaced ONE gap: in Firefox one wheel click panned ~3× the
  Windows/Chrome notch — Firefox emits `DOM_DELTA_LINE` with 3 lines per click and eframe
  passes the DOM delta as-is (verified against eframe 0.37.0 sources), so a `Line`-unit
  delta hit `wheel_notches` as 3 notches. Fix inside the INTERIM F12-B convention: on the
  WEB backend only (`web_backend` = `cfg!(target_arch = "wasm32")`), `Line` deltas divide
  by the new `WEB_LINE_UNIT_LINES_PER_NOTCH` 3.0; native stays pass-through; the approved
  Windows/Chrome feel (15 pt/notch, F11 folded) is UNCHANGED; fractional lines stay
  proportional. (2) The rename (owner: the abbreviation was unclear): cargo feature `diag`
  → `diagnostics`, fn `diag_main_resize` → `diagnostics_main_resize`, log prefix `[diag]` →
  `[diagnostics]`; SPEC §3.4 + Cargo.toml + both DEBTS files updated. How-to: native
  `cargo run --features diagnostics`; web `trunk serve --features diagnostics` → logs in
  the browser console. F12 STAYS OPEN (the interim constants retune from the logs).
- **Files:** `src/app/camera.rs` (+ tests — args updated, +1 test), `src/app.rs`,
  `src/app/placement.rs`, `Cargo.toml`, `SPEC.md`, `DEBTS-functional.md`,
  `DEBTS-verification.md`, `PROGRESS.md`, `PROGRESS-archive.md` (the Shift-proportional
  entry moved verbatim).
- **Verified:** verify.sh ALL GATES GREEN — 99 default / 111 mcp; `--features diagnostics`
  clippy+test green (99); wasm check green with and without the feature;
  `trunk serve --features diagnostics` smoke-tested in-sandbox (builds + serves on
  localhost:8080; trunk 0.22.1 re-installed after the sandbox recycle); rules.sh
  HARD-green before/after; delivery byte-compared as usual.
- **Owner-checks:** none born — V23 narrowed (re-check Firefox after the divisor; the
  Linux leg), V24 narrowed (only the Linux/Wayland leg) — updated in DEBTS-verification.md.
- **Commit title:** `Fix Firefox wheel-pan overshoot (web Line ÷ 3 lines/notch) and rename the diag feature to diagnostics`
