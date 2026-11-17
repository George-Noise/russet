//! Tests of the magnetized tool-settings panel (owner request 2026-11-17):
//! the pure per-tool content decisions, and headless-egui regression tests
//! of the panel builder's magnet semantics — `Window::fixed_pos` re-applies
//! the position on EVERY frame, so no stored area position (a drag, a stale
//! state) can ever win and the panel cannot be detached from the Tools
//! window.
//!
//! The headless tests run real egui passes through
//! [`egui::Context::run_pass`] — no renderer, no App; the window builder is
//! the exact function the app uses.

use super::{tool_has_settings, tool_settings_window_builder, tool_uses_size};
use crate::tools::Tool;
use eframe::egui::{self, Pos2, Rect, Vec2};

/// The screen every test runs on (bigger than the 500×400 main-window
/// minimum, so a mispositioned panel cannot accidentally look right).
const SCREEN: Vec2 = Vec2::new(1280.0, 800.0);

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
