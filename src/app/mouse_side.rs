//! The Mouse4/Mouse5 Undo/Redo opt-in (owner request 2026-11-23, SPEC
//! §3.9): the mouse's side buttons — winit `Back`/`Forward`, egui
//! `PointerButton::Extra1`/`Extra2` — are bound to Undo/Redo ONLY after
//! the user accepts the first-press modal, because not everyone likes
//! that binding: the very first side press does not act, it asks. The
//! choice is SESSION-ONLY (a field on `App`, never persisted — SPEC §6
//! keeps "saving application settings" out of scope; the framework's
//! `persistence` route is described in the PROGRESS.md entry of this
//! round, not wired up).
//!
//! Mapping (the browser back/forward convention the owner picked — "сделай
//! как у других"): Mouse4 (back) = Undo, Mouse5 (forward) = Redo — one
//! node up/down the active layer's history tree, exactly like the
//! Ctrl+Z / Ctrl+Y hotkeys (SPEC §3.9).
//!
//! The pure state machine is [`MouseSideButtons`]; the App-side press
//! handler consumes the raw press events (the buttons belong to this
//! feature — a declined session, a pending ask and a mid-gesture press
//! are all equally inert) and guards the canvas gestures: an undo or
//! redo while a gesture is in flight would corrupt the pending history
//! node the gesture records on release.

use eframe::egui::{self, Context, PointerButton};

use super::App;
use super::dialogs::{dialog_button_row, dialog_id, dialog_window, forget_dialog_position};

/// Which side button was pressed: Mouse4 (`Extra1`, winit `Back`) or
/// Mouse5 (`Extra2`, winit `Forward`). A dedicated enum keeps the pure
/// state machine free of egui types.
#[derive(Clone, Copy, PartialEq)]
pub(super) enum SideButton {
    Back,
    Forward,
}

impl SideButton {
    /// The egui pointer button this side button is.
    fn pointer_button(self) -> PointerButton {
        match self {
            SideButton::Back => PointerButton::Extra1,
            SideButton::Forward => PointerButton::Extra2,
        }
    }
}

/// The user's answer to the first-press modal: ON binds Mouse4/Mouse5 to
/// Undo/Redo for the session, OFF leaves the buttons inert.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum Choice {
    On,
    Off,
}

/// What a side-button press resolves to right now.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum PressOutcome {
    /// No answer yet: open the ask modal instead of acting — the press
    /// that opens the modal never undoes/redoes itself.
    Ask,
    Undo,
    Redo,
    /// A declined session or an open modal: the press is swallowed.
    Inert,
}

/// The session-only opt-in state: `choice` is `None` until the user
/// answers the modal; `dialog_open` holds the modal on screen (while it
/// is open, every side press is inert — the dialog owns the screen).
#[derive(Default)]
pub(super) struct MouseSideButtons {
    choice: Option<Choice>,
    dialog_open: bool,
}

impl MouseSideButtons {
    /// The pure press resolver: what a Mouse4/Mouse5 press does given the
    /// current opt-in state. Shared by the press handler and the tests.
    fn press_outcome(&self, button: SideButton) -> PressOutcome {
        if self.dialog_open {
            return PressOutcome::Inert;
        }
        match self.choice {
            None => PressOutcome::Ask,
            Some(Choice::On) => match button {
                SideButton::Back => PressOutcome::Undo,
                SideButton::Forward => PressOutcome::Redo,
            },
            Some(Choice::Off) => PressOutcome::Inert,
        }
    }

    fn dialog_is_open(&self) -> bool {
        self.dialog_open
    }

    fn open_dialog(&mut self) {
        self.dialog_open = true;
    }

    /// The modal was answered: record the choice and close.
    fn answer(&mut self, enable: bool) {
        self.choice = Some(if enable { Choice::On } else { Choice::Off });
        self.dialog_open = false;
    }

    /// The modal was dismissed with its ✕ button: NO decision — the
    /// choice stays `None`, so the next side press asks again. Only the
    /// two explicit buttons settle the question (there is no settings UI
    /// to change the answer later; restarting the app resets it).
    fn dismissed(&mut self) {
        self.dialog_open = false;
    }
}

impl App {
    /// The Mouse4/Mouse5 press hook, called right after
    /// [`App::handle_hotkeys`] every frame: the side buttons are the
    /// pointer twin of the Ctrl+Z / Ctrl+Y hotkeys and work anywhere in
    /// the window, not only over the canvas (the dialogs and floating
    /// windows do not consume the extra buttons — nothing in egui does).
    pub(super) fn handle_mouse_side_buttons(&mut self, ctx: &Context) {
        let back_pressed = ctx.input(|i| i.pointer.button_pressed(PointerButton::Extra1));
        let forward_pressed = ctx.input(|i| i.pointer.button_pressed(PointerButton::Extra2));
        if !back_pressed && !forward_pressed {
            return;
        }
        // The presses are consumed UNCONDITIONALLY (a declined session, a
        // pending ask and a mid-gesture press are all equally inert) so
        // nothing else ever reacts to the side buttons.
        for button in [SideButton::Back, SideButton::Forward] {
            consume_presses_of(ctx, button.pointer_button());
        }
        // An undo/redo mid-gesture would corrupt the pending history node
        // (the stroke/preview/move commit records its own snapshot on
        // release): the press is swallowed until the canvas is idle. An
        // open Text entry counts too — its commit is a pending node.
        if self.canvas_gesture_active() {
            return;
        }
        // A double press in one frame is physically impossible; if it
        // ever happens, Back (undo) wins — the undo-first resolution the
        // exhausted-stack convention implies for the keyboard pair.
        let outcome = if back_pressed {
            self.mouse_side.press_outcome(SideButton::Back)
        } else {
            self.mouse_side.press_outcome(SideButton::Forward)
        };
        match outcome {
            PressOutcome::Inert => {}
            PressOutcome::Ask => self.mouse_side.open_dialog(),
            PressOutcome::Undo => {
                if self.layers.can_undo() {
                    self.undo();
                }
            }
            PressOutcome::Redo => {
                if self.layers.can_redo() {
                    self.redo();
                }
            }
        }
    }

    /// A canvas gesture is in flight (a freehand stroke, a Line/Shape
    /// preview drag, a Select region/move drag or an open Text entry):
    /// side-button presses must not undo/redo under it.
    fn canvas_gesture_active(&self) -> bool {
        self.is_stroking
            || self.preview.is_active()
            || self.select_gesture.is_active()
            || self.text_edit.is_some()
    }

    /// The first-press modal (owner request 2026-11-23): "do you want the
    /// mouse side buttons to undo/redo?" — built through the shared
    /// [`dialog_window`] / [`dialog_button_row`] helpers like every other
    /// dialog. YES enables the binding for this session, NO disables it,
    /// and the ✕ button decides NOTHING (the dialog closes unanswered and
    /// the next press asks again). The choice is session-only: nothing is
    /// persisted.
    pub(super) fn mouse_side_dialog(&mut self, ctx: &Context) {
        if !self.mouse_side.dialog_is_open() {
            return;
        }
        let mut yes = false;
        let mut no = false;
        let mut open = true;
        let id = dialog_id("mouse_side");
        dialog_window("Mouse Side Buttons", id, ctx)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.label("Use the mouse side buttons for Undo and Redo?");
                ui.weak(
                    "Mouse4 (back) - Undo; Mouse5 (forward) - Redo, like a browser's back/forward.",
                );
                ui.weak("The choice lasts until the app is closed.");
                let (yes_hit, no_hit) = dialog_button_row(ui, "Yes", "No");
                yes = yes_hit;
                no = no_hit;
            });
        if yes {
            self.mouse_side.answer(true);
            self.action_log.push("Mouse side buttons: Undo/Redo on");
            forget_dialog_position(ctx, id);
        } else if no {
            self.mouse_side.answer(false);
            self.action_log.push("Mouse side buttons: off");
            forget_dialog_position(ctx, id);
        } else if !open {
            self.mouse_side.dismissed();
            forget_dialog_position(ctx, id);
        }
    }
}

/// Drops every press event of ONE pointer button from the frame's raw
/// event stream (the release events stay — `any_released` bookkeeping and
/// egui's own button state are untouched). The same retain pattern the
/// clipboard dedicated-events path uses ([`super::input`]).
fn consume_presses_of(ctx: &Context, button: PointerButton) {
    ctx.input_mut(|i| {
        i.events.retain(|event| match event {
            egui::Event::PointerButton {
                button: pressed_button,
                pressed: true,
                ..
            } => *pressed_button != button,
            _ => true,
        });
    });
}

#[cfg(test)]
mod tests;
