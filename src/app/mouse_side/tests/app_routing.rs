//! The App-level press routing: the real handler over the real history,
//! the press consumption and the gesture guard.

use super::super::*;
use super::{accept, app_with_one_node, decline, input_with_presses};
use eframe::egui::PointerButton;

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
    accept(&mut app.mouse_side);
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
    decline(&mut app.mouse_side);
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
    accept(&mut app.mouse_side);
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
    decline(&mut app.mouse_side);
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
