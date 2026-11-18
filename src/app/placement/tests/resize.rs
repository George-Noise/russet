//! The F13 variant A pin logic (`resize_step`): the pre-resize state is
//! pinned on the first changed frame, every resize frame re-derives the
//! target FROM THE PIN (non-monotonic Wayland/Web series cannot drift),
//! the pin clears on the first stable frame and repins fresh.

use super::common::rect;
use crate::app::placement::{WindowAnchor, resize_step};
use eframe::egui::pos2;

#[test]
fn resize_pins_the_reference_on_the_first_changed_frame() {
    let stable_area = rect(0.0, 0.0, 1000.0, 800.0);
    let win = rect(100.0, 100.0, 200.0, 200.0);
    let grown = rect(0.0, 0.0, 1200.0, 800.0);
    // First changed frame: the PRE-resize state is pinned and the free
    // axis keeps its gap ratio (100/800 = 0.125 of the new 1000 pt free run).
    let (pin, pos) = resize_step(
        Some(win),
        Some(stable_area),
        grown,
        None,
        WindowAnchor::default(),
    );
    assert_eq!(pin, Some((win, stable_area)));
    assert_eq!(pos, Some(pos2(125.0, 100.0)));
    // The series turns back (non-monotonic Wayland/Web configure events) and
    // the drawn rect is stale — the target is re-derived FROM THE PIN, so the
    // content returning to the pre-resize size lands the window EXACTLY at
    // its pre-resize position instead of drifting (the "wobbles" fix).
    let (_, pos_back) = resize_step(
        Some(rect(300.0, 100.0, 200.0, 200.0)),
        Some(grown),
        stable_area,
        pin,
        WindowAnchor::default(),
    );
    assert_eq!(pos_back, Some(pos2(100.0, 100.0)));
}

#[test]
fn resize_pin_clears_when_stable_and_repins_fresh() {
    let a = rect(0.0, 0.0, 1000.0, 800.0);
    let win = rect(100.0, 100.0, 200.0, 200.0);
    // Stable frame: the pin clears, nothing moves — the last computed
    // position persists until the NEXT resize.
    assert_eq!(
        resize_step(
            Some(win),
            Some(a),
            a,
            Some((win, a)),
            WindowAnchor::default()
        ),
        (None, None)
    );
    // The next resize pins the CURRENT state fresh (the window was moved
    // between resizes): the new reference, not the stale one.
    let b = rect(0.0, 0.0, 500.0, 800.0);
    let moved = rect(200.0, 100.0, 200.0, 200.0);
    let (pin, pos) = resize_step(Some(moved), Some(a), b, None, WindowAnchor::default());
    assert_eq!(pin, Some((moved, a)));
    assert_eq!(pos, Some(pos2(75.0, 100.0)));
}

#[test]
fn resize_step_needs_history_on_both_sides() {
    let a = rect(0.0, 0.0, 1000.0, 800.0);
    let b = rect(0.0, 0.0, 500.0, 800.0);
    // First frame ever (no previous content): nothing to compare.
    assert_eq!(
        resize_step(None, None, b, None, WindowAnchor::default()),
        (None, None)
    );
    // First frame of a just-shown window (no drawn rect yet): no pin
    // material either — the next changed frame pins normally.
    assert_eq!(
        resize_step(None, Some(a), b, None, WindowAnchor::default()),
        (None, None)
    );
}
