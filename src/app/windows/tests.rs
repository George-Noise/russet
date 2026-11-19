//! Tests of the magnetized tool-settings panel (owner request 2026-11-17):
//! the pure per-tool content decisions, and headless-egui regression tests
//! of the panel builder's magnet semantics — `Window::fixed_pos` re-applies
//! the position on EVERY frame, so no stored area position (a drag, a stale
//! state) can ever win and the panel cannot be detached from the Tools
//! window. Since the owner request 2026-11-19 also the collapse rule: while
//! the Tools window is collapsed to its title bar the panel hides with it.
//!
//! The headless tests run real egui passes through
//! [`egui::Context::run_pass`] — no renderer, no App; the window builders are
//! the exact functions the app uses.

use super::{
    settings_panel_shown, tool_has_settings, tool_settings_window_builder, tool_uses_size,
    tools_window_builder,
};
use crate::tools::Tool;
use eframe::egui::{self, Pos2, Rect, Vec2};

/// The screen every test runs on (bigger than the 500×400 main-window
/// minimum, so a mispositioned panel cannot accidentally look right).
const SCREEN: Vec2 = Vec2::new(1280.0, 800.0);

/// Shows the Tools window (the exact builder the app runs) for one pass and
/// returns its outer rect plus whether egui reported a body (`inner.is_some()`
/// — the flag the settings-panel visibility keys on).
fn show_tools_once(ctx: &egui::Context, input: &egui::RawInput) -> (Rect, bool) {
    let mut rect = Rect::NOTHING;
    let mut body_shown = false;
    let mut output = ctx.run_pass(input.clone(), |ctx| {
        let mut open = true;
        let shown = tools_window_builder(ctx, &mut open).show(ctx, |ui| {
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

/// Owner request 2026-11-19: while the Tools window is COLLAPSED to its
/// title bar (the collapse button / a title double-click), the magnetized
/// settings panel must hide with it. The guard keys on `Window::show`
/// reporting `inner: None` while collapsed (`window.rs` show-doc: "Returns
/// `Some(InnerResponse { inner: None })` if the window is collapsed") — this
/// pins that semantic to the exact builder the app runs (expanded: a body is
/// reported and the window is taller than its title bar; seeded collapsed via
/// the same `CollapsingState` egui stores under `<window id> + "collapsing"`:
/// NO body and a title-bar-only height), plus the pure decision both ways.
#[test]
fn the_settings_panel_hides_while_the_tools_window_is_collapsed() {
    // The pure decision: the panel needs BOTH a visible tools body AND a
    // tool with settings.
    assert!(settings_panel_shown(true, Tool::Brush));
    assert!(
        !settings_panel_shown(false, Tool::Brush),
        "collapsed: no panel"
    );
    assert!(
        !settings_panel_shown(true, Tool::Pencil),
        "no settings: no panel"
    );

    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };

    // Expanded (egui default): a body IS reported, the window is taller
    // than its title bar. Three passes — egui sizes a fresh window on the
    // frame after its first layout, so the settled geometry is asserted.
    let expanded_ctx = egui::Context::default();
    for _ in 0..3 {
        let (rect, body) = show_tools_once(&expanded_ctx, &input);
        assert!(body, "the expanded Tools window must report a body");
        assert!(rect.height() > 60.0, "expanded height {}", rect.height());
    }

    // Seeded COLLAPSED: NO body is reported and the window shrinks to its
    // title bar — the exact condition the panel guard hides on.
    let collapsed_ctx = egui::Context::default();
    let mut state = egui::collapsing_header::CollapsingState::load_with_default_open(
        &collapsed_ctx,
        super::tools_window_id().with("collapsing"),
        true,
    );
    state.set_open(false);
    state.store(&collapsed_ctx);
    for _ in 0..3 {
        let (rect, body) = show_tools_once(&collapsed_ctx, &input);
        assert!(!body, "the collapsed Tools window must report NO body");
        assert!(rect.height() < 60.0, "collapsed height {}", rect.height());
    }
}

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
    let grid_w = super::TOOL_COLUMNS as f32 * super::TOOL_BUTTON_PT
        + (super::TOOL_COLUMNS - 1) as f32 * super::TOOL_GRID_GAP;
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
                let frame = super::tools_title_frame(ctx);
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
            if let Some(click) = ctx.read_response(super::tools_window_id().with("__title_click")) {
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

#[test]
fn size_is_shown_only_for_the_size_affected_tools() {
    assert!(tool_uses_size(Tool::Brush));
    assert!(tool_uses_size(Tool::Eraser));
    assert!(tool_uses_size(Tool::Line));
    assert!(tool_uses_size(Tool::Shape));
    // The others get NO slider, not a disabled one (owner request
    // 2026-11-17).
    assert!(!tool_uses_size(Tool::Pencil));
    assert!(!tool_uses_size(Tool::Eyedropper));
    assert!(!tool_uses_size(Tool::Text));
}

#[test]
fn pencil_and_eyedropper_draw_no_settings_panel() {
    for tool in [Tool::Pencil, Tool::Eyedropper] {
        assert!(!tool_has_settings(tool), "{tool:?} must show no panel");
    }
    for tool in [
        Tool::Brush,
        Tool::Eraser,
        Tool::Line,
        Tool::Shape,
        Tool::Text,
    ] {
        assert!(tool_has_settings(tool), "{tool:?} must show the panel");
    }
}

/// Shows the settings panel for one pass at `pos` and returns its outer rect.
fn show_once(ctx: &egui::Context, input: &egui::RawInput, pos: Pos2) -> Rect {
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

#[test]
fn the_settings_panel_follows_its_fixed_pos_every_frame() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let settled = show_once(&ctx, &input, Pos2::new(20.0, 30.0));
    let _ = show_once(&ctx, &input, Pos2::new(20.0, 30.0));

    // The magnet: even though the area state now carries the old position,
    // the builder's `fixed_pos` wins — the panel lands exactly where the
    // next position says, so it can never be dragged off the Tools window.
    let moved = show_once(&ctx, &input, Pos2::new(500.0, 300.0));
    assert_eq!(moved.min, Pos2::new(500.0, 300.0));
    assert_eq!(
        moved.size(),
        settled.size(),
        "the panel size must not change with the position"
    );
}

#[test]
fn the_settings_panel_hugs_its_content() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut rect = Rect::NOTHING;
    // egui sizes a fresh window on the frame after its first layout, so
    // run three passes and assert the settled geometry.
    for _ in 0..3 {
        rect = show_once(&ctx, &input, Pos2::new(20.0, 30.0));
    }
    // One label is ~20 pt tall; egui's 420-pt default outer height (or
    // anything near the 800-pt screen) means the window did NOT hug its
    // content.
    assert!(rect.height() < 100.0, "panel height {}", rect.height());
    // The default outer width is 340 pt; anything screen-wide means a
    // first-pass available-width leak.
    assert!(rect.width() < 400.0, "panel width {}", rect.width());
}
