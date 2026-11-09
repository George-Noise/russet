use super::*;
use eframe::egui::pos2;

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
    assert!(draw_segment(&mut pixmap, Tool::Pencil, 1.0, Color32::BLACK, from, to).is_some());
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
    );
    assert!(dirty.is_some());
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 0).count();
    assert!(painted >= 15, "the horizontal line is drawn: {painted}");
}

#[test]
fn polyline_single_point_draws_a_dot() {
    let mut pixmap = Pixmap::new(20, 20).expect("20x20 pixmap");
    assert!(
        rasterize_polyline(&mut pixmap, &[pos2(10.0, 10.0)], 1.0, Color32::BLACK).is_some()
    );
    // A width-1 dot straddles the pixel grid: a 2×2 block of partial
    // coverage (the brush's round-cap look).
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 0).count();
    assert!((1..=4).contains(&painted), "a dot, not a line: {painted}");
}

#[test]
fn empty_polyline_draws_nothing() {
    let mut pixmap = Pixmap::new(20, 20).expect("20x20 pixmap");
    assert!(rasterize_polyline(&mut pixmap, &[], 1.0, Color32::BLACK).is_none());
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
    )
    .expect("eraser stroke rasterizes");

    // The erased line is fully transparent; pixels off the stroke stay opaque.
    assert_eq!(pixmap.pixel(8, 8).expect("pixel in bounds").alpha(), 0);
    assert_eq!(pixmap.pixel(8, 1).expect("pixel in bounds").alpha(), 255);
}
