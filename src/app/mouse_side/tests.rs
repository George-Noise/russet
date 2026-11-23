use super::*;

use crate::app::App;
use crate::history_tree::ActionKind;
use eframe::egui::{Modifiers, Pos2};

/// A raw press event of a side button, as winit delivers it
/// (Back → `Extra1`, Forward → `Extra2`).
fn side_press(button: PointerButton) -> egui::Event {
    egui::Event::PointerButton {
        pos: Pos2::ZERO,
        button,
        pressed: true,
        modifiers: Modifiers::default(),
    }
}

/// One headless frame's input: the given side-button presses delivered
/// through RawInput — the REAL winit → egui path (the frame's derived
/// `pointer.button_pressed` state is built from it before the closure).
fn input_with_presses(buttons: &[PointerButton]) -> egui::RawInput {
    let mut input = egui::RawInput::default();
    input.events.extend(buttons.iter().map(|&b| side_press(b)));
    input
}

/// A headless app with ONE recorded stroke node: `can_undo` is true,
/// `can_redo` is false. A successful undo leaves a redo available.
fn app_with_one_node() -> App {
    let mut app = App::new_headless();
    app.record_active_action(ActionKind::Pencil, eframe::egui::Color32::BLACK, None);
    app
}

#[test]
fn a_fresh_session_answers_nothing_until_asked() {
    let state = MouseSideButtons::default();

    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Ask);
    assert_eq!(state.press_outcome(SideButton::Forward), PressOutcome::Ask);
}

#[test]
fn a_press_while_the_dialog_is_open_is_inert() {
    let mut state = MouseSideButtons::default();
    state.open_dialog();

    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Inert);
    assert_eq!(
        state.press_outcome(SideButton::Forward),
        PressOutcome::Inert
    );
}

#[test]
fn an_accept_enables_undo_on_mouse4_and_redo_on_mouse5() {
    let mut state = MouseSideButtons::default();
    state.answer(true);

    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Undo);
    assert_eq!(state.press_outcome(SideButton::Forward), PressOutcome::Redo);
}

#[test]
fn a_decline_disables_both_side_buttons() {
    let mut state = MouseSideButtons::default();
    state.answer(false);

    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Inert);
    assert_eq!(
        state.press_outcome(SideButton::Forward),
        PressOutcome::Inert
    );
}

#[test]
fn closing_the_dialog_without_answering_asks_again() {
    let mut state = MouseSideButtons::default();
    state.open_dialog();
    state.dismissed();

    assert_eq!(state.choice, None, "the close button decides nothing");
    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Ask);
    assert_eq!(
        state.press_outcome(SideButton::Forward),
        PressOutcome::Ask,
        "the next press asks again"
    );
}

#[test]
fn an_answer_closes_the_dialog() {
    let mut state = MouseSideButtons::default();
    state.open_dialog();
    state.answer(false);

    assert!(!state.dialog_is_open(), "the choice settles the question");
}

// --- App-level: the real press handler over the real history ------------

#[test]
fn the_first_side_press_opens_the_modal_instead_of_acting() {
    let mut app = app_with_one_node();
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_pass(input_with_presses(&[PointerButton::Extra1]), |ctx| {
        app.handle_mouse_side_buttons(ctx);
    });
    output.textures_delta.clear();

    assert!(
        app.mouse_side.dialog_is_open(),
        "the first press asks, it does not undo"
    );
    assert!(
        !app.layers.can_redo(),
        "the asking press itself must not undo"
    );
}

#[test]
fn an_accepted_session_routes_mouse4_to_undo_and_mouse5_to_redo() {
    let mut app = app_with_one_node();
    app.mouse_side.answer(true);
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_pass(input_with_presses(&[PointerButton::Extra1]), |ctx| {
        app.handle_mouse_side_buttons(ctx);
    });
    output.textures_delta.clear();
    assert!(app.layers.can_redo(), "Mouse4 undid the recorded node");

    let mut output = ctx.run_pass(input_with_presses(&[PointerButton::Extra2]), |ctx| {
        app.handle_mouse_side_buttons(ctx);
    });
    output.textures_delta.clear();
    assert!(!app.layers.can_redo(), "Mouse5 redid what Mouse4 undid");
}

#[test]
fn a_declined_session_ignores_side_presses() {
    let mut app = app_with_one_node();
    app.mouse_side.answer(false);
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_pass(input_with_presses(&[PointerButton::Extra1]), |ctx| {
        app.handle_mouse_side_buttons(ctx);
    });
    output.textures_delta.clear();

    assert!(!app.layers.can_redo(), "a declined session must not undo");
    assert!(
        !app.mouse_side.dialog_is_open(),
        "a declined session is never asked again"
    );
}

#[test]
fn a_side_press_during_a_stroke_is_swallowed() {
    let mut app = app_with_one_node();
    app.mouse_side.answer(true);
    app.is_stroking = true;
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_pass(input_with_presses(&[PointerButton::Extra1]), |ctx| {
        app.handle_mouse_side_buttons(ctx);
    });
    output.textures_delta.clear();

    assert!(
        !app.layers.can_redo(),
        "an undo mid-stroke would corrupt the pending node"
    );
}

#[test]
fn a_side_press_while_the_modal_is_open_changes_nothing() {
    let mut app = app_with_one_node();
    app.mouse_side.open_dialog();
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_pass(input_with_presses(&[PointerButton::Extra1]), |ctx| {
        app.handle_mouse_side_buttons(ctx);
    });
    output.textures_delta.clear();

    assert!(!app.layers.can_redo(), "the open modal swallows presses");
    assert!(app.mouse_side.dialog_is_open(), "the modal stays on screen");
}

#[test]
fn the_press_events_are_consumed_so_nothing_else_sees_them() {
    let mut app = app_with_one_node();
    app.mouse_side.answer(false);
    let ctx = eframe::egui::Context::default();

    let mut output = ctx.run_pass(
        input_with_presses(&[PointerButton::Extra1, PointerButton::Extra2]),
        |ctx| {
            app.handle_mouse_side_buttons(ctx);
            ctx.input(|i| {
                let presses_left = i
                    .events
                    .iter()
                    .filter(|event| {
                        matches!(event, egui::Event::PointerButton { pressed: true, .. })
                    })
                    .count();
                assert_eq!(
                    presses_left, 0,
                    "side-button presses are consumed unconditionally"
                );
            });
        },
    );
    output.textures_delta.clear();
}
