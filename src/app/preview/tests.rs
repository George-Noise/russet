//! Unit tests of the Shape-tool gesture wells ([`super::shape_gesture_wells`]):
//! the right-button inverted-wells rule (owner request 2026-11-15: the
//! colors are used the other way around WITHOUT swapping the wells), the
//! Fill/Outline checkbox gating (owner request 2026-11-10), and the
//! recorded-color mapping ([`super::shape_record_colors`] — owner request
//! 2026-11-16: a two-color gesture carries BOTH colors on its history
//! node).

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
