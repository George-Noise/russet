//! The "History" and "Layers" windows. History is the read-only action
//! log; Layers is the REAL layer system's control panel (owner request
//! 2026-11-13, paying functional debt F1): rows in display order (top
//! layer first), click to select the active layer — every History Tree
//! window follows — plus visibility, rename (double-click), reorder and
//! delete-with-confirmation. Both are plain floating windows that since
//! the owner request 2026-11-11 follow the same §3.11 placement as "Tools"
//! and "Palette" (drop-time edge anchoring + resize repositioning, via
//! [`place_floating_window`]); their FIRST-open positions stay the edge
//! anchors below. Child module of `app`, so it can touch `App`'s private
//! fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Color32, Context, Vec2};

use super::App;
use super::placement::{FIRST_OPEN_GAP, LAYERS_FIRST_OPEN_LIFT, place_floating_window};

/// Stable ids, same idea as `windows.rs`: independent of the title text so
/// placement / area state lookups stay stable.
fn history_window_id() -> egui::Id {
    egui::Id::unique("history_window")
}

fn layers_window_id() -> egui::Id {
    egui::Id::unique("layers_window")
}

/// What one frame of the Layers window asked for. The rows are rendered
/// from a CLONED snapshot of the layer list, so every mutation applies
/// AFTER the window closes its layout pass (no borrow fights inside the
/// egui closure).
#[derive(Default)]
struct LayerRequest {
    add: bool,
    select: Option<usize>,
    toggle: Option<usize>,
    up: Option<usize>,
    down: Option<usize>,
    delete: Option<usize>,
    rename_start: Option<usize>,
    rename_commit: Option<(usize, String)>,
    rename_cancel: bool,
}

impl App {
    /// Read-only action log (newest first). Entries are plain labels —
    /// clicking them does nothing yet by design.
    pub(super) fn history_window(&mut self, ctx: &Context) {
        if !self.history_window_open {
            return;
        }
        let mut open = true;
        let content = ctx.content_rect();
        // First open (owner request 2026-11-12): the OLD History window now
        // sits at the BOTTOM CENTER of the main window (the new "History
        // Tree" took its old right-top spot). `CENTER_BOTTOM` pivot +
        // `default_pos` anchor the bottom edge midpoint 0.5rem above the
        // bottom edge. Afterwards the same §3.11 magnetism and resize
        // repositioning as Tools/Palette (owner request 2026-11-11).
        let shown = egui::Window::new("History")
            .id(history_window_id())
            .open(&mut open)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .default_pos(egui::pos2(
                content.center().x,
                content.bottom() - FIRST_OPEN_GAP,
            ))
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.set_min_width(180.0);
                let entries = self.action_log.items();
                if entries.is_empty() {
                    ui.weak("No actions yet.");
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .show(ui, |ui| {
                            // Newest at the top, so the latest action is
                            // always visible without scrolling.
                            for entry in entries.iter().rev() {
                                ui.label(format!("{}. {}", entry.id, entry.label));
                            }
                        });
                }
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                history_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.history_placement,
            );
        }
        if !open {
            self.history_window_open = false;
        }
    }

    /// The Layers window (owner request 2026-11-13, real layers): one row
    /// per layer, TOP layer first. The row's click target is the layer
    /// NAME (a wide selectable label); the eye, arrows and "×" sit next to
    /// it and never change the active layer.
    pub(super) fn layers_window(&mut self, ctx: &Context) {
        if !self.layers_window_open {
            return;
        }
        let mut open = true;
        let content = ctx.content_rect();
        // First open: 0.5rem from the right edge; the bottom edge sits
        // 0.5rem + 1.75rem above the window's bottom edge so it never
        // covers the zoom % label painted in the canvas's bottom-right
        // corner (owner request 2026-11-10). Afterwards the same §3.11
        // magnetism and resize repositioning as Tools/Palette (owner
        // request 2026-11-11).
        let count = self.layers.count();
        let active = self.layers.active_layer();
        // Display snapshot (name, visible) — the closure only mutates the
        // rename buffer and collects the request.
        let info: Vec<(String, bool)> = (0..count)
            .filter_map(|i| {
                self.layers
                    .layer(i)
                    .map(|layer| (layer.name().to_string(), layer.visible()))
            })
            .collect();
        let mut request = LayerRequest::default();
        let shown = egui::Window::new("Layers")
            .id(layers_window_id())
            .open(&mut open)
            .pivot(egui::Align2::RIGHT_BOTTOM)
            .default_pos(egui::pos2(
                content.right() - FIRST_OPEN_GAP,
                content.bottom() - FIRST_OPEN_GAP - LAYERS_FIRST_OPEN_LIFT,
            ))
            .auto_sized()
            .show(ctx, |ui| {
                ui.set_min_width(190.0);
                if ui.button("+ Add layer").clicked() {
                    request.add = true;
                }
                ui.separator();
                if info.is_empty() {
                    ui.weak("No layers.");
                }
                let row_height = ui.style().spacing.interact_size.y.max(18.0);
                for i in (0..count).rev() {
                    let (name, visible) = &info[i];
                    ui.horizontal(|ui| {
                        self.layer_row(
                            ui,
                            i,
                            name,
                            *visible,
                            i == active,
                            count,
                            row_height,
                            &mut request,
                        );
                    });
                }
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                layers_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.layers_placement,
            );
        }
        if !open {
            self.layers_window_open = false;
        }
        self.apply_layer_request(request);
    }

    /// One Layers-window row: visibility eye, name (click = select,
    /// double-click = rename), reorder arrows, delete button.
    #[allow(clippy::too_many_arguments)]
    fn layer_row(
        &mut self,
        ui: &mut egui::Ui,
        index: usize,
        name: &str,
        visible: bool,
        is_active: bool,
        count: usize,
        row_height: f32,
        request: &mut LayerRequest,
    ) {
        let renaming = self.layer_rename.as_ref().is_some_and(|(i, _)| *i == index);

        // Visibility eye: a filled dot when visible, a ring when hidden —
        // painted, so no font glyph is needed.
        let (rect, eye) = ui.allocate_exact_size(Vec2::splat(16.0), egui::Sense::click());
        let painter = ui.painter();
        let center = rect.center();
        let eye_color = if visible {
            ui.visuals().text_color()
        } else {
            Color32::from_gray(120)
        };
        if visible {
            painter.circle_filled(center, 5.0, eye_color);
        } else {
            painter.circle_stroke(center, 5.0, egui::Stroke::new(1.5, eye_color));
        }
        if eye.clicked() {
            request.toggle = Some(index);
        }
        if eye.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        eye.on_hover_text(if visible { "Hide layer" } else { "Show layer" });

        // Name: click selects, double-click renames. While renaming, the
        // name becomes an inline TextEdit. The row label is painted by hand
        // (a wide click target with the selection highlight) — egui 0.37's
        // `SelectableLabel` widget is sized to its text, which would make
        // the row's empty half dead space.
        if renaming {
            if let Some((_, buffer)) = self.layer_rename.as_mut() {
                let response = ui.add(
                    egui::TextEdit::singleline(buffer)
                        .desired_width(120.0)
                        .font(egui::TextStyle::Body),
                );
                let enter = ui.input(|i| i.key_pressed(egui::Key::Enter));
                let escape = ui.input(|i| i.key_pressed(egui::Key::Escape));
                if (response.lost_focus() && !escape) || enter {
                    request.rename_commit = Some((index, buffer.trim().to_string()));
                }
                if escape {
                    request.rename_cancel = true;
                }
                if request.rename_commit.is_some() || request.rename_cancel {
                    self.layer_rename = None;
                }
            }
        } else {
            let width = (ui.available_width() - 58.0).max(40.0);
            let (rect, label) =
                ui.allocate_exact_size(Vec2::new(width, row_height), egui::Sense::click());
            if is_active {
                ui.painter()
                    .rect_filled(rect, 3.0, ui.style().visuals.selection.bg_fill);
            }
            ui.painter().text(
                egui::pos2(rect.left() + 6.0, rect.center().y),
                egui::Align2::LEFT_CENTER,
                name,
                egui::FontId::proportional(row_height - 2.0),
                ui.visuals().text_color(),
            );
            if label.clicked() {
                request.select = Some(index);
            }
            if label.double_clicked() {
                request.rename_start = Some(index);
            }
            if label.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
        }

        // Reorder: painted triangles (no glyph risk); "up" = toward the
        // top of the list = later in the composite.
        let can_up = index + 1 < count;
        let can_down = index > 0;
        if self.triangle_button(ui, true, can_up) {
            request.up = Some(index);
        }
        if self.triangle_button(ui, false, can_down) {
            request.down = Some(index);
        }

        // Delete (refused for the last layer, confirmed in a dialog).
        if ui
            .add_enabled(
                count > 1,
                egui::Button::new("×").small().sense(egui::Sense::click()),
            )
            .clicked()
        {
            request.delete = Some(index);
        }
    }

    /// A small painted-arrow button: `up` draws the upward triangle.
    /// Returns true when clicked (disabled arrows never do).
    fn triangle_button(&mut self, ui: &mut egui::Ui, up: bool, enabled: bool) -> bool {
        let (rect, response) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::click());
        let color = if enabled {
            ui.visuals().text_color()
        } else {
            Color32::from_gray(90)
        };
        let c = rect.center();
        let half = 3.5;
        let points = if up {
            [
                egui::pos2(c.x, c.y - half),
                egui::pos2(c.x - half, c.y + half),
                egui::pos2(c.x + half, c.y + half),
            ]
        } else {
            [
                egui::pos2(c.x, c.y + half),
                egui::pos2(c.x - half, c.y - half),
                egui::pos2(c.x + half, c.y - half),
            ]
        };
        ui.painter().add(egui::Shape::convex_polygon(
            points.to_vec(),
            color,
            egui::Stroke::NONE,
        ));
        let clicked = enabled && response.clicked();
        if response.hovered() && enabled {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        clicked
    }

    /// Applies what the Layers window asked for this frame (after the
    /// window's layout pass — see [`LayerRequest`]).
    fn apply_layer_request(&mut self, request: LayerRequest) {
        if request.add {
            self.add_layer();
        }
        if let Some(index) = request.select {
            self.switch_active_layer(index);
        }
        if let Some(index) = request.toggle {
            self.toggle_layer_visibility(index);
        }
        if let Some(index) = request.up {
            self.move_layer(index, 1);
        }
        if let Some(index) = request.down {
            self.move_layer(index, -1);
        }
        if let Some(index) = request.delete {
            self.layer_delete_confirm = Some(index);
        }
        if let Some(index) = request.rename_start {
            let name = self
                .layers
                .layer(index)
                .map(|layer| layer.name().to_string())
                .unwrap_or_default();
            self.layer_rename = Some((index, name));
        }
        if let Some((index, name)) = request.rename_commit
            && !name.is_empty()
        {
            self.layers.rename_layer(index, name);
            self.dirty = true;
            self.action_log.push("Layer renamed");
        }
        let _ = request.rename_cancel; // the buffer was already cleared
    }

    /// "+ Add layer": a transparent layer on top, active, fresh history.
    fn add_layer(&mut self) {
        let (width, height) = (self.canvas.width, self.canvas.height);
        let index = self.layers.add_layer(width, height);
        self.switch_active_layer(index);
        self.refresh_composite();
        self.dirty = true;
        self.action_log.push("Layer added");
    }

    fn toggle_layer_visibility(&mut self, index: usize) {
        let visible = self.layers.layer(index).map(|layer| layer.visible());
        if let Some(visible) = visible {
            self.layers.set_visible(index, !visible);
            self.refresh_composite();
            self.dirty = true;
        }
    }

    fn move_layer(&mut self, index: usize, delta: isize) {
        self.layers.move_layer(index, delta);
        self.refresh_composite();
        self.dirty = true;
        self.action_log.push("Layer moved");
    }

    /// The confirmed half of layer deletion ([`layer_delete_dialog`] did
    /// the asking).
    fn remove_layer_confirmed(&mut self, index: usize) {
        if !self.layers.remove_layer(index) {
            return;
        }
        let active = self.layers.active_layer();
        self.switch_active_layer(active);
        // Anchors may point into the removed layer's tree — the switch
        // above already retargeted every window to the new end.
        self.refresh_composite();
        self.dirty = true;
        self.action_log.push("Layer deleted");
    }

    /// "Delete layer?" confirmation (mirrors the unsaved-changes dialog):
    /// deleting removes the layer's WHOLE history, so it asks first.
    pub(super) fn layer_delete_dialog(&mut self, ctx: &Context) {
        let Some(index) = self.layer_delete_confirm else {
            return;
        };
        let name = self
            .layers
            .layer(index)
            .map(|layer| layer.name().to_string())
            .unwrap_or_else(|| "?".to_string());
        let mut confirmed = false;
        let mut cancel = false;
        egui::Window::new("Delete Layer")
            .collapsible(false)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label(format!("Delete layer \"{name}\"?"));
                ui.weak("Its pixels and history will be removed for good.");
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
            self.layer_delete_confirm = None;
            self.remove_layer_confirmed(index);
        } else if cancel {
            self.layer_delete_confirm = None;
        }
    }
}
