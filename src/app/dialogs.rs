//! Modal dialogs (SPEC §3.7 / §3.8): "New", "Resize Canvas", the
//! unsaved-changes confirmation and the "Error" popup. All of them are
//! built through [`dialog_window`] (centered on the main window, owner
//! request 2026-11-15) and all their affirmative/dismissive button rows go
//! through [`dialog_button_row`] (owner request 2026-11-13: the IBM Carbon
//! / Apple HIG / Google Material order — right-aligned, the primary button
//! rightmost; the Text tool window and the Delete-layer confirmation use
//! the same helpers). A child module of `app`, so it can touch `App`'s
//! private fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Align2, AreaState, Context, Id, LayerId, Order, Vec2};

use super::App;
use super::document::{MAX_CANVAS, MIN_CANVAS, PendingAction};
use crate::history_tree::{ResizeAnchor, ResizeFilter, ResizeMode};

/// The GitHub repository the About window links (owner request 2026-11-18):
/// the `ui.hyperlink_to` target — opens the system browser on native builds
/// (eframe's `links` feature) and a new tab on web.
pub(super) const ABOUT_REPO_URL: &str = "https://github.com/George-Noise/russet";

/// The About window hyperlink's visible label (owner request 2026-11-18).
pub(super) const ABOUT_REPO_LABEL: &str = "GitHub: George-Noise/russet";

/// The About window's one-line description (owner request 2026-11-18: the
/// window must say the project is written in Rust — SPEC §3.1 / §7.20).
pub(super) const ABOUT_DESCRIPTION: &str =
    "A tiny bitmap paint program written in Rust with egui/eframe.";

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

    /// "Resize Canvas…" (the Canvas menu): prefill the fields from the
    /// current canvas and reset every choice to its default (Canvas mode,
    /// center anchor, bicubic filter, aspect lock on).
    pub(super) fn open_resize_dialog(&mut self) {
        self.resize_width = self.canvas.width as f32;
        self.resize_height = self.canvas.height as f32;
        self.resize_mode = ResizeMode::Canvas;
        self.resize_anchor = ResizeAnchor::Center;
        self.resize_filter = ResizeFilter::Bicubic;
        self.resize_aspect = true;
        self.show_resize_dialog = true;
    }

    pub(super) fn resize_dialog(&mut self, ctx: &Context) {
        if !self.show_resize_dialog {
            return;
        }
        let mut open = true;
        let mut ok = false;
        let mut cancel = false;
        let id = dialog_id("resize");
        // Aspect-lock anchor: the canvas ratio at dialog open (the fields
        // are prefilled from the canvas, so the lock keeps THAT ratio).
        let canvas_ratio = self.canvas.height as f32 / self.canvas.width as f32;
        let (w0, h0) = (self.resize_width, self.resize_height);
        dialog_window("Resize Canvas", id, ctx)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Mode");
                    ui.radio_value(&mut self.resize_mode, ResizeMode::Canvas, "Canvas size");
                    ui.radio_value(&mut self.resize_mode, ResizeMode::Image, "Image size");
                });
                ui.horizontal(|ui| {
                    ui.label("Width");
                    ui.add(
                        egui::DragValue::new(&mut self.resize_width)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                    ui.label("Height");
                    ui.add(
                        egui::DragValue::new(&mut self.resize_height)
                            .range(MIN_CANVAS as f32..=MAX_CANVAS as f32)
                            .speed(1.0)
                            .max_decimals(0),
                    );
                });
                if self.resize_mode == ResizeMode::Canvas {
                    // The Paint.NET 3×3 anchor: which part of the picture
                    // stays in place while the rest crops/pads.
                    ui.label("Anchor — the part that stays in place");
                    anchor_grid(ui, &mut self.resize_anchor);
                } else {
                    ui.checkbox(&mut self.resize_aspect, "Maintain aspect ratio");
                    ui.label("Resampling");
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut self.resize_filter,
                            ResizeFilter::Nearest,
                            "Nearest neighbor",
                        );
                        ui.radio_value(&mut self.resize_filter, ResizeFilter::Bilinear, "Bilinear");
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut self.resize_filter, ResizeFilter::Bicubic, "Bicubic");
                        ui.radio_value(&mut self.resize_filter, ResizeFilter::Lanczos3, "Lanczos3");
                    });
                }
                let (ok_hit, cancel_hit) = dialog_button_row(ui, "OK", "Cancel");
                ok = ok_hit;
                cancel = cancel_hit;
            });
        // Aspect lock (Image mode only): whichever field the user CHANGED
        // this frame drives the other one along the canvas ratio.
        if self.resize_mode == ResizeMode::Image && self.resize_aspect && !ok && !cancel {
            let (w, h) = aspect_linked(w0, h0, self.resize_width, self.resize_height, canvas_ratio);
            self.resize_width = w;
            self.resize_height = h;
        }
        if ok {
            self.apply_resize(
                self.resize_mode,
                self.resize_width.round() as u32,
                self.resize_height.round() as u32,
                self.resize_anchor,
                self.resize_filter,
            );
            self.show_resize_dialog = false;
            forget_dialog_position(ctx, id);
        } else if cancel || !open {
            self.show_resize_dialog = false;
            forget_dialog_position(ctx, id);
        }
    }

    /// The base-choice dialog after deleting the BOTTOM layer whose
    /// successor has a DIFFERENT size (owner request 2026-11-17): keep the
    /// canvas size (the new bottom layer crop/pads to it, top-left — the
    /// composite never shifts) or use the layer's size as the new canvas.
    /// No dismiss button: the deletion already happened, the size model
    /// needs a decision (same-size successors never open this dialog).
    pub(super) fn layer_base_dialog(&mut self, ctx: &Context) {
        if !self.layer_base_choice {
            return;
        }
        let (name, layer_w, layer_h) = self
            .layers
            .layer(0)
            .map(|l| {
                (
                    l.name().to_string(),
                    l.pixmap().width(),
                    l.pixmap().height(),
                )
            })
            .unwrap_or_else(|| ("?".to_string(), 0, 0));
        let (canvas_w, canvas_h) = (self.canvas.width, self.canvas.height);
        let mut keep = false;
        let mut use_layer = false;
        let id = dialog_id("layer_base");
        dialog_window("Canvas Size Source", id, ctx).show(ctx, |ui| {
            ui.label(format!(
                "Layer \"{name}\" is {layer_w}×{layer_h}, the canvas is {canvas_w}×{canvas_h}."
            ));
            ui.weak("Keep the canvas size, or make this layer's size the canvas size?");
            let (primary_hit, secondary_hit) =
                dialog_button_row(ui, "Keep canvas size", "Use layer as base");
            keep = primary_hit;
            use_layer = secondary_hit;
        });
        if keep || use_layer {
            self.layer_base_choice = false;
            forget_dialog_position(ctx, id);
            if keep {
                self.layers.fit_bottom_to(canvas_w, canvas_h);
                self.refresh_composite();
                self.action_log.push(format!(
                    "Canvas kept at {canvas_w}×{canvas_h} (layer adapted)"
                ));
            } else {
                self.sync_canvas_to_bottom();
                self.refresh_composite();
                self.action_log.push(format!(
                    "Canvas sized to the base layer {layer_w}×{layer_h}"
                ));
            }
            self.dirty = true;
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

    /// The "About Russet" window (owner request 2026-11-09). The CONTENT —
    /// name, version, the one-line description ("written in Rust", owner
    /// request 2026-11-18), the GitHub repository hyperlink (same request),
    /// the platform/renderer lines and the Close row — lives in the free
    /// [`about_window_content`], so the headless-egui test in `dialogs/tests.rs`
    /// renders the REAL body without an `App`. The platform/renderer strings
    /// come from `main.rs` (`crate::platform_label` / `crate::renderer_label`),
    /// the place where platform knowledge is allowed to live (AGENTS.md
    /// "Repo facts").
    pub(super) fn about_dialog(&mut self, ctx: &Context) {
        if !self.show_about {
            return;
        }
        let mut close = false;
        let id = dialog_id("about");
        dialog_window("About Russet", id, ctx).show(ctx, |ui| {
            close = about_window_content(ui);
        });
        if close {
            self.show_about = false;
            forget_dialog_position(ctx, id);
        }
    }
}

/// The About window's whole content, minus the window plumbing (owner
/// request 2026-11-09 for the window; 2026-11-18 for the language line and
/// the repository link). Free of `App` state on purpose: the only variable
/// inputs are the crate version (compile-time) and the platform/renderer
/// labels, so the headless-egui test can show this through the SAME
/// [`dialog_window`] builder the real dialog uses. Returns whether the
/// Close row was clicked.
fn about_window_content(ui: &mut egui::Ui) -> bool {
    ui.heading("Russet");
    ui.label(format!("Version {}", env!("CARGO_PKG_VERSION")));
    ui.add_space(4.0);
    ui.label(ABOUT_DESCRIPTION);
    ui.hyperlink_to(ABOUT_REPO_LABEL, ABOUT_REPO_URL);
    ui.add_space(4.0);
    ui.label(format!("Platform: {}", crate::platform_label()));
    ui.label(format!("Renderer: {}", crate::renderer_label()));
    ui.add_space(6.0);
    let (close, _) = dialog_button_row(ui, "Close", "");
    close
}

/// The Paint.NET-style 3×3 anchor grid of the Canvas-mode resize (owner
/// request 2026-11-17): nine clickable cells, one painted dot each — the
/// selected cell's dot is filled with the selection color plus a ring, the
/// others stay small text-colored dots. Painted (no font glyphs), like the
/// Layers window's eye and triangle buttons.
fn anchor_grid(ui: &mut egui::Ui, selected: &mut ResizeAnchor) {
    const CELL: f32 = 18.0;
    let rows: [[ResizeAnchor; 3]; 3] = [
        [
            ResizeAnchor::TopLeft,
            ResizeAnchor::TopCenter,
            ResizeAnchor::TopRight,
        ],
        [
            ResizeAnchor::MiddleLeft,
            ResizeAnchor::Center,
            ResizeAnchor::MiddleRight,
        ],
        [
            ResizeAnchor::BottomLeft,
            ResizeAnchor::BottomCenter,
            ResizeAnchor::BottomRight,
        ],
    ];
    ui.scope(|ui| {
        ui.spacing_mut().item_spacing = egui::vec2(3.0, 3.0);
        for row in rows {
            ui.horizontal(|ui| {
                for anchor in row {
                    let (rect, response) =
                        ui.allocate_exact_size(Vec2::splat(CELL), egui::Sense::click());
                    let is_selected = *selected == anchor;
                    let cell_fill = if response.hovered() {
                        ui.visuals().widgets.hovered.bg_fill
                    } else {
                        ui.visuals().extreme_bg_color
                    };
                    ui.painter().rect_filled(rect, 3.0, cell_fill);
                    ui.painter().rect_stroke(
                        rect,
                        3.0,
                        egui::Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color),
                        egui::StrokeKind::Inside,
                    );
                    let center = rect.center();
                    if is_selected {
                        ui.painter()
                            .circle_filled(center, 5.0, ui.visuals().selection.bg_fill);
                        ui.painter().circle_stroke(
                            center,
                            5.0,
                            egui::Stroke::new(1.5, ui.visuals().text_color()),
                        );
                    } else {
                        ui.painter()
                            .circle_filled(center, 2.5, ui.visuals().text_color());
                    }
                    if response.clicked() {
                        *selected = anchor;
                    }
                    if response.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                    }
                    response.on_hover_text(anchor.label());
                }
            });
        }
    });
}

/// The Image mode's aspect-ratio lock: whichever field the user changed
/// this frame drives the other one along the canvas ratio (`ratio` =
/// canvas height / width); a field the user did not touch follows, a
/// simultaneous edit of both wins as-is. Rounded to whole points, min 1.
fn aspect_linked(w0: f32, h0: f32, w1: f32, h1: f32, ratio: f32) -> (f32, f32) {
    let width_changed = (w1 - w0).abs() > f32::EPSILON;
    let height_changed = (h1 - h0).abs() > f32::EPSILON;
    match (width_changed, height_changed) {
        (true, false) => (w1, (w1 * ratio).round().max(1.0)),
        (false, true) => ((h1 / ratio).round().max(1.0), h1),
        _ => (w1, h1),
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
