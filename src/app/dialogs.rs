//! Modal dialogs (SPEC §3.7 / §3.8): "New", "Resize Canvas", the
//! unsaved-changes confirmation and the "Error" popup. A child module of
//! `app`, so it can touch `App`'s private fields; `app.rs` stays the
//! orchestrator.

use eframe::egui::{self, Context};

use super::{App, MAX_CANVAS, MIN_CANVAS, PendingAction};

impl App {
    pub(super) fn new_dialog(&mut self, ctx: &Context) {
        if !self.show_new_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        egui::Window::new("New")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut self.new_width)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut self.new_height)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        ok = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if ok {
            let width = self.new_width.round() as u32;
            let height = self.new_height.round() as u32;
            self.reset_canvas(width, height);
            self.show_new_dialog = false;
        } else if cancel || !open {
            self.show_new_dialog = false;
        }
    }

    pub(super) fn resize_dialog(&mut self, ctx: &Context) {
        if !self.show_resize_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        egui::Window::new("Resize Canvas")
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut self.resize_width)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut self.resize_height)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        ok = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if ok {
            let width = self.resize_width.round() as u32;
            let height = self.resize_height.round() as u32;
            self.history.push_snapshot(self.canvas.pixmap().clone());
            self.canvas.resize_canvas(width, height);
            self.repaint_full();
            self.dirty = true;
            self.show_resize_dialog = false;
        } else if cancel || !open {
            self.show_resize_dialog = false;
        }
    }

    pub(super) fn discard_dialog(&mut self, ctx: &Context) {
        let Some(action) = self.pending_action else {
            return;
        };
        let mut confirmed = false;
        let mut cancel = false;
        egui::Window::new("Unsaved Changes")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("You have unsaved changes. Discard them and continue?");
                ui.horizontal(|ui| {
                    if ui.button("OK").clicked() {
                        confirmed = true;
                    }
                    if ui.button("Cancel").clicked() {
                        cancel = true;
                    }
                });
            });
        if confirmed {
            self.pending_action = None;
            match action {
                PendingAction::New => self.open_new_dialog(),
                PendingAction::Open => self.open_now(ctx),
            }
        } else if cancel {
            self.pending_action = None;
        }
    }

    pub(super) fn error_dialog(&mut self, ctx: &Context) {
        let Some(message) = self.error_popup.clone() else {
            return;
        };
        let mut close = false;
        egui::Window::new("Error")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(message);
                if ui.button("OK").clicked() {
                    close = true;
                }
            });
        if close {
            self.error_popup = None;
        }
    }
}
