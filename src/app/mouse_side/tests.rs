//! The Mouse4/Mouse5 tests, split by topic (the ~300-line test-file guard,
//! the `app/windows/tests.rs` pattern): `machine` — the pure state machine
//! and the Settings setter; `persistence` — the ONE saved setting's
//! storage behavior; `app_routing` — the real press handler over the real
//! history. The shared helpers live here.

use super::*;

use crate::app::App;
use crate::history_tree::ActionKind;
use eframe::egui::{Modifiers, Pos2};

mod app_routing;
mod machine;
mod persistence;

/// A raw press event of a side button, as winit delivers it
/// (Back → `Extra1`, Forward → `Extra2`).
pub(super) fn side_press(button: PointerButton) -> egui::Event {
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
pub(super) fn input_with_presses(buttons: &[PointerButton]) -> egui::RawInput {
    let mut input = egui::RawInput::default();
    input.events.extend(buttons.iter().map(|&b| side_press(b)));
    input
}

/// A headless app with ONE recorded stroke node: `can_undo` is true,
/// `can_redo` is false. A successful undo leaves a redo available.
pub(super) fn app_with_one_node() -> App {
    let mut app = App::new_headless();
    app.record_active_action(ActionKind::Pencil, eframe::egui::Color32::BLACK, None);
    app
}

/// A minimal test double of `eframe::Storage` that separates the content
/// PRE-SEEDED before the call (what a previous session wrote) from the
/// writes the call makes (a private twin of the one in `app/tests.rs` —
/// the doubles stay local to their test module).
#[derive(Default)]
pub(super) struct MemoryStorage {
    pub(super) seed: Vec<(String, String)>,
    pub(super) sets: Vec<(String, String)>,
    pub(super) removes: Vec<String>,
}

impl eframe::Storage for MemoryStorage {
    fn get_string(&self, key: &str) -> Option<String> {
        self.sets
            .iter()
            .rev()
            .chain(self.seed.iter().rev())
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.clone())
    }

    fn set_string(&mut self, key: &str, value: String) {
        self.sets.push((key.to_owned(), value));
    }

    fn remove_string(&mut self, key: &str) {
        self.removes.push(key.to_owned());
    }

    fn flush(&mut self) {}
}

/// An ACCEPTED question (the modal's Yes / the Settings pick): the state
/// machine move + the dialog close in one call, exactly as the modal path
/// runs (`set_mode` from `Ask` always changes).
pub(super) fn accept(state: &mut MouseSideButtons) {
    state.open_dialog();
    state.set_mode(MouseSideMode::Always);
}

/// A DECLINED question (the modal's No / the Settings pick).
pub(super) fn decline(state: &mut MouseSideButtons) {
    state.open_dialog();
    state.set_mode(MouseSideMode::Never);
}
