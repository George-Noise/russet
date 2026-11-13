//! Unit tests of the Shape-tool gesture wells ([`super::shape_gesture_wells`]):
//! the right-button well-swap rule (owner request 2026-11-13) and the
//! Fill/Outline checkbox gating (owner request 2026-11-10).

use super::*;

#[test]
fn left_button_draws_outline_from_primary_and_fill_from_secondary() {
    let (outline, fill, swapped) = shape_gesture_wells(
        PointerButton::Primary,
        true,
        true,
        Color32::RED,
        Color32::BLUE,
    );

    assert_eq!(outline, Some(Color32::RED));
    assert_eq!(fill, Some(Color32::BLUE));
    assert!(!swapped);
}

#[test]
fn right_button_swaps_the_wells() {
    let (outline, fill, swapped) = shape_gesture_wells(
        PointerButton::Secondary,
        true,
        true,
        Color32::RED,
        Color32::BLUE,
    );

    assert_eq!(outline, Some(Color32::BLUE));
    assert_eq!(fill, Some(Color32::RED));
    assert!(swapped);
}

#[test]
fn the_fill_outline_checkboxes_still_gate_after_the_swap() {
    let (outline, fill, swapped) = shape_gesture_wells(
        PointerButton::Secondary,
        true,
        false,
        Color32::RED,
        Color32::BLUE,
    );

    assert_eq!(outline, Some(Color32::BLUE));
    assert_eq!(fill, None);
    assert!(swapped);
}
