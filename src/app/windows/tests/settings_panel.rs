//! Tests of the magnetized tool-settings panel (owner request 2026-11-17):
//! the pure per-tool content decisions, and headless-egui regression tests
//! of the panel builder's magnet semantics — `Window::fixed_pos` re-applies
//! the position on EVERY frame, so no stored area position (a drag, a stale
//! state) can ever win and the panel cannot be detached from the Tools
//! window. Since the owner request 2026-11-19 also the collapse rule: while
//! the Tools window is collapsed to its title bar the panel hides with it.

use super::super::{
    settings_panel_shown, shape_settings_ui, toggled_fill, toggled_outline, tool_has_settings,
    tool_uses_size,
};
use super::common::{SCREEN, show_once, show_tools_once};
use crate::tools::Tool;
use eframe::egui::{self, Pos2, Rect};

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
        super::super::tools_window_id().with("collapsing"),
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

/// The Shape panel's toggle rule (owner request 2026-11-10, now driven by
/// the icon toggles — owner request 2026-11-20): flipping a mode OFF
/// re-checks the OTHER mode, so the tool can never end up drawing nothing;
/// flipping a mode ON keeps the other as it was. PURE — the exact helpers
/// the panel's click handlers call.
#[test]
fn toggling_a_shape_mode_off_re_checks_the_other() {
    assert_eq!(toggled_fill(true, true), (false, true));
    assert_eq!(toggled_outline(true, true), (true, false));
    assert_eq!(toggled_fill(false, true), (true, true));
    assert_eq!(toggled_outline(true, false), (true, true));
    for (fill, outline) in [(true, true), (false, true), (true, false)] {
        let (f, o) = toggled_fill(fill, outline);
        assert!(f || o, "fill flip from ({fill}, {outline})");
        let (f, o) = toggled_outline(fill, outline);
        assert!(f || o, "outline flip from ({fill}, {outline})");
    }
}

/// The Shape panel's two icon rows (owner request 2026-11-20) render
/// COMPACT headless — the exact `shape_settings_ui` the app runs, atlas
/// omitted (the buttons draw without icons the same way the Tools window
/// does before the atlas lands). A leaked width/height (egui's 340-pt
/// default window width, the 800-pt screen) fails here.
#[test]
fn the_shape_rows_render_compact_headless() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut rect = Rect::NOTHING;
    // Two passes: egui settles an area's geometry on the second one.
    for _ in 0..2 {
        let mut output = ctx.run_pass(input.clone(), |ctx| {
            let mut tools = crate::tools::ToolState::new();
            tools.tool = Tool::Shape;
            egui::Area::new(egui::Id::unique("shape_rows_probe"))
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    let min = ui.cursor().min;
                    shape_settings_ui(ui, &mut tools, &None);
                    rect = Rect::from_min_size(min, ui.min_size());
                });
        });
        output.textures_delta.clear();
    }
    // Two 28-pt button rows + the group gap — nothing near a full window.
    assert!(rect.height() < 80.0, "shape rows height {}", rect.height());
    assert!(rect.width() < 100.0, "shape rows width {}", rect.width());
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
