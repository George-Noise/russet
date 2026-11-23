//! Tests of the Select tool's magnetized-panel rows (owner request
//! 2026-11-20): the REAL `select_settings_ui` runs headless — the MODE row
//! and the OPS row separate by the Shape panels' 12-pt group step, the ops'
//! disabled state follows the availability (no clicks land without a
//! selection / a clipboard), and an injected click on a mode button writes
//! `ToolState::select_mode`.

use super::super::select_panel::{SelectAvail, SelectOps, select_settings_ui};
use super::super::{SHAPE_GROUP_GAP, TOOL_BUTTON_PT};
use super::common::SCREEN;
use crate::tools::{SelectMode, ToolState};
use eframe::egui::{self, Pos2, Rect, Shape};

/// Extracts the filled-button rects from one pass's shapes (icon_button
/// fills its square — the icon/stroke shapes carry no fill).
fn button_rects(shapes: &[egui::epaint::ClippedShape]) -> Vec<Rect> {
    shapes
        .iter()
        .filter_map(|clipped| match &clipped.shape {
            Shape::Rect(rect) if rect.fill != egui::Color32::TRANSPARENT => Some(rect.rect),
            _ => None,
        })
        .collect()
}

/// Runs the REAL panel fn for three passes: layout, then a settled pass
/// whose shapes yield the button rects, then a pass with `events` injected
/// (the click targets come from the settled rects). Returns the last pass's
/// op clicks and the settled button rects.
fn run_panel(
    tools: &mut ToolState,
    avail: SelectAvail,
    events: &[egui::Event],
) -> (SelectOps, Vec<Rect>) {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut rects = Vec::new();
    let mut ops = SelectOps::default();
    for pass in 0..3 {
        let mut this_input = input.clone();
        if pass == 2 {
            this_input.events = events.to_vec();
        }
        let mut output = ctx.run_pass(this_input, |ctx| {
            egui::Area::new(egui::Id::unique("select_panel_probe"))
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    ops = select_settings_ui(ui, tools, &None, &avail);
                });
        });
        if pass == 1 {
            rects = button_rects(&output.shapes);
        }
        output.textures_delta.clear();
    }
    (ops, rects)
}

/// The press-release click pair at a rect's center.
fn click_at(rect: Rect) -> Vec<egui::Event> {
    let pos = rect.center();
    vec![
        egui::Event::PointerMoved(pos),
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::default(),
        },
        egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::default(),
        },
    ]
}

#[test]
fn the_panel_rows_keep_the_group_gap_between_mode_and_ops() {
    let mut tools = ToolState::new();
    let (ops, rects) = run_panel(
        &mut tools,
        SelectAvail {
            has_selection: true,
            has_clipboard: true,
        },
        &[],
    );

    assert_eq!(rects.len(), 7, "3 mode + 4 ops buttons");
    assert!(
        !ops.copy && !ops.cut && !ops.paste && !ops.crop,
        "no clicks without interaction"
    );
    // The rows: the mode row's buttons share a y; the ops row sits the
    // Shape panels' 12-pt step below (the group gap + egui's item spacing).
    let mode_y = rects[0].top();
    let ops_y = rects[3].top();
    let gap = ops_y - (mode_y + TOOL_BUTTON_PT);
    assert!(
        (gap - SHAPE_GROUP_GAP - 3.0).abs() < 1.0,
        "the rows separate by the 12-pt step, got {gap}"
    );
    // All buttons are the Tools window's button size.
    assert!(rects.iter().all(|r| r.width() == TOOL_BUTTON_PT));
}

#[test]
fn disabled_ops_take_no_clicks_and_report_none() {
    let mut tools = ToolState::new();
    let (ops, rects) = run_panel(
        &mut tools,
        SelectAvail {
            has_selection: false,
            has_clipboard: false,
        },
        &[],
    );
    assert!(!ops.copy && !ops.cut && !ops.paste && !ops.crop);
    assert_eq!(rects.len(), 7, "the buttons still draw (grayed)");
}

#[test]
fn an_injected_click_on_a_mode_button_writes_select_mode() {
    let mut tools = ToolState::new();
    assert_eq!(tools.select_mode, SelectMode::Rect);
    // Pass 1 settles the rects; the events inject in pass 2 at the LASSO
    // button's center (rects[1] in the draw order: Rect, Lasso).
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut rects = Vec::new();
    for pass in 0..3 {
        let mut this_input = input.clone();
        if pass == 2
            && let Some(rect) = rects.get(1)
        {
            this_input.events = click_at(*rect);
        }
        let mut output = ctx.run_pass(this_input, |ctx| {
            egui::Area::new(egui::Id::unique("select_panel_click_probe"))
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    let avail = SelectAvail {
                        has_selection: false,
                        has_clipboard: false,
                    };
                    let _ = select_settings_ui(ui, &mut tools, &None, &avail);
                });
        });
        if pass == 1 {
            rects = button_rects(&output.shapes);
        }
        output.textures_delta.clear();
    }
    assert_eq!(
        tools.select_mode,
        SelectMode::Lasso,
        "the lasso button's click wrote the mode"
    );
}
