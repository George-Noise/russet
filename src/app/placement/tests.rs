//! §3.2/§3.11 placement tests, split BY TOPIC (owner-approved 2026-11-17,
//! "Tests" rule 7): this file keeps only the `mod` declarations — the
//! shared helpers live in [`common`], the suites in the topic files
//! ([`anchors`], [`reposition`], [`resize`], [`first_open`], [`panel`]).
//! The topic files reach the placement items with
//! `crate::app::placement::…` (descendant modules see the private items).

mod anchors;
mod common;
mod first_open;
mod panel;
mod reposition;
mod resize;
