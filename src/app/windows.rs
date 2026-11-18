//! The floating "Tools" and "Palette" windows (SPEC §3.2 / §3.3), their
//! stable area ids and the primary/secondary color well with the built-in
//! egui color picker. A child module of `app`, so it can touch `App`'s
//! private fields; `app.rs` stays the orchestrator. Their first-open
//! positions are set here (§3.2/§3.3: `Window::pivot` + `default_pos`);
//! where they sit afterwards (drop-time anchoring, resize repositioning)
//! is §3.11 and lives in [`super::placement`]. The "Tools" window also owns
//! its magnetized per-tool settings panel (owner request 2026-11-17): the
//! position math is `super::placement::magnet_settings_pos`, the window
//! itself is `tool_settings_window` below.

use eframe::egui::{self, Color32, Context, Id, Rect, Sense};

use super::App;
use super::placement::{FIRST_OPEN_GAP, place_floating_window};
use super::tool_icons;
use crate::palette::{COLORS, COLS, GRID_WIDTH, ROWS, TILE};
use crate::tools::{LineMode, ShapeKind, Tool};

/// INTERIM layout (owner request 2026-11-18: "two columns for now — it will
/// be reworked later, I have a better idea"): the tool buttons sit in a
/// two-column grid. The rework is debt F15.
const TOOL_COLUMNS: usize = 2;

/// The tool button's square size in logical points: the 1.5rem icon plus a
/// small padding on every side.
const TOOL_BUTTON_PT: f32 = tool_icons::ICON_PT + 4.0;

/// Stable ids for the two floating windows. They are needed to read the
/// windows' area state and title-bar drag responses from the placement
/// logic (§3.11), so they must not depend on the (changing) title text.
fn tools_window_id() -> Id {
    Id::unique("tools_window")
}

fn palette_window_id() -> Id {
    Id::unique("palette_window")
}

/// Stable id of the magnetized tool-settings panel (owner request
/// 2026-11-17); needed to read and persist its area state, so it must not
/// depend on the (changing) title text.
fn tool_settings_window_id() -> Id {
    Id::unique("tool_settings_window")
}

/// The builder of the magnetized settings panel (owner request 2026-11-17):
/// a title-bar-less, immovable, content-hugging window. `fixed_pos`
/// re-applies the position on EVERY frame — that is the magnet: no stored
/// area position can ever win over it, so the panel cannot be dragged or
/// otherwise detached from the Tools window ("with no way to detach it,
/// for now"). Exposed as a function so the regression tests
/// (`windows/tests.rs`) run the exact builder the app runs.
fn tool_settings_window_builder(pos: egui::Pos2) -> egui::Window<'static> {
    egui::Window::new("Tool settings")
        .id(tool_settings_window_id())
        .title_bar(false)
        .collapsible(false)
        .movable(false)
        .fixed_pos(pos)
        .auto_sized() // size follows the content; no resize handles
}

/// Whether the tool's settings panel shows the "Size" slider (owner request
/// 2026-11-17: the slider is SHOWN for these tools and NOT SHOWN for the
/// others — the former disabled-when-unapplicable slider is retired).
fn tool_uses_size(tool: Tool) -> bool {
    matches!(tool, Tool::Brush | Tool::Eraser | Tool::Line | Tool::Shape)
}

/// Whether the tool has anything for the settings panel to display at all:
/// Pencil and Eyedropper have no settings, so the magnetized panel is not
/// drawn for them (owner request 2026-11-17).
fn tool_has_settings(tool: Tool) -> bool {
    !matches!(tool, Tool::Pencil | Tool::Eyedropper)
}

impl App {
    pub(super) fn tools_window(&mut self, ctx: &Context) {
        if !self.tools_window_open {
            return;
        }
        // The icon atlas is (re)rastered once per scale change in
        // `App::ui` — before the canvas, so the Eyedropper's pipette cursor
        // exists too; here every frame just blits the cached textures.
        let shown = egui::Window::new("Tools")
            .id(tools_window_id())
            // Owner v4 (2026-11-18): NO title bar — the header was WIDER
            // than the tiny window (a visual artifact). The title still
            // names the window's Id: egui derives it from the title EVEN
            // when the bar is hidden (`window.rs` on `Window::new`), so the
            // explicit `tools_window_id()` keeps the identity stable; egui's
            // `WindowDrag::TitleBar` silently falls back to `Anywhere` for
            // title-bar-less windows (`window.rs`), so the window stays
            // draggable by its surface, and closing runs through
            // View ▸ Tools (the bar's ✕ is gone with the bar).
            .title_bar(false)
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
                // The Tools window itself is ONLY the tool list: the per-tool
                // settings live in the magnetized panel below/above it (owner
                // request 2026-11-17, see `tool_settings_window`).
                egui::Grid::new("tools_grid")
                    .num_columns(TOOL_COLUMNS)
                    // egui floors every column at `interact_size.x` (40 pt),
                    // which padded the 28 pt buttons with invisible width and
                    // made the column gap 18 pt while the row gap stayed 6 pt
                    // (owner request 2026-11-18: equal gaps both ways).
                    .min_col_width(TOOL_BUTTON_PT)
                    .spacing([6.0, 6.0])
                    .show(ui, |ui| {
                        for row in tool_icons::TOOLS.chunks(TOOL_COLUMNS) {
                            for tool in row {
                                self.tool_button(ui, *tool);
                            }
                            ui.end_row();
                        }
                    });
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
            // The panel magnets to the Tools window's rect AS DRAWN THIS
            // frame: a title-bar drag moves both in the same frame, and the
            // §3.11 resize repositioning (which persists into the next frame)
            // is followed the frame it becomes visible.
            self.tool_settings_window(ctx, response.response.rect);
        }
    }

    /// One tool button: a square, the cached icon texture on top, the
    /// selectable_label visual states (selected = the selection fill, hover
    /// = the hovered fill) and the name+key tooltip (owner request
    /// 2026-11-18). Clicking selects the tool — the same state the old
    /// text `selectable_label` list drove.
    fn tool_button(&mut self, ui: &mut egui::Ui, tool: Tool) {
        let selected = self.tools.tool == tool;
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(TOOL_BUTTON_PT, TOOL_BUTTON_PT), Sense::click());
        // Owner request 2026-11-18 (after the V36 pass): the tool buttons
        // keep the LIGHT-theme look in BOTH themes — one background lets the
        // single icon set (its dark rim) read well everywhere, instead of
        // drawing for two backgrounds. The states (selected/hover/rest) come
        // from the light widget visuals, so the button is theme-independent.
        // Owner v4 (same day): the resting/hover fills dim A STEP (−14 per
        // channel) — the full gray(230) light-theme button contrasted hard
        // against BOTH themes' window background; the selection fill keeps
        // the bright selection blue so the active tool still pops.
        const DIM: u8 = 14;
        let dim = |c: Color32| {
            Color32::from_rgb(
                c.r().saturating_sub(DIM),
                c.g().saturating_sub(DIM),
                c.b().saturating_sub(DIM),
            )
        };
        let light = egui::Visuals::light();
        let (fill, stroke) = if selected {
            (light.selection.bg_fill, light.selection.stroke)
        } else if response.hovered() {
            (
                dim(light.widgets.hovered.bg_fill),
                light.widgets.hovered.bg_stroke,
            )
        } else {
            (
                dim(light.widgets.inactive.bg_fill),
                light.widgets.inactive.bg_stroke,
            )
        };
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, fill);
        painter.rect_stroke(rect, 3.0, stroke, egui::StrokeKind::Inside);
        if let Some(id) = tool_icons::texture_id(&self.tool_icons, tool) {
            let uv = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0));
            // The icon is square; center it inside the padded button.
            let icon = egui::Rect::from_center_size(
                rect.center(),
                egui::vec2(tool_icons::ICON_PT, tool_icons::ICON_PT),
            );
            painter.image(id, icon, uv, Color32::WHITE);
        }
        if response.clicked() {
            self.tools.tool = tool;
        }
        response.on_hover_text(tool_icons::label(tool));
    }

    /// The magnetized settings panel of the "Tools" window (owner request
    /// 2026-11-17): the per-tool controls that used to sit below the tool
    /// list inside the Tools window, now in their own window glued to the
    /// Tools window by [`tool_settings_window_builder`] and
    /// `placement::magnet_settings_pos`. Drawn only while the Tools window
    /// is shown AND the current tool has settings — for Pencil/Eyedropper
    /// the panel is not drawn at all.
    fn tool_settings_window(&mut self, ctx: &Context, tools_rect: Rect) {
        if !tool_has_settings(self.tools.tool) {
            return;
        }
        let pos = self
            .tool_settings_placement
            .pos(tools_rect, ctx.content_rect());
        let shown = tool_settings_window_builder(pos).show(ctx, |ui| {
            let tool = self.tools.tool;
            if tool_uses_size(tool) {
                // Always enabled when shown: a tool without a size setting
                // does not get a disabled slider, it gets NO slider (owner
                // request 2026-11-17).
                ui.add(egui::Slider::new(&mut self.tools.size, 1.0..=100.0).text("Size"));
            }
            match tool {
                Tool::Line => {
                    ui.horizontal(|ui| {
                        ui.label("Mode");
                        ui.radio_value(&mut self.tools.line_mode, LineMode::Straight, "Straight");
                        ui.radio_value(&mut self.tools.line_mode, LineMode::Curve, "Curve");
                    });
                    if self.tools.line_mode == LineMode::Curve {
                        // Short lines: one long hint stretches the panel (as
                        // it once stretched the Tools window — owner
                        // request 2026-11-10).
                        ui.weak("Click adds points.");
                        ui.weak("Enter/double-click places, Esc cancels.");
                    }
                }
                Tool::Shape => {
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
                Tool::Text => {
                    // Two short lines: the single long hint would make the
                    // auto-sized panel too wide (owner 2026-11-10).
                    ui.weak("Click the canvas to place text;");
                    ui.weak("font size is set in the Text window.");
                }
                Tool::Pencil | Tool::Brush | Tool::Eraser | Tool::Eyedropper => {}
            }
        });
        if let Some(response) = &shown {
            self.tool_settings_placement.record(response.response.rect);
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
                // No hotkey behind it — §3.9 stays exhaustive and the label
                // names no key either (owner request 2026-11-15).
                ui.horizontal(|ui| {
                    color_swell_button(ui, &mut self.tools.primary, 32.0);
                    color_swell_button(ui, &mut self.tools.secondary, 32.0);
                    if ui.button("Swap").clicked() {
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

#[cfg(test)]
mod tests;
