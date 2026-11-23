//! The Mouse4/Mouse5 Undo/Redo setting (owner request 2026-11-23, SPEC
//! §3.9 + §3.14): the mouse's side buttons — winit `Back`/`Forward`, egui
//! `PointerButton::Extra1`/`Extra2` — bind to Undo/Redo per the user's
//! choice: `Ask` (the default — the first side press of a session opens
//! the opt-in modal), `Always` (bound, no modal) or `Never` (inert, no
//! modal). Not everyone likes such a binding, so it is OFF until the user
//! settles the question one way or another.
//!
//! Mapping (the browser back/forward convention the owner picked — "сделай
//! как у других"): Mouse4 (back) = Undo, Mouse5 (forward) = Redo — one
//! node up/down the active layer's history tree, exactly like the
//! Ctrl+Z / Ctrl+Y hotkeys (SPEC §3.9).
//!
//! The choice is the app's ONE persisted setting (owner request
//! 2026-11-23: "только Mouse4/Mouse5 выбор сохранялся"; SPEC §3.14): the
//! mode is written through [`App::save`] into eframe's storage as the
//! plain string `"always"`/`"never"` under [`STORAGE_KEY`] — `Ask` REMOVES
//! the key instead, so a user who never decided (or reset to Ask) keeps
//! the storage byte-empty and no `app.ron` is written at all. eframe does
//! not re-export serde, so no `set_value`/RON of our own: a plain string
//! under one key is the whole format (missing or unrecognized values load
//! as `Ask`). eframe calls `save` on its 30-s autosave and at exit — the
//! choice lands within that window. The OTHER channels stay off
//! (`persist_window: false` in main.rs, `persist_egui_memory() -> false`
//! in app.rs).
//!
//! The pure state machine is [`MouseSideButtons`]; the App-side press
//! handler consumes the raw press events (the buttons belong to this
//! feature — a `Never` session, a pending ask and a mid-gesture press are
//! all equally inert) and guards the canvas gestures: an undo or redo
//! while a gesture is in flight would corrupt the pending history node
//! the gesture records on release. The Settings window (§3.14,
//! `app/settings_window.rs`) edits the mode through
//! [`App::set_mouse_side_mode`].

use eframe::egui::{self, Context, PointerButton};

use super::App;
use super::dialogs::{dialog_button_row, dialog_id, dialog_window, forget_dialog_position};

/// The storage key of the persisted mode — the app's ONE saved setting
/// (SPEC §3.14). A single flat key under eframe's `app.ron` (native) or
/// one localStorage entry (web).
const STORAGE_KEY: &str = "settings/mouse_side_buttons";

/// The persisted Mouse4/Mouse5 choice (owner request 2026-11-23, SPEC
/// §3.14): ask on first use, always use, never use. The order in [`ALL`]
/// is the dropdown order (Ask first — the default and the gentlest).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub(super) enum MouseSideMode {
    /// Each session starts undecided: the first side press opens the ask
    /// modal, and answering it settles the choice (Yes → Always, No →
    /// Never). The STARTUP default.
    #[default]
    Ask,
    /// Mouse4 = Undo, Mouse5 = Redo, no modal ever.
    Always,
    /// The side buttons are inert, no modal ever.
    Never,
}

impl MouseSideMode {
    /// Every mode in dropdown order.
    pub(super) const ALL: [Self; 3] = [Self::Ask, Self::Always, Self::Never];

    /// The Settings dropdown label (owner request 2026-11-23: three
    /// variants "типо спросить, использовать или не использовать" — the
    /// wording is ours, the meaning is his).
    pub(super) fn label(self) -> &'static str {
        match self {
            MouseSideMode::Ask => "Ask every session",
            MouseSideMode::Always => "Use for Undo / Redo",
            MouseSideMode::Never => "Don't use",
        }
    }

    /// The per-choice description shown under the Settings dropdown —
    /// the window explains the CURRENT choice live.
    pub(super) fn description(self) -> &'static str {
        match self {
            MouseSideMode::Ask => {
                "Each session starts undecided: the first Mouse4 or Mouse5 \
                 press opens a dialog asking whether to bind them."
            }
            MouseSideMode::Always => {
                "Mouse4 (back) undoes, Mouse5 (forward) redoes - like a \
                 browser's back/forward. No dialog is shown."
            }
            MouseSideMode::Never => {
                "The side buttons do nothing, and they are never asked \
                 about."
            }
        }
    }

    /// The plain-string encoding stored under [`STORAGE_KEY`]. `Ask` is
    /// never WRITTEN (the key is removed instead), but it still decodes
    /// from this string so a hand-written value round-trips.
    fn storage_str(self) -> &'static str {
        match self {
            MouseSideMode::Ask => "ask",
            MouseSideMode::Always => "always",
            MouseSideMode::Never => "never",
        }
    }

    /// The inverse of [`storage_str`]: anything unknown is `None`, which
    /// the load path treats as `Ask` (the default — a corrupt value must
    /// never disable or enable the buttons on its own).
    fn from_storage_str(value: &str) -> Option<Self> {
        MouseSideMode::ALL
            .iter()
            .copied()
            .find(|mode| mode.storage_str() == value)
    }
}

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

/// What a side-button press resolves to right now.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(super) enum PressOutcome {
    /// The session is undecided (`Ask` mode, no dialog yet): open the ask
    /// modal instead of acting — the press that opens the modal never
    /// undoes/redoes itself.
    Ask,
    Undo,
    Redo,
    /// A `Never` session or an open modal: the press is swallowed.
    Inert,
}

/// The Mouse4/Mouse5 state: the persisted [`MouseSideMode`] plus the ask
/// modal's open flag (the modal only ever exists in `Ask` mode; while it
/// is open, every side press is inert — the dialog owns the screen).
#[derive(Default)]
pub(super) struct MouseSideButtons {
    mode: MouseSideMode,
    dialog_open: bool,
}

impl MouseSideButtons {
    /// The current persisted mode (read by the Settings window's dropdown).
    pub(super) fn mode(&self) -> MouseSideMode {
        self.mode
    }

    /// The pure press resolver: what a Mouse4/Mouse5 press does given the
    /// current mode. Shared by the press handler and the tests.
    fn press_outcome(&self, button: SideButton) -> PressOutcome {
        if self.dialog_open {
            return PressOutcome::Inert;
        }
        match self.mode {
            MouseSideMode::Ask => PressOutcome::Ask,
            MouseSideMode::Always => match button {
                SideButton::Back => PressOutcome::Undo,
                SideButton::Forward => PressOutcome::Redo,
            },
            MouseSideMode::Never => PressOutcome::Inert,
        }
    }

    fn dialog_is_open(&self) -> bool {
        self.dialog_open
    }

    fn open_dialog(&mut self) {
        self.dialog_open = true;
    }

    /// The modal was dismissed with its ✕ button: NO decision — the mode
    /// stays `Ask`, so the next side press asks again. Only the two
    /// explicit buttons settle the question (the Settings window can
    /// change it deliberately).
    fn dismissed(&mut self) {
        self.dialog_open = false;
    }

    /// The Settings dropdown picked a mode (SPEC §3.14): the state moves
    /// immediately, and leaving `Ask` closes a possibly-open ask dialog —
    /// its question is settled by the pick. Returns whether anything
    /// changed (the caller logs only real changes).
    fn set_mode(&mut self, mode: MouseSideMode) -> bool {
        if self.mode == mode {
            return false;
        }
        self.mode = mode;
        if mode != MouseSideMode::Ask {
            self.dialog_open = false;
        }
        true
    }

    /// Writes the mode to eframe's storage (called from [`App::save`] —
    /// the app's ONE persisted setting, SPEC §3.14): `always`/`never` as
    /// the key's plain string; `Ask` REMOVES the key instead — and only
    /// when the key is actually present, because eframe's `remove_string`
    /// marks the file storage dirty unconditionally (a repeated remove of
    /// an absent key would rewrite `app.ron` every autosave — the empty
    /// default must stay a zero-write state, SPEC §1).
    pub(super) fn persist(&self, storage: &mut dyn eframe::Storage) {
        match self.mode {
            MouseSideMode::Ask => {
                if storage.get_string(STORAGE_KEY).is_some() {
                    storage.remove_string(STORAGE_KEY);
                }
            }
            mode => storage.set_string(STORAGE_KEY, mode.storage_str().to_owned()),
        }
    }

    /// Reads the mode back (the startup load in `App::new_with`): a
    /// missing or unrecognized string means `Ask` — the fresh-session
    /// default (SPEC §3.14).
    pub(super) fn load(&mut self, storage: &dyn eframe::Storage) {
        if let Some(saved) = storage.get_string(STORAGE_KEY)
            && let Some(mode) = MouseSideMode::from_storage_str(&saved)
        {
            self.mode = mode;
        }
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
        // The presses are consumed UNCONDITIONALLY (a `Never` session, a
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

    /// The app-level mode setter (SPEC §3.14): the Settings window's
    /// dropdown runs through it, and so does the ask modal's answer — one
    /// state move + one Tool Log line per settled choice. The log strings
    /// are the modal's originals ("Undo/Redo on" / "off"); the `Ask` pick
    /// reads like the modal's absence — the question is back.
    pub(super) fn set_mouse_side_mode(&mut self, mode: MouseSideMode) {
        if self.mouse_side.set_mode(mode) {
            self.action_log.push(match mode {
                MouseSideMode::Ask => "Mouse side buttons: ask every session",
                MouseSideMode::Always => "Mouse side buttons: Undo/Redo on",
                MouseSideMode::Never => "Mouse side buttons: off",
            });
        }
    }

    /// The first-press modal (owner request 2026-11-23): "do you want the
    /// mouse side buttons to undo/redo?" — built through the shared
    /// [`dialog_window`] / [`dialog_button_row`] helpers like every other
    /// dialog. YES settles the choice as `Always`, NO as `Never` — both
    /// PERSIST (SPEC §3.14) — and the ✕ button decides NOTHING (the
    /// dialog closes unanswered and the next press asks again).
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
                ui.weak("The choice is saved between sessions.");
                let (yes_hit, no_hit) = dialog_button_row(ui, "Yes", "No");
                yes = yes_hit;
                no = no_hit;
            });
        if yes {
            self.set_mouse_side_mode(MouseSideMode::Always);
            forget_dialog_position(ctx, id);
        } else if no {
            self.set_mouse_side_mode(MouseSideMode::Never);
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
