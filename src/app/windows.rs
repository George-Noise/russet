//! The floating "Tools" window (SPEC §3.2) — its stable area id, the icon
//! grid and the magnetized per-tool settings panel — plus the child
//! [`palette_window`] module owning the "Palette" window (SPEC §3.3). Both
//! are child modules of `app`, so they can touch `App`'s private fields;
//! `app.rs` stays the orchestrator. The windows' first-open positions are
//! set here (§3.2/§3.3: `Window::pivot` + `default_pos`); where they sit
//! afterwards (drop-time anchoring, resize repositioning) is §3.11 and
//! lives in [`super::placement`]. The "Tools" window also owns its
//! magnetized per-tool settings panel (owner request 2026-11-17): the
//! position math is `super::placement::magnet_settings_pos`, the window
//! itself is `tool_settings_window` below, and the panel hides while the
//! Tools window is collapsed to its title bar (owner request 2026-11-19).
//!
//! FILE LAYOUT (the owner-approved 2026-11-19 file-guard split, pure code
//! motion): the palette half of this file — the window itself, the
//! primary/secondary color wells with egui's picker popup (the alpha
//! channel editable, `Alpha::OnlyBlend`) and the hex field — moved VERBATIM
//! to the child [`palette_window`]; the parent keeps the Tools window and
//! the shared placement plumbing, and the `App` method call sites
//! (`app.rs`) are unchanged.

use eframe::egui::{self, Color32, Context, Id, Rect, Sense, TextureId};

use super::App;
use super::placement::{FIRST_OPEN_GAP, place_floating_window};
use super::tool_icons::{self, LineIcon, ShapeIcon};
use crate::tools::{LineMode, ShapeKind, ShapeOutline, ShapePaint, Tool, ToolState};

mod palette_window;

/// INTERIM layout (owner request 2026-11-18: "two columns for now — it will
/// be reworked later, I have a better idea"): the tool buttons sit in a
/// two-column grid. The rework is debt F15.
const TOOL_COLUMNS: usize = 2;

/// The gap between the tool-grid columns and rows (owner request 2026-11-18:
/// equal gaps both ways).
const TOOL_GRID_GAP: f32 = 6.0;

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
const SHAPE_GROUP_GAP: f32 = 9.0;

/// The tool button's square size in logical points: the 1.5rem icon plus a
/// small padding on every side.
const TOOL_BUTTON_PT: f32 = tool_icons::ICON_PT + 4.0;

/// Stable ids for the two floating windows. They are needed to read the
/// windows' area state and title-bar drag responses from the placement
/// logic (§3.11), so they must not depend on the (changing) title text.
fn tools_window_id() -> Id {
    Id::unique("tools_window")
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

/// The title strip's frame (owner v6 2026-11-19, "variant A2"): egui's default
/// window frame with the HORIZONTAL inner margin zeroed. The strip's padding
/// IS the title frame's inner margin (`window.rs::title_ui` lays the button
/// atoms out in `Frame::NONE.inner_margin(frame.inner_margin)`), and an
/// auto-sized window's strip is free to PAINT wider than the window — with the
/// default 6-pt side padding the both-buttons strip (2 Heading-height atoms +
/// 4 item-spacing gaps = 73.4 pt) painted 85.4 pt wide against the ~74-pt
/// window and the ✕ landed half outside it (AGENTS.md quirk; owner report
/// 2026-11-19). Zeroing the side padding makes the strip's minimum width
/// (73.4 pt) fit the grid-width window, so the strip paints exactly the
/// window's width and both buttons sit inside it. The VERTICAL padding stays
/// (variant A2 keeps the strip's height, the fill and the rounded top corners
/// exactly as the default window frame draws them — variant A, `Margin::ZERO`,
/// additionally shrank the strip vertically and shifted the window's first-open
/// position; the owner picked A2).
fn tools_title_frame(ctx: &Context) -> egui::Frame {
    let mut frame = egui::Frame::window(&ctx.global_style());
    frame.inner_margin.left = 0;
    frame.inner_margin.right = 0;
    frame
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
fn settings_panel_shown(tools_body_shown: bool, tool: Tool) -> bool {
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
fn icon_button(
    ui: &mut egui::Ui,
    size: f32,
    icon: Option<TextureId>,
    tooltip: &'static str,
    selected: bool,
) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
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
        painter.image(id, icon_rect, uv, Color32::WHITE);
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
/// further apart vertically ([`SHAPE_GROUP_GAP`] above).
///
/// Extracted as a free fn over [`ToolState`] so the regression tests run
/// the exact UI the app runs (headless; the atlas is optional — the buttons
/// render without icons the same way the Tools window does before the atlas
/// lands).
fn shape_settings_ui(ui: &mut egui::Ui, tools: &mut ToolState, icons: &Option<tool_icons::Atlas>) {
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
fn line_settings_ui(ui: &mut egui::Ui, tools: &mut ToolState, icons: &Option<tool_icons::Atlas>) {
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

/// The builder of the "Tools" window itself (§3.2). Exposed as a function so
/// the regression tests (`windows/tests.rs`) run the exact builder the app
/// runs.
fn tools_window_builder<'a>(ctx: &Context, open: &'a mut bool) -> egui::Window<'a> {
    egui::Window::new("")
        .id(tools_window_id())
        // Owner v5 (2026-11-18): the v4 `title_bar(false)` removed the
        // WHOLE bar (drag strip + buttons), but the owner wanted only
        // the LABEL hidden — the header (drag + close) is part of the
        // window's look. An EMPTY title string does exactly that:
        // `title_ui` builds the bar from the button atoms and the drag
        // strip regardless of the text (`window.rs`), the text atom is
        // just empty. The Id stays EXPLICIT — egui derives it from the
        // title (an empty string would collide with any other
        // empty-titled window; `Window::id` doc: "Required if the title
        // changes, or is shared with another window").
        .open(open)
        // The strip fits the window — see `tools_title_frame` (owner v6,
        // 2026-11-19, "variant A2"): without this the both-buttons strip paints
        // ~11 pt past the window's right edge and the ✕ lands half outside it.
        .title_frame(tools_title_frame(ctx))
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
}

impl App {
    pub(super) fn tools_window(&mut self, ctx: &Context) {
        if !self.tools_window_open {
            return;
        }
        // The icon atlas is (re)rastered once per scale change in
        // `App::ui` — before the canvas, so the Eyedropper's pipette cursor
        // exists too; here every frame just blits the cached textures.
        let mut open = true;
        let shown = tools_window_builder(ctx, &mut open).show(ctx, |ui| {
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
                .spacing([TOOL_GRID_GAP, TOOL_GRID_GAP])
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
            if settings_panel_shown(response.inner.is_some(), self.tools.tool) {
                self.tool_settings_window(ctx, response.response.rect);
            }
        }
        if !open {
            self.tools_window_open = false;
        }
    }

    /// One tool button: a square, the cached icon texture on top, the
    /// selectable_label visual states (selected = the selection fill, hover
    /// = the hovered fill) and the name+key tooltip (owner request
    /// 2026-11-18). Clicking selects the tool — the same state the old
    /// text `selectable_label` list drove.
    fn tool_button(&mut self, ui: &mut egui::Ui, tool: Tool) {
        let selected = self.tools.tool == tool;
        let response = icon_button(
            ui,
            TOOL_BUTTON_PT,
            tool_icons::texture_id(&self.tool_icons, tool),
            tool_icons::label(tool),
            selected,
        );
        if response.clicked() {
            self.tools.tool = tool;
        }
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
                    line_settings_ui(ui, &mut self.tools, &self.tool_icons);
                }
                Tool::Shape => {
                    shape_settings_ui(ui, &mut self.tools, &self.tool_icons);
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
}

#[cfg(test)]
mod tests;
