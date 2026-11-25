//! The Select tool's magnetized-panel rows (owner request 2026-11-20): the
//! MODE row — the region kinds as icon buttons (rectangle / lasso, the
//! same marks the Tools window's Select button draws, one geometry per
//! [`tool_icons::SelectIcon`]) — then, after a [`SHAPE_GROUP_GAP`], the OPS
//! row — Copy / Cut / Paste / Crop as icon buttons, grayed while their
//! precondition is missing (no selection / no clipboard), the same
//! affordance the Edit menu's `add_enabled` rows give. The buttons are the
//! Tools window's icon buttons ([`super::icon_button`]); the mode buttons
//! write [`ToolState::select_mode`] directly, the OPS need the whole
//! [`App`] — this free fn RETURNS the clicks and the panel dispatch in
//! [`super::App::tool_settings_window`] wires them (the same split the
//! tests run headless: [`super::App`] is not constructible in the window
//! tests).

use eframe::egui;

use super::settings_panel::tolerance_block_ui;
use super::tool_icons::{self, SelectIcon};
use super::{SHAPE_GROUP_GAP, TOOL_BUTTON_PT, TOOL_GRID_GAP, icon_button, icon_button_state};
use crate::tools::{SelectMode, ToolState};

/// What the ops row can do right now (computed by the caller from the App
/// state before the borrow of `tools`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SelectAvail {
    pub has_selection: bool,
    pub has_clipboard: bool,
}

/// Which op buttons the user clicked this frame (the caller wires each to
/// its App method; several are possible in one frame — each runs).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SelectOps {
    pub copy: bool,
    pub cut: bool,
    pub paste: bool,
    pub crop: bool,
}

/// The ops row's buttons in order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OpButton {
    Copy,
    Cut,
    Paste,
    Crop,
}

/// Draws the Select panel: the MODE row, the group gap, the OPS row.
/// Returns the op clicks.
pub(super) fn select_settings_ui(
    ui: &mut egui::Ui,
    tools: &mut ToolState,
    icons: &Option<tool_icons::Atlas>,
    avail: &SelectAvail,
) -> SelectOps {
    // The same in-row gap the Shape/Line panels use: EXACTLY the Tools
    // window's element gap (owner request 2026-11-20, the spacing pass).
    ui.spacing_mut().item_spacing.x = TOOL_GRID_GAP;
    ui.horizontal(|ui| {
        for (mode, icon) in [
            (SelectMode::Rect, SelectIcon::Rect),
            (SelectMode::Lasso, SelectIcon::Lasso),
            (SelectMode::Wand, SelectIcon::Wand),
        ] {
            let response = icon_button(
                ui,
                TOOL_BUTTON_PT,
                tool_icons::select_texture_id(icons, icon),
                tool_icons::select_icon_label(icon),
                tools.select_mode == mode,
            );
            if response.clicked() {
                tools.select_mode = mode;
            }
        }
    });
    // The wand's threshold slider (owner request 2026-11-23: "for the fill and the magic wand, add a
    // slider like the Size one that serves as the trigger threshold in %") — shown ONLY while the wand mode is
    // active (the rect/lasso drags have no threshold), the same shared
    // setting the Fill panel shows.
    if tools.select_mode == SelectMode::Wand {
        tolerance_block_ui(ui, tools);
    }
    ui.add_space(SHAPE_GROUP_GAP);
    let mut ops = SelectOps::default();
    ui.horizontal(|ui| {
        for (button, icon, enabled) in [
            (OpButton::Copy, SelectIcon::Copy, avail.has_selection),
            (OpButton::Cut, SelectIcon::Cut, avail.has_selection),
            (OpButton::Paste, SelectIcon::Paste, avail.has_clipboard),
            (OpButton::Crop, SelectIcon::Crop, avail.has_selection),
        ] {
            // Disabled = hover-only sense + the gray tint, via the enabled
            // variant of the shared icon button (owner request 2026-11-20:
            // "the same affordance the Edit menu's add_enabled rows give").
            let response = icon_button_state(
                ui,
                TOOL_BUTTON_PT,
                tool_icons::select_texture_id(icons, icon),
                tool_icons::select_icon_label(icon),
                false,
                enabled,
            );
            // The disabled state is a hover-sense rect: no clicks land, the
            // gray tint + the tooltip still show (owner request 2026-11-20).
            let clicked = enabled && response.clicked();
            match button {
                OpButton::Copy => ops.copy = clicked,
                OpButton::Cut => ops.cut = clicked,
                OpButton::Paste => ops.paste = clicked,
                OpButton::Crop => ops.crop = clicked,
            }
        }
    });
    ops
}
