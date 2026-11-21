use super::*;
use eframe::egui::pos2;

mod clip;
mod stroke_size;

fn line_points(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<(i64, i64)> {
    let mut pts = Vec::new();
    bresenham_line(x0, y0, x1, y1, |x, y| pts.push((x, y)));
    pts
}

#[test]
fn diagonal_is_a_one_pixel_staircase() {
    assert_eq!(
        line_points(0, 0, 3, 3),
        vec![(0, 0), (1, 1), (2, 2), (3, 3)]
    );
}

#[test]
fn shallow_line_has_exactly_one_pixel_per_column() {
    let pts = line_points(0, 0, 7, 3);
    assert_eq!(pts.len(), 8);
    for col in 0..=7 {
        assert_eq!(pts.iter().filter(|&&(x, _)| x == col).count(), 1);
    }
}

#[test]
fn steep_line_has_exactly_one_pixel_per_row() {
    let pts = line_points(0, 0, 3, 7);
    assert_eq!(pts.len(), 8);
    for row in 0..=7 {
        assert_eq!(pts.iter().filter(|&&(_, y)| y == row).count(), 1);
    }
}

#[test]
fn single_point_plots_one_pixel() {
    assert_eq!(line_points(5, 5, 5, 5), vec![(5, 5)]);
}

#[test]
fn pencil_draws_hard_one_pixel_line_on_pixmap() {
    let mut pixmap = Pixmap::new(16, 16).expect("16x16 pixmap");
    let from = pos2(0.4, 0.6); // rounds to (0, 1)
    let to = pos2(7.6, 3.4); // rounds to (8, 3)
    assert!(
        draw_segment(
            &mut pixmap,
            Tool::Pencil,
            1.0,
            Color32::BLACK,
            from,
            to,
            None
        )
        .is_some()
    );
    let black: Vec<(usize, usize)> = pixmap
        .pixels()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.alpha() == 255 && c.red() == 0)
        .map(|(i, _)| (i % 16, i / 16))
        .collect();
    assert_eq!(black.len(), 9); // shallow line: one pixel per column
    for col in 0..16 {
        assert!(black.iter().filter(|&&(x, _)| x == col).count() <= 1);
    }
}

#[test]
fn polyline_draws_a_line() {
    let mut pixmap = Pixmap::new(20, 20).expect("20x20 pixmap");
    let dirty = rasterize_polyline(
        &mut pixmap,
        &[pos2(2.0, 10.2), pos2(17.0, 10.0)],
        2.0,
        Color32::BLACK,
        None,
    );
    assert!(dirty.is_some());
    // Two mostly-solid pixel rows per column from 2..=17 (a width-2
    // stroke); the slightly diagonal path lowers some edge alphas.
    let painted: Vec<usize> = pixmap
        .pixels()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.alpha() > 100)
        .map(|(i, _)| i)
        .collect();
    assert!(
        (24..=36).contains(&painted.len()),
        "painted={}",
        painted.len()
    );
    for &i in &painted {
        // Pixel index = row * 20 + column; a width-2 stroke around
        // y≈10 covers rows 9..=11.
        assert!((8..=12).contains(&(i / 20)));
    }
}

#[test]
fn polyline_handles_perfectly_horizontal_line() {
    let mut pixmap = Pixmap::new(20, 20).expect("20x20 pixmap");
    // Zero-height path bounds are rejected by tiny-skia; the rasterizer
    // must still draw the line (via a subpixel nudge).
    let dirty = rasterize_polyline(
        &mut pixmap,
        &[pos2(2.0, 10.0), pos2(17.0, 10.0)],
        1.0,
        Color32::BLACK,
        None,
    );
    assert!(dirty.is_some());
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 0).count();
    assert!(painted >= 15, "the horizontal line is drawn: {painted}");
}

#[test]
fn polyline_single_point_draws_a_dot() {
    let mut pixmap = Pixmap::new(20, 20).expect("20x20 pixmap");
    assert!(
        rasterize_polyline(&mut pixmap, &[pos2(10.0, 10.0)], 1.0, Color32::BLACK, None).is_some()
    );
    // A width-1 dot straddles the pixel grid: a 2×2 block of partial
    // coverage (the brush's round-cap look).
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 0).count();
    assert!((1..=4).contains(&painted), "a dot, not a line: {painted}");
}

#[test]
fn empty_polyline_draws_nothing() {
    let mut pixmap = Pixmap::new(20, 20).expect("20x20 pixmap");
    assert!(rasterize_polyline(&mut pixmap, &[], 1.0, Color32::BLACK, None).is_none());
    assert!(pixmap.pixels().iter().all(|c| c.alpha() == 0));
}

#[test]
fn rectangle_outline_keeps_center_transparent() {
    let mut pixmap = Pixmap::new(40, 40).expect("40x40 pixmap");
    let dirty = rasterize_shape(
        &mut pixmap,
        ShapeKind::Rectangle,
        pos2(5.0, 5.0),
        pos2(34.0, 34.0),
        2.0,
        Some(Color32::BLACK),
        None,
        ShapeOutline::Center,
        None,
    );
    assert!(dirty.is_some());
    let at = |x: usize, y: usize| pixmap.pixels()[y * 40 + x];
    assert!(at(5, 5).alpha() > 200, "corner is stroked");
    assert!(at(20, 5).alpha() > 200, "top edge is stroked");
    assert!(at(5, 20).alpha() > 200, "left edge is stroked");
    assert_eq!(at(20, 20).alpha(), 0, "center stays transparent");
}

#[test]
fn filled_rectangle_paints_the_center() {
    let mut pixmap = Pixmap::new(40, 40).expect("40x40 pixmap");
    let dirty = rasterize_shape(
        &mut pixmap,
        ShapeKind::Rectangle,
        pos2(5.0, 5.0),
        pos2(34.0, 34.0),
        2.0,
        None,
        Some(Color32::BLACK),
        ShapeOutline::Center,
        None,
    );
    assert!(dirty.is_some());
    let at = |x: usize, y: usize| pixmap.pixels()[y * 40 + x];
    assert!(at(20, 20).alpha() > 200, "center is filled");
}

#[test]
fn fill_and_outline_paint_both() {
    let mut pixmap = Pixmap::new(40, 40).expect("40x40 pixmap");
    let dirty = rasterize_shape(
        &mut pixmap,
        ShapeKind::Ellipse,
        pos2(5.0, 5.0),
        pos2(34.0, 34.0),
        2.0,
        Some(Color32::BLACK),
        Some(Color32::WHITE),
        ShapeOutline::Center,
        None,
    );
    assert!(dirty.is_some());
    let at = |x: usize, y: usize| pixmap.pixels()[y * 40 + x];
    assert!(at(20, 20).alpha() > 200, "center is filled");
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 0).count();
    assert!(painted > 20, "the ring is drawn on top of the fill");
    assert_eq!(at(6, 6).alpha(), 0, "corner stays transparent");
}

#[test]
fn shape_without_outline_or_fill_draws_nothing() {
    let mut pixmap = Pixmap::new(40, 40).expect("40x40 pixmap");
    assert!(
        rasterize_shape(
            &mut pixmap,
            ShapeKind::Rectangle,
            pos2(5.0, 5.0),
            pos2(34.0, 34.0),
            2.0,
            None,
            None,
            ShapeOutline::Center,
            None
        )
        .is_none()
    );
    assert!(pixmap.pixels().iter().all(|c| c.alpha() == 0));
}

#[test]
fn ellipse_outline_keeps_center_and_corners_transparent() {
    let mut pixmap = Pixmap::new(40, 40).expect("40x40 pixmap");
    let dirty = rasterize_shape(
        &mut pixmap,
        ShapeKind::Ellipse,
        pos2(5.0, 5.0),
        pos2(34.0, 34.0),
        2.0,
        Some(Color32::BLACK),
        None,
        ShapeOutline::Center,
        None,
    );
    assert!(dirty.is_some());
    let at = |x: usize, y: usize| pixmap.pixels()[y * 40 + x];
    assert_eq!(at(20, 20).alpha(), 0, "center stays transparent");
    assert_eq!(at(6, 6).alpha(), 0, "corner stays transparent");
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 0).count();
    assert!(painted > 20, "the ring is drawn somewhere: {painted}");
}

#[test]
fn zero_size_shape_draws_nothing() {
    let mut pixmap = Pixmap::new(40, 40).expect("40x40 pixmap");
    assert!(
        rasterize_shape(
            &mut pixmap,
            ShapeKind::Rectangle,
            pos2(10.0, 10.0),
            pos2(10.0, 10.0),
            2.0,
            Some(Color32::BLACK),
            None,
            ShapeOutline::Center,
            None
        )
        .is_none()
    );
    assert!(pixmap.pixels().iter().all(|c| c.alpha() == 0));
}

#[test]
fn pick_color_demultiplies_the_pixel() {
    let mut pixmap = Pixmap::new(4, 4).expect("4x4 pixmap");
    // Half-transparent red: premultiplied (128, 0, 0, 128).
    let src = PremultipliedColorU8::from_rgba(128, 0, 0, 128).expect("valid premultiplied");
    pixmap.pixels_mut()[5] = src;
    assert_eq!(
        pick_color(&pixmap, 1, 1),
        Some(Color32::from_rgba_unmultiplied(255, 0, 0, 128))
    );
    assert_eq!(pick_color(&pixmap, 4, 4), None, "out of bounds");
}

#[test]
fn brush_size_controls_stroke_thickness() {
    let mut pixmap = Pixmap::new(32, 32).expect("32x32 pixmap");
    draw_segment(
        &mut pixmap,
        Tool::Brush,
        10.0,
        Color32::BLACK,
        pos2(6.0, 16.0),
        pos2(26.0, 16.0),
        None,
    )
    .expect("brush stroke rasterizes");

    // The row through the stroke centre is painted; rows farther than the
    // 10 px stroke width from it are not.
    assert!(pixmap.pixel(16, 16).expect("pixel in bounds").alpha() > 0);
    assert_eq!(pixmap.pixel(16, 9).expect("pixel in bounds").alpha(), 0);
    let painted_rows = (0..32)
        .filter(|&y| pixmap.pixel(16, y).expect("pixel in bounds").alpha() > 0)
        .count();
    assert!(
        (9..=12).contains(&painted_rows),
        "10 px brush painted {painted_rows} rows"
    );
}

#[test]
fn eraser_clears_pixels_to_transparency() {
    let mut pixmap = Pixmap::new(16, 16).expect("16x16 pixmap");
    pixmap.fill(tiny_skia::Color::BLACK);
    draw_segment(
        &mut pixmap,
        Tool::Eraser,
        4.0,
        Color32::BLACK,
        pos2(3.0, 8.0),
        pos2(12.0, 8.0),
        None,
    )
    .expect("eraser stroke rasterizes");

    // The erased line is fully transparent; pixels off the stroke stay opaque.
    assert_eq!(pixmap.pixel(8, 8).expect("pixel in bounds").alpha(), 0);
    assert_eq!(pixmap.pixel(8, 1).expect("pixel in bounds").alpha(), 255);
}

/// The outline placement math ([`outline_stroke_bounds`], owner request
/// 2026-11-20): the stroke straddles its path, so Outside shifts the path
/// half a width outward, Inside half a width inward — clamped so a shape
/// thinner than the stroke keeps a valid non-degenerate path — and Center
/// keeps the boundary itself.
#[test]
fn the_outline_placement_shifts_the_stroked_path_by_half_a_width() {
    assert_eq!(
        outline_stroke_bounds(10.0, 20.0, 30.0, 40.0, 4.0, ShapeOutline::Center),
        (10.0, 20.0, 30.0, 40.0),
        "Center strokes the boundary itself (the old look)"
    );
    assert_eq!(
        outline_stroke_bounds(10.0, 20.0, 30.0, 40.0, 4.0, ShapeOutline::Outside),
        (8.0, 18.0, 32.0, 42.0),
        "Outside shifts the path half a width outward: the stroke's inner edge lands on the boundary"
    );
    assert_eq!(
        outline_stroke_bounds(10.0, 20.0, 30.0, 40.0, 4.0, ShapeOutline::Inside),
        (12.0, 22.0, 28.0, 38.0),
        "Inside shifts the path half a width inward: the stroke's outer edge lands on the boundary"
    );
}

#[test]
fn the_inside_placement_clamps_thin_shapes_to_a_valid_path() {
    // A 2-px-wide shape with a 6-px stroke: the raw half-width inset (3)
    // would invert the bounds; the clamp keeps the 0.1-px non-degenerate
    // path, so the stroke covers the whole tiny shape instead of vanishing.
    let (x0, y0, x1, y1) = outline_stroke_bounds(10.0, 10.0, 12.0, 40.0, 6.0, ShapeOutline::Inside);
    assert_eq!(
        (x0, x1),
        (10.95, 11.05),
        "x inset clamped to the 0.1-px span"
    );
    assert_eq!(
        (y0, y1),
        (13.0, 37.0),
        "the 30-px span insets the full half width"
    );
}

/// The Outside placement (owner request 2026-11-20): the outline lies
/// fully BEYOND the two mouse points — its inner edge is the boundary, so
/// the band paints outside the shape and the interior stays untouched.
#[test]
fn the_outside_outline_lies_fully_beyond_the_two_points() {
    let mut pixmap = Pixmap::new(48, 48).expect("48x48 pixmap");
    let dirty = rasterize_shape(
        &mut pixmap,
        ShapeKind::Rectangle,
        pos2(12.0, 12.0),
        pos2(35.0, 35.0),
        4.0,
        Some(Color32::BLACK),
        None,
        ShapeOutline::Outside,
        None,
    );
    assert!(dirty.is_some());
    let dirty = dirty.expect("the outside shape rasterizes");
    let at = |x: usize, y: usize| pixmap.pixels()[y * 48 + x];
    // The band spans [8, 12] around the boundary at 12: pixels just
    // outside are painted, pixels just inside are not (no fill here).
    assert!(
        at(9, 20).alpha() > 200,
        "the band outside the boundary is painted"
    );
    assert!(
        at(20, 9).alpha() > 200,
        "the top band outside the boundary is painted"
    );
    assert_eq!(
        at(14, 20).alpha(),
        0,
        "just inside the boundary stays transparent"
    );
    assert_eq!(at(20, 14).alpha(), 0, "the interior stays transparent");
    // The dirty rect must cover the full outside reach (the band reaches 8;
    // the shared pad rule adds the reach + 2 around the bounds).
    assert!(
        dirty.x <= 6 && dirty.y <= 6,
        "dirty {:?} covers the outside reach",
        dirty
    );
}

/// The Inside placement (owner request 2026-11-20): the outline lies fully
/// WITHIN the two mouse points — its outer edge is the boundary, so nothing
/// is painted outside the shape. With the fill both stay exactly the two
/// points.
#[test]
fn the_inside_outline_lies_fully_within_the_two_points() {
    let mut pixmap = Pixmap::new(48, 48).expect("48x48 pixmap");
    let dirty = rasterize_shape(
        &mut pixmap,
        ShapeKind::Rectangle,
        pos2(12.0, 12.0),
        pos2(35.0, 35.0),
        4.0,
        Some(Color32::BLACK),
        Some(Color32::WHITE),
        ShapeOutline::Inside,
        None,
    );
    assert!(dirty.is_some());
    let at = |x: usize, y: usize| pixmap.pixels()[y * 48 + x];
    assert!(
        at(13, 20).alpha() > 200,
        "the band inside the boundary is painted"
    );
    assert!(
        at(20, 13).alpha() > 200,
        "the top band inside the boundary is painted"
    );
    assert_eq!(at(9, 20).alpha(), 0, "nothing outside the boundary");
    assert_eq!(at(20, 9).alpha(), 0, "nothing above the boundary");
    assert!(at(20, 20).alpha() > 200, "the fill reaches the interior");
}

/// The Center placement keeps the classic look: the band straddles the
/// boundary, painting BOTH just outside and just inside.
#[test]
fn the_center_outline_straddles_the_two_points() {
    let mut pixmap = Pixmap::new(48, 48).expect("48x48 pixmap");
    rasterize_shape(
        &mut pixmap,
        ShapeKind::Rectangle,
        pos2(12.0, 12.0),
        pos2(35.0, 35.0),
        4.0,
        Some(Color32::BLACK),
        None,
        ShapeOutline::Center,
        None,
    )
    .expect("the shape rasterizes");
    let at = |x: usize, y: usize| pixmap.pixels()[y * 48 + x];
    assert!(at(11, 20).alpha() > 200, "half the band outside");
    assert!(at(13, 20).alpha() > 200, "half the band inside");
    assert_eq!(at(8, 20).alpha(), 0, "the outside reach stays half a width");
}

/// The paint radio's halves (owner request 2026-11-20): exactly one of the
/// three paints is ever active, and each names its wells — Fill fills only,
/// Outline outlines only, Fill + Outline does both (a shape always paints
/// something by construction — the invariant the old at-least-one toggle
/// rule enforced by hand).
#[test]
fn the_shape_paint_radio_names_its_wells() {
    assert_eq!(
        (ShapePaint::Fill.fills(), ShapePaint::Fill.outlines()),
        (true, false)
    );
    assert_eq!(
        (ShapePaint::Outline.fills(), ShapePaint::Outline.outlines()),
        (false, true)
    );
    assert_eq!(
        (
            ShapePaint::FillOutline.fills(),
            ShapePaint::FillOutline.outlines()
        ),
        (true, true),
        "the explicit third choice paints BOTH wells"
    );
    for paint in [
        ShapePaint::Fill,
        ShapePaint::Outline,
        ShapePaint::FillOutline,
    ] {
        assert!(
            paint.fills() || paint.outlines(),
            "{paint:?} must paint something"
        );
    }
}
