//! The per-tool icon DRAWINGS tests (owner request 2026-11-18; the

//! Shape-panel set lives in [`super::shape_panel`]): colors, geometry and

//! the tooltip labels, judged through the pure rasterization pipeline.

use super::super::{SelectIcon, label, rasterize_select_icon, rasterize_tool_icon};
use super::*;
use crate::tools::Tool;
use eframe::egui::Color32;

#[test]
fn pencil_is_the_classic_yellow_wood_graphite_eraser_pencil() {
    let pm = rasterize_tool_icon(Tool::Pencil, 48, Color32::WHITE).expect("px is in 1..=256");
    assert!(has_color(&pm, (253, 194, 92), 6), "yellow body missing");
    assert!(
        has_color(&pm, (236, 159, 43), 6),
        "darker body facet missing"
    );
    assert!(has_color(&pm, (231, 196, 151), 6), "wood cone missing");
    assert!(has_color(&pm, (60, 60, 60), 6), "black graphite missing");
    assert!(has_color(&pm, (240, 144, 138), 6), "eraser end missing");
    assert!(has_color(&pm, (175, 180, 189), 6), "metal ferrule missing");
}

#[test]
fn brush_is_wood_handle_metal_ferrule_and_bristles() {
    let pm = rasterize_tool_icon(Tool::Brush, 48, Color32::WHITE).expect("px is in 1..=256");
    assert!(
        has_color(&pm, (180, 113, 60), 6),
        "varnished-wood handle missing"
    );
    assert!(has_color(&pm, (185, 190, 198), 6), "metal ferrule missing");
    assert!(has_color(&pm, (216, 169, 107), 6), "bristles missing");
    assert!(has_color(&pm, (110, 74, 40), 6), "loaded tip missing");
}

#[test]
fn eraser_is_the_classic_pink_slab() {
    let pm = rasterize_tool_icon(Tool::Eraser, 48, Color32::WHITE).expect("px is in 1..=256");
    assert!(
        has_color(&pm, (249, 183, 196), 6),
        "light pink face missing"
    );
    assert!(has_color(&pm, (239, 140, 164), 6), "main pink face missing");
}

#[test]
fn pipette_has_rubber_bulb_glass_and_liquid() {
    let pm = rasterize_tool_icon(Tool::Eyedropper, 48, Color32::WHITE).expect("px is in 1..=256");
    assert!(has_color(&pm, (217, 87, 68), 6), "rubber bulb missing");
    assert!(has_color(&pm, (95, 168, 220), 6), "liquid stripe missing");
    assert!(has_color(&pm, (213, 223, 231), 6), "glass barrel missing");
    assert!(
        has_color(&pm, (94, 103, 112), 6),
        "the wide metal collar missing"
    );
}

#[test]
fn monochrome_icons_follow_the_given_text_color() {
    // The SAME pipeline, the color comes from the theme (owner request:
    // no duplicated draw code for colored vs monochrome icons). The pixels
    // come back PREMULTIPLIED, so the comparison runs on the
    // demultiplied (straight) color; a thin AA stroke may never reach full
    // pixel coverage, which premultiplication would otherwise scale.
    for tool in [Tool::Line, Tool::Shape, Tool::Text] {
        for text in [Color32::BLACK, Color32::WHITE] {
            let pm = rasterize_tool_icon(tool, 32, text).expect("px is in 1..=256");
            let opaque: Vec<_> = pm
                .pixels()
                .iter()
                .filter(|p| p.alpha() > 220)
                .map(|p| p.demultiply())
                .collect();
            assert!(!opaque.is_empty(), "{tool:?} must draw opaque pixels");
            for p in &opaque {
                assert!(
                    p.red().abs_diff(text.r()) <= 10
                        && p.green().abs_diff(text.g()) <= 10
                        && p.blue().abs_diff(text.b()) <= 10,
                    "{tool:?} must be monochrome in the given text color"
                );
            }
        }
    }
}

#[test]
fn labels_carry_the_name_and_the_keyboard_key() {
    let keys = [
        (Tool::Pencil, "P", "Pencil"),
        (Tool::Brush, "B", "Brush"),
        (Tool::Eraser, "E", "Eraser"),
        (Tool::Eyedropper, "I", "Eyedropper"),
        (Tool::Line, "L", "Line"),
        (Tool::Shape, "U", "Shape"),
        (Tool::Text, "T", "Text"),
    ];
    for (tool, key, name) in keys {
        let label = label(tool);
        assert_eq!(
            label,
            &format!("{name} ({key})"),
            "tooltip label of {tool:?}"
        );
    }
}

/// The v2 refinements (owner request 2026-11-18 after the V35 pass).
#[test]
fn line_icon_is_a_curve_bulging_off_the_chord() {
    let pm = rasterize_tool_icon(Tool::Line, 48, Color32::BLACK).expect("px is in 1..=256");
    // The S of two quadratic arcs deviates from the straight chord on BOTH
    // sides: the first arc dips to ≈(11.6, 16.9) — the chord passes y ≈ 12.1
    // there — and the second rises to ≈(15.9, 6.2) against a chord y ≈ 8.0.
    assert!(
        has_opaque_near(&pm, (11.6, 16.9), 1.0) && has_opaque_near(&pm, (15.9, 6.2), 1.0),
        "the S-curve's bends are missing — the icon reads as a straight line"
    );
    assert!(
        has_opaque_near(&pm, (4.8, 18.6), 1.0) && has_opaque_near(&pm, (19.2, 4.8), 1.0),
        "the curve must span the icon's diagonal"
    );
}

#[test]
fn shape_icon_is_a_rectangle_and_a_circle_not_a_diamond() {
    let pm = rasterize_tool_icon(Tool::Shape, 48, Color32::BLACK).expect("px is in 1..=256");
    // On the circle's 45° diagonal the diamond HAD ink (its edge band ran
    // through Manhattan distance 4.05..5.95; the point below sits at
    // Manhattan 4.24 — on the old edge — but at Euclidean 3.0, deep inside
    // the circle's empty hole).
    assert!(
        has_opaque_near(&pm, (19.1, 14.1), 0.8),
        "the circle's 45° arc is missing"
    );
    assert!(
        !has_opaque_near(&pm, (17.72, 8.48), 0.4),
        "the diamond's edge is still there — the circle did not replace it"
    );
    assert!(
        has_opaque_near(&pm, (3.6, 9.2), 0.5) && has_opaque_near(&pm, (13.8, 19.4), 0.5),
        "the rectangle half is missing"
    );
}

#[test]
fn text_icon_is_a_serif_curved_t_with_a_small_t() {
    let pm = rasterize_tool_icon(Tool::Text, 48, Color32::BLACK).expect("px is in 1..=256");
    // The serif hooks dip BELOW the bar line (the old plain bar had nothing
    // there outside the stem).
    assert!(
        has_opaque_near(&pm, (4.0, 9.8), 0.8),
        "the left serif hook is missing"
    );
    assert!(
        has_opaque_near(&pm, (14.2, 9.8), 0.8),
        "the right serif hook is missing"
    );
    // The small lowercase t: ink on its stem and its crossbar.
    assert!(
        has_opaque_near(&pm, (17.2, 11.0), 0.7) && has_opaque_near(&pm, (16.6, 13.0), 0.9),
        "the small lowercase t is missing"
    );
}

/// The Shape panel's icons (owner request 2026-11-20): every one draws a
/// substantial mark for BOTH kinds, the tooltips carry EXACTLY the old
/// labels, and the KIND buttons are the neutral gray previews.

#[test]
fn colored_outlines_are_dark_in_both_themes() {
    // Owner v3 (after the V36 pass): ONE dark rim in BOTH themes — the tool
    // buttons carry the light-theme background in the dark theme
    // (windows.rs::tool_button), so the rim never needs to follow the theme.
    // The Pencil's silhouette carries the rim; the raster color (which
    // tracks the theme for the monochrome icons) must not change it.
    for raster in [Color32::WHITE, Color32::BLACK] {
        let pm = rasterize_tool_icon(Tool::Pencil, 48, raster).expect("px is in 1..=256");
        assert!(
            has_color(&pm, (75, 58, 40), 10),
            "the dark rim is missing (raster {raster:?})"
        );
        assert!(
            !has_color(&pm, (216, 222, 228), 10),
            "the v2 light rim is gone — no theme-aware rims anymore"
        );
    }
    // The Eraser's rim joins the family (owner v4: it carried a light pink
    // outline that blended with its faces and broke the set's style).
    let eraser = rasterize_tool_icon(Tool::Eraser, 48, Color32::WHITE).expect("px is in 1..=256");
    assert!(
        has_color(&eraser, (92, 52, 66), 10),
        "the eraser needs the dark plum rim"
    );
    assert!(
        !has_color(&eraser, (180, 90, 116), 10),
        "the old pink rim is gone"
    );
}

#[test]
fn fill_is_the_metal_pail_with_liquid_band_and_drop() {
    // Owner request 2026-11-23: the bucket joins the Tools grid LAST —
    // the pour glyph: gray body, dark rim, the pipette's liquid blue.
    let pm = rasterize_tool_icon(Tool::Fill, 48, Color32::WHITE).expect("px is in 1..=256");
    assert!(has_color(&pm, (168, 178, 188), 6), "the pail body missing");
    assert!(has_color(&pm, (94, 103, 112), 6), "the dark rim missing");
    assert!(has_color(&pm, (95, 168, 220), 6), "the liquid blue missing");
    assert!(has_color(&pm, (52, 58, 64), 6), "the ink outline missing");
}

#[test]
fn the_wand_mark_is_a_rod_with_a_four_point_sparkle() {
    // The wand's panel row + the Select button's mode-aware mark —
    // monochrome ink like the rest of the Select family.
    let ink = Color32::from_gray(64);
    let pm = rasterize_select_icon(SelectIcon::Wand, 48, ink).expect("px is in 1..=256");
    assert!(
        has_color(&pm, (64, 64, 64), 8),
        "the ink rod/sparkle missing"
    );
}
