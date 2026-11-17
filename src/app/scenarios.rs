//! The whole-app scenario suite (owner request 2026-11-17): ONE test file
//! ([`tests`]) that drives a REAL headless `App` through the owner's manual
//! walkthrough scenarios — layers + canvas-mode Resize + per-layer history
//! trees + undo/redo/node jumps in FULL runs, exactly as a user chains them.
//! The model lives in [`crate::layers`], the App orchestration in [`super`];
//! this shim only mounts the suite as a child module of `app` (the
//! established `<module>/tests.rs` pattern of AGENTS.md "Tests").
//!
//! OWNER-LOCKED (owner request 2026-11-17): editing the suite WITHOUT the
//! owner's explicit permission is forbidden — the agent only ADDS new
//! scenarios on an explicit owner request. The file is exempt from BOTH
//! size guards (the tests' ~300-line guard and the 500-line file guard) and
//! splitting it is NOT a debt (AGENTS.md "Tests" rule 7 + "Repo facts";
//! `scripts/rules.sh` skips it in the size-guard loop).

// The suite reaches `App` through its own `super` (this shim): one private
// re-import here, so `tests.rs` keeps the plain `use super::App;` of the
// AGENTS.md child-module pattern. Test-only — no production code.
#[cfg(test)]
use super::App;

#[cfg(test)]
mod tests;
