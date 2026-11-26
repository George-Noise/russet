//! The multi-touch gesture MACHINE tests (owner requests
//! 2026-11-25/2026-11-26, the F20 web slices): the pure [`TouchGesture`]
//! session machine — the SECOND finger's raw Touch-stream landing fires the
//! takeover and poisons the session EVEN WHEN egui's gesture never engages
//! (the owner's 2026-11-26 second report: a phone's fingers land at
//! DIFFERENT times, and the gesture's seeding may lag or fail), the session
//! outlives the finger count dropping to one, everything clears when the
//! last finger lifts, a pinch frame carries its zoom factor plus the
//! PREVIOUS center as the zoom's anchor, and the WEB pipeline's per-tap
//! double press is deduped (the 2026-11-26 third report: "the fill
//! applies twice on a single finger's touch" — eframe-web runs the
//! browser's real `pointerdown` as an app pass BEFORE any touch exists,
//! then the touchstart's emulated press follows while the touch is down).
//! The REAL-pass regressions live in `touch_gesture_pass.rs`; the mouse
//! drag-pan regression in `drag_pan.rs`.

use super::super::{TouchGesture, fold_touch_ids};
use eframe::egui::{Event, MultiTouchInfo, Pos2, TouchId, TouchPhase, Vec2};

/// One raw Touch-stream id in the eframe-web shape.
fn t(id: u64) -> TouchId {
    TouchId::from(id)
}

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

    // Finger 1 lands ALONE (a lone finger paints).
    let birth = pan.step(&[t(1)], &[], true, false, None);
    assert!(!birth.press_duplicate, "no bare press, no duplicate");
    assert!(!birth.second_finger);
    assert!(!birth.session, "one finger paints — it is no gesture");

    // Finger 2 lands in a LATER frame (the staggered reality) and the
    // gesture reports from this frame on.
    let frame = pan.step(
        &[t(2)],
        &[],
        true,
        false,
        Some(gesture(Vec2::new(10.0, 4.0), 1.0, Pos2::ZERO)),
    );

    assert!(
        frame.second_finger,
        "the second finger's landing flags the takeover"
    );
    assert!(frame.session);
    assert_eq!(frame.delta, Vec2::new(10.0, 4.0));

    let next = pan.step(
        &[],
        &[],
        true,
        false,
        Some(gesture(Vec2::new(6.0, 2.0), 1.0, Pos2::ZERO)),
    );

    assert!(!next.second_finger, "a later gesture frame is no takeover");
    assert!(next.session);
    assert_eq!(next.delta, Vec2::new(6.0, 2.0));
}

#[test]
fn the_second_finger_fires_the_takeover_without_any_gesture() {
    // THE owner report 2026-11-26, second pass: a phone's fingers land at
    // DIFFERENT times, and egui's multi-touch gesture may engage LATE or
    // never (its seeding needs a prior pointer position — backends
    // differ). The takeover and the suppression must ride the RAW Touch
    // stream alone: the second finger's `Start` id transition IS the
    // trigger, gesture or no gesture.
    let mut pan = TouchGesture::default();

    pan.step(&[t(1)], &[], true, false, None);
    let frames = [
        pan.step(&[], &[], true, false, None),
        pan.step(&[], &[], true, false, None),
    ];
    assert!(frames.iter().all(|f| !f.second_finger && !f.session));

    // The second finger lands — the gesture NEVER reports on any frame.
    let takeover = pan.step(&[t(2)], &[], true, false, None);

    assert!(
        takeover.second_finger,
        "the raw 1→2 id transition is the trigger"
    );
    assert!(takeover.session, "the session poisons without any gesture");
    assert_eq!(takeover.delta, Vec2::ZERO, "no gesture report, no delta");

    // Still poisoned on the hold frames — and a lone leftover finger after
    // one lifts keeps the poison until the last finger lifts.
    let hold = pan.step(&[], &[], true, false, None);
    assert!(hold.session, "the poisoned session holds with no gesture");
    let leftover = pan.step(&[], &[t(2)], true, false, None);
    assert!(leftover.session, "the leftover finger keeps the session");
    assert!(!leftover.second_finger);

    let cleared = pan.step(&[], &[t(1)], false, false, None);
    assert!(!cleared.session, "the last finger's lift ends the session");
}

#[test]
fn the_session_outlives_the_finger_count_dropping_to_one() {
    let mut pan = TouchGesture::default();

    pan.step(&[t(1)], &[], true, false, None);
    pan.step(
        &[t(2)],
        &[],
        true,
        false,
        Some(gesture(Vec2::new(10.0, 4.0), 1.0, Pos2::ZERO)),
    );

    // One finger lifts: egui stops reporting a multi-touch gesture, but the
    // touch session is still "panning" — the leftover finger must not
    // resume painting (the paint-app convention).
    let leftover = pan.step(&[], &[t(2)], true, false, None);

    assert_eq!(leftover.delta, Vec2::ZERO, "no gesture, no delta");
    assert_eq!(leftover.zoom_factor, 1.0, "no gesture, no zoom");
    assert!(!leftover.second_finger);
    assert!(leftover.session, "the session holds while a finger is down");

    // The LAST finger lifts: the session ends — painting may resume.
    let cleared = pan.step(&[], &[t(1)], false, false, None);

    assert!(!cleared.session);
    assert_eq!(cleared.delta, Vec2::ZERO);
}

#[test]
fn a_fresh_lone_finger_after_the_session_draws_again() {
    // The poison lifts with the LAST finger: a NEW touch session's lone
    // finger paints again — the suppression must not leak across sessions.
    let mut pan = TouchGesture::default();

    pan.step(&[t(1)], &[], true, false, None);
    pan.step(&[t(2)], &[], true, false, None);
    pan.step(&[], &[t(1), t(2)], false, false, None);

    let fresh = pan.step(&[t(7)], &[], true, false, None);
    assert!(!fresh.session, "the new lone finger paints again");
    assert!(!fresh.second_finger);
}

#[test]
fn a_cancelled_finger_leaves_the_glass() {
    // A browser `touchcancel` (the browser reclaiming the touch) ends the
    // finger the same as a touchend: the session survives while any other
    // finger is down and clears with the last one.
    let mut pan = TouchGesture::default();

    pan.step(&[t(1)], &[], true, false, None);
    pan.step(&[t(2)], &[], true, false, None);

    let cancelled = pan.step(&[], &[t(2)], true, false, None);
    assert!(cancelled.session, "the cancelled finger ends, one remains");

    let cleared = pan.step(&[], &[t(1)], false, false, None);
    assert!(!cleared.session);
}

#[test]
fn a_stale_id_set_reconciles_against_eguis_touch_state() {
    // The missed-End safety: if the raw fold somehow never saw an End,
    // egui's `any_touches()` (fed by the same events, but battle-tested
    // across backends) is the authority — an empty egui touch state
    // clears the set, so a lost end cannot wedge the session poisoned
    // forever.
    let mut pan = TouchGesture::default();

    pan.step(&[t(1)], &[], true, false, None);
    pan.step(&[t(2)], &[], true, false, None);
    assert!(pan.step(&[], &[], true, false, None).session);

    // egui says no finger is down; the set is stale.
    let healed = pan.step(&[], &[], false, false, None);

    assert!(!healed.session, "the reconcile clears the stale set");
}

#[test]
fn finger_count_change_frames_carry_zeroed_deltas() {
    // egui itself zeroes BOTH deltas on finger add/remove frames (the
    // gesture re-baselines: `previous` is taken from the CURRENT state);
    // the machine passes such a frame through as a normal gesture frame —
    // no special case, no jump.
    let mut pan = TouchGesture::default();

    pan.step(&[t(1)], &[], true, false, None);
    let add = pan.step(
        &[t(2)],
        &[],
        true,
        false,
        Some(gesture(Vec2::ZERO, 1.0, Pos2::new(150.0, 150.0))),
    );

    assert!(add.second_finger);
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

    pan.step(&[t(1)], &[], true, false, None);
    let first = pan.step(
        &[t(2)],
        &[],
        true,
        false,
        Some(gesture(Vec2::ZERO, 1.0, Pos2::new(150.0, 150.0))),
    );
    assert_eq!(first.anchor, None, "no previous center on the first frame");

    let pinch = pan.step(
        &[],
        &[],
        true,
        false,
        Some(gesture(Vec2::ZERO, 1.4, Pos2::new(160.0, 160.0))),
    );

    assert_eq!(pinch.zoom_factor, 1.4);
    assert_eq!(
        pinch.anchor,
        Some(Pos2::new(150.0, 150.0)),
        "the zoom anchors at the PREVIOUS frame's center"
    );

    // The last finger lifts: the anchor state clears with the session.
    pan.step(&[], &[t(1), t(2)], false, false, None);
    pan.step(&[t(1)], &[], true, false, None);
    let fresh = pan.step(
        &[t(2)],
        &[],
        true,
        false,
        Some(gesture(Vec2::ZERO, 1.0, Pos2::new(40.0, 40.0))),
    );
    assert_eq!(fresh.anchor, None, "a fresh session starts centerless");
}

#[test]
fn the_webs_second_press_of_a_tap_is_the_duplicate() {
    // THE owner report 2026-11-26, third pass ("the fill applies twice on
    // a single finger's touch"): eframe-web fires a tap's press TWICE —
    // the browser's real `pointerdown` runs a whole app pass in its own
    // handler (eframe-web's `install_pointerdown` calls `runner.logic()`
    // synchronously), BEFORE any Touch event exists; the following
    // `touchstart` then pushes a second, EMULATED press alongside the
    // Touch start. A press observed while the touch is down, AFTER a bare
    // press was seen, is the tap's duplicate — the caller must not
    // re-fire a stateless click tool on it.
    let mut pan = TouchGesture::default();

    // The browser's real pointerdown: a press with NO touch down — the
    // tap's first press dispatches the tool.
    let bare = pan.step(&[], &[], false, true, None);
    assert!(
        !bare.press_duplicate,
        "the bare press is the tap's one real press"
    );

    // The touchstart's pass: the touch is now down and its emulated press
    // rides along — the DUPLICATE.
    let dupe = pan.step(&[t(1)], &[], true, true, None);
    assert!(
        dupe.press_duplicate,
        "the touchstart's emulated press is the tap's duplicate"
    );

    // A second finger's own real pointerdown (before its touchstart) also
    // arrives while touches are down — suppressed the same way.
    let second_fingers = pan.step(&[t(2)], &[], true, true, None);
    assert!(second_fingers.press_duplicate);
    assert!(second_fingers.second_finger, "and it takes over anyway");
}

#[test]
fn a_native_press_sharing_the_touch_pass_is_no_duplicate() {
    // Native winit (egui-winit's `on_touch`) emits the touch and its ONE
    // emulated press in the SAME pass — no bare press precedes it, so the
    // single press dispatches the tool normally: the dedupe must not kill
    // native touch drawing.
    let mut pan = TouchGesture::default();

    let native = pan.step(&[t(1)], &[], true, true, None);
    assert!(
        !native.press_duplicate,
        "the touch's own pass is the press's first sighting — no duplicate"
    );

    // A quiet pass clears the bookkeeping: the next tap starts fresh.
    pan.step(&[], &[t(1)], false, false, None);
    let fresh = pan.step(&[t(4)], &[], true, true, None);
    assert!(!fresh.press_duplicate, "a fresh tap's press dispatches");
}

#[test]
fn the_duplicate_bookkeeping_clears_on_a_quiet_pass() {
    // A quiet pass (no touches, no press) ends the interaction — the next
    // bare press belongs to a FRESH tap, and only the press AFTER that
    // one (while a touch is down) is the duplicate again.
    let mut pan = TouchGesture::default();

    pan.step(&[], &[], false, true, None);
    pan.step(&[t(1)], &[], true, true, None);

    // The tap lifts; a quiet pass follows (the touchend pass carries no
    // press).
    pan.step(&[], &[t(1)], false, false, None);

    // A new tap: bare press, then the touchstart's duplicate.
    let bare = pan.step(&[], &[], false, true, None);
    assert!(!bare.press_duplicate);
    let dupe = pan.step(&[t(2)], &[], true, true, None);
    assert!(dupe.press_duplicate, "the fresh tap's shape repeats");
}

#[test]
fn a_simultaneous_landing_births_and_takes_over_in_one_pass() {
    // Both fingers landing in ONE pass (the rare simultaneous case): the
    // takeover still fires on that same pass.
    let mut pan = TouchGesture::default();

    let frame = pan.step(
        &[t(1), t(2)],
        &[],
        true,
        false,
        Some(gesture(Vec2::ZERO, 1.0, Pos2::new(150.0, 150.0))),
    );

    assert!(frame.second_finger, "the takeover fires at once");
    assert!(frame.session);
}

#[test]
fn fold_touch_ids_collects_starts_and_ends_only() {
    // The raw per-frame fold: `Start` ids accumulate, `End`/`Cancel` ids
    // accumulate, `Move` events (however many, coalesced) carry no id
    // change, and non-touch events are ignored.
    let touch = |id: u64, phase: TouchPhase| Event::Touch {
        device_id: eframe::egui::TouchDeviceId(0),
        id: TouchId::from(id),
        phase,
        pos: Pos2::ZERO,
        force: None,
    };

    let (started, ended) = fold_touch_ids(&[
        touch(1, TouchPhase::Start),
        touch(1, TouchPhase::Move),
        touch(1, TouchPhase::Move),
        touch(2, TouchPhase::Start),
        touch(3, TouchPhase::Cancel),
        Event::PointerMoved(Pos2::ZERO),
    ]);

    assert_eq!(started, vec![t(1), t(2)]);
    assert_eq!(ended, vec![t(3)]);

    // A duplicated Start of one id in one frame folds once.
    let (dupe_start, _) =
        fold_touch_ids(&[touch(4, TouchPhase::Start), touch(4, TouchPhase::Start)]);
    assert_eq!(dupe_start, vec![t(4)]);

    // An End and a Start of the SAME id in one frame: the fold reports
    // both; the machine's order-proof set application leaves the finger
    // live (a browser re-press within one frame).
    let mut pan = TouchGesture::default();
    pan.step(&[t(9)], &[], true, false, None);
    let repress = pan.step(&[t(9)], &[t(9)], true, false, None);
    assert!(!repress.session, "one live finger is no session");
}
