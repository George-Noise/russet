//! Unit tests of the Shape-tool gesture wells ([`super::shape_gesture_wells`]):
//! the right-button inverted-wells rule (owner request 2026-11-15: the
//! colors are used the other way around WITHOUT swapping the wells), the
//! Fill/Outline gating (owner request 2026-11-10; the two checkboxes became
//! the THREE-way paint radio 2026-11-20), the recorded-color mapping
//! ([`super::shape_record_colors`] — owner request 2026-11-16: a two-color
//! gesture carries BOTH colors on its history node), the
//! Shift-proportional drag rule ([`super::proportional_current`] — owner
//! request 2026-11-16, Paint.NET style), and the TESTING Shift diameter-circle
//! rule for the Ellipse kind ([`super::diameter_rect`] — owner request
//! 2026-11-21).

use super::*;
use crate::tools::{ShapeKind, ShapeOutline, ShapePaint, rasterize_shape};

/// The EXACT well-decoding the gesture start runs (preview.rs): the radio's
/// `outlines()`/`fills()` halves feed [`shape_gesture_wells`], so each of
/// the three paints lands the right wells — Fill = the secondary well only,
/// Outline = the primary only, Fill + Outline = both (owner request
/// 2026-11-20).
#[test]
fn the_paint_radio_feeds_the_gesture_the_right_wells() {
    for (paint, want_outline, want_fill) in [
        (ShapePaint::Fill, None, Some(Color32::BLUE)),
        (ShapePaint::Outline, Some(Color32::RED), None),
        (
            ShapePaint::FillOutline,
            Some(Color32::RED),
            Some(Color32::BLUE),
        ),
    ] {
        let (outline, fill) = shape_gesture_wells(
            PointerButton::Primary,
            paint.outlines(),
            paint.fills(),
            Color32::RED,
            Color32::BLUE,
        );

        assert_eq!(outline, want_outline, "{paint:?}: outline well");
        assert_eq!(fill, want_fill, "{paint:?}: fill well");
    }
}

#[test]
fn left_button_draws_outline_from_primary_and_fill_from_secondary() {
    let (outline, fill) = shape_gesture_wells(
        PointerButton::Primary,
        true,
        true,
        Color32::RED,
        Color32::BLUE,
    );

    assert_eq!(outline, Some(Color32::RED));
    assert_eq!(fill, Some(Color32::BLUE));
}

#[test]
fn right_button_uses_the_wells_inverted() {
    let (outline, fill) = shape_gesture_wells(
        PointerButton::Secondary,
        true,
        true,
        Color32::RED,
        Color32::BLUE,
    );

    assert_eq!(outline, Some(Color32::BLUE));
    assert_eq!(fill, Some(Color32::RED));
}

#[test]
fn the_fill_outline_checkboxes_gate_the_inverted_wells_too() {
    let (outline, fill) = shape_gesture_wells(
        PointerButton::Secondary,
        true,
        false,
        Color32::RED,
        Color32::BLUE,
    );

    assert_eq!(outline, Some(Color32::BLUE));
    assert_eq!(fill, None);
}

#[test]
fn a_shape_commit_that_used_both_wells_records_both_colors() {
    let (color, color2) =
        shape_record_colors(Some(Color32::RED), Some(Color32::BLUE), Color32::GREEN);

    assert_eq!(color, Color32::RED, "the outline is the node's main color");
    assert_eq!(
        color2,
        Some(Color32::BLUE),
        "the fill rides as the second color"
    );
}

#[test]
fn a_shape_commit_with_the_outline_only_records_one_color() {
    let (color, color2) = shape_record_colors(Some(Color32::RED), None, Color32::GREEN);

    assert_eq!(color, Color32::RED);
    assert_eq!(
        color2, None,
        "a single-color gesture carries no second color"
    );
}

#[test]
fn a_shape_commit_with_the_fill_only_records_the_fill_as_main() {
    let (color, color2) = shape_record_colors(None, Some(Color32::BLUE), Color32::GREEN);

    assert_eq!(color, Color32::BLUE);
    assert_eq!(color2, None);
}

#[test]
fn a_shape_commit_without_wells_falls_back_to_the_primary() {
    let (color, color2) = shape_record_colors(None, None, Color32::GREEN);

    assert_eq!(color, Color32::GREEN);
    assert_eq!(color2, None);
}

#[test]
fn shift_pulls_the_free_corner_onto_the_dominant_axis_square() {
    let pulled = proportional_current(pos2(10.0, 20.0), pos2(110.0, 70.0));

    assert_eq!(
        pulled,
        pos2(110.0, 120.0),
        "dx=100 dominates dy=50; both axes grow by 100 toward the pointer"
    );
}

#[test]
fn shift_grows_the_square_toward_the_pointer_in_every_direction() {
    let up_left = proportional_current(pos2(50.0, 50.0), pos2(30.0, 10.0));
    let down_left = proportional_current(pos2(50.0, 50.0), pos2(10.0, 70.0));

    assert_eq!(
        up_left,
        pos2(10.0, 10.0),
        "dy=-40 dominates dx=-20; the square grows up-left"
    );
    assert_eq!(
        down_left,
        pos2(10.0, 90.0),
        "dx=-40 dominates dy=+20; the square grows down-left"
    );
}

#[test]
fn an_equal_delta_drag_stays_exactly_on_the_pointer() {
    let pulled = proportional_current(pos2(0.0, 0.0), pos2(7.0, 7.0));

    assert_eq!(
        pulled,
        pos2(7.0, 7.0),
        "already proportional: nothing to pull"
    );
}

#[test]
fn a_strictly_one_sided_shift_drag_still_yields_a_square() {
    let pulled = proportional_current(pos2(5.0, 5.0), pos2(25.0, 5.0));

    assert_eq!(
        pulled,
        pos2(25.0, 25.0),
        "dy=0 still grows a 20x20 square (Rust's signum maps +0.0 to +1.0): Paint.NET never draws a degenerate shape"
    );
}

#[test]
fn shift_does_not_move_the_anchor_corner() {
    let start = pos2(3.0, 4.0);
    let pulled = proportional_current(start, pos2(103.0, 54.0));

    assert_eq!(pulled.x - start.x, 100.0);
    assert_eq!(
        pulled.y - start.y,
        100.0,
        "the side equals the dominant delta on both axes"
    );
}

// --- the TESTING Shift diameter-circle rule (owner request 2026-11-21) ---

#[test]
fn the_diameter_circle_centers_on_the_drag_segment_midpoint() {
    let rect = diameter_rect(pos2(40.0, 30.0), pos2(60.0, 50.0));

    assert_eq!(
        rect.center(),
        pos2(50.0, 40.0),
        "the circle's center is the segment between the two points' midpoint"
    );
    assert_eq!(
        rect.width(),
        rect.height(),
        "the bounding square of a circle: the side is the diameter"
    );
    assert_eq!(
        rect.width(),
        (pos2(60.0, 50.0) - pos2(40.0, 30.0)).length(),
        "the square's side equals the drag segment — its diameter"
    );
}

#[test]
fn both_drag_points_lie_on_the_diameter_circle_boundary() {
    let start = pos2(40.0, 30.0);
    let current = pos2(60.0, 50.0);
    let rect = diameter_rect(start, current);
    let radius = (current - start).length() * 0.5;

    let off_boundary = |p: Pos2| ((p - rect.center()).length() - radius).abs();
    assert!(
        off_boundary(start) < 0.001,
        "the press point is on the circle"
    );
    assert!(
        off_boundary(current) < 0.001,
        "the release point is on the circle too"
    );
}

#[test]
fn a_zero_length_diameter_drag_stays_degenerate() {
    let rect = diameter_rect(pos2(12.0, 34.0), pos2(12.0, 34.0));

    assert_eq!(rect.width(), 0.0);
    assert_eq!(
        rect.height(),
        0.0,
        "a click without movement still draws nothing, Shift or not"
    );
}

#[test]
fn the_committed_diameter_circle_passes_through_both_drag_points() {
    // The exact geometry the commit rasterizes (commit_drag): the bounding
    // square of `diameter_rect`, outline-only in the Center placement — the
    // band straddles the boundary, so both drag points (boundary points)
    // must be painted, while the segment's midpoint (the CENTER) stays
    // unpainted with the outline-only paint.
    let start = pos2(40.0, 30.0);
    let current = pos2(60.0, 50.0);
    let rect = diameter_rect(start, current);
    let mut pixmap = tiny_skia::Pixmap::new(100, 100).expect("100x100 pixmap");
    rasterize_shape(
        &mut pixmap,
        ShapeKind::Ellipse,
        rect.min,
        rect.max,
        2.0,
        Some(Color32::BLACK),
        None,
        ShapeOutline::Center,
        None,
    );

    assert!(
        pixmap
            .pixel(40, 30)
            .expect("the press point is in bounds")
            .alpha()
            > 0,
        "the press point lies on the committed circle"
    );
    assert!(
        pixmap
            .pixel(60, 50)
            .expect("the release point is in bounds")
            .alpha()
            > 0,
        "the release point lies on the committed circle"
    );
    assert_eq!(
        pixmap
            .pixel(50, 40)
            .expect("the midpoint is in bounds")
            .alpha(),
        0,
        "the drag segment's midpoint is the circle's CENTER — no paint there"
    );
}

#[test]
fn the_diameter_guides_draw_the_measured_chord_and_center() {
    let a = pos2(100.0, 200.0);
    let b = pos2(300.0, 240.0);
    let shapes = diameter_guides(a, b, Color32::RED);

    // The chord: dashed line segments, every dash ENDPOINT collinear with
    // and between a and b — the dashes lie ON the measured segment.
    let mut chord_dashes = 0usize;
    let mut dot_centers = Vec::new();
    for shape in &shapes {
        match shape {
            Shape::LineSegment { points, .. } => {
                chord_dashes += 1;
                let ab = b - a;
                for &p in points.iter() {
                    let t = (p - a).dot(ab) / ab.dot(ab);
                    assert!(
                        (0.0..=1.0).contains(&t),
                        "a dash endpoint strays past the chord: t={t}"
                    );
                    let projected = a + ab * t;
                    assert!(
                        (p - projected).length() < 0.01,
                        "a dash endpoint strays off the chord line"
                    );
                }
            }
            Shape::Circle(circle) => dot_centers.push(circle.center),
            _ => {}
        }
    }
    assert!(
        chord_dashes > 0,
        "a 200+ px chord must produce dashes, not an empty path"
    );
    assert_eq!(
        dot_centers.len(),
        2,
        "the center dot is the white core + the dark ring"
    );
    let mid = Pos2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    for center in &dot_centers {
        assert_eq!(
            *center, mid,
            "the dot sits at the segment's midpoint — the circle's center"
        );
    }
}

#[test]
fn the_diameter_guides_survive_a_zero_length_drag() {
    let shapes = diameter_guides(pos2(50.0, 60.0), pos2(50.0, 60.0), Color32::RED);

    let dashes = shapes
        .iter()
        .filter(|s| matches!(s, Shape::LineSegment { .. }))
        .count();
    assert_eq!(
        dashes, 0,
        "a zero-length chord has no room for dashes (the commit draws nothing anyway)"
    );
    let mid = Pos2::new(50.0, 60.0);
    for shape in &shapes {
        if let Shape::Circle(circle) = shape {
            assert_eq!(circle.center, mid, "the dot marks the click point");
        }
    }
}

// -----------------------------------------------------------------------
// The selection-clipped commits (owner request 2026-11-21): the drag
// gesture snapshots the selection mask at start and the commit transfers
// only masked pixels — outside them the canvas stays byte-identical.
// -----------------------------------------------------------------------

/// A half-canvas selection (the left 20 columns) on the default app.
fn left_half_clip(app: &App) -> crate::selection::Mask {
    let (w, h) = (app.canvas.width, app.canvas.height);
    let mut sel = crate::selection::Selection::default();
    sel.replace(
        Some(crate::selection::Region::Rect {
            a: pos2(0.0, 0.0),
            b: pos2(20.0, h as f32),
        }),
        w,
        h,
    );
    sel.build_mask(w, h).expect("non-empty selection")
}

#[test]
fn a_line_commit_confines_to_the_selection() {
    let mut app = App::new_headless();
    let mask = left_half_clip(&app);
    let slate = app
        .layers
        .active()
        .pixmap()
        .pixel(30, 20)
        .expect("in bounds");
    app.commit_drag(
        DragKind::Line,
        pos2(6.0, 20.0),
        pos2(34.0, 20.0),
        Some(Color32::BLACK),
        None,
        2.0,
        ShapeOutline::Center,
        false,
        Some(&mask),
    );
    for x in [20, 26, 33] {
        assert_eq!(
            app.layers
                .active()
                .pixmap()
                .pixel(x, 20)
                .expect("in bounds"),
            slate,
            "outside the selection the canvas must stay untouched at {x}"
        );
    }
    assert!(
        app.layers
            .active()
            .pixmap()
            .pixel(10, 20)
            .expect("in bounds")
            .alpha()
            > 0,
        "inside the selection the line paints"
    );
}

#[test]
fn a_shape_commit_confines_to_the_selection() {
    let mut app = App::new_headless();
    let mask = left_half_clip(&app);
    let slate = app
        .layers
        .active()
        .pixmap()
        .pixel(30, 4)
        .expect("in bounds");
    app.commit_drag(
        DragKind::Shape(ShapeKind::Rectangle),
        pos2(4.0, 4.0),
        pos2(36.0, 30.0),
        Some(Color32::BLACK),
        None,
        2.0,
        ShapeOutline::Center,
        false,
        Some(&mask),
    );
    assert_eq!(
        app.layers
            .active()
            .pixmap()
            .pixel(30, 4)
            .expect("in bounds"),
        slate,
        "the outline's right half must stay untouched"
    );
    assert!(
        app.layers
            .active()
            .pixmap()
            .pixel(10, 4)
            .expect("in bounds")
            .alpha()
            > 0,
        "the outline's left half paints"
    );
}
