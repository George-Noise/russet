//! Tests of the floating-window behaviors owned by `windows.rs`, split by
//! topic (AGENTS.md "Tests" rule 7 — the file grew past ~300 lines):
//! [`common`] carries the shared headless-pass helpers, [`settings_panel`]
//! the magnetized tool-settings panel and the collapse rule, [`size_block`]
//! the panel's Size block (label above, narrowed slider, the manual-entry
//! box, the rows gap), [`tools_strip`] the Tools window's title-strip fit.
//! The Palette window's color-well tests live in the child module
//! (`palette_window/tests.rs`) — they followed the palette code there in
//! the owner-approved 2026-11-19 split (AGENTS.md "Tests" rule 2: move the
//! test, not the visibility).

mod common;
mod settings_panel;
mod size_block;
mod tools_strip;
