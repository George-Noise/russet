//! Tests of the tool-icon rasterization (owner request 2026-11-18): the
//! pure tiny-skia pipeline (colored + monochrome icons through the SAME
//! primitives), the atlas lifecycle — rasterized once per scale change
//! (theme-independent since the v4 pass), served as cached egui textures —
//! and, since owner request 2026-11-20, the Shape panel's icon set. The
//! headless part runs real egui passes through [`egui::Context::run_pass`]
//! (no renderer, no App), the same pattern the dialog tests use.
//!
//! TOPICS (Tests rule 7; the file grew past the ~300-line guard with the
//! Shape-panel round): the per-tool drawings in [`drawings`], the
//! atlas/cursor pipeline + the icon dump in [`atlas`], the Shape panel's
//! set in [`shape_panel`]. The pixel-poking helpers below are shared by all
//! three topics (children reach them with `use super::*;`).

use super::{SHAPE_ICONS, TOOL_INK, TOOLS, rasterize_shape_icon, rasterize_tool_icon};
use eframe::egui::Color32;

mod atlas;
mod drawings;
mod shape_panel;

/// Opaque-pixel threshold: every icon must draw a substantial mark in its
/// 24×24 logical space (guards against a silently-empty raster).
const MIN_OPAQUE_PIXELS: usize = 40;

fn count_opaque(pm: &tiny_skia::Pixmap) -> usize {
    pm.pixels().iter().filter(|p| p.alpha() > 0).count()
}

fn has_color(pm: &tiny_skia::Pixmap, rgb: (u8, u8, u8), tol: u8) -> bool {
    let (r, g, b) = rgb;
    pm.pixels().iter().any(|p| {
        p.alpha() > 200
            && p.red().abs_diff(r) <= tol
            && p.green().abs_diff(g) <= tol
            && p.blue().abs_diff(b) <= tol
    })
}

/// Whether any mostly-opaque pixel sits within `tol` LOGICAL units of the
/// logical-space point `pt` (the tests speak the icons' 24×24 language;
/// the helper maps it through the raster's scale).
fn has_opaque_near(pm: &tiny_skia::Pixmap, pt: (f32, f32), tol: f32) -> bool {
    let k = pm.width() as f32 / 24.0;
    let (x, y) = (pt.0 * k, pt.1 * k);
    pm.pixels().iter().enumerate().any(|(i, p)| {
        p.alpha() > 200
            && ((i as f32 % pm.width() as f32) - x).abs() <= tol * k
            && ((i as f32 / pm.width() as f32) - y).abs() <= tol * k
    })
}

/// How many mostly-opaque pixels fall INSIDE the logical-space box `bx`
/// (the icons' 24×24 language, mapped by the raster's scale) — the crisp
/// solid-vs-hollow discriminator: the interior box holds no AA fringe.
fn count_opaque_in_box(pm: &tiny_skia::Pixmap, bx: (f32, f32, f32, f32)) -> usize {
    let k = pm.width() as f32 / 24.0;
    let (x0, y0, x1, y1) = (bx.0 * k, bx.1 * k, bx.2 * k, bx.3 * k);
    pm.pixels()
        .iter()
        .enumerate()
        .filter(|(i, p)| {
            let (x, y) = (
                (*i as f32) % pm.width() as f32,
                (*i as f32) / pm.width() as f32,
            );
            p.alpha() > 200 && x >= x0 && x < x1 && y >= y0 && y < y1
        })
        .count()
}

/// The shared smoke pass over EVERY raster the pipeline serves: the seven
/// tool icons (the pipeline's original contract) and the Shape panel's five
/// fixed drawings (owner request 2026-11-20 — kind-independent).
#[test]
fn every_icon_rasterizes_with_a_substantial_mark() {
    for tool in TOOLS {
        let pm = rasterize_tool_icon(tool, 24, Color32::WHITE).expect("px is in 1..=256");
        assert!(
            count_opaque(&pm) >= MIN_OPAQUE_PIXELS,
            "{tool:?} must draw more than {MIN_OPAQUE_PIXELS} opaque pixels"
        );
    }
    for icon in SHAPE_ICONS {
        let pm = rasterize_shape_icon(icon, 24, TOOL_INK).expect("px is in 1..=256");
        assert!(
            count_opaque(&pm) >= MIN_OPAQUE_PIXELS,
            "{icon:?} must draw more than {MIN_OPAQUE_PIXELS} opaque pixels"
        );
    }
}
