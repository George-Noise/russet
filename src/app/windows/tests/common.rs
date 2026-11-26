//! Shared helpers of the `windows` test topics: the screen size and the
//! headless one-pass runners. A real [`egui::Context::run_pass`] per call —
//! no renderer, no App; the window builders are the exact functions the app
//! uses.

use super::super::settings_panel::tool_settings_window_builder;
use super::super::tools_window_builder;
use eframe::egui::{self, Pos2, Rect, Vec2};

/// The screen every test runs on (bigger than the 500×400 main-window
/// minimum, so a mispositioned panel cannot accidentally look right).
pub(super) const SCREEN: Vec2 = Vec2::new(1280.0, 800.0);

/// Shows the Tools window (the exact builder the app runs) for one pass and
/// returns its outer rect plus whether egui reported a body (`inner.is_some()`
/// — the flag the settings-panel visibility keys on).
pub(super) fn show_tools_once(ctx: &egui::Context, input: &egui::RawInput) -> (Rect, bool) {
    let mut rect = Rect::NOTHING;
    let mut body_shown = false;
    let mut output = ctx.run_pass(input.clone(), |ctx| {
        let mut open = true;
        let shown = tools_window_builder(ctx, &mut open, false).show(ctx, |ui| {
            ui.label("tools");
        });
        if let Some(response) = &shown {
            rect = response.response.rect;
            body_shown = response.inner.is_some();
        }
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (`clear` is the sanctioned "drop intentionally"
    // path — epaint's TexturesDelta asserts when unhandled deltas drop).
    output.textures_delta.clear();
    (rect, body_shown)
}

/// Shows the settings panel for one pass at `pos` and returns its outer rect.
pub(super) fn show_once(ctx: &egui::Context, input: &egui::RawInput, pos: Pos2) -> Rect {
    let mut rect = Rect::NOTHING;
    let mut output = ctx.run_pass(input.clone(), |ctx| {
        let response = tool_settings_window_builder(pos)
            .show(ctx, |ui| {
                ui.label("Size");
            })
            .expect("the settings panel shows");
        rect = response.response.rect;
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (`clear` is the sanctioned "drop intentionally"
    // path — epaint's TexturesDelta asserts when unhandled deltas drop).
    output.textures_delta.clear();
    rect
}
