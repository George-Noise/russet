//! Unit tests of the Shape-tool gesture wells ([`super::shape_gesture_wells`]):
//! the right-button inverted-wells rule (owner request 2026-11-15: the
//! colors are used the other way around WITHOUT swapping the wells) and the
//! Fill/Outline checkbox gating (owner request 2026-11-10).

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
