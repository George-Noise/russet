//! The magnetized per-tool settings panel (pure code motion, owner-approved
//! 2026-11-21 file-guard split): the panel window builder + the magnet
//! contract (`tool_settings_window`, [`settings_panel_shown`]), the icon
//! button pair every panel row uses ([`icon_button`] /
//! [`icon_button_state`]) and the per-tool UIs ([`shape_settings_ui`] /
//! [`line_settings_ui`] / [`size_block_ui`]). A child of `windows` (and of
//! `app`), so it touches `App`'s private fields; the Tools window itself
//! stayed in the parent. The position math is
//! `super::placement::magnet_settings_pos`; the panel hides while the
//! Tools window is collapsed (owner request 2026-11-19).

use eframe::egui::{self, Color32, Context, Id, Rect, Sense, TextureId};

use super::{App, TOOL_BUTTON_PT, TOOL_GRID_GAP};

/// The Size block's slider width — the Shape panel's widest button row
/// (moved with the block, owner-approved 2026-11-21 split).
pub(super) const SIZE_SLIDER_W: f32 = TOOL_BUTTON_PT * 3.0 + TOOL_GRID_GAP * 2.0;
use super::tool_icons::{self, LineIcon, ShapeIcon};
use crate::tools::{
    LineMode, MAX_STROKE_SIZE, ShapeKind, ShapeOutline, ShapePaint, Tool, ToolState,
};

/// The vertical gap that separates the Shape panel's button rows into
/// GROUPS — the KIND choice, the paint MODE and the outline PLACEMENT — so
/// the three groups read apart (the same distance-separates-meaning
/// principle the owner set for the Save dialog's encoding group, V50, and
/// extended to the Shape panel 2026-11-20): the rows' own gap is egui's 3-pt
/// item spacing, this adds 9 more between the button rows — 12 pt total.
/// Owner request 2026-11-20 (the spacing pass): WITHIN a row the buttons sit
/// EXACTLY the Tools window's element gap apart ([`TOOL_GRID_GAP`], set via
/// `item_spacing.x` in [`shape_settings_ui`]), and VERTICALLY the rows sit
/// "чуть дальше" — a step further apart than that 6-pt horizontal gap.
pub(super) const SHAPE_GROUP_GAP: f32 = 9.0;
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
pub(super) fn tool_settings_window_builder(pos: egui::Pos2) -> egui::Window<'static> {
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
pub(super) fn tool_uses_size(tool: Tool) -> bool {
    matches!(tool, Tool::Brush | Tool::Eraser | Tool::Line | Tool::Shape)
}
/// Whether the tool has anything for the settings panel to display at all:
/// Pencil and Eyedropper have no settings, so the magnetized panel is not
/// drawn for them (owner request 2026-11-17).
pub(super) fn tool_has_settings(tool: Tool) -> bool {
    !matches!(tool, Tool::Pencil | Tool::Eyedropper)
}
/// Whether the magnetized settings panel is drawn THIS frame (owner request
/// 2026-11-19): only while the Tools window shows its BODY — `tools_body_shown`
/// is `Window::show`'s `inner.is_some()`, which egui reports as `None` exactly
/// while the window is collapsed to its title bar (the collapse button /
/// double-click state; `window.rs` show-doc: "Returns
/// `Some(InnerResponse { inner: None })` if the window is collapsed") — and
/// while the current tool has settings at all. A collapsed window shows no
/// tool list, so the panel glued to it has nothing to attach to; it comes
/// back the frame the window is expanded again.
pub(super) fn settings_panel_shown(tools_body_shown: bool, tool: Tool) -> bool {
    tools_body_shown && tool_has_settings(tool)
}
/// One ICON BUTTON — the shared look of the Tools window's tool buttons and
/// the Shape panel's kind/mode buttons (owner request 2026-11-20: the
/// panel's word radio and checkboxes became "the same buttons as in the
/// Tools window"). A square of `size` pt, the cached icon texture centered
/// inside, the LIGHT-theme visual states in BOTH themes (selected = the
/// selection fill + stroke, hover = the hovered fill), rounded 3.0 corners,
/// and the tooltip the caller passes. Returns the response; the caller
/// wires the click (what the button SETS is the caller's state, not this
/// fn's).
pub(super) fn icon_button(
    ui: &mut egui::Ui,
    size: f32,
    icon: Option<TextureId>,
    tooltip: &'static str,
    selected: bool,
) -> egui::Response {
    icon_button_state(ui, size, icon, tooltip, selected, true)
}
/// The disabled-capable body of [`icon_button`] (owner request 2026-11-20:
/// the Select panel's copy/cut/crop buttons gray out while no selection
/// exists and paste while the clipboard is empty — the same affordance the
/// Edit menu's `add_enabled` rows give). Disabled = hover-only sense + a
/// gray icon tint; the tooltip still reads.
pub(super) fn icon_button_state(
    ui: &mut egui::Ui,
    size: f32,
    icon: Option<TextureId>,
    tooltip: &'static str,
    selected: bool,
    enabled: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(
        egui::vec2(size, size),
        if enabled {
            Sense::click()
        } else {
            Sense::hover()
        },
    );
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
    if let Some(id) = icon {
        let uv = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1.0, 1.0));
        // The icon is square; center it inside the padded button.
        let icon_rect = egui::Rect::from_center_size(
            rect.center(),
            egui::vec2(tool_icons::ICON_PT, tool_icons::ICON_PT),
        );
        // A disabled button grays its icon (the tint multiplies the
        // texture — the selection blue and the ink both dim).
        painter.image(
            id,
            icon_rect,
            uv,
            if enabled {
                Color32::WHITE
            } else {
                Color32::GRAY
            },
        );
    }
    response.on_hover_text(tooltip)
}
/// The Shape tool's settings rows (owner requests 2026-11-20): the KIND
/// row — two icon buttons, the gray-filled square / circle previews — then,
/// after a [`SHAPE_GROUP_GAP`], the MODE row — a THREE-way radio ("не 2
/// выбора, а 3 и это уже будет radio") over [`ShapePaint`] — Fill, Outline,
/// Fill + Outline — then, after another [`SHAPE_GROUP_GAP`], the PLACEMENT
/// row — a second THREE-way radio over [`ShapeOutline`] (the owner: three
/// outline modes — outside the two mouse points / exactly as before /
/// inside them; "делать их в виде угла квадратика вне зависимости от
/// выбранной фигуры"). The MODE radios' icons draw the Shape TOOL's emblem
/// ([`tool_icons::ShapeIcon::Fill`] / [`Outline`] / [`FillOutline`]); the
/// PLACEMENT radios' icons draw the corner composition
/// ([`ShapeIcon::OutlineOutside`] / [`OutlineCenter`] / [`OutlineInside`]).
/// The buttons are the Tools window's icon buttons ([`icon_button`]); the
/// tooltips carry the words ([`tool_icons::shape_icon_label`]). The SPACING
/// is the owner's 2026-11-20 pass: WITHIN a row the buttons sit EXACTLY the
/// Tools window's element gap apart ([`TOOL_GRID_GAP`] — "ровно такой же,
/// как в окошке Tools"; the row is laid out by egui's `item_spacing.x`, so
/// it is set here once and the three rows inherit it), and the rows sit
/// further apart vertically ([`SHAPE_GROUP_GAP`] above). While the
/// diameter-circle feature is under test (owner request 2026-11-21) a FOURTH
/// group — a plain checkbox — closes the panel, shown for the ELLIPSE kind
/// only: the rectangle's Shift rule is untouched, so the toggle must not
/// tempt anyone into reading it through the rectangle.
///
/// Extracted as a free fn over [`ToolState`] so the regression tests run
/// the exact UI the app runs (headless; the atlas is optional — the buttons
/// render without icons the same way the Tools window does before the atlas
/// lands).
pub(super) fn shape_settings_ui(
    ui: &mut egui::Ui,
    tools: &mut ToolState,
    icons: &Option<tool_icons::Atlas>,
) {
    // The horizontal gap between the buttons of every row below: EXACTLY
    // the Tools window's grid gap (owner request 2026-11-20). egui's
    // default `item_spacing.x` (8 pt) applied here instead; the vertical
    // stacking is untouched (the rows separate through the item spacing's
    // y half + `SHAPE_GROUP_GAP`).
    ui.spacing_mut().item_spacing.x = TOOL_GRID_GAP;
    ui.horizontal(|ui| {
        for (kind, icon) in [
            (ShapeKind::Rectangle, ShapeIcon::Rectangle),
            (ShapeKind::Ellipse, ShapeIcon::Ellipse),
        ] {
            let response = icon_button(
                ui,
                TOOL_BUTTON_PT,
                tool_icons::shape_texture_id(icons, icon),
                tool_icons::shape_icon_label(icon),
                tools.shape_kind == kind,
            );
            if response.clicked() {
                tools.shape_kind = kind;
            }
        }
    });
    ui.add_space(SHAPE_GROUP_GAP);
    ui.horizontal(|ui| {
        for (paint, icon) in [
            (ShapePaint::Fill, ShapeIcon::Fill),
            (ShapePaint::Outline, ShapeIcon::Outline),
            (ShapePaint::FillOutline, ShapeIcon::FillOutline),
        ] {
            let response = icon_button(
                ui,
                TOOL_BUTTON_PT,
                tool_icons::shape_texture_id(icons, icon),
                tool_icons::shape_icon_label(icon),
                tools.shape_paint == paint,
            );
            // Radio semantics: clicking a choice selects it — selecting the
            // already-active one is a no-op, and a shape always paints
            // something because EXACTLY one of the three is ever active.
            if response.clicked() {
                tools.shape_paint = paint;
            }
        }
    });
    ui.add_space(SHAPE_GROUP_GAP);
    ui.horizontal(|ui| {
        for (outline, icon) in [
            (ShapeOutline::Outside, ShapeIcon::OutlineOutside),
            (ShapeOutline::Center, ShapeIcon::OutlineCenter),
            (ShapeOutline::Inside, ShapeIcon::OutlineInside),
        ] {
            let response = icon_button(
                ui,
                TOOL_BUTTON_PT,
                tool_icons::shape_texture_id(icons, icon),
                tool_icons::shape_icon_label(icon),
                tools.shape_outline == outline,
            );
            // The same radio semantics as the paint row above. The choice
            // stays clickable with the Fill-only paint too: it is remembered
            // for the moment an outline is painted again.
            if response.clicked() {
                tools.shape_outline = outline;
            }
        }
    });
    // TESTING FEATURE (owner request 2026-11-21): the diameter-circle
    // toggle, bound straight to the tool state the gesture reads live — a
    // flip applies to the very next frame's geometry, mid-drag included.
    // ELLIPSE KIND ONLY (the owner: "чтобы никто случайно не пытался её
    // понять через прямоугольник"): with the Rectangle kind selected the
    // row does not exist at all — not a disabled checkbox, the same
    // no-dead-controls rule the Size slider follows.
    if tools.shape_kind == ShapeKind::Ellipse {
        ui.add_space(SHAPE_GROUP_GAP);
        ui.checkbox(&mut tools.shape_diameter_circle, "Diameter circle (Shift)")
            .on_hover_text(
                "Under test: Shift-drag makes the press and release points the endpoints \
                 of the circle's diameter (the center is the segment's midpoint). \
                 Unchecked = the old proportional circle growing from the corner.",
            );
    }
}
/// The Line tool's settings panel (owner request 2026-11-20: the word
/// "Mode" radio became icon buttons in the Tools window's style — "такие же
/// по стилю и отступам" as the reshaped Shape panel): ONE row of two icon
/// buttons — Straight and Curve — sitting EXACTLY the Tools window's
/// element gap apart ([`TOOL_GRID_GAP`] via `item_spacing.x`, the same
/// one-liner [`shape_settings_ui`] uses; the panel has a single button row,
/// so no group gap). The buttons draw the LINE TOOL's emblem per mode
/// ([`tool_icons::LineIcon`]: the Curve choice strokes it pixel-for-pixel
/// as the Tools window's Line button does, the Straight choice draws the
/// same stroke unbent); the tooltips carry the words the old radio showed
/// ([`tool_icons::line_icon_label`]). While the Curve mode is active the
/// panel keeps its two hint lines under the row — the multi-point
/// interaction's copy, unchanged by this pass.
///
/// Extracted as a free fn over [`ToolState`] so the regression tests run
/// the exact UI the app runs (headless; the atlas is optional — the buttons
/// render without icons the same way [`shape_settings_ui`] does).
pub(super) fn line_settings_ui(
    ui: &mut egui::Ui,
    tools: &mut ToolState,
    icons: &Option<tool_icons::Atlas>,
) {
    // The horizontal gap between the buttons: EXACTLY the Tools window's
    // grid gap (owner request 2026-11-20), the same setting the Shape rows
    // use.
    ui.spacing_mut().item_spacing.x = TOOL_GRID_GAP;
    ui.horizontal(|ui| {
        for (mode, icon) in [
            (LineMode::Straight, LineIcon::Straight),
            (LineMode::Curve, LineIcon::Curve),
        ] {
            let response = icon_button(
                ui,
                TOOL_BUTTON_PT,
                tool_icons::line_texture_id(icons, icon),
                tool_icons::line_icon_label(icon),
                tools.line_mode == mode,
            );
            // Radio semantics: clicking a choice selects it — selecting the
            // already-active one is a no-op, and EXACTLY one of the two is
            // ever active (the enum's own invariant).
            if response.clicked() {
                tools.line_mode = mode;
            }
        }
    });
    if tools.line_mode == LineMode::Curve {
        // Short lines: one long hint stretches the panel (as it once
        // stretched the Tools window — owner request 2026-11-10).
        ui.weak("Click adds points.");
        ui.weak("Enter/double-click places, Esc cancels.");
    }
}
/// The Size block of the settings panel (owner request 2026-11-20): the
/// "Size" label sits ABOVE the slider and the slider is narrowed to the
/// button grid's row width ([`SIZE_SLIDER_W`]); right of the slider a
/// click-to-type number box ([`egui::DragValue`]) accepts any size ≥ 1 —
/// including above 100 ([`crate::tools::MAX_STROKE_SIZE`]) — while the
/// SLIDER itself stays limited to 1..=100: it binds to a clamp COPY and
/// writes back only when dragged. (egui's `Slider` with its default `SliderClamping::Always`
/// would otherwise clamp the bound value EVERY frame — a typed 250 would
/// snap back to 100 — and with the copy the thumb just pins at the right
/// end while the entered size stays.) For Line and Shape a
/// [`SHAPE_GROUP_GAP`] is added after the block, so it separates from the
/// button rows by the same 12-pt step that separates the Shape panel's own
/// button rows (owner request 2026-11-20: "такой же высоты что и между
/// кнопками в Shape" — the 9-pt group gap + egui's 3-pt item spacing; the
/// Brush/Eraser panels have nothing below the slider, so no gap).
///
/// Extracted as a free fn over [`ToolState`] so the regression tests run
/// the exact UI the app runs (headless; the same pattern as
/// [`shape_settings_ui`]).
pub(super) fn size_block_ui(ui: &mut egui::Ui, tools: &mut ToolState, tool: Tool) {
    ui.label("Size");
    ui.horizontal(|ui| {
        // The same element gap the button rows use, so the slider and the
        // number box read as one row of the panel's grid.
        ui.spacing_mut().item_spacing.x = TOOL_GRID_GAP;
        // egui's Slider takes its width ONLY from `spacing.slider_width`
        // (`SliderCore::desired_size` — the default 100 pt; a container's
        // max width cannot narrow it), so the scope pins it to the button
        // grid's row width (owner request 2026-11-20).
        ui.spacing_mut().slider_width = SIZE_SLIDER_W;
        let mut slider_size = tools.size.clamp(1.0, 100.0);
        if ui
            .add(egui::Slider::new(&mut slider_size, 1.0..=100.0).show_value(false))
            .changed()
        {
            tools.size = slider_size;
        }
        ui.add(
            egui::DragValue::new(&mut tools.size)
                .range(1.0..=MAX_STROKE_SIZE)
                .speed(1.0),
        );
    });
    if matches!(tool, Tool::Line | Tool::Shape) {
        ui.add_space(SHAPE_GROUP_GAP);
    }
}

impl App {
    /// The magnetized settings panel of the "Tools" window (owner request
    /// 2026-11-17): the per-tool controls that used to sit below the tool
    /// list inside the Tools window, now in their own window glued to the
    /// Tools window by [`tool_settings_window_builder`] and
    /// `placement::magnet_settings_pos`. Drawn only while the Tools window
    /// is shown AND the current tool has settings — for Pencil/Eyedropper
    /// the panel is not drawn at all.
    pub(super) fn tool_settings_window(&mut self, ctx: &Context, tools_rect: Rect) {
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
                size_block_ui(ui, &mut self.tools, tool);
            }
            match tool {
                Tool::Line => {
                    line_settings_ui(ui, &mut self.tools, &self.tool_icons);
                }
                Tool::Shape => {
                    shape_settings_ui(ui, &mut self.tools, &self.tool_icons);
                }
                Tool::Select => {
                    // The mode row + the ops row (owner request 2026-11-20);
                    // the ops act on the WHOLE app state, so the free UI fn
                    // returns the clicks and the dispatch happens here.
                    let avail = super::select_panel::SelectAvail {
                        has_selection: self.selection.bounds().is_some(),
                        has_clipboard: self.clipboard.is_some(),
                    };
                    let ops = super::select_panel::select_settings_ui(
                        ui,
                        &mut self.tools,
                        &self.tool_icons,
                        &avail,
                    );
                    if ops.copy {
                        self.copy_selection();
                    }
                    if ops.cut {
                        self.cut_selection();
                    }
                    if ops.paste {
                        self.paste_clipboard();
                    }
                    if ops.crop {
                        self.crop_to_selection();
                    }
                }
                Tool::Text => {
                    // Two short lines: the single long hint would make the
                    // auto-sized panel too wide (owner 2026-11-10).
                    ui.weak("Click the canvas to place text;");
                    ui.weak("font size is set in the Text window.");
                }
                // The Fill has no panel knobs (owner request 2026-11-23:
                // the flood tolerance is the shared fixed FLOOD_TOLERANCE
                // — a setting lands only if the owner asks).
                Tool::Pencil | Tool::Brush | Tool::Eraser | Tool::Eyedropper | Tool::Fill => {}
            }
        });
        if let Some(response) = &shown {
            self.tool_settings_placement.record(response.response.rect);
        }
    }
}
