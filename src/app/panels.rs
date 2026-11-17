//! The "Tool Log" and "Layers" windows. Tool Log is the read-only action
//! log — it was titled "History" until the owner rename request 2026-11-13
//! (the egui area id and the internal `history_*` names stay unchanged so
//! placement / area state survive); Layers is the REAL layer system's
//! control panel (owner request 2026-11-13, paying functional debt F1):
//! rows in display order (top layer first), click to select the active
//! layer — every History Tree window follows — plus visibility, rename
//! (double-click), reorder and delete-with-confirmation. Both are plain
//! floating windows that since the owner request 2026-11-11 follow the same
//! §3.11 placement as "Tools" and "Palette" (drop-time edge anchoring +
//! resize repositioning, via [`place_floating_window`]); their FIRST-open
//! positions stay the edge anchors below. Child module of `app`, so it can
//! touch `App`'s private fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Color32, Context, Vec2};

use super::App;
use super::dialogs::dialog_button_row;
use super::placement::{FIRST_OPEN_GAP, LAYERS_FIRST_OPEN_LIFT, place_floating_window};

/// Stable ids, same idea as `windows.rs`: independent of the title text so
/// placement / area state lookups stay stable.
fn history_window_id() -> egui::Id {
    egui::Id::unique("history_window")
}

fn layers_window_id() -> egui::Id {
    egui::Id::unique("layers_window")
}

/// Outer width of the Layers window. The window used to be `.auto_sized()`
/// while its rows derived their width from `ui.available_width()` — and an
/// auto-sized window's FIRST layout pass sees `available_width` = the whole
/// `content_rect` (its `Resize` default size is INFINITY clamped to the
/// screen) while `desired_size` never shrinks below the first frame. The
/// row widths then fed themselves every frame: the window opened screen-
/// wide and grew ~one row-overflow per frame FOREVER, spilling past the
/// main window (owner report 2026-11-13). A fixed `default_width` with
/// fixed-width rows cannot feed itself — the quirk is recorded in
/// AGENTS.md "eframe/egui 0.37 API quirks".
const LAYERS_DEFAULT_WIDTH: f32 = 240.0;

/// Fixed width of a row's name click target — the former
/// `available_width - 58.0` was the auto-size feedback loop above.
/// Long names are clipped to the column instead of running under the
/// row's buttons.
const LAYER_NAME_WIDTH: f32 = 120.0;

/// Max height of the Tool Log's entry list. Was 280 pt, HALVED to 140 pt
/// by the owner request 2026-11-13 («уменьшить максимальную высоту окошка
/// вдвое»).
const TOOL_LOG_MAX_HEIGHT: f32 = 140.0;

/// Font size of the Tool Log's entries: 0.5rem = 8 pt (1 rem = 16 pt — egui
/// 0.37 has no rem API, the same convention as `placement::REM`; owner
/// request 2026-11-13). The window's vertical row spacing is zeroed with it
/// (owner request: «прижать в 0»), so consecutive log lines sit flush — each
/// row is the 8 pt font's own line height, the closest egui gets to a
/// 0.5rem line height.
const TOOL_LOG_FONT_SIZE: f32 = 8.0;

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
    /// The Tool Log window (owner request 2026-11-13 renamed it from
    /// "History"): the read-only action log, newest first. Entries are
    /// plain labels — clicking them does nothing yet by design.
    pub(super) fn history_window(&mut self, ctx: &Context) {
        if !self.history_window_open {
            return;
        }
        let mut open = true;
        let content = ctx.content_rect();
        // First open (owner request 2026-11-12, amended 2026-11-13): the
        // window sits at the BOTTOM CENTER of the main window and its
        // bottom edge is FLUSH with the bottom edge — the former 0.5rem
        // gap is now 0 («прижать в 0 к низу», owner request 2026-11-13).
        // §3.11 anchors the dropped gap, so it stays flush through
        // main-window resizes until the user drags it away. Afterwards the
        // same §3.11 magnetism and resize repositioning as Tools/Palette
        // (owner request 2026-11-11).
        let shown = egui::Window::new("Tool Log")
            .id(history_window_id())
            .open(&mut open)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .default_pos(egui::pos2(content.center().x, content.bottom()))
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.set_min_width(180.0);
                // Only this window (owner request 2026-11-13): zero vertical
                // row spacing, so the 8 pt log lines sit flush.
                ui.style_mut().spacing.item_spacing.y = 0.0;
                let entries = self.action_log.items();
                if entries.is_empty() {
                    ui.label(egui::RichText::new("No actions yet.").size(TOOL_LOG_FONT_SIZE));
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(TOOL_LOG_MAX_HEIGHT)
                        .show(ui, |ui| {
                            // Newest at the top, so the latest action is
                            // always visible without scrolling.
                            for entry in entries.iter().rev() {
                                ui.label(
                                    egui::RichText::new(format!("{}. {}", entry.id, entry.label))
                                        .size(TOOL_LOG_FONT_SIZE),
                                );
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
    /// NAME (a fixed-width column, [`LAYER_NAME_WIDTH`]); the eye, arrows
    /// and "×" sit next to it and never change the active layer. Fixed
    /// width, no resize handle — see [`LAYERS_DEFAULT_WIDTH`] for why the
    /// former `.auto_sized()` could not stay narrow.
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
            .default_width(LAYERS_DEFAULT_WIDTH)
            .resizable(false) // `.auto_sized()` was non-resizable too; see LAYERS_DEFAULT_WIDTH
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
            // Fixed-width name column (`LAYER_NAME_WIDTH`): width derived
            // from `available_width` inside this window fed the auto-size
            // growth loop (owner report 2026-11-13). The name text is
            // clipped to the column so a long name cannot run under the
            // row's buttons.
            let (rect, label) = ui.allocate_exact_size(
                Vec2::new(LAYER_NAME_WIDTH, row_height),
                egui::Sense::click(),
            );
            if is_active {
                ui.painter()
                    .rect_filled(rect, 3.0, ui.style().visuals.selection.bg_fill);
            }
            ui.painter().with_clip_rect(rect).text(
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
    /// the asking). Deleting the BOTTOM layer (owner request 2026-11-17:
    /// the bottom layer owns the canvas size) checks the successor: the
    /// SAME-size successor silently becomes the new base, a DIFFERENT-size
    /// one opens the base-choice dialog ([`super::dialogs::layer_base_dialog`]).
    fn remove_layer_confirmed(&mut self, index: usize) {
        // The successor's size, read BEFORE the removal (it is the layer
        // at index 1 when the bottom one goes).
        let successor = if index == 0 {
            self.layers
                .layer(1)
                .map(|layer| (layer.pixmap().width(), layer.pixmap().height()))
        } else {
            None
        };
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
        if let Some((width, height)) = successor
            && (width, height) != (self.canvas.width, self.canvas.height)
        {
            self.layer_base_choice = true;
        }
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
        let id = super::dialogs::dialog_id("delete_layer");
        super::dialogs::dialog_window("Delete Layer", id, ctx).show(ctx, |ui| {
            ui.label(format!("Delete layer \"{name}\"?"));
            ui.weak("Its pixels and history will be removed for good.");
            let (ok_hit, cancel_hit) = dialog_button_row(ui, "OK", "Cancel");
            confirmed = ok_hit;
            cancel = cancel_hit;
        });
        if confirmed {
            self.layer_delete_confirm = None;
            super::dialogs::forget_dialog_position(ctx, id);
            self.remove_layer_confirmed(index);
        } else if cancel {
            self.layer_delete_confirm = None;
            super::dialogs::forget_dialog_position(ctx, id);
        }
    }
}
