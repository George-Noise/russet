//! Tests of the floating-window behaviors owned by `windows.rs`, split by
//! topic (AGENTS.md "Tests" rule 7 — the file grew past ~300 lines):
//! [`common`] carries the shared headless-pass helpers, [`settings_panel`]
//! the magnetized tool-settings panel and the collapse rule, [`tools_strip`]
//! the Tools window's title-strip fit, [`color_well`] the Palette window's
//! color well (the picker alpha + the hex field, owner request 2026-11-19).

mod color_well;
mod common;
mod settings_panel;
mod tools_strip;
