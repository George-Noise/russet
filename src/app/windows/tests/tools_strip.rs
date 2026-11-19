//! Regression test of the Tools window's title strip fit (owner request
//! 2026-11-19, "variant A2"): the strip must stay INSIDE the window —
//! with egui's default title frame it painted ~11 pt past the window's
//! right edge and the ✕ landed half outside it (AGENTS.md quirk,
//! pixel-measured headless AND under Xvfb).

use super::super::{
    TOOL_BUTTON_PT, TOOL_COLUMNS, TOOL_GRID_GAP, tools_title_frame, tools_window_builder,
    tools_window_id,
};
use super::common::SCREEN;
use eframe::egui::{self, Pos2, Rect};

/// Owner request 2026-11-19 (the owner picked "variant A2"): the Tools
/// window's title strip must fit INSIDE the window — with egui's default
/// title frame the both-buttons strip painted ~11 pt past the window's right
/// edge and the ✕ landed half outside it (AGENTS.md quirk, pixel-measured).
///
/// Two guards. (1) The GEOMETRY proof, measured on the output of the exact
/// builder the app runs: `title_ui` makes the strip's drag response
/// (`area_id.with("__title_click")` — the id AGENTS.md documents) span from
/// the collapse ICON's right edge to the close ICON's left edge, where each
/// icon is its button atom shrunk by `(atom - icon_width) / 2` per side — so
/// the ▼ atom's left edge is `click.min_x - (heading_h + icon_width) / 2` and
/// the ✕ atom's right edge is `click.max_x + (heading_h + icon_width) / 2`.
/// Both must stay within the window — with the default 6-pt side padding the
/// ✕ atom's right edge lands ~4.3 pt past the window's right edge (the
/// pixel-measured "✕ half outside it" of the AGENTS.md quirk). (2) The
/// variant-A2 contract of `tools_title_frame`: the strip's horizontal padding
/// is zero and the vertical padding keeps egui's window margin (the strip's
/// height is unchanged — variant A, `Margin::ZERO`, shrank it), which is
/// exactly what makes the two atoms fit the grid-width window (the strip's
/// minimum content: 2 Heading-height button atoms + 4 item-spacing gaps of
/// the empty-title strip, `window.rs::title_ui`).
#[test]
fn the_title_strip_fits_inside_the_tools_window() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };

    let mut window_rect = Rect::NOTHING;
    let mut click_rect = Rect::NOTHING;
    let mut click_found = false;
    // The button atoms are Heading-row-height squares; their icons shrink
    // them by `(atom - icon_width) / 2` per side (`window.rs::title_ui`).
    let mut atom_side_offset = 0.0;
    let mut horizontal_padding = 0;
    let mut vertical_padding = (0, 0);
    let mut window_margin_vertical = (0, 0);
    // The app's body width: the two-column tool grid — the widest widget, so
    // it is what fixes the window's outer width.
    let grid_w = TOOL_COLUMNS as f32 * TOOL_BUTTON_PT + (TOOL_COLUMNS - 1) as f32 * TOOL_GRID_GAP;
    // egui sizes a fresh window on the frame after its first layout, so run
    // three passes and assert the settled geometry.
    for _ in 0..3 {
        let mut output = ctx.run_pass(input.clone(), |ctx| {
            let mut open = true;
            let shown = tools_window_builder(ctx, &mut open).show(ctx, |ui| {
                ui.allocate_exact_size(egui::vec2(grid_w, 10.0), egui::Sense::hover());
                // The same atom math `title_ui` runs: the atoms are
                // Heading-row-height squares, the icons shrink them.
                let heading_h =
                    ui.fonts_mut(|f| f.row_height(&egui::TextStyle::Heading.resolve(ui.style())));
                atom_side_offset = (heading_h + ui.spacing().icon_width) / 2.0;
                // The variant-A2 contract of the frame the builder feeds
                // `Window::title_frame` with.
                let frame = tools_title_frame(ctx);
                horizontal_padding = frame.inner_margin.left + frame.inner_margin.right;
                vertical_padding = (frame.inner_margin.top, frame.inner_margin.bottom);
                let margin = ctx.global_style().spacing.window_margin;
                window_margin_vertical = (margin.top, margin.bottom);
            });
            if let Some(shown) = shown {
                window_rect = shown.response.rect;
            }
            // The strip's drag strip: the collapse atom's right edge .. the
            // close atom's left edge (window.rs::title_ui's title_click_rect).
            if let Some(click) = ctx.read_response(tools_window_id().with("__title_click")) {
                click_rect = click.rect;
                click_found = true;
            }
        });
        // No renderer here: the font-atlas deltas each pass hands back must be
        // explicitly discarded (`clear` is the sanctioned "drop intentionally"
        // path — epaint's TexturesDelta asserts when unhandled deltas drop).
        output.textures_delta.clear();
    }

    assert!(click_found, "the title strip's drag response was not found");
    let close_right = click_rect.max.x + atom_side_offset;
    let collapse_left = click_rect.min.x - atom_side_offset;
    assert!(
        close_right <= window_rect.max.x + 0.5,
        "the close atom's right edge {close_right} must not pass the window's \
         right edge {} (the default title frame put the ✕ half outside)",
        window_rect.max.x
    );
    assert!(
        collapse_left >= window_rect.min.x - 0.5,
        "the collapse atom's left edge {collapse_left} must not pass the \
         window's left edge {}",
        window_rect.min.x
    );
    assert_eq!(horizontal_padding, 0, "variant A2: no side padding");
    assert_eq!(
        vertical_padding, window_margin_vertical,
        "variant A2: the strip keeps egui's vertical window padding"
    );
}
