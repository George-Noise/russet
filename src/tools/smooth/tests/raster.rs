//! Tests of [`rasterize_smooth_pieces`]: the pencil's 1 px drifting-pen
//! trail along the flattened centerline (the even staircase, gap-freedom,
//! the translucent-safe joint dedupe), the brush/eraser single-path stroke
//! and the union dirty rectangle.

use super::super::{PencilTrail, SmoothPiece, Tool, pencil_pixel, rasterize_smooth_pieces};
use eframe::egui::{Color32, pos2};
use tiny_skia::Pixmap;

fn plotted_black(pixmap: &Pixmap) -> Vec<(usize, usize)> {
    pixmap
        .pixels()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.alpha() == 255 && c.red() == 0)
        .map(|(i, _)| (i % pixmap.width() as usize, i / pixmap.width() as usize))
        .collect()
}

#[test]
fn pencil_line_piece_plots_one_pixel_per_column_between_its_endpoints() {
    let mut pixmap = Pixmap::new(32, 32).expect("32x32 pixmap");
    let piece = SmoothPiece::Line {
        from: pos2(2.4, 3.6),
        to: pos2(20.7, 11.3),
    };
    let mut trail = PencilTrail::default();
    rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Pencil,
        1.0,
        Color32::BLACK,
        &[piece],
        &mut trail,
        Some(pos2(20.7, 11.3)),
        None,
    )
    .expect("piece rasterizes");
    let mut got = plotted_black(&pixmap);
    got.sort();
    // The same pixel COUNT as the classic Bresenham run between the rounded
    // endpoints (one per dominant-axis step: |dx| = 19 from column 2 to 21),
    // the same endpoints — the drifting pen's staircase hugs the same chord
    // (individual pixels may sit one row away from Bresenham's tie-breaking,
    // both are valid staircases of the same line).
    let expected_count = ((21i64 - 2).abs().max(11i64 - 4) + 1) as usize;
    assert_eq!(got.len(), expected_count, "one pixel per dominant step");
    assert!(
        got.contains(&(2, 4)),
        "the trail starts at the rounded from"
    );
    assert!(
        got.contains(&(21, 11)),
        "the pinned trail ends at the rounded to"
    );
    let per_column = got
        .iter()
        .fold(std::collections::BTreeMap::new(), |mut map, (x, y)| {
            map.entry(*x).or_insert_with(Vec::new).push(*y);
            map
        });
    assert_eq!(per_column.len(), expected_count, "every column is covered");
    for (x, ys) in &per_column {
        assert_eq!(ys.len(), 1, "column {x} carries exactly one pixel");
    }
}

#[test]
fn pencil_quad_centerline_is_a_gap_free_pixel_chain() {
    let mut pixmap = Pixmap::new(64, 64).expect("64x64 pixmap");
    let piece = SmoothPiece::Quad {
        from: pos2(3.0, 40.0),
        control: pos2(30.0, 5.0),
        to: pos2(55.0, 45.0),
    };
    let mut trail = PencilTrail::default();
    rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Pencil,
        1.0,
        Color32::BLACK,
        &[piece],
        &mut trail,
        Some(pos2(55.0, 45.0)),
        None,
    )
    .expect("piece rasterizes");
    let black = plotted_black(&pixmap);
    // The chain starts at the rounded `from` and reaches the rounded `to`.
    assert!(black.contains(&(3, 40)));
    assert!(black.contains(&(55, 45)));
    // 8-connected flood fill from the first pixel covers the whole set:
    // no gaps anywhere along the flattened curve.
    let set: std::collections::HashSet<(usize, usize)> = black.iter().copied().collect();
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![(3, 40)];
    while let Some(p @ (x, y)) = queue.pop() {
        if !seen.insert(p) {
            continue;
        }
        for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                let n = (x as i64 + dx, y as i64 + dy);
                if n.0 >= 0
                    && n.1 >= 0
                    && (n.0 as usize, n.1 as usize) != p
                    && set.contains(&(n.0 as usize, n.1 as usize))
                {
                    queue.push((n.0 as usize, n.1 as usize));
                }
            }
        }
    }
    assert_eq!(seen.len(), black.len(), "the pixel chain must be gap-free");
}

#[test]
fn pencil_joint_pixels_blend_once_for_translucent_colors() {
    let mut pixmap = Pixmap::new(32, 32).expect("32x32 pixmap");
    let color = Color32::from_rgba_unmultiplied(0, 0, 255, 128);
    let pieces = [
        SmoothPiece::Line {
            from: pos2(4.0, 4.0),
            to: pos2(16.0, 4.0),
        },
        SmoothPiece::Line {
            from: pos2(16.0, 4.0),
            to: pos2(16.0, 16.0),
        },
    ];
    let mut trail = PencilTrail::default();
    rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Pencil,
        1.0,
        color,
        &pieces,
        &mut trail,
        None,
        None,
    )
    .expect("pieces rasterize");
    let alpha_at = |x: u32, y: u32| pixmap.pixel(x, y).expect("in bounds").alpha();
    assert_eq!(
        alpha_at(4, 4),
        alpha_at(16, 4),
        "the joint pixel blends once, not twice"
    );
    assert_eq!(alpha_at(16, 4), 128, "one 50% blend over transparency");
}

#[test]
fn brush_pieces_stroke_one_continuous_path() {
    let mut pixmap = Pixmap::new(64, 64).expect("64x64 pixmap");
    let pieces = [
        SmoothPiece::Line {
            from: pos2(6.0, 32.0),
            to: pos2(12.0, 32.0),
        },
        SmoothPiece::Quad {
            from: pos2(12.0, 32.0),
            control: pos2(24.0, 32.0),
            to: pos2(32.0, 32.0),
        },
    ];
    let mut trail = PencilTrail::default();
    let dirty = rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Brush,
        10.0,
        Color32::BLACK,
        &pieces,
        &mut trail,
        None,
        None,
    )
    .expect("pieces rasterize");
    assert!(
        dirty.x <= 6 && dirty.y <= 27,
        "the union rect covers the path start"
    );
    assert!(
        dirty.x + dirty.w >= 32,
        "the union rect covers the path end"
    );
    assert!(
        pixmap.pixel(20, 32).expect("in bounds").alpha() > 200,
        "the quad's middle is painted"
    );
    assert_eq!(
        pixmap.pixel(20, 25).expect("in bounds").alpha(),
        0,
        "outside the 10 px stroke stays transparent"
    );
}

#[test]
fn eraser_pieces_erase_to_transparency() {
    let mut pixmap = Pixmap::new(32, 32).expect("32x32 pixmap");
    pixmap.fill(tiny_skia::Color::BLACK);
    let pieces = [SmoothPiece::Line {
        from: pos2(4.0, 16.0),
        to: pos2(12.0, 16.0),
    }];
    let mut trail = PencilTrail::default();
    rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Eraser,
        6.0,
        Color32::BLACK,
        &pieces,
        &mut trail,
        None,
        None,
    )
    .expect("pieces rasterize");
    assert_eq!(pixmap.pixel(8, 16).expect("in bounds").alpha(), 0);
    assert_eq!(pixmap.pixel(8, 1).expect("in bounds").alpha(), 255);
}

#[test]
fn dirty_rect_covers_the_quad_control_point() {
    let mut pixmap = Pixmap::new(64, 64).expect("64x64 pixmap");
    let piece = SmoothPiece::Quad {
        from: pos2(10.0, 50.0),
        control: pos2(40.0, 10.0),
        to: pos2(50.0, 50.0),
    };
    let mut trail = PencilTrail::default();
    let dirty = rasterize_smooth_pieces(
        &mut pixmap,
        Tool::Brush,
        2.0,
        Color32::BLACK,
        &[piece],
        &mut trail,
        None,
        None,
    )
    .expect("piece rasterizes");
    assert!(
        dirty.y <= 10,
        "dirty.y={} must cover the control row",
        dirty.y
    );
    assert!(dirty.y + dirty.h >= 50);
    assert!(dirty.x <= 10 && dirty.x + dirty.w >= 50);
}

#[test]
fn no_pieces_rasterize_nothing() {
    let mut pixmap = Pixmap::new(8, 8).expect("8x8 pixmap");
    let mut trail = PencilTrail::default();
    assert!(
        rasterize_smooth_pieces(
            &mut pixmap,
            Tool::Brush,
            4.0,
            Color32::BLACK,
            &[],
            &mut trail,
            None,
            None
        )
        .is_none()
    );
    assert!(pixmap.pixels().iter().all(|c| c.alpha() == 0));
}

#[test]
fn pencil_pixel_rounds_like_the_draw_segment_endpoints() {
    assert_eq!(pencil_pixel(pos2(2.4, 3.6)), (2, 4));
    assert_eq!(pencil_pixel(pos2(-0.4, 10.5)), (0, 11));
}
