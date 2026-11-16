//! Modal dialogs (SPEC §3.7 / §3.8): "New", "Resize Canvas", the
//! unsaved-changes confirmation and the "Error" popup. All of them are
//! built through [`dialog_window`] (centered on the main window, owner
//! request 2026-11-15) and all their affirmative/dismissive button rows go
//! through [`dialog_button_row`] (owner request 2026-11-13: the IBM Carbon
//! / Apple HIG / Google Material order — right-aligned, the primary button
//! rightmost; the Text tool window and the Delete-layer confirmation use
//! the same helpers). A child module of `app`, so it can touch `App`'s
//! private fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Align2, AreaState, Context, Id, LayerId, Order};

use super::App;
use super::document::{MAX_CANVAS, MIN_CANVAS, PendingAction};

/// Stable area id of a modal dialog (needed to reset its stored position
/// on close — see [`forget_dialog_position`]).
pub(super) fn dialog_id(name: &str) -> Id {
    // `Id::new` is deprecated in egui 0.37; `Id::unique` is its
    // deterministic replacement (same hash every frame — stable ids are
    // what lets [`forget_dialog_position`] address the area).
    Id::unique(format!("russet_dialog_{name}"))
}

/// The shared builder of every modal dialog (SPEC §3.7–§3.8): not
/// collapsible, not resizable, and positioned so that it ARISES at the
/// main window's center (owner request 2026-11-15: dialogs
/// «возникают по центру главного окна», not at egui's cascade position
/// slightly right/below of the floating windows). The CENTER_CENTER pivot
/// makes `default_pos` mean "put the dialog's center here", so the center
/// lands exactly on `content_rect().center()` even before the window size
/// is known. While open the dialog stays draggable like any other window;
/// [`forget_dialog_position`] on every close path re-centers the NEXT open.
pub(super) fn dialog_window<'a>(title: &'a str, id: Id, ctx: &'a Context) -> egui::Window<'a> {
    egui::Window::new(title)
        .id(id)
        .collapsible(false)
        .resizable(false)
        .pivot(Align2::CENTER_CENTER)
        .default_pos(ctx.content_rect().center())
}

/// Wipes a dialog's stored area position. Called on EVERY close path of a
/// modal dialog, so the next open consumes `default_pos` again and is
/// centered (egui keeps an `AreaState` for an area that was shown before;
/// `default_pos` is only ever consumed while `pivot_pos` is None — the
/// fresh-state condition). Runs AFTER `Window::show` on the closing frame:
/// the next frame the dialog is skipped entirely, so nothing re-stores a
/// position. A fresh [`AreaState`] is exactly what `Area::begin` sees
/// before the first show ever (same field pattern as egui's own fallback).
pub(super) fn forget_dialog_position(ctx: &Context, id: Id) {
    ctx.memory_mut(|mem| {
        mem.areas_mut().set_state(
            LayerId::new(Order::Middle, id),
            AreaState {
                pivot_pos: None,
                pivot: Align2::CENTER_CENTER,
                size: None,
                interactable: true,
                last_became_visible_at: None,
            },
        );
    });
}

impl App {
    pub(super) fn new_dialog(&mut self, ctx: &Context) {
        if !self.show_new_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        let id = dialog_id("new");
        dialog_window("New", id, ctx)
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
                let (ok_hit, cancel_hit) = dialog_button_row(ui, "OK", "Cancel");
                ok = ok_hit;
                cancel = cancel_hit;
            });
        if ok {
            let width = self.new_width.round() as u32;
            let height = self.new_height.round() as u32;
            self.reset_canvas(width, height);
            self.show_new_dialog = false;
            forget_dialog_position(ctx, id);
        } else if cancel || !open {
            self.show_new_dialog = false;
            forget_dialog_position(ctx, id);
        }
    }

    pub(super) fn resize_dialog(&mut self, ctx: &Context) {
        if !self.show_resize_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        let id = dialog_id("resize");
        dialog_window("Resize Canvas", id, ctx)
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
                let (ok_hit, cancel_hit) = dialog_button_row(ui, "OK", "Cancel");
                ok = ok_hit;
                cancel = cancel_hit;
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
            forget_dialog_position(ctx, id);
        } else if cancel || !open {
            self.show_resize_dialog = false;
            forget_dialog_position(ctx, id);
        }
    }

    pub(super) fn discard_dialog(&mut self, ctx: &Context) {
        let Some(action) = self.pending_action else {
            return;
        };
        let mut confirmed = false;
        let mut cancel = false;
        let id = dialog_id("unsaved");
        dialog_window("Unsaved Changes", id, ctx).show(ctx, |ui| {
            ui.label("You have unsaved changes. Discard them and continue?");
            let (ok_hit, cancel_hit) = dialog_button_row(ui, "OK", "Cancel");
            confirmed = ok_hit;
            cancel = cancel_hit;
        });
        if confirmed {
            self.pending_action = None;
            forget_dialog_position(ctx, id);
            match action {
                PendingAction::New => self.open_new_dialog(),
                PendingAction::Open => self.open_now(ctx),
            }
        } else if cancel {
            self.pending_action = None;
            forget_dialog_position(ctx, id);
        }
    }

    pub(super) fn error_dialog(&mut self, ctx: &Context) {
        let Some(message) = self.error_popup.clone() else {
            return;
        };
        let mut close = false;
        let id = dialog_id("error");
        dialog_window("Error", id, ctx).show(ctx, |ui| {
            ui.label(message);
            let (close_hit, _) = dialog_button_row(ui, "OK", "");
            close = close_hit;
        });
        if close {
            self.error_popup = None;
            forget_dialog_position(ctx, id);
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
        let id = dialog_id("about");
        dialog_window("About Russet", id, ctx).show(ctx, |ui| {
            ui.heading("Russet");
            ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
            ui.add_space(4.0);
            ui.label("A tiny bitmap paint program built with egui/eframe.");
            ui.add_space(4.0);
            ui.label(format!("Platform: {}", crate::platform_label()));
            ui.label(format!("Renderer: {}", crate::renderer_label()));
            ui.add_space(6.0);
            let (close_hit, _) = dialog_button_row(ui, "Close", "");
            close = close_hit;
        });
        if close {
            self.show_about = false;
            forget_dialog_position(ctx, id);
        }
    }
}

/// The affirmative/dismissive button row of every dialog and of the Text
/// tool window, laid out per the IBM Carbon / Apple HIG / Google Material
/// convention (owner request 2026-11-13: "чтобы как у всех"): the row is
/// right-aligned and the AFFIRMATIVE (primary) button is the RIGHTMOST one,
/// with the dismissive button to its left. Microsoft Windows's opposite
/// order (primary leftmost) is deliberately not followed. Returns
/// `(primary_clicked, secondary_clicked)`; pass an empty `secondary` label
/// for a single-button row.
///
/// The row is laid out in a BOUNDED scope, one interact-size tall across
/// the available width. A bare [`egui::Ui::with_layout`] inherits the WHOLE
/// remaining available rect — inside a Window that is taller than its
/// content (egui's default outer window size is 340×420) the row is
/// anchored into that empty space, the content's measured size never
/// shrinks back and the dialog stays tall with the buttons detached below
/// the fields (owner report 2026-11-15: dialogs "растягиваются до конца
/// вниз"). Reading `available_width` is safe here: every caller is a
/// window with egui's stored default width — never an `.auto_sized()`
/// window, whose first-pass available width is the whole screen (the
/// quirk recorded in AGENTS.md).
pub(super) fn dialog_button_row(ui: &mut egui::Ui, primary: &str, secondary: &str) -> (bool, bool) {
    let mut primary_clicked = false;
    let mut secondary_clicked = false;
    // `right_to_left`: the FIRST widget added lands rightmost, so the
    // primary button is added first. The height may overflow onto the
    // buttons' real height — overflow only grows the scope, never clips.
    ui.allocate_ui_with_layout(
        egui::vec2(ui.available_width(), ui.spacing().interact_size.y),
        egui::Layout::right_to_left(egui::Align::Center),
        |ui| {
            if ui.button(primary).clicked() {
                primary_clicked = true;
            }
            if !secondary.is_empty() && ui.button(secondary).clicked() {
                secondary_clicked = true;
            }
        },
    );
    (primary_clicked, secondary_clicked)
}

#[cfg(test)]
mod tests;
