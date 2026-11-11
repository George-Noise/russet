//! The floating "Tools" and "Palette" windows (SPEC §3.2 / §3.3), their
//! stable area ids and the primary/secondary color well with the built-in
//! egui color picker. A child module of `app`, so it can touch `App`'s
//! private fields; `app.rs` stays the orchestrator. Their first-open
//! positions are set here (§3.2/§3.3: `Window::pivot` + `default_pos`);
//! where they sit afterwards (drop-time anchoring, resize repositioning)
//! is §3.11 and lives in [`super::placement`].

use eframe::egui::{self, Color32, Context, Id, Rect, Sense};

use super::App;
use super::placement::{FIRST_OPEN_GAP, place_floating_window};
use crate::palette::{COLORS, COLS, GRID_WIDTH, ROWS, TILE};
use crate::tools::{LineMode, ShapeKind, Tool};

/// Tools are stacked in one column (SPEC §3.2).
const TOOL_COLUMNS: usize = 1;

/// Stable ids for the two floating windows. They are needed to read the
/// windows' area state and title-bar drag responses from the placement
/// logic (§3.11), so they must not depend on the (changing) title text.
fn tools_window_id() -> Id {
    Id::unique("tools_window")
}

fn palette_window_id() -> Id {
    Id::unique("palette_window")
}

impl App {
    pub(super) fn tools_window(&mut self, ctx: &Context) {
        if !self.tools_window_open {
            return;
        }
        let tools = [
            (Tool::Pencil, "Pencil (P)"),
            (Tool::Brush, "Brush (B)"),
            (Tool::Eraser, "Eraser (E)"),
            (Tool::Eyedropper, "Eyedropper (I)"),
            (Tool::Line, "Line (L)"),
            (Tool::Shape, "Shape (U)"),
            (Tool::Text, "Text (T)"),
        ];
        let mut open = true;
        let shown = egui::Window::new("Tools")
            .id(tools_window_id())
            .open(&mut open)
            // First open (§3.2): 0.5rem from the left edge; vertically a bit
            // ABOVE center — 40% of the free vertical space above the window
            // and 60% below it (owner request 2026-11-11). The LEFT_CENTER
            // pivot + `default_pos` set the left gap and a centered start;
            // the exact 40/60 split is applied once by the placement glue on
            // the first frame (`placement::TOOLS_FIRST_OPEN_RATIO`), because
            // the auto-sized height is only known after the first layout.
            // `default_pos` is consumed only while the window has no stored
            // position — after any user move or §3.11 repositioning the
            // stored position always wins.
            .pivot(egui::Align2::LEFT_CENTER)
            .default_pos(ctx.content_rect().left_center() + egui::vec2(FIRST_OPEN_GAP, 0.0))
            .auto_sized() // size follows the content; no resize handles
            .show(ctx, |ui| {
                egui::Grid::new("tools_grid")
                    .num_columns(TOOL_COLUMNS)
                    .spacing([4.0, 2.0])
                    .show(ui, |ui| {
                        for row in tools.chunks(TOOL_COLUMNS) {
                            for (tool, label) in row {
                                if ui
                                    .selectable_label(self.tools.tool == *tool, *label)
                                    .clicked()
                                {
                                    self.tools.tool = *tool;
                                }
                            }
                            ui.end_row();
                        }
                    });

                let size_affected = matches!(
                    self.tools.tool,
                    Tool::Brush | Tool::Eraser | Tool::Line | Tool::Shape
                );
                ui.add_enabled(
                    size_affected,
                    egui::Slider::new(&mut self.tools.size, 1.0..=100.0).text("Size"),
                );

                if self.tools.tool == Tool::Line {
                    ui.horizontal(|ui| {
                        ui.label("Mode");
                        ui.radio_value(&mut self.tools.line_mode, LineMode::Straight, "Straight");
                        ui.radio_value(&mut self.tools.line_mode, LineMode::Curve, "Curve");
                    });
                    if self.tools.line_mode == LineMode::Curve {
                        // Short lines: one long hint stretches the auto-sized
                        // Tools window (owner request 2026-11-10).
                        ui.weak("Click adds points.");
                        ui.weak("Enter/double-click places, Esc cancels.");
                    }
                }
                if self.tools.tool == Tool::Shape {
                    ui.horizontal(|ui| {
                        ui.label("Kind");
                        ui.radio_value(
                            &mut self.tools.shape_kind,
                            ShapeKind::Rectangle,
                            "Rectangle",
                        );
                        ui.radio_value(&mut self.tools.shape_kind, ShapeKind::Ellipse, "Ellipse");
                    });
                    // The primary color paints the outline, the secondary
                    // color the fill. At least one of the two must stay on:
                    // unchecking Fill re-checks Outline and unchecking
                    // Outline re-checks Fill, so the tool can never end up
                    // drawing nothing (owner request 2026-11-10).
                    if ui.checkbox(&mut self.tools.shape_fill, "Fill").changed()
                        && !self.tools.shape_fill
                    {
                        self.tools.shape_outline = true;
                    }
                    if ui
                        .checkbox(&mut self.tools.shape_outline, "Outline")
                        .changed()
                        && !self.tools.shape_outline
                    {
                        self.tools.shape_fill = true;
                    }
                }
                if self.tools.tool == Tool::Text {
                    // Two short lines: the single long hint made the
                    // auto-sized Tools window too wide (owner 2026-11-10).
                    ui.weak("Click the canvas to place text;");
                    ui.weak("font size is set in the Text window.");
                }
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                tools_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.tools_placement,
            );
        }
        if !open {
            self.tools_window_open = false;
        }
    }

    pub(super) fn palette_window(&mut self, ctx: &Context) {
        if !self.palette_window_open {
            return;
        }
        let mut open = true;
        let shown = egui::Window::new("Palette")
            .id(palette_window_id())
            .open(&mut open)
            // First open (§3.3): the window's bottom-left corner sits 0.5rem
            // from the main window's left and bottom edges (`Align2::LEFT_BOTTOM`
            // pivot interprets `default_pos` as that corner's position).
            // As with Tools, the stored position wins after any move.
            .pivot(egui::Align2::LEFT_BOTTOM)
            .default_pos(
                ctx.content_rect().left_bottom() + egui::vec2(FIRST_OPEN_GAP, -FIRST_OPEN_GAP),
            )
            .auto_sized() // size follows the content; no resize handles
            .show(ctx, |ui| {
                // Top: the primary/secondary color wells. A click on a well
                // opens egui's color picker for that color (the former
                // "Custom…" button, one per color); Swap sits to the right.
                ui.horizontal(|ui| {
                    color_swell_button(ui, &mut self.tools.primary, 32.0);
                    color_swell_button(ui, &mut self.tools.secondary, 32.0);
                    if ui.button("Swap (X)").clicked() {
                        std::mem::swap(&mut self.tools.primary, &mut self.tools.secondary);
                    }
                });

                // A plain ui.separator() fills all available width; inside an
                // auto-sized window that width feeds back into the window size
                // until the window spans the whole screen. Pin the divider to
                // the swatch-grid width instead (6.0 pt = default height).
                ui.add_sized([GRID_WIDTH, 6.0], egui::Separator::default());

                // Bottom: the seamless preset grid — tiles touch each other
                // (gap 0) and have no outline. LMB sets the primary color,
                // RMB the secondary. Not an egui::Grid: Grid enforces a
                // minimum column width (`interact_size.x`, 40 pt by default),
                // which leaves gaps between the 20 pt tiles. One exact-size
                // block is allocated instead; neighboring tiles share edges,
                // and clicks are resolved manually from the pointer position.
                let grid_size = egui::vec2(GRID_WIDTH, ROWS as f32 * TILE);
                let (grid_rect, grid) = ui.allocate_exact_size(grid_size, Sense::click());
                for row in 0..ROWS {
                    for col in 0..COLS {
                        let tile = Rect::from_min_size(
                            grid_rect.min + egui::vec2(col as f32 * TILE, row as f32 * TILE),
                            egui::vec2(TILE, TILE),
                        );
                        ui.painter()
                            .rect_filled(tile, 0.0, COLORS[row * COLS + col]);
                    }
                }
                if let Some(pos) = grid.interact_pointer_pos() {
                    let dx = pos.x - grid_rect.min.x;
                    let dy = pos.y - grid_rect.min.y;
                    if dx >= 0.0 && dy >= 0.0 {
                        let col = (dx / TILE) as usize;
                        let row = (dy / TILE) as usize;
                        if col < COLS && row < ROWS {
                            let color = COLORS[row * COLS + col];
                            if grid.clicked() {
                                self.tools.primary = color;
                            }
                            if grid.secondary_clicked() {
                                self.tools.secondary = color;
                            }
                        }
                    }
                }
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                palette_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.palette_placement,
            );
        }
        if !open {
            self.palette_window_open = false;
        }
    }
}

/// A primary/secondary color well: keeps the classic swatch look, but a
/// click opens egui's built-in color picker for THIS color (each well edits
/// its own color; there are no captions).
fn color_swell_button(ui: &mut egui::Ui, color: &mut Color32, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
    ui.painter().rect_filled(rect, 2.0, *color);
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0, Color32::from_gray(40)),
        egui::StrokeKind::Inside,
    );

    // Same wiring as egui's own color button (color_edit_button_hsva): a
    // click toggles the popup; the HSV state is re-derived from the current
    // color every frame and written back only when the picker changed it.
    let mut hsva = egui::ecolor::Hsva::from(*color);
    let mut button_response = response;
    egui::Popup::menu(&button_response)
        .id(button_response.id.with("color_popup"))
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = 275.0;
            if egui::color_picker::color_picker_hsva_2d(
                ui,
                &mut hsva,
                egui::color_picker::Alpha::Opaque,
            ) {
                *color = Color32::from(hsva);
                button_response.mark_changed();
            }
        });
    button_response
}
