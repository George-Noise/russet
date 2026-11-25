//! The Settings window (owner request 2026-11-23, SPEC §3.14): opened
//! from the top menu bar's Settings menu ("Settings…" — the one item), it
//! lists the app's settings WITH their descriptions. It currently holds
//! exactly ONE — the Mouse4/Mouse5 Undo/Redo choice as a three-option
//! dropdown (ask / use / don't use; the owner: "decide yourself what exactly this
//! dropdown says — what matters to me is that the meaning is preserved").
//!
//! The window is built through the shared [`dialog_window`] /
//! [`dialog_button_row`] helpers (centered on the main window at open,
//! non-collapsible, non-resizable — the About-window pattern), so a fresh
//! open is always centered; the ✕ and the Close button both close it.
//! The dropdown edits the mode through [`App::set_mouse_side_mode`]
//! (state machine + Tool Log in [`super::mouse_side`], persistence in
//! [`App::save`]).

use eframe::egui::{self, Context};

use super::App;
use super::dialogs::{dialog_button_row, dialog_id, dialog_window, forget_dialog_position};
use super::mouse_side::MouseSideMode;

impl App {
    pub(super) fn settings_window(&mut self, ctx: &Context) {
        if !self.settings_window_open {
            return;
        }
        let mut open = true;
        let mut close = false;
        let id = dialog_id("settings");
        dialog_window("Settings", id, ctx)
            .open(&mut open)
            .show(ctx, |ui| {
                // ONE setting for now (owner request 2026-11-23): the
                // Mouse4/Mouse5 choice. The name is a strong label; the
                // dropdown is the Save dialog's ComboBox pattern
                // (app/save_dialog.rs).
                ui.label(egui::RichText::new("Mouse side buttons (Undo / Redo)").strong());
                let mut selected = self.mouse_side.mode();
                egui::ComboBox::from_id_salt("settings_mouse_side")
                    .selected_text(selected.label())
                    .width(160.0)
                    .show_ui(ui, |ui| {
                        for mode in MouseSideMode::ALL {
                            if ui
                                .selectable_value(&mut selected, mode, mode.label())
                                .clicked()
                            {
                                self.set_mouse_side_mode(mode);
                            }
                        }
                    });
                // The description explains the CURRENT choice live — the
                // owner asked for a window of settings WITH descriptions.
                ui.weak(self.mouse_side.mode().description());
                ui.weak("The choice is saved between sessions.");
                ui.add_space(6.0);
                let (close_hit, _) = dialog_button_row(ui, "Close", "");
                close = close_hit;
            });
        if close || !open {
            self.settings_window_open = false;
            forget_dialog_position(ctx, id);
        }
    }
}

#[cfg(test)]
mod tests;
