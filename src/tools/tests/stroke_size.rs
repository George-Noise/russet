//! The manual-entry regression (owner bug report 2026-11-20: a typed 200
//! still drew at radius 100 — the UI accepted the value, the pixels ignored
//! it): a size above 100 must reach the DRAWN stroke. Every rasterizer
//! clamped the width to the slider-era 1..=100, a legacy of the time the
//! slider was the only size input. The probes sample a point only a stroke
//! wider than 160 can cover (perpendicular offset ≥ 80 px — beyond the
//! old clamp's 50-px half-width reach, with a ≥ 15 px anti-aliasing margin
//! from the new band's edge).

use super::super::{
    MAX_STROKE_SIZE, PencilTrail, ShapeKind, ShapeOutline, SmoothPiece, Tool, draw_segment,
    rasterize_polyline, rasterize_shape, rasterize_smooth_pieces,
};
use eframe::egui::{Color32, pos2};
use tiny_skia::Pixmap;

/// The alpha at one pixel — 0 means unpainted.
fn alpha(pixmap: &Pixmap, x: u32, y: u32) -> u8 {
    pixmap.pixel(x, y).map_or(0, |p| p.alpha())
}

#[test]
fn smooth_brush_stroke_honors_a_width_above_100() {
    let mut pixmap = Pixmap::new(300, 200).expect("300x200 pixmap");
    let pieces = [SmoothPiece::Line {
        from: pos2(20.0, 100.0),
        to: pos2(280.0, 100.0),
    }];
    let mut trail = PencilTrail::default();
    rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Brush,
        200.0,
        Color32::BLACK,
        &pieces,
        &mut trail,
        None,
    )
    .expect("stroke rasterizes");
    // 85 px below the centerline: inside a 200-wide band (half-width 100,
    // the edge at y = 200), outside the old clamp's 50-px half-width band.
    assert_eq!(alpha(&pixmap, 150, 185), 255);
}

#[test]
fn draw_segment_stroke_honors_a_width_above_100() {
    let mut pixmap = Pixmap::new(300, 200).expect("300x200 pixmap");
    draw_segment(
        &mut pixmap,
        Tool::Brush,
        200.0,
        Color32::BLACK,
        pos2(150.0, 20.0),
        pos2(150.0, 180.0),
    )
    .expect("segment rasterizes");
    // 85 px right of the vertical centerline at x = 150 (the band edge at
    // x = 250): painted at width 200, unpainted under the old 100 clamp.
    assert_eq!(alpha(&pixmap, 235, 100), 255);
}

#[test]
fn line_tool_polyline_honors_a_width_above_100() {
    let mut pixmap = Pixmap::new(300, 200).expect("300x200 pixmap");
    rasterize_polyline(
        &mut pixmap,
        &[pos2(20.0, 100.0), pos2(280.0, 100.0)],
        200.0,
        Color32::BLACK,
    )
    .expect("polyline rasterizes");
    assert_eq!(alpha(&pixmap, 150, 185), 255);
}

#[test]
fn shape_outline_honors_a_width_above_100() {
    let mut pixmap = Pixmap::new(400, 400).expect("400x400 pixmap");
    rasterize_shape(
        &mut pixmap,
        ShapeKind::Rectangle,
        pos2(100.0, 100.0),
        pos2(200.0, 200.0),
        200.0,
        Some(Color32::BLACK),
        None,
        ShapeOutline::Center,
    )
    .expect("shape rasterizes");
    // 70 px left of the boundary at x = 100: the centered band spans
    // half-width 100 around it (the edge at x = 0); the old clamp's
    // half-width 50 stopped at x = 50.
    assert_eq!(alpha(&pixmap, 30, 150), 255);
}

#[test]
fn the_sanity_ceiling_stays_the_entry_box_bound() {
    // MAX_STROKE_SIZE itself rasterizes without panicking: the dirty rect
    // saturates at the pixmap bounds (the same arithmetic the UI entry
    // box's 1..=MAX_STROKE_SIZE range feeds).
    let mut pixmap = Pixmap::new(50, 50).expect("50x50 pixmap");
    let dirty = draw_segment(
        &mut pixmap,
        Tool::Brush,
        MAX_STROKE_SIZE,
        Color32::BLACK,
        pos2(0.0, 0.0),
        pos2(49.0, 49.0),
    )
    .expect("ceiling width rasterizes");
    assert_eq!((dirty.x, dirty.y, dirty.w, dirty.h), (0, 0, 50, 50));
}
