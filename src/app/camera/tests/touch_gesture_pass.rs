//! The multi-touch gesture REAL-pass regressions (owner requests
//! 2026-11-25/2026-11-26, the F20 web slices), driven through
//! `Context::run_pass` in the app's frame order: the two-finger
//! translation pans the view, the pinch SPREAD zooms it anchored at the
//! gesture's center, the combined move-and-spread frame holds the content
//! under the fingers, and a gesture beginning mid-stroke commits the
//! in-flight stroke as its node. The pure machine tests live in
//! `touch_gesture.rs`.

use super::super::App;
use crate::tools::{SmoothStroke, Tool};
use eframe::egui::{self, Color32, PointerButton, Pos2, Rect, TouchPhase, Vec2};

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

/// Runs ONE headless frame in the app's frame order hosting `method` in a
/// frame-less area at the origin (the pointer/touch events carry their own
/// positions; the area's rect hosts the interaction).
fn pass(ctx: &egui::Context, events: Vec<egui::Event>, app: &mut App, panel_rect: Rect) {
    let input = egui::RawInput {
        time: None,
        events,
        ..Default::default()
    };
    let mut output = ctx.run_pass(input, |ctx| {
        egui::Area::new(egui::Id::unique("touch-gesture-test"))
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| {
                app.handle_touch_gesture(ui, panel_rect);
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
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));

    // A pointer position must exist BEFORE the gesture starts (egui seeds
    // the touch-gesture state from the PREVIOUS frame's interact pos — on
    // the web backend the primary touch itself drives the pointer, so this
    // frame mirrors that reality).
    pass(
        &ctx,
        vec![egui::Event::PointerMoved(Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );

    // Two fingers land (the finger-add frame — egui zeroes both deltas).
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
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
        panel_rect,
    );
    assert_eq!(app.pan, Vec2::new(10.0, 4.0));

    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Move, Pos2::new(120.0, 108.0)),
            touch(2, TouchPhase::Move, Pos2::new(220.0, 208.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_eq!(app.pan, Vec2::new(20.0, 8.0));

    // One finger lifts: no multi-touch gesture anymore (the leftover
    // finger holds the session but pans nothing).
    pass(
        &ctx,
        vec![touch(2, TouchPhase::End, Pos2::new(220.0, 208.0))],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.pan,
        Vec2::new(20.0, 8.0),
        "a single finger is no gesture"
    );
    assert!(
        app.touch_gesture.session,
        "the session outlives the dropped finger"
    );

    // The last finger lifts: the session clears.
    pass(
        &ctx,
        vec![touch(1, TouchPhase::End, Pos2::new(120.0, 108.0))],
        &mut app,
        panel_rect,
    );
    assert!(!app.touch_gesture.session, "no finger down — session over");
}

#[test]
fn multi_touch_pinch_spread_zooms_the_view_at_the_gesture_center() {
    // The pinch zoom (owner request 2026-11-26): two fingers SPREAD —
    // the view zooms in by the distance ratio, ANCHORED at the gesture's
    // center (the canvas point under the midpoint stays under it).
    let mut app = App::new_headless();
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));

    pass(
        &ctx,
        vec![egui::Event::PointerMoved(Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );

    // Two fingers land at (100,100) and (200,200) — center (150,150).
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_eq!(app.zoom, 1.0, "the finger-add frame zooms nothing");
    // `canvas_pos` takes the CANVAS rect (the drawn image's rect), not the
    // panel rect the handlers receive.
    let canvas_under_center = app.canvas_pos(app.canvas_rect(panel_rect), Pos2::new(150.0, 150.0));

    // Both fingers spread symmetrically to (80,80) and (220,220): the
    // finger distance grows 100->140 units per axis — an exact ×1.4 zoom,
    // the center (150,150) unmoved.
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Move, Pos2::new(80.0, 80.0)),
            touch(2, TouchPhase::Move, Pos2::new(220.0, 220.0)),
        ],
        &mut app,
        panel_rect,
    );

    assert!(
        (app.zoom - 1.4).abs() < 1e-3,
        "the spread ratio 140/100 = x1.4 zooms the view, got {}",
        app.zoom
    );
    assert_eq!(
        app.canvas_pos(app.canvas_rect(panel_rect), Pos2::new(150.0, 150.0)),
        canvas_under_center,
        "the canvas point under the gesture's center stays under it"
    );

    // Pinching back below 1.0 zooms out; the zoom's floor is MIN_ZOOM.
    // (One frame suffices: the composition is multiplicative.)
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Move, Pos2::new(99.0, 99.0)),
            touch(2, TouchPhase::Move, Pos2::new(101.0, 101.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert!(
        app.zoom < 1.4,
        "the pinch-together frame zooms back out (x(2/140) clamped at MIN_ZOOM)"
    );
}

#[test]
fn a_move_and_spread_frame_zooms_and_pans_together() {
    // The combined gesture (owner request 2026-11-26): the same two
    // fingers BOTH move and spread — the zoom anchors at the previous
    // center, the translation delta carries the content to the NEW
    // center: the canvas point that was under the OLD midpoint lands
    // exactly under the NEW one.
    let mut app = App::new_headless();
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));

    pass(
        &ctx,
        vec![egui::Event::PointerMoved(Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );

    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
    );
    let canvas_under_old_center =
        app.canvas_pos(app.canvas_rect(panel_rect), Pos2::new(150.0, 150.0));

    // Both fingers scale by ×1.1 around the screen origin — the pair
    // distance grows 100->110 units per axis (an exact x1.1 zoom) while
    // the gesture center rides (150,150)->(165,165) (a (15,15)
    // translation): BOTH jobs in one frame.
    pass(
        &ctx,
        vec![
            touch(1, TouchPhase::Move, Pos2::new(110.0, 110.0)),
            touch(2, TouchPhase::Move, Pos2::new(220.0, 220.0)),
        ],
        &mut app,
        panel_rect,
    );

    assert!(
        (app.zoom - 1.1).abs() < 1e-3,
        "the spread ratio 110/100 = x1.1, got {}",
        app.zoom
    );
    assert!(
        app.pan != Vec2::ZERO,
        "the frame's translation moved the view too"
    );
    assert_eq!(
        app.canvas_pos(app.canvas_rect(panel_rect), Pos2::new(165.0, 165.0)),
        canvas_under_old_center,
        "the content under the old midpoint (150,150) follows to the new one (165,165)"
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
