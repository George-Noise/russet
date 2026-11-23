//! The pure state machine: what a Mouse4/Mouse5 press does given the
//! current mode, plus the Settings dropdown's setter semantics.

use super::super::*;
use super::{accept, decline};

#[test]
fn a_fresh_session_answers_nothing_until_asked() {
    let state = MouseSideButtons::default();

    assert_eq!(state.mode, MouseSideMode::Ask, "the startup default asks");
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
    accept(&mut state);

    assert_eq!(state.mode, MouseSideMode::Always);
    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Undo);
    assert_eq!(state.press_outcome(SideButton::Forward), PressOutcome::Redo);
}

#[test]
fn a_decline_disables_both_side_buttons() {
    let mut state = MouseSideButtons::default();
    decline(&mut state);

    assert_eq!(state.mode, MouseSideMode::Never);
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

    assert_eq!(
        state.mode,
        MouseSideMode::Ask,
        "the close button decides nothing"
    );
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
    decline(&mut state);

    assert!(!state.dialog_is_open(), "the choice settles the question");
}

#[test]
fn every_mode_names_its_own_dropdown_label_and_description() {
    for mode in MouseSideMode::ALL {
        let (label, description) = (mode.label(), mode.description());

        assert!(!label.is_empty());
        assert!(!description.is_empty());
        assert_ne!(
            label, description,
            "the description explains the label, it does not repeat it"
        );
    }
}

#[test]
fn switching_the_mode_in_settings_closes_the_open_question() {
    let mut state = MouseSideButtons::default();
    state.open_dialog();

    assert!(state.set_mode(MouseSideMode::Never));
    assert_eq!(state.mode, MouseSideMode::Never);
    assert!(
        !state.dialog_is_open(),
        "a settled question must not stay on screen"
    );
}

#[test]
fn picking_the_same_mode_again_changes_nothing() {
    let mut state = MouseSideButtons::default();

    assert!(
        !state.set_mode(MouseSideMode::Ask),
        "the default re-picked is not a change"
    );
    assert_eq!(state.mode, MouseSideMode::Ask);
}

#[test]
fn returning_to_ask_from_a_settled_mode_asks_again() {
    let mut state = MouseSideButtons::default();
    decline(&mut state);

    assert!(state.set_mode(MouseSideMode::Ask));
    assert_eq!(state.press_outcome(SideButton::Back), PressOutcome::Ask);
}
