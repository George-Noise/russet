use super::*;

use crate::selection::{Region, Selection};
use crate::tools::{PencilTrail, SmoothPiece, Tool, draw_segment, rasterize_smooth_pieces};
use eframe::egui::pos2;

fn white_layer(w: u32, h: u32) -> Pixmap {
    let mut pm = Pixmap::new(w, h).expect("pixmap allocates");
    pm.fill(tiny_skia::Color::from_rgba8(255, 255, 255, 255));
    pm
}

/// Hand-sets the scratch's coverage at one pixel (the composite reads only
/// the alpha — the stroke color rides `composite_region`).
fn set_coverage(wet: &mut StrokeWet, x: u32, y: u32, alpha: u8) {
    let stride = wet.scratch.width() as usize;
    let px = PremultipliedColorU8::from_rgba(0, 0, 0, alpha).expect("valid premultiplied");
    wet.scratch.pixels_mut()[y as usize * stride + x as usize] = px;
}

fn premult(pm: &Pixmap, x: u32, y: u32) -> [u8; 4] {
    let p = pm.pixel(x, y).expect("in bounds");
    [p.red(), p.green(), p.blue(), p.alpha()]
}

#[test]
fn wet_start_clones_the_layer_and_starts_an_empty_scratch() {
    let layer = white_layer(16, 8);

    let wet = StrokeWet::start(&layer).expect("buffers allocate");

    assert_eq!((wet.scratch.width(), wet.scratch.height()), (16, 8));
    assert!(
        wet.scratch.pixels().iter().all(|p| p.alpha() == 0),
        "the scratch starts empty"
    );
}

#[test]
fn wet_paint_blends_the_stroke_alpha_once_over_the_base() {
    let mut layer = white_layer(16, 8);
    let mut wet = StrokeWet::start(&layer).expect("buffers allocate");
    set_coverage(&mut wet, 3, 3, 128);
    let region = DirtyRect {
        x: 0,
        y: 0,
        w: 8,
        h: 8,
    };
    let color = Color32::from_rgba_unmultiplied(10, 20, 30, 128);

    wet.composite_region(&mut layer, region, color, false, None)
        .expect("region non-empty");

    // α 128 · coverage 128 → effective 64; premultiplied src (3, 5, 8, 64)
    // over the white base (255, 255, 255, 255) — exactly one blend.
    assert_eq!(premult(&layer, 3, 3), [194, 196, 199, 255]);
    assert_eq!(premult(&layer, 10, 3), [255, 255, 255, 255], "untouched");

    // Recompositing the same region restarts from the base — idempotent.
    wet.composite_region(&mut layer, region, color, false, None)
        .expect("region non-empty");
    assert_eq!(premult(&layer, 3, 3), [194, 196, 199, 255]);
}

#[test]
fn wet_eraser_fades_the_base_by_the_coverage_once() {
    let mut layer = Pixmap::new(16, 8).expect("pixmap allocates");
    let base_px = PremultipliedColorU8::from_rgba(200, 100, 50, 200).expect("valid premultiplied");
    layer.pixels_mut().fill(base_px);
    let mut wet = StrokeWet::start(&layer).expect("buffers allocate");
    set_coverage(&mut wet, 5, 5, 128);
    let region = DirtyRect {
        x: 0,
        y: 0,
        w: 16,
        h: 8,
    };

    wet.composite_region(&mut layer, region, Color32::WHITE, true, None)
        .expect("region non-empty");

    // Destination-out by coverage 128: every channel scales by 1 − 128/255.
    assert_eq!(premult(&layer, 5, 5), [100, 50, 25, 100]);
    assert_eq!(premult(&layer, 10, 5), [200, 100, 50, 200], "untouched");

    // A second pass recomputes from the base — the erase does not stack.
    wet.composite_region(&mut layer, region, Color32::WHITE, true, None)
        .expect("region non-empty");
    assert_eq!(premult(&layer, 5, 5), [100, 50, 25, 100]);
}

#[test]
fn wet_composite_confines_to_the_selection_mask() {
    let mut layer = white_layer(32, 8);
    let mut wet = StrokeWet::start(&layer).expect("buffers allocate");
    set_coverage(&mut wet, 3, 3, 255);
    set_coverage(&mut wet, 15, 3, 255);
    let mut selection = Selection::default();
    selection.replace(
        Some(Region::Rect {
            a: pos2(0.0, 0.0),
            b: pos2(10.0, 8.0),
        }),
        32,
        8,
    );
    let mask = selection.build_mask(32, 8).expect("non-empty selection");
    let region = DirtyRect {
        x: 0,
        y: 0,
        w: 32,
        h: 8,
    };
    let color = Color32::from_rgba_unmultiplied(255, 0, 0, 255);

    let touched = wet
        .composite_region(&mut layer, region, color, false, Some(&mask))
        .expect("region non-empty");

    // The returned region is the dirty rect ∩ the mask's tight bounds.
    assert_eq!((touched.x, touched.y, touched.w, touched.h), (0, 0, 10, 8));
    assert_eq!(premult(&layer, 3, 3), [255, 0, 0, 255], "inside: painted");
    assert_eq!(
        premult(&layer, 15, 3),
        [255, 255, 255, 255],
        "outside: byte-identical base"
    );
}

/// The owner's bug report 2026-11-23, at the tools level: a translucent
/// brush gesture pressed at (50, 20) and dragged through the shared
/// midpoint (60, 20) in TWO frame batches — the joint cap disc, the press
/// dot and the segment body must all read identically, whatever the frame
/// batching (the direct per-frame path darkened the overlaps).
#[test]
fn translucent_wet_joints_match_the_stroke_body() {
    let mut layer = white_layer(100, 40);
    let mut wet = StrokeWet::start(&layer).expect("buffers allocate");
    let color = Color32::from_rgba_unmultiplied(10, 20, 30, 128);
    let full = Color32::from_rgba_unmultiplied(10, 20, 30, 255);
    let mut trail = PencilTrail::default();

    // The press dot (full-alpha coverage in the scratch)...
    let dirty = draw_segment(
        wet.scratch_mut(),
        Tool::Brush,
        8.0,
        full,
        pos2(50.0, 20.0),
        pos2(50.0, 20.0),
        None,
    );
    wet.composite_region(&mut layer, dirty.expect("dot dirties"), color, false, None)
        .expect("region non-empty");
    // ...frame one ending ON the joint midpoint...
    let dirty = rasterize_smooth_pieces(
        wet.scratch_mut(),
        Tool::Brush,
        8.0,
        full,
        &[SmoothPiece::Line {
            from: pos2(50.0, 20.0),
            to: pos2(60.0, 20.0),
        }],
        &mut trail,
        None,
        None,
    );
    wet.composite_region(&mut layer, dirty.expect("pieces dirty"), color, false, None)
        .expect("region non-empty");
    // ...and frame two starting from it.
    let dirty = rasterize_smooth_pieces(
        wet.scratch_mut(),
        Tool::Brush,
        8.0,
        full,
        &[SmoothPiece::Line {
            from: pos2(60.0, 20.0),
            to: pos2(70.0, 20.0),
        }],
        &mut trail,
        None,
        None,
    );
    wet.composite_region(&mut layer, dirty.expect("pieces dirty"), color, false, None)
        .expect("region non-empty");

    let body = premult(&layer, 55, 20);
    assert_eq!(
        premult(&layer, 60, 20),
        body,
        "the joint cap matches the body"
    );
    assert_eq!(
        premult(&layer, 50, 20),
        body,
        "the press dot matches the body"
    );
}
