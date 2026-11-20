//! Tests of the smoothed-stroke module, split by topic (AGENTS.md
//! "Tests" rule 7 — the file grew past ~300 lines): [`window`] tests the
//! midpoint-quadratic emission window (pure geometry), [`raster`] the
//! piece rasterizer for all three tools, [`trail`] the pencil's drifting
//! pen (the 1 px staircase quality + the frame-batch invariance).

mod raster;
mod trail;
mod window;
