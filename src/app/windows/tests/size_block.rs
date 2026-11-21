//! Tests of the settings panel's Size block (owner request 2026-11-20):
//! the "Size" label sits ABOVE the slider and the slider is narrowed to the
//! button grid's row width, the block separates from the Line/Shape button
//! rows by the Shape rows' own 12-pt step (the 9-pt group gap + egui's 3-pt
//! item spacing), and a size entered above 100 is NOT clamped back — the
//! slider itself stays limited to 1..=100 (it binds to a clamp copy and
//! writes back only when dragged; egui's default `SliderClamping::Always`
//! would otherwise snap the bound value every frame).

use super::super::SHAPE_GROUP_GAP;
use super::super::settings_panel::{
    SIZE_SLIDER_W, line_settings_ui, shape_settings_ui, size_block_ui,
};
use super::common::SCREEN;
use crate::tools::{Tool, ToolState};
use eframe::egui::{self, Pos2, Rect};

/// Runs `draw` (the exact UI the app draws) for two settle passes inside an
/// Area anchored at ZERO; returns the content rect from the second pass and
/// that pass's clipped paint shapes (the pixel-level assertions run on the
/// settled geometry — the same runner pattern the shape-rows probe uses).
fn run_two_passes(mut draw: impl FnMut(&mut egui::Ui)) -> (Rect, Vec<egui::epaint::ClippedShape>) {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut rect = Rect::NOTHING;
    let mut shapes = Vec::new();
    for pass in 0..2 {
        let mut output = ctx.run_pass(input.clone(), |ctx| {
            egui::Area::new(egui::Id::unique("size_block_probe"))
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    let min = ui.cursor().min;
                    draw(ui);
                    rect = Rect::from_min_size(min, ui.min_size());
                });
        });
        // No renderer here: the font-atlas deltas each pass hands back must
        // be explicitly discarded (`clear` is the sanctioned "drop
        // intentionally" path — epaint's TexturesDelta asserts when
        // unhandled deltas drop).
        output.textures_delta.clear();
        if pass == 1 {
            shapes = output.shapes;
        }
    }
    (rect, shapes)
}

/// The widest painted rect of the run: the slider's bar (nothing else in
/// the Size block paints that wide).
fn widest_rect(shapes: &[egui::epaint::ClippedShape]) -> Rect {
    shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::epaint::Shape::Rect(r) => Some(r.rect),
            _ => None,
        })
        .max_by_key(|r| (r.width() * 16.0) as i64)
        .expect("the slider paints its bar")
}

/// The painted text glyphs as (string, screen rect) pairs.
fn texts(shapes: &[egui::epaint::ClippedShape]) -> Vec<(String, Rect)> {
    shapes
        .iter()
        .filter_map(|cs| match &cs.shape {
            egui::epaint::Shape::Text(t) => Some((
                t.galley.text().to_owned(),
                Rect::from_min_size(t.pos, t.galley.size()),
            )),
            _ => None,
        })
        .collect()
}

/// The pass's headline rule: the "Size" label sits fully ABOVE the slider's
/// bar (the owner: "нужно саму надпись Size разместить над слайдером" — the
/// label-to-the-left slider made the element the panel's widest row), the
/// bar is narrowed to the button grid's row width ([`SIZE_SLIDER_W`]) and
/// the whole block stays far narrower than the old stretch.
#[test]
fn the_size_label_sits_above_a_slider_wide_as_the_button_grid() {
    let mut tools = ToolState::new();
    tools.size = 5.0;
    let (rect, shapes) = run_two_passes(|ui| {
        size_block_ui(ui, &mut tools, Tool::Brush);
    });

    let track = widest_rect(&shapes);
    assert!(
        (track.width() - SIZE_SLIDER_W).abs() <= 2.0,
        "bar width {} vs {SIZE_SLIDER_W}",
        track.width()
    );
    assert!(
        track.height() <= 12.0,
        "the bar is thin: {}",
        track.height()
    );

    let (label, label_rect) = texts(&shapes)
        .into_iter()
        .find(|(text, _)| text == "Size")
        .expect("the Size label paints");
    assert_eq!(label, "Size");
    assert!(
        label_rect.bottom() <= track.top() + 0.5,
        "the label must sit ABOVE the slider: label bottom {} vs bar top {}",
        label_rect.bottom(),
        track.top()
    );

    assert!(rect.width() < 160.0, "block width {}", rect.width());
}

/// Owner request 2026-11-20: between the Size block and the Line/Shape
/// button rows the SAME 12-pt step that separates the Shape panel's own
/// button rows ("такой же высоты что и между кнопками в Shape") — the
/// 9-pt [`SHAPE_GROUP_GAP`] plus egui's 3-pt item spacing. Measured as the
/// full panel's height minus the block's and the rows' own heights (the
/// widget-level gap, the same arithmetic the Shape rows' compact test
/// encodes as 9 + 3).
#[test]
fn the_size_block_clears_the_rows_gap_above_the_button_rows() {
    // The Shape panel, the app's exact sequence: size block, then the rows.
    let mut tools_full = ToolState::new();
    tools_full.tool = Tool::Shape;
    let (full, _) = run_two_passes(|ui| {
        size_block_ui(ui, &mut tools_full, Tool::Shape);
        shape_settings_ui(ui, &mut tools_full, &None);
    });
    let mut tools_rows = ToolState::new();
    tools_rows.tool = Tool::Shape;
    let (rows, _) = run_two_passes(|ui| {
        shape_settings_ui(ui, &mut tools_rows, &None);
    });
    let mut tools_size = ToolState::new();
    tools_size.tool = Tool::Brush;
    let (size_only, _) = run_two_passes(|ui| {
        size_block_ui(ui, &mut tools_size, Tool::Brush);
    });
    let gap = full.height() - rows.height() - size_only.height();
    let want = SHAPE_GROUP_GAP + 3.0;
    assert!((gap - want).abs() <= 0.75, "shape gap {gap} vs {want}");

    // The Line panel: the same rule, the owner asked for BOTH panels.
    let mut tools_full = ToolState::new();
    tools_full.tool = Tool::Line;
    let (full, _) = run_two_passes(|ui| {
        size_block_ui(ui, &mut tools_full, Tool::Line);
        line_settings_ui(ui, &mut tools_full, &None);
    });
    let mut tools_rows = ToolState::new();
    tools_rows.tool = Tool::Line;
    let (rows, _) = run_two_passes(|ui| {
        line_settings_ui(ui, &mut tools_rows, &None);
    });
    let mut tools_size = ToolState::new();
    tools_size.tool = Tool::Brush;
    let (size_only, _) = run_two_passes(|ui| {
        size_block_ui(ui, &mut tools_size, Tool::Brush);
    });
    let gap = full.height() - rows.height() - size_only.height();
    assert!((gap - want).abs() <= 0.75, "line gap {gap} vs {want}");
}

/// Owner request 2026-11-20: a size entered ABOVE 100 must survive — the
/// slider stays limited to 1..=100 but must NOT clamp the stored value back
/// (egui's default `SliderClamping::Always` would snap a bound 250 to 100
/// every frame; the block binds the slider to a clamp copy instead). The
/// 250 here stands in for a typed value: the number box writes this same
/// field. The slider still renders — thumb pinned at the 100 end — and the
/// row keeps the button-grid width.
#[test]
fn a_size_entered_above_100_survives_and_the_row_stays_narrow() {
    let mut tools = ToolState::new();
    tools.size = 250.0;
    let (rect, shapes) = run_two_passes(|ui| {
        size_block_ui(ui, &mut tools, Tool::Brush);
    });

    assert_eq!(
        tools.size, 250.0,
        "the entered size must not be clamped back"
    );

    let track = widest_rect(&shapes);
    assert!(
        (track.width() - SIZE_SLIDER_W).abs() <= 2.0,
        "bar width {} vs {SIZE_SLIDER_W}",
        track.width()
    );
    assert!(rect.width() < 160.0, "block width {}", rect.width());
}
