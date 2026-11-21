//! The selection-CLIP commit tests (owner request 2026-11-21): an active
//! selection confines every canvas-changing tool to its region. The pencil
//! filters its plots against the mask, the thick strokes rasterize onto a
//! transparent bbox scratch and transfer through the mask, and — with a
//! mask covering everything — the clipped result must be pixel-identical
//! to the direct paint (the transfer's rounding and the integer
//! translation reproduce it exactly). The transfer primitives themselves
//! (`stamp_masked` / `erase_masked_by`) are pinned in `selection/tests.rs`.

use super::*;
use crate::selection::{Mask, Region, Selection};
use eframe::egui::pos2;
use tiny_skia::PremultipliedColorU8;

/// A mask selecting the whole canvas.
fn full_mask(w: u32, h: u32) -> Mask {
    rect_mask(w, h, (0.0, 0.0), (w as f32, h as f32))
}

/// A mask selecting the rect between two corners.
fn rect_mask(w: u32, h: u32, a: (f32, f32), b: (f32, f32)) -> Mask {
    let mut sel = Selection::default();
    sel.replace(
        Some(Region::Rect {
            a: pos2(a.0, a.1),
            b: pos2(b.0, b.1),
        }),
        w,
        h,
    );
    sel.build_mask(w, h).expect("the rect is never empty")
}

/// An opaque slate canvas (the "untouched" probe reads this back).
fn filled(w: u32, h: u32, color: Color32) -> Pixmap {
    let mut pm = Pixmap::new(w, h).expect("the pixmap builds");
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    // to_srgba gives straight alpha; the pixmap stores premultiplied.
    let px = PremultipliedColorU8::from_rgba(
        (r as u16 * a as u16 / 255) as u8,
        (g as u16 * a as u16 / 255) as u8,
        (b as u16 * a as u16 / 255) as u8,
        a,
    )
    .expect("opaque color");
    for p in pm.pixels_mut() {
        *p = px;
    }
    pm
}

/// The maximum per-channel difference between two pixmaps (the eraser's
/// clipped AA edge may differ from the direct destination-out by one
/// rounding step; everything else must match exactly).
fn max_channel_diff(a: &Pixmap, b: &Pixmap) -> i32 {
    assert_eq!(a.width(), b.width());
    assert_eq!(a.height(), b.height());
    a.pixels()
        .iter()
        .zip(b.pixels().iter())
        .map(|(x, y)| {
            (x.red() as i32 - y.red() as i32)
                .abs()
                .max((x.green() as i32 - y.green() as i32).abs())
                .max((x.blue() as i32 - y.blue() as i32).abs())
                .max((x.alpha() as i32 - y.alpha() as i32).abs())
        })
        .max()
        .unwrap_or(0)
}

/// With a mask covering EVERYTHING, the clipped commit must reproduce the
/// direct paint: same geometry through an integer translation onto a
/// scratch, transferred src-over. The pencil is pixel-IDENTICAL (the same
/// `blend_pixel` plots, just mask-filtered); the AA strokes (brush,
/// eraser) may differ by ONE rounding step at their anti-aliased edges —
/// the direct path blends once through tiny-skia, the clipped path blends
/// the scratch's coverage through `blend_pixel` — while their interiors
/// and everything outside the stroke match exactly.
#[test]
fn clipped_paint_with_a_full_mask_matches_the_direct_paint() {
    let color = Color32::from_rgb(180, 40, 40);
    let pieces = [
        SmoothPiece::Line {
            from: pos2(4.0, 6.0),
            to: pos2(24.0, 10.0),
        },
        SmoothPiece::Quad {
            from: pos2(24.0, 10.0),
            control: pos2(30.0, 30.0),
            to: pos2(8.0, 28.0),
        },
    ];
    for (tool, size) in [(Tool::Pencil, 1.0), (Tool::Brush, 3.0), (Tool::Eraser, 3.0)] {
        // The eraser needs content to erase; the paint tools draw over the
        // same slate so both bitmaps share the base.
        let base = filled(40, 40, Color32::from_rgb(20, 90, 20));
        let mut direct = base.clone();
        let mut clipped = base;
        let mask = full_mask(40, 40);
        let mut trail_a = PencilTrail::default();
        let mut trail_b = PencilTrail::default();
        rasterize_smooth_pieces(
            &mut direct,
            tool,
            size,
            color,
            &pieces,
            &mut trail_a,
            None,
            None,
        );
        rasterize_smooth_pieces(
            &mut clipped,
            tool,
            size,
            color,
            &pieces,
            &mut trail_b,
            None,
            Some(&mask),
        );
        let diff = max_channel_diff(&direct, &clipped);
        if matches!(tool, Tool::Pencil) {
            // The pencil plots through the SAME `blend_pixel` both ways —
            // byte-identical.
            assert_eq!(direct.data(), clipped.data(), "{tool:?}: identical plots");
        } else {
            assert!(
                diff <= 1,
                "{tool:?}: the clipped paint must match the direct one within one rounding step (got {diff})"
            );
        }
    }
}

/// A brush stroke crossing the selection's right edge paints ONLY inside:
/// outside pixels keep the slate byte-for-byte (the AA ring included — the
/// mask cuts before the stroke's coverage reaches them).
#[test]
fn a_clipped_stroke_leaves_outside_pixels_untouched() {
    let slate = Color32::from_rgb(20, 90, 20);
    let mut pm = filled(40, 40, slate);
    let mask = rect_mask(40, 40, (0.0, 0.0), (20.0, 40.0));
    let color = Color32::from_rgb(180, 40, 40);
    let dirty = draw_segment(
        &mut pm,
        Tool::Brush,
        4.0,
        color,
        pos2(6.0, 20.0),
        pos2(34.0, 20.0),
        Some(&mask),
    );
    assert!(dirty.is_some(), "the inside half must paint");
    let green = pm.pixel(2, 2).expect("in bounds");
    for x in [20, 26, 33] {
        assert_eq!(
            pm.pixel(x, 20).expect("in bounds"),
            green,
            "pixel {x} is outside the selection and must stay slate"
        );
    }
    let ink = pm.pixel(10, 20).expect("in bounds");
    assert!(
        ink.alpha() == 255 && ink.red() > 100,
        "inside must be inked"
    );
}

/// The pencil plots only masked pixels: the run crossing the mask's right
/// edge stops there exactly (no AA — the pencil is a 1-px Bresenham).
#[test]
fn a_clipped_pencil_plots_only_masked_pixels() {
    let slate = Color32::from_rgb(20, 90, 20);
    let mut pm = filled(40, 40, slate);
    let mask = rect_mask(40, 40, (0.0, 0.0), (20.0, 40.0));
    let color = Color32::BLACK;
    draw_segment(
        &mut pm,
        Tool::Pencil,
        1.0,
        color,
        pos2(5.0, 10.0),
        pos2(35.0, 10.0),
        Some(&mask),
    );
    let black = PremultipliedColorU8::from_rgba(0, 0, 0, 255).expect("opaque");
    for x in 5..20 {
        assert_eq!(pm.pixel(x, 10), Some(black), "inside must be plotted");
    }
    for x in 20..35 {
        assert_ne!(
            pm.pixel(x, 10),
            Some(black),
            "outside must not be plotted at {x}"
        );
    }
}

/// The clipped eraser clears ONLY masked pixels: the black slate stays
/// opaque beyond the selection's right edge and fades to transparent
/// inside (the stroke's solid core).
#[test]
fn a_clipped_eraser_erases_only_masked_pixels() {
    let black = Color32::BLACK;
    let mut pm = filled(40, 40, black);
    let mask = rect_mask(40, 40, (0.0, 0.0), (20.0, 40.0));
    let dirty = draw_segment(
        &mut pm,
        Tool::Eraser,
        4.0,
        black, // the eraser ignores the color; the param is the API's shape
        pos2(6.0, 20.0),
        pos2(34.0, 20.0),
        Some(&mask),
    );
    assert!(dirty.is_some());
    for x in [21, 26, 33] {
        assert_eq!(
            pm.pixel(x, 20).expect("in bounds").alpha(),
            255,
            "outside must stay opaque at {x}"
        );
    }
    assert_eq!(
        pm.pixel(10, 20).expect("in bounds").alpha(),
        0,
        "inside the stroke's core must be erased fully"
    );
}

/// A shape commit with a half-canvas mask: the outline's right half is
/// absent, the left half present.
#[test]
fn a_clipped_shape_outline_stops_at_the_mask() {
    let slate = Color32::from_rgb(20, 90, 20);
    let mut pm = filled(40, 40, slate);
    let mask = rect_mask(40, 40, (0.0, 0.0), (20.0, 40.0));
    let color = Color32::BLACK;
    let dirty = rasterize_shape(
        &mut pm,
        ShapeKind::Rectangle,
        pos2(4.0, 4.0),
        pos2(36.0, 30.0),
        2.0,
        Some(color),
        None,
        ShapeOutline::Center,
        Some(&mask),
    );
    assert!(dirty.is_some(), "the inside half must paint");
    let green = pm.pixel(2, 2).expect("in bounds");
    // The outline runs along the boundary (Center): the top edge's right
    // part must stay slate, the left part must be inked.
    assert_eq!(pm.pixel(30, 4), Some(green), "outside must stay slate");
    assert_ne!(pm.pixel(10, 4), Some(green), "inside must be inked");
    assert_eq!(pm.pixel(36, 20), Some(green), "the right edge is outside");
    assert_ne!(pm.pixel(4, 20), Some(green), "the left edge is inside");
}
