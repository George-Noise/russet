//! Unit tests of the Shape-tool gesture wells ([`super::shape_gesture_wells`]):
//! the right-button inverted-wells rule (owner request 2026-11-15: the
//! colors are used the other way around WITHOUT swapping the wells), the
//! Fill/Outline checkbox gating (owner request 2026-11-10), the
//! recorded-color mapping ([`super::shape_record_colors`] — owner request
//! 2026-11-16: a two-color gesture carries BOTH colors on its history
//! node), and the Shift-proportional drag rule
//! ([`super::proportional_current`] — owner request 2026-11-16, Paint.NET
//! style).

use super::*;

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
