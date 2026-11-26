//! The multi-touch TAKEOVER regressions (owner bug report 2026-11-26:
//! "when trying to use zoom or pan on a smartphone in web, the selected
//! tool is still used"), driven through `Context::run_pass` hosting
//! `draw_canvas` — the app's REAL frame order — with the raw events in
//! the eframe-WEB shape: every touchstart carries an EMULATED primary
//! pointer press FIRST (the web backend's first touch drives the
//! pointer; `eframe/src/web/events.rs::install_touchstart`), then the
//! Touch events. eframe-web pushes that emulated press even when the
//! gesture's second finger lands, and egui's multi-touch gesture only
//! engages on the frame AFTER the second finger — so the FIRST finger
//! of every zoom/pan attempt paints through the ordinary tool path.
//! The takeover must leave no trace: the canvas, the history head and
//! the tool state return to the pre-touch moment, and the gesture
//! frames themselves paint nothing. The pure machine tests live in
//! `touch_gesture.rs`; the pass-shape regressions in
//! `touch_gesture_pass.rs`.

use super::super::App;
use crate::tools::Tool;
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

/// eframe-web's emulated primary press of a touchstart: the backend pushes
/// it FIRST (before the Touch events), at the PRIMARY touch's position —
/// the web backend's first finger drives the pointer.
fn press_at(pos: Pos2) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    }
}

/// Runs ONE headless frame in the app's frame order hosting `draw_canvas`
/// in an immovable area (a movable area's root widget steals the press) —
/// the full gesture → takeover → tool-dispatch pipeline of a real frame.
fn canvas_pass(ctx: &egui::Context, events: Vec<egui::Event>, app: &mut App, panel_rect: Rect) {
    let input = egui::RawInput {
        time: None,
        events,
        ..Default::default()
    };
    let mut output = ctx.run_pass(input, |ctx| {
        egui::Area::new(egui::Id::unique("touch-takeover-test"))
            .movable(false)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| app.draw_canvas(ui, panel_rect));
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (`clear` is the sanctioned "drop intentionally"
    // path — epaint's TexturesDelta asserts when unhandled deltas drop).
    output.textures_delta.clear();
}

#[test]
fn a_second_finger_reverts_the_first_fingers_pencil_dot() {
    let mut app = App::new_headless();
    app.tools.tool = Tool::Pencil;
    app.tools.primary = Color32::RED;
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
    let baseline = app.layers.head();
    let baseline_nodes = app.layers.layer_nodes(0).expect("layer 0 exists").len();
    // The canvas (800×600 at zoom 100%) centers in the 300×300 panel, so
    // the screen press (100,100) lands at its own CANVAS coordinates —
    // the same mapping the app uses.
    let dot = app.canvas_pos(app.canvas_rect(panel_rect), Pos2::new(100.0, 100.0));
    let (dot_x, dot_y) = (dot.x.floor() as u32, dot.y.floor() as u32);
    let untouched = app.layers.active().pixmap().pixel(dot_x, dot_y);

    // Two warm-up passes: the interact widget must sit in the hit-test
    // list before the press pass reads it (egui hit-tests the PREVIOUS
    // pass's widgets).
    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(&ctx, vec![], &mut app, panel_rect);

    // Finger 1 lands (the emulated primary press, then the touch) — one
    // finger is the phone's only drawing lever, so it paints: the pencil
    // press dot is in the layer.
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert!(app.is_stroking, "one finger still paints");
    assert_eq!(
        app.touch_baseline,
        Some((0, baseline)),
        "the session's birth pass latched the pre-touch history head"
    );
    assert_ne!(
        app.layers.active().pixmap().pixel(dot_x, dot_y),
        untouched,
        "the emulated press painted the pencil dot"
    );

    // Finger 2 lands: the gesture takes over — the dot reverts, the head
    // returns to the baseline, no stroke state survives. (The emulated
    // press comes along — eframe-web pushes one on EVERY touchstart.)
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert!(!app.is_stroking, "the takeover ended the stroke");
    assert_eq!(
        app.layers.head(),
        baseline,
        "the history returned to the pre-touch head"
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "the reverted dot stays as an ordinary tree fork (undo semantics)"
    );
    assert_eq!(
        app.layers.active().pixmap().pixel(dot_x, dot_y),
        untouched,
        "the dot is gone — a zoom/pan attempt leaves no tool marks"
    );

    // The gesture frames themselves paint nothing (finger 1's emulated
    // moves keep coming) — and the view pans by the fingers' average.
    canvas_pass(
        &ctx,
        vec![
            egui::Event::PointerMoved(Pos2::new(110.0, 104.0)),
            touch(1, TouchPhase::Move, Pos2::new(110.0, 104.0)),
            touch(2, TouchPhase::Move, Pos2::new(210.0, 204.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.layers.active().pixmap().pixel(dot_x, dot_y),
        untouched,
        "the gesture paints nothing"
    );
    assert_eq!(app.pan, Vec2::new(10.0, 4.0), "the gesture pans");

    // The last fingers lift (eframe-web: the emulated release, the
    // pointer-gone, then the touch ends): the session and the baseline
    // clear — a fresh finger paints again.
    canvas_pass(
        &ctx,
        vec![
            egui::Event::PointerButton {
                pos: Pos2::new(110.0, 104.0),
                button: PointerButton::Primary,
                pressed: false,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::PointerGone,
            touch(2, TouchPhase::End, Pos2::new(210.0, 204.0)),
            touch(1, TouchPhase::End, Pos2::new(110.0, 104.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert!(
        app.touch_baseline.is_none(),
        "the session's end clears the baseline"
    );
}

#[test]
fn a_second_finger_reverts_a_fill_that_fired_on_the_first_tap() {
    // The INSTANT-tool class of the same bug: a fill floods on the first
    // finger's press — a whole history node, not just a dot — and the
    // takeover reverts it the same way.
    let mut app = App::new_headless();
    app.tools.tool = Tool::Fill;
    app.tools.primary = Color32::RED;
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
    let baseline = app.layers.head();
    let baseline_nodes = app.layers.layer_nodes(0).expect("layer 0 exists").len();
    let (probe_x, probe_y) = (100u32, 100u32);
    let untouched = app.layers.active().pixmap().pixel(probe_x, probe_y);

    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(&ctx, vec![], &mut app, panel_rect);

    // Finger 1 lands: the fill floods the whole transparent canvas.
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_ne!(
        app.layers.active().pixmap().pixel(probe_x, probe_y),
        untouched,
        "the fill fired on the first finger's tap"
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "the flood recorded its node"
    );

    // Finger 2 lands: the takeover reverts the flood — pixels, head, all.
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.layers.head(),
        baseline,
        "the history returned to the pre-touch head"
    );
    assert_eq!(
        app.layers.active().pixmap().pixel(probe_x, probe_y),
        untouched,
        "the flood is gone — the tap that became a gesture painted nothing"
    );
}

#[test]
fn the_second_finger_reverts_the_stroke_with_no_gesture_at_all() {
    // The owner's SECOND report 2026-11-26 ("It got worse — a user cannot
    // press both fingers in the same frame; the fingers touch at
    // DIFFERENT times"): a stream where egui's multi-touch gesture NEVER
    // engages — the raw Touch events carry no pointer events at all, so
    // `touch_state.rs` has no prior pointer position to seed from and
    // `multi_touch()` stays `None` the whole session (a native-touchscreen
    // shape; on the phone the same hole appears whenever the gesture
    // seeds late or never). The takeover must still fire on the raw
    // 1→2 id transition — the second finger's `Touch{Start}` frame — and
    // the session must poison without the gesture.
    let mut app = App::new_headless();
    app.tools.tool = Tool::Pencil;
    app.tools.primary = Color32::RED;
    // The first finger's in-flight stroke (on the phone eframe-web's
    // emulated pointer press paints it; the pointer-less harness seeds
    // the same gesture state directly — the takeover must end it).
    app.stroke_button = Some(PointerButton::Primary);
    app.is_stroking = true;
    app.stroke_paint = Some(Color32::RED);
    app.stroke_tool = Some(Tool::Pencil);
    app.stroke_smooth = crate::tools::SmoothStroke::new(Pos2::new(10.0, 10.0));
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
    let baseline = app.layers.head();
    let baseline_nodes = app.layers.layer_nodes(0).expect("layer 0 exists").len();

    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(&ctx, vec![], &mut app, panel_rect);

    // Finger 1 lands — NO pointer events anywhere in this stream: the
    // session's birth latches the baseline, and no gesture can seed.
    canvas_pass(
        &ctx,
        vec![touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.touch_baseline,
        Some((0, baseline)),
        "the birth pass latched the pre-touch head — pointer or no pointer"
    );

    // Finger 2 lands in a LATER frame (the staggered reality). The gesture
    // NEVER engages: the takeover rides the raw id transition instead,
    // committing the in-flight stroke as its node and reverting it.
    canvas_pass(
        &ctx,
        vec![touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0))],
        &mut app,
        panel_rect,
    );
    assert!(!app.is_stroking, "the takeover ended the stroke");
    assert_eq!(
        app.layers.head(),
        baseline,
        "the history returned to the pre-touch head with NO gesture engaged"
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "the reverted stroke stays as an ordinary tree fork"
    );

    // The session poisons WITHOUT the gesture: a later primary press (an
    // emulated touchstart press while both fingers hold) must not start a
    // stroke until every finger has lifted.
    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(120.0, 120.0))],
        &mut app,
        panel_rect,
    );
    assert!(
        !app.is_stroking,
        "the poisoned session suppresses the press — no gesture needed"
    );

    // Every finger lifts; the next lone press paints again.
    canvas_pass(
        &ctx,
        vec![
            touch(1, TouchPhase::End, Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::End, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
    );
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(120.0, 120.0))],
        &mut app,
        panel_rect,
    );
    assert!(app.is_stroking, "a fresh lone finger paints again");
}
