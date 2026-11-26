//! The REAL eframe-web touch pipeline regressions (owner bug report
//! 2026-11-26, third pass: "nothing changed in my manual tests — a new
//! node is still created, the fill applies twice on a single finger's
//! touch"): the browser fires a tap's events STAGGERED — the REAL
//! `pointerdown` first (eframe-web's `install_pointerdown` handler runs
//! `runner.logic()` synchronously, so the press lands in a pass of its
//! own, BEFORE any Touch event exists), then the `touchstart` with its
//! EMULATED press (the web backend's first touch drives the pointer)
//! alongside the Touch events. The earlier takeover rounds tested the
//! single-pass shape (`touch_takeover.rs` — the native-winit pipeline),
//! which is why they stayed green while the phone kept failing: on the
//! real phone the first press paints in its OWN pass and the SECOND
//! (emulated) press re-fired stateless click tools — the double fill —
//! and the takeover's baseline (latched on the touch birth pass) came
//! one pass too late to cover the first press's node. These tests drive
//! the exact staggered stream through `draw_canvas`.

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

/// A primary pointer press at `pos` — both the browser's REAL
/// `pointerdown` (its own pass) and the touchstart's EMULATED press use
/// this shape.
fn press_at(pos: Pos2) -> egui::Event {
    egui::Event::PointerButton {
        pos,
        button: PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    }
}

/// The emulated release of a touchend (eframe-web pushes the release,
/// then `PointerGone`, then the Touch ends).
fn release_at(pos: Pos2) -> Vec<egui::Event> {
    vec![
        egui::Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        },
        egui::Event::PointerGone,
    ]
}

/// Runs ONE headless frame in the app's frame order hosting `draw_canvas`
/// in an immovable area — the full gesture → takeover → tool-dispatch
/// pipeline of a real frame.
fn canvas_pass(ctx: &egui::Context, events: Vec<egui::Event>, app: &mut App, panel_rect: Rect) {
    let input = egui::RawInput {
        time: None,
        events,
        ..Default::default()
    };
    let mut output = ctx.run_pass(input, |ctx| {
        egui::Area::new(egui::Id::unique("touch-web-test"))
            .movable(false)
            .fixed_pos(Pos2::ZERO)
            .show(ctx, |ui| app.draw_canvas(ui, panel_rect));
    });
    output.textures_delta.clear();
}

#[test]
fn a_lone_tap_fills_once() {
    // The owner's exact words: "the fill applies twice when touched with
    // one finger". The real web pipeline fires the fill TWICE on one tap —
    // once on the browser's real `pointerdown` (its own pass, no touch
    // evidence yet) and once on the touchstart's emulated press — because
    // a stateless click tool answers every primary-press pass. The dedupe
    // (the press while a touch is down, after a bare press was seen)
    // must leave exactly ONE fill per tap.
    let mut app = App::new_headless();
    app.tools.tool = Tool::Fill;
    app.tools.primary = Color32::RED;
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
    let baseline_nodes = app.layers.layer_nodes(0).expect("layer 0 exists").len();
    let baseline_log = app.action_log.items().len();
    let (probe_x, probe_y) = (100u32, 100u32);
    let untouched = app.layers.active().pixmap().pixel(probe_x, probe_y);

    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(&ctx, vec![], &mut app, panel_rect);

    // The browser's REAL pointerdown: an app pass with a press and NO
    // touch evidence — the tap's one and only dispatch.
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );
    assert_ne!(
        app.layers.active().pixmap().pixel(probe_x, probe_y),
        untouched,
        "the tap's first press filled"
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "the first press recorded its node"
    );

    // The touchstart pass: the emulated press rides along with the Touch
    // start — it must NOT fill again.
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "the emulated press did not fill a second time"
    );
    assert_eq!(
        app.action_log.items().len() - baseline_log,
        1,
        "exactly ONE Fill row per tap"
    );

    // The tap lifts (pointerup pass, then the touchend's emulated release
    // + PointerGone + the Touch end): the fill of a DELIBERATE lone tap
    // survives — node and log row both.
    canvas_pass(
        &ctx,
        vec![egui::Event::PointerButton {
            pos: Pos2::new(100.0, 100.0),
            button: PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
        &mut app,
        panel_rect,
    );
    canvas_pass(
        &ctx,
        {
            let mut events = release_at(Pos2::new(100.0, 100.0));
            events.push(touch(1, TouchPhase::End, Pos2::new(100.0, 100.0)));
            events
        },
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "a deliberate tap keeps its node"
    );

    // The NEXT tap fills again exactly once (the dedupe never leaks
    // across taps).
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(150.0, 150.0))],
        &mut app,
        panel_rect,
    );
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(150.0, 150.0)),
            touch(5, TouchPhase::Start, Pos2::new(150.0, 150.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 2,
        "the second tap filled once more — still exactly one node per tap"
    );
}

#[test]
fn a_two_finger_gesture_leaves_no_node_at_all() {
    // The owner's exact words: "with the pencil selected a new node is
    // still created, even though undo runs — no new node should happen at
    // all on web smartphones during a two-finger pan/zoom". The whole
    // staggered pipeline: pointerdown pass → touchstart pass → finger 2's
    // pointerdown pass → finger 2's touchstart pass (the takeover) →
    // gesture frames → lifts. Nothing may survive: no pixels, no node, no
    // Tool Log row.
    let mut app = App::new_headless();
    app.tools.tool = Tool::Pencil;
    app.tools.primary = Color32::RED;
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
    let baseline = app.layers.head();
    let baseline_nodes = app.layers.layer_nodes(0).expect("layer 0 exists").len();
    let baseline_log = app.action_log.items().len();
    let dot = app.canvas_pos(app.canvas_rect(panel_rect), Pos2::new(100.0, 100.0));
    let (dot_x, dot_y) = (dot.x.floor() as u32, dot.y.floor() as u32);
    let untouched = app.layers.active().pixmap().pixel(dot_x, dot_y);

    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(&ctx, vec![], &mut app, panel_rect);

    // Finger 1: the browser's real pointerdown pass paints the pencil dot.
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );
    assert!(app.is_stroking, "the first press started the stroke");
    assert_ne!(
        app.layers.active().pixmap().pixel(dot_x, dot_y),
        untouched,
        "the pencil dot painted"
    );

    // The touchstart pass (the emulated press is a duplicate — no double
    // dot, the stroke just continues).
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
        ],
        &mut app,
        panel_rect,
    );

    // Finger 2's own pointerdown: an app pass while finger 1's touch is
    // down — no tool dispatch (its press is the session's duplicate).
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(200.0, 200.0))],
        &mut app,
        panel_rect,
    );

    // Finger 2's touchstart: the takeover fires on the raw 1→2
    // transition — and leaves NOTHING.
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(2, TouchPhase::Start, Pos2::new(200.0, 200.0)),
        ],
        &mut app,
        panel_rect,
    );
    assert!(!app.is_stroking, "the takeover discarded the stroke");
    assert_eq!(
        app.layers.head(),
        baseline,
        "the history returned to the pre-touch head"
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes,
        "NO node at all — not even an undone fork"
    );
    assert_eq!(
        app.action_log.items().len(),
        baseline_log,
        "no Tool Log row for the gesture attempt"
    );
    assert_eq!(
        app.layers.active().pixmap().pixel(dot_x, dot_y),
        untouched,
        "the pencil dot reverted"
    );

    // The gesture frames paint nothing and pan the view.
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
        "the gesture frames paint nothing"
    );

    // Both fingers lift (finger 2 first, finger 1 last): the session
    // ends, and a FRESH lone finger paints again.
    canvas_pass(
        &ctx,
        {
            let mut events = release_at(Pos2::new(210.0, 204.0));
            events.push(touch(2, TouchPhase::End, Pos2::new(210.0, 204.0)));
            events
        },
        &mut app,
        panel_rect,
    );
    canvas_pass(
        &ctx,
        {
            let mut events = release_at(Pos2::new(110.0, 104.0));
            events.push(touch(1, TouchPhase::End, Pos2::new(110.0, 104.0)));
            events
        },
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

#[test]
fn the_fill_of_the_first_press_pass_is_removed_too() {
    // The instant-tool class on the REAL pipeline: the fill fires (and
    // records its node) on the browser's pointerdown pass — BEFORE any
    // touch exists — so the takeover's baseline must be the ROLLING
    // pre-tap latch, not the touch-birth pass, or the node predates the
    // baseline and survives the revert (the owner's "extra nodes"
    // report).
    let mut app = App::new_headless();
    app.tools.tool = Tool::Fill;
    app.tools.primary = Color32::RED;
    let ctx = egui::Context::default();
    let panel_rect = Rect::from_min_size(Pos2::ZERO, Vec2::new(300.0, 300.0));
    let baseline = app.layers.head();
    let baseline_nodes = app.layers.layer_nodes(0).expect("layer 0 exists").len();
    let baseline_log = app.action_log.items().len();
    let (probe_x, probe_y) = (100u32, 100u32);
    let untouched = app.layers.active().pixmap().pixel(probe_x, probe_y);

    canvas_pass(&ctx, vec![], &mut app, panel_rect);
    canvas_pass(&ctx, vec![], &mut app, panel_rect);

    // Finger 1: the browser's real pointerdown pass — the fill floods and
    // records its node BEFORE the touch session exists.
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(100.0, 100.0))],
        &mut app,
        panel_rect,
    );
    assert_eq!(
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes + 1,
        "the first press recorded its fill node"
    );

    // The touchstart pass (the duplicate press is suppressed — no second
    // fill even here).
    canvas_pass(
        &ctx,
        vec![
            press_at(Pos2::new(100.0, 100.0)),
            touch(1, TouchPhase::Start, Pos2::new(100.0, 100.0)),
        ],
        &mut app,
        panel_rect,
    );

    // Finger 2's pointerdown and touchstart: the takeover — the fill
    // node from BEFORE the touch existed must be REMOVED (the rolling
    // baseline covers it).
    canvas_pass(
        &ctx,
        vec![press_at(Pos2::new(200.0, 200.0))],
        &mut app,
        panel_rect,
    );
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
        app.layers.layer_nodes(0).expect("layer 0 exists").len(),
        baseline_nodes,
        "the fill node created before the touch existed is gone — no fork"
    );
    assert_eq!(
        app.action_log.items().len(),
        baseline_log,
        "the fill's Tool Log row is gone too"
    );
    assert_eq!(
        app.layers.active().pixmap().pixel(probe_x, probe_y),
        untouched,
        "the flood is gone — nothing survives the gesture attempt"
    );
}
