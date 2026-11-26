//! The multi-touch gesture MACHINE tests (owner requests
//! 2026-11-25/2026-11-26, the F20 web slices): the pure [`TouchGesture`]
//! session machine — a gesture begins, the session outlives the finger
//! count dropping to one, everything clears when the last finger lifts,
//! and a pinch frame carries its zoom factor plus the PREVIOUS center as
//! the zoom's anchor. The REAL-pass regressions live in
//! `touch_gesture_pass.rs`; the mouse drag-pan regression in
//! `drag_pan.rs`.

use super::super::TouchGesture;
use eframe::egui::{MultiTouchInfo, Pos2, Vec2};

/// One frame of `InputState::multi_touch()` output in the shape the
/// egui backend reports it: the gesture's translation delta, its pinch
/// factor and its CURRENT center (the average of the touch points).
fn gesture(translation: Vec2, zoom: f32, center: Pos2) -> MultiTouchInfo {
    MultiTouchInfo {
        start_time: 0.0,
        start_pos: Pos2::ZERO,
        center_pos: center,
        num_touches: 2,
        zoom_delta: zoom,
        zoom_delta_2d: Vec2::splat(zoom),
        rotation_delta: 0.0,
        translation_delta: translation,
        force: 0.0,
    }
}

#[test]
fn a_two_finger_gesture_begins_and_pans() {
    let mut pan = TouchGesture::default();

    let frame = pan.step(Some(gesture(Vec2::new(10.0, 4.0), 1.0, Pos2::ZERO)), true);

    assert!(frame.began, "the gesture's first frame flags the takeover");
    assert!(frame.session);
    assert_eq!(frame.delta, Vec2::new(10.0, 4.0));

    let next = pan.step(Some(gesture(Vec2::new(6.0, 2.0), 1.0, Pos2::ZERO)), true);

    assert!(!next.began, "the second gesture frame is no takeover");
    assert!(next.session);
    assert_eq!(next.delta, Vec2::new(6.0, 2.0));
}

#[test]
fn the_session_outlives_the_finger_count_dropping_to_one() {
    let mut pan = TouchGesture::default();

    pan.step(Some(gesture(Vec2::new(10.0, 4.0), 1.0, Pos2::ZERO)), true);

    // One finger lifts: egui stops reporting a multi-touch gesture, but the
    // touch session is still "panning" — the leftover finger must not
    // resume painting (the paint-app convention).
    let leftover = pan.step(None, true);

    assert_eq!(leftover.delta, Vec2::ZERO, "no gesture, no delta");
    assert_eq!(leftover.zoom_factor, 1.0, "no gesture, no zoom");
    assert!(!leftover.began);
    assert!(leftover.session, "the session holds while a finger is down");

    // The LAST finger lifts: the session ends — painting may resume.
    let cleared = pan.step(None, false);

    assert!(!cleared.session);
    assert_eq!(cleared.delta, Vec2::ZERO);
}

#[test]
fn a_single_finger_never_starts_a_session() {
    let mut pan = TouchGesture::default();

    for _ in 0..5 {
        let frame = pan.step(None, true);

        assert!(!frame.session, "one finger paints — it is no gesture");
        assert!(!frame.began);
        assert_eq!(frame.delta, Vec2::ZERO);
    }

    // A gesture AFTER all fingers lifted is a fresh takeover again.
    let frame = pan.step(None, false);
    assert!(!frame.session);
    let fresh = pan.step(Some(gesture(Vec2::new(1.0, 1.0), 1.0, Pos2::ZERO)), true);
    assert!(fresh.began);
    assert!(fresh.session);
}

#[test]
fn finger_count_change_frames_carry_zeroed_deltas() {
    // egui itself zeroes BOTH deltas on finger add/remove frames (the
    // gesture re-baselines: `previous` is taken from the CURRENT state);
    // the machine passes such a frame through as a normal gesture frame —
    // no special case, no jump.
    let mut pan = TouchGesture::default();

    let add = pan.step(
        Some(gesture(Vec2::ZERO, 1.0, Pos2::new(150.0, 150.0))),
        true,
    );

    assert!(add.began);
    assert_eq!(add.delta, Vec2::ZERO);
    assert_eq!(add.zoom_factor, 1.0);
}

#[test]
fn a_pinch_frame_carries_its_factor_and_the_previous_center() {
    // The anchor law: the FIRST gesture frame has no previous center to
    // anchor at (its zoom factor is 1.0 by the re-baseline above), and
    // every later pinch frame reports the PREVIOUS frame's center as its
    // `anchor` — the zoom keeps THAT screen point fixed, so the content
    // follows the fingers exactly.
    let mut pan = TouchGesture::default();

    let first = pan.step(
        Some(gesture(Vec2::ZERO, 1.0, Pos2::new(150.0, 150.0))),
        true,
    );
    assert_eq!(first.anchor, None, "no previous center on the first frame");

    let pinch = pan.step(
        Some(gesture(Vec2::ZERO, 1.4, Pos2::new(160.0, 160.0))),
        true,
    );

    assert_eq!(pinch.zoom_factor, 1.4);
    assert_eq!(
        pinch.anchor,
        Some(Pos2::new(150.0, 150.0)),
        "the zoom anchors at the PREVIOUS frame's center"
    );

    // The last finger lifts: the anchor state clears with the session.
    pan.step(None, false);
    let fresh = pan.step(Some(gesture(Vec2::ZERO, 1.0, Pos2::new(40.0, 40.0))), true);
    assert_eq!(fresh.anchor, None, "a fresh session starts centerless");
}
