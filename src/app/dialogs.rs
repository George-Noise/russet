//! Modal dialogs (SPEC §3.7 / §3.8): "New", "Resize Canvas", the
//! unsaved-changes confirmation and the "Error" popup. A child module of
//! `app`, so it can touch `App`'s private fields; `app.rs` stays the
//! orchestrator.

use eframe::egui::{self, Context};

use super::App;
use super::document::{MAX_CANVAS, MIN_CANVAS, PendingAction};

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
            self.canvas.resize_canvas(width, height);
            // Owner decision 2026-11-13 ("keep the nodes"): the layer
            // buffers crop/pad to the new size while every node PNG keeps
            // its original size — switching to an old node shows the fitted
            // (cropped/padded) image, and drawing continues from that
            // fitted state, so the NEXT node is stored at the new size.
            self.layers.resize_all(width, height);
            // Pending gestures reference canvas coordinates that may no
            // longer be meaningful — drop them.
            self.cancel_preview();
            self.cancel_text_edit();
            self.refresh_composite();
            self.dirty = true;
            self.action_log
                .push(format!("Canvas resized to {width}×{height}"));
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

    /// The "About Russet" window (owner request 2026-11-09): name, version
    /// and the platform the app is running on — Wayland, X11, Windows, Web
    /// (WASM), etc. The platform/renderer strings come from `main.rs`
    /// (`crate::platform_label` / `crate::renderer_label`), the place where
    /// platform knowledge is allowed to live (AGENTS.md "Repo facts").
    pub(super) fn about_dialog(&mut self, ctx: &Context) {
        if !self.show_about {
            return;
        }
        let mut close = false;
        egui::Window::new("About Russet")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.heading("Russet");
                ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
                ui.add_space(4.0);
                ui.label("A tiny bitmap paint program built with egui/eframe.");
                ui.add_space(4.0);
                ui.label(format!("Platform: {}", crate::platform_label()));
                ui.label(format!("Renderer: {}", crate::renderer_label()));
                ui.add_space(6.0);
                if ui.button("Close").clicked() {
                    close = true;
                }
            });
        if close {
            self.show_about = false;
        }
    }
}
