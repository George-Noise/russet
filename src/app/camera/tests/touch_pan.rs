//! The multi-touch pan tests (owner request 2026-11-25, the F20 web
//! slice): the pure [`TouchPan`] session machine (gesture begins, the
//! session outlives the finger count dropping to one, everything clears
//! when the last finger lifts), two REAL-pass regressions through
//! `Context::run_pass` in the app's frame order (the two-finger
//! translation pans the view; the MIDDLE-DRAG pan — whose delta
//! application was lost in the 2026-11-13 app.rs split and restored
//! 2026-11-25 — moves the view again), and the stroke takeover (a
//! two-finger gesture beginning mid-stroke commits the in-flight gesture
//! as its node instead of orphaning the painted pixels).

use super::super::{App, TouchPan};
use crate::tools::{SmoothStroke, Tool};
use eframe::egui::{self, Color32, PointerButton, Pos2, Rect, Sense, TouchPhase, Vec2};

/// One Touch event in the eframe-web shape (device 0, the only device the
/// web backend ever reports).
fn touch(id: u64, phase: TouchPhase, pos: Pos2) -> egui::Event {
    egui::Event::Touch {
        device_id: egui::TouchDeviceId(0),
        id: egui::TouchId::from(id),
        phase,
        pos,
        force: None,
    }
}

#[test]
fn a_two_finger_gesture_begins_and_pans() {
    let mut pan = TouchPan::default();

    let frame = pan.step(Some(Vec2::new(10.0, 4.0)), true);

    assert!(frame.began, "the gesture's first frame flags the takeover");
    assert!(frame.session);
    assert_eq!(frame.delta, Vec2::new(10.0, 4.0));

    let next = pan.step(Some(Vec2::new(6.0, 2.0)), true);

    assert!(!next.began, "the second gesture frame is no takeover");
    assert!(next.session);
    assert_eq!(next.delta, Vec2::new(6.0, 2.0));
}

#[test]
fn the_session_outlives_the_finger_count_dropping_to_one() {
    let mut pan = TouchPan::default();

    pan.step(Some(Vec2::new(10.0, 4.0)), true);

    // One finger lifts: egui stops reporting a multi-touch gesture, but the
    // touch session is still "panning" — the leftover finger must not
    // resume painting (the paint-app convention).
    let leftover = pan.step(None, true);

    assert_eq!(leftover.delta, Vec2::ZERO, "no gesture, no delta");
    assert!(!leftover.began);
    assert!(leftover.session, "the session holds while a finger is down");

    // The LAST finger lifts: the session ends — painting may resume.
    let cleared = pan.step(None, false);

    assert!(!cleared.session);
    assert_eq!(cleared.delta, Vec2::ZERO);
}

#[test]
fn a_single_finger_never_starts_a_session() {
    let mut pan = TouchPan::default();

    for _ in 0..5 {
        let frame = pan.step(None, true);

        assert!(!frame.session, "one finger paints — it is no gesture");
        assert!(!frame.began);
        assert_eq!(frame.delta, Vec2::ZERO);
    }

    // A gesture AFTER all fingers lifted is a fresh takeover again.
    let frame = pan.step(None, false);
    assert!(!frame.session);
    let fresh = pan.step(Some(Vec2::new(1.0, 1.0)), true);
    assert!(fresh.began);
    assert!(fresh.session);
}

#[test]
fn finger_count_change_frames_carry_a_zero_delta() {
    // egui itself zeroes `translation_delta` on finger add/remove frames
    // (the average re-baselines); the machine passes such a frame through
    // as a normal gesture frame — no special case, no jump.
    let mut pan = TouchPan::default();

    let add = pan.step(Some(Vec2::ZERO), true);

    assert!(add.began);
    assert_eq!(add.delta, Vec2::ZERO);
}

/// Runs ONE headless frame in the app's frame order hosting `method` in a
/// frame-less area at the origin (the pointer/touch events carry their own
/// positions; the area's rect hosts the interaction).
fn pass(ctx: &egui::Context, events: Vec<egui::Event>, app: &mut App) {
    let input = egui::RawInput {
        time: None,
        events,
        ..Default::default()
    };
    let mut output = ctx.run_pass(input, |ctx| {
        egui::Area::new(egui::Id::unique("touch-pan-test"))
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                app.handle_touch_pan(ui);
            });
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (`clear` is the sanctioned "drop intentionally"
    // path — epaint's TexturesDelta asserts when unhandled deltas drop).
    output.textures_delta.clear();
}

#[test]
fn multi_touch_translation_pans_the_view() {
    let mut app = App::new_headless();
    let ctx = egui::Context::default();

    // A pointer position must exist BEFORE the gesture starts (egui seeds
    // the touch-gesture state from the PREVIOUS frame's interact pos — on
    // the web backend the primary touch itself drives the pointer, so this
    // frame mirrors that reality).
    pass(
        &ctx,
        vec![egui::Event::PointerMoved(Pos2::new(100.0, 100.0))],
        &mut app,
    );

    // Two fingers land (the finger-add frame — egui zeroes the delta).
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
    );
    assert_eq!(app.pan, Vec2::ZERO, "the finger-add frame pans nothing");

    // Both fingers move by (10, 4): the average moves (10, 4) — the
    // content follows the fingers (the drag convention, not the wheel's).
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Move, Pos2::new(110.0, 104.0)),
            touch(2, TouchPhase::Move, Pos2::new(210.0, 204.0)),
        ],
        &mut app,
    );
    assert_eq!(app.pan, Vec2::new(10.0, 4.0));

    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Move, Pos2::new(120.0, 108.0)),
            touch(2, TouchPhase::Move, Pos2::new(220.0, 208.0)),
        ],
        &mut app,
    );
    assert_eq!(app.pan, Vec2::new(20.0, 8.0));

    // One finger lifts: no multi-touch gesture anymore (the leftover
    // finger holds the session but pans nothing).
    pass(
        &ctx,
        vec![touch(2, TouchPhase::End, Pos2::new(220.0, 208.0))],
        &mut app,
    );
    assert_eq!(
        app.pan,
        Vec2::new(20.0, 8.0),
        "a single finger is no gesture"
    );
    assert!(
        app.touch_pan.session,
        "the session outlives the dropped finger"
    );

    // The last finger lifts: the session clears.
    pass(
        &ctx,
        vec![touch(1, TouchPhase::End, Pos2::new(120.0, 108.0))],
        &mut app,
    );
    assert!(!app.touch_pan.session, "no finger down — session over");
}

#[test]
fn a_middle_button_drag_pans_the_view() {
    // Regression (2026-11-25): the 2026-11-13 app.rs split carried the
    // drag-pan DETECTION, the Grabbing cursor and the stroke suppression
    // into camera.rs but dropped the `self.pan += response.drag_delta()`
    // application — the middle-drag and Space+LMB pans moved NOTHING
    // since. The line is restored; this pins it.
    let mut app = App::new_headless();
    let ctx = egui::Context::default();
    let rect = Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(300.0, 300.0));

    // One pass helper rendering the interactable rect each frame — a
    // warm-up pass first so the NEXT pass's press event hit-tests THIS
    // widget (egui assigns the drag to whatever was under the pointer on
    // the previous pass).
    fn drag_pass(ctx: &egui::Context, events: Vec<egui::Event>, app: &mut App, rect: Rect) {
        let input = egui::RawInput {
            events,
            ..Default::default()
        };
        let mut output = ctx.run_pass(input, |ctx| {
            // `movable(false)` keeps the area's own root widget from
            // stealing the drag (a movable Area senses drags to move
            // itself, and its root covers the whole pass).
            egui::Area::new(egui::Id::unique("drag-pan-test"))
                .movable(false)
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    // The allocated rect IS the area's content, so the
                    // widget is laid out, registered for the next pass's
                    // hit-test and draggable exactly like a real canvas
                    // widget.
                    let response = ui.allocate_rect(rect, Sense::click_and_drag());
                    app.handle_pan(ui, &response);
                });
        });
        output.textures_delta.clear();
    }

    // TWO warm-up passes: an auto-sized area's first pass is a sizing
    // pass whose widgets are not hit-testable, so the press must arrive on
    // the SECOND pass at the earliest — the widget then sits in the
    // hit-test list the press reads.
    drag_pass(&ctx, vec![], &mut app, rect);
    drag_pass(&ctx, vec![], &mut app, rect);
    // The press at (100, 100) — inside the rect.
    drag_pass(
        &ctx,
        vec![egui::Event::PointerButton {
            pos: Pos2::new(100.0, 100.0),
            button: PointerButton::Middle,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
        &mut app,
        rect,
    );
    assert_eq!(app.pan, Vec2::ZERO, "the press itself pans nothing");

    // The drag to (130, 116) — delta (30, 16) — pans the view with the
    // pointer (the Space+LMB drag rides the same line).
    drag_pass(
        &ctx,
        vec![egui::Event::PointerMoved(Pos2::new(130.0, 116.0))],
        &mut app,
        rect,
    );
    assert_eq!(
        app.pan,
        Vec2::new(30.0, 16.0),
        "the middle drag moves the view with the pointer"
    );
}

#[test]
fn a_gesture_takeover_commits_the_in_flight_stroke_as_a_node() {
    // The takeover rule: a two-finger gesture beginning mid-stroke ENDS the
    // stroke as a proper history node (the painted pixels stay, undoable)
    // instead of orphaning them under the gesture's stroke suppression.
    let mut app = App::new_headless();
    app.stroke_button = Some(PointerButton::Primary);
    app.is_stroking = true;
    app.stroke_paint = Some(Color32::BLACK);
    app.stroke_tool = Some(Tool::Pencil);
    app.stroke_smooth = SmoothStroke::new(Pos2::new(10.0, 10.0));

    let before = app.layers.layer_nodes(0).expect("layer 0 exists").len();

    app.commit_stroke_gesture();

    let after = app.layers.layer_nodes(0).expect("layer 0 exists").len();
    assert_eq!(after, before + 1, "one node for the taken-over stroke");
    assert!(!app.is_stroking, "the gesture state is ground zero again");
    assert!(app.stroke_paint.is_none());
    assert!(app.stroke_button.is_none() && app.stroke_tool.is_none());
}
