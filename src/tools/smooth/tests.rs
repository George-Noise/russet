//! Tests of the smoothed-stroke module, split by topic (AGENTS.md
//! "Tests" rule 7 — the file grew past ~300 lines): [`window`] tests the
//! midpoint-quadratic emission window (pure geometry), [`raster`] the
//! piece rasterizer for all three tools.

mod raster;
mod window;
