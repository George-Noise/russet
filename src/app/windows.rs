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

use eframe::egui::{self, Context, Id};

use super::App;
use super::placement::{FIRST_OPEN_GAP, place_floating_window};
use super::tool_icons;
use crate::tools::{SelectMode, Tool};

mod palette_window;
mod select_panel;

mod settings_panel;

use settings_panel::{SHAPE_GROUP_GAP, icon_button, icon_button_state, settings_panel_shown};

/// INTERIM layout (owner request 2026-11-18: "two columns for now — it will
/// be reworked later, I have a better idea"): the tool buttons sit in a
/// two-column grid. The rework is debt F15.
const TOOL_COLUMNS: usize = 2;

/// The gap between the tool-grid columns and rows (owner request 2026-11-18:
/// equal gaps both ways).
const TOOL_GRID_GAP: f32 = 6.0;

/// The tool button's square size in logical points: the 1.5rem icon plus a
/// small padding on every side.
const TOOL_BUTTON_PT: f32 = tool_icons::ICON_PT + 4.0;

/// Stable ids for the two floating windows. They are needed to read the
/// windows' area state and title-bar drag responses from the placement
/// logic (§3.11), so they must not depend on the (changing) title text.
fn tools_window_id() -> Id {
    Id::unique("tools_window")
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

/// The width of the Size block's slider: EXACTLY the Shape panel's widest
/// button row (3 × [`TOOL_BUTTON_PT`] + 2 × [`TOOL_GRID_GAP`]) — the owner's
/// 2026-11-20 pass moved the "Size" label ABOVE the slider and narrowed the
/// slider to the button grid's own rhythm (with the label to its left the
/// Size element was the panel's widest row).
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
    /// text `selectable_label` list drove. The SELECT button (owner
    /// request 2026-11-20) is mode-aware: its icon draws the CURRENT
    /// sub-mode's mark, and while the tool is ALREADY active a left-click
    /// cycles the sub-mode forward and a right-click cycles it backward
    /// ("there will be more than 2 selection kinds later" — the cycle runs over
    /// [`SelectMode::ALL`]).
    fn tool_button(&mut self, ui: &mut egui::Ui, tool: Tool) {
        if tool == Tool::Select {
            self.select_tool_button(ui);
            return;
        }
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

    /// The Select tool's mode-aware button (owner request 2026-11-20): the
    /// icon = the current sub-mode's selection mark; an INACTIVE button's
    /// left-click activates the tool, an ACTIVE one's left-click cycles the
    /// sub-mode FORWARD, the right-click cycles BACKWARD.
    fn select_tool_button(&mut self, ui: &mut egui::Ui) {
        let selected = self.tools.tool == Tool::Select;
        let icon = match self.tools.select_mode {
            SelectMode::Rect => tool_icons::SelectIcon::Rect,
            SelectMode::Lasso => tool_icons::SelectIcon::Lasso,
            SelectMode::Wand => tool_icons::SelectIcon::Wand,
        };
        let response = icon_button(
            ui,
            TOOL_BUTTON_PT,
            tool_icons::select_texture_id(&self.tool_icons, icon),
            tool_icons::select_icon_label(icon),
            selected,
        );
        if response.clicked() {
            if selected {
                self.tools.select_mode = self.tools.select_mode.next();
            } else {
                self.tools.tool = Tool::Select;
            }
        }
        if response.secondary_clicked() && selected {
            self.tools.select_mode = self.tools.select_mode.prev();
        }
    }
}

#[cfg(test)]
mod tests;
