//! Tests of the tool-icon rasterization (owner request 2026-11-18): the
//! pure tiny-skia pipeline (colored + monochrome icons through the SAME
//! primitives) and the atlas lifecycle — rasterized once per scale change
//! (theme-independent since the v4 pass), served as cached egui textures.
//! The headless part runs real egui passes through [`egui::Context::run_pass`]
//! (no renderer, no App), the same pattern the dialog tests use.

use eframe::egui::{Color32, Context, RawInput, TextureHandle, Theme, Visuals};

use super::{TOOL_INK, TOOLS, cursor_bitmap, ensure, icon_transform, label, rasterize_tool_icon};
use crate::tools::Tool;

/// Opaque-pixel threshold: every icon must draw a substantial mark in its
/// 24×24 logical space (guards against a silently-empty raster).
const MIN_OPAQUE_PIXELS: usize = 40;

fn count_opaque(pm: &tiny_skia::Pixmap) -> usize {
    pm.pixels().iter().filter(|p| p.alpha() > 0).count()
}

fn has_color(pm: &tiny_skia::Pixmap, rgb: (u8, u8, u8), tol: u8) -> bool {
    let (r, g, b) = rgb;
    pm.pixels().iter().any(|p| {
        p.alpha() > 200
            && p.red().abs_diff(r) <= tol
            && p.green().abs_diff(g) <= tol
            && p.blue().abs_diff(b) <= tol
    })
}

#[test]
fn every_tool_rasterizes_with_a_substantial_mark() {
    for tool in TOOLS {
        let pm = rasterize_tool_icon(tool, 24, Color32::WHITE).expect("px is in 1..=256");
        assert!(
            count_opaque(&pm) >= MIN_OPAQUE_PIXELS,
            "{tool:?} must draw more than {MIN_OPAQUE_PIXELS} opaque pixels"
        );
    }
}

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

#[test]
fn cursor_bitmap_is_straight_rgba_with_the_hotspot_at_the_pipette_tip() {
    let pm = rasterize_tool_icon(Tool::Eyedropper, 24, Color32::WHITE).expect("px is in 1..=256");
    let image = cursor_bitmap(&pm).expect("a 24×24 pixmap always converts");
    let (w, h) = (pm.width() as usize, pm.height() as usize);
    assert_eq!(
        image.rgba.len(),
        w * h * 4,
        "straight RGBA, 4 bytes per pixel"
    );
    assert_eq!(image.size, [w as u16, h as u16]);
    assert!(
        image.hotspot[0] < w as u16 && image.hotspot[1] < h as u16,
        "the hotspot must sit inside the bitmap"
    );
    // The hotspot is the pipette tip mapped by the same transform the
    // drawing used (scale ∘ rotate-about-center).
    let mut tip = tiny_skia::Point::from_xy(super::PIPETTE_TIP.0, super::PIPETTE_TIP.1);
    icon_transform(w, super::TOOL_ANGLE).map_point(&mut tip);
    assert_eq!(
        image.hotspot,
        [tip.x.round() as u16, tip.y.round() as u16],
        "the hotspot must be the pipette tip's pixel"
    );
    // Straight (non-premultiplied) encoding: an AA edge pixel's straight
    // components are >= its premultiplied ones.
    for p in pm.pixels() {
        if p.alpha() > 0 && p.alpha() < 255 {
            let straight = p.demultiply();
            assert!(straight.red() >= p.red() && straight.green() >= p.green());
            break;
        }
    }
}

#[test]
fn the_icon_transform_maps_the_logical_space_onto_the_raster() {
    let ts = icon_transform(48, 0.0);
    assert_eq!(ts.sx, 2.0, "48 px over the 24 pt logical space = ×2");
    let ts45 = icon_transform(24, super::TOOL_ANGLE);
    // The rotation about the center keeps the center fixed.
    let mut c = tiny_skia::Point::from_xy(12.0, 12.0);
    ts45.map_point(&mut c);
    assert!((c.x - 12.0).abs() < 1e-3 && (c.y - 12.0).abs() < 1e-3);
}

/// The atlas lifecycle: built on the first frame, untouched while the ppp
/// key matches, rebuilt when the scale changes. THEME-INDEPENDENT since the
/// v4 pass (fixed ink + fixed rims): a theme switch must not rebuild it.
#[test]
fn atlas_rebuilds_only_on_scale_change() {
    let ctx = Context::default();
    // ensure() loads textures; headless egui hands the deltas back with
    // every pass and asserts when they drop unhandled (the dialog tests'
    // convention: explicitly clear them). The return value is how many
    // textures the pass uploaded — 0 means nothing re-rasterized. The
    // first pass also uploads egui's font atlas (8); later passes carry
    // only what re-rasterized, so the 0-assertions below stay a clean
    // "nothing rebuilt" discriminator.
    fn pass(ctx: &Context) -> usize {
        let mut delta = ctx.run_pass(RawInput::default(), |_| {}).textures_delta;
        let uploaded = delta.set.len();
        delta.clear();
        uploaded
    }
    let mut atlas = None;
    ensure(&mut atlas, &ctx);
    assert_eq!(pass(&ctx), 8, "7 tool icons + egui's font atlas");
    let first = atlas.as_ref().expect("the first ensure builds the atlas");
    assert_eq!(first.key, 1.0, "egui's headless default: ppp 1.0");
    let cursor0 = first.pipette_cursor.as_ref().expect("the pipette cursor");
    assert_eq!(cursor0.size, [24, 24]);

    // Same conditions: a cheap no-op (the key match must keep the atlas).
    ensure(&mut atlas, &ctx);
    assert_eq!(pass(&ctx), 0);
    assert_eq!(atlas.as_ref().expect("kept").key, 1.0);

    // Scale change: the WHOLE atlas re-rasterizes (24 pt × 2.0 = 48 px).
    // set_pixels_per_point becomes active at the START of the next pass,
    // so a pass runs before ensure() can observe the new scale.
    ctx.set_pixels_per_point(2.0);
    pass(&ctx);
    ensure(&mut atlas, &ctx);
    assert_eq!(
        pass(&ctx),
        7,
        "rebuilt after the scale change (the font atlas only uploads once)"
    );
    let scaled = atlas.as_ref().expect("rebuilt after the scale change");
    assert_eq!(scaled.key, 2.0);
    let cursor1 = scaled
        .pipette_cursor
        .as_ref()
        .expect("cursor re-rasterized");
    assert_eq!(cursor1.size, [48, 48]);

    // Theme change: NOT a rebuild anymore — the fixed ink and the fixed
    // rims make the icons theme-independent (owner v4: one drawing reads
    // well in both themes; the old theme re-raster died with its reason).
    // The pass still uploads ONE texture: egui's font atlas follows the
    // theme too. OUR seven icons would make it 8 — the count stays the
    // "did the atlas rebuild?" discriminator.
    ctx.set_theme(Theme::Light);
    ensure(&mut atlas, &ctx);
    assert_eq!(pass(&ctx), 1, "only egui's font atlas, not our 7 icons");
    assert_eq!(atlas.as_ref().expect("kept").key, 2.0);
}

/// The monochrome ink is FIXED (owner v4) and sits a bit BELOW the light
/// theme's text color (egui light's noninteractive text, gray 80) — "the
/// light-theme color, just a bit darker".
#[test]
fn mono_ink_is_fixed_below_the_light_theme_text() {
    let light_text = Visuals::light().widgets.noninteractive.fg_stroke.color;
    assert_eq!(
        light_text,
        Color32::from_gray(80),
        "egui light's text color"
    );
    assert!(
        TOOL_INK.r() < light_text.r() && TOOL_INK.r() > 32,
        "TOOL_INK = {:?}",
        TOOL_INK.to_array()
    );
}

/// All seven atlas textures exist and are distinct (one per tool).
#[test]
fn atlas_ids_are_one_per_tool() {
    let ctx = Context::default();
    let mut atlas = None;
    ensure(&mut atlas, &ctx);
    let atlas = atlas.expect("built");
    let mut ids: Vec<_> = atlas.handles.iter().map(TextureHandle::id).collect();
    let n = ids.len();
    ids.sort_by_key(|id| format!("{id:?}"));
    ids.dedup();
    assert_eq!(ids.len(), n, "every tool owns its own texture");
    assert_eq!(n, TOOLS.len());
}

/// The icon contact sheet, kept PERMANENTLY as an ignored test (owner
/// request 2026-11-18). A plain `cargo test` — the done-gate — SKIPS it,
/// so it costs verify.sh nothing; run it explicitly when redraw/adjusting
/// an icon to SEE it instead of judging coordinates:
///
///     cargo test --profile gates -- --ignored icons_dump
///
/// It rasterizes every tool at 256 px (the design view) and at the real
/// button size 24 px (cleared first). The raster color is BLACK so the
/// monochrome icons (Line/Shape/Text take the raster color) stay legible;
/// the colored icons paint their own palette. The output folder is
/// `RUSSET_ICON_DUMP_DIR` (a leading `~/` expands to HOME/USERPROFILE —
/// the owner dumps straight into e.g. ~/Pictures), default /tmp/icon_dump.
#[test]
#[ignore = "a diagnostic dump, not a check — run: RUSSET_ICON_DUMP_DIR=~/Pictures cargo test --profile gates -- --ignored icons_dump"]
fn icons_dump_when_requested() {
    let dir = dump_dir();
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("the dump folder must be creatable");
    for tool in TOOLS {
        for px in [256, 24] {
            let pm = rasterize_tool_icon(tool, px, Color32::BLACK).expect("px is in 1..=256");
            let path = dir.join(format!("{px}px_{tool:?}.png"));
            pm.save_png(&path)
                .expect("tiny-skia's default features include png");
        }
    }
    let n = std::fs::read_dir(&dir)
        .expect("the dir was just created")
        .count();
    assert_eq!(n, TOOLS.len() * 2, "one 256 px + one 24 px PNG per tool");
}

/// The dump folder: `RUSSET_ICON_DUMP_DIR` when set (a leading `~/` expands
/// to HOME, falling back to USERPROFILE on Windows), else /tmp/icon_dump.
fn dump_dir() -> std::path::PathBuf {
    match std::env::var("RUSSET_ICON_DUMP_DIR") {
        Ok(dir) if !dir.is_empty() => {
            if let Some(rest) = dir.strip_prefix("~/") {
                let home = std::env::var("HOME")
                    .or_else(|_| std::env::var("USERPROFILE"))
                    .expect("set HOME/USERPROFILE — cannot expand ~ in RUSSET_ICON_DUMP_DIR");
                std::path::PathBuf::from(home).join(rest)
            } else {
                std::path::PathBuf::from(dir)
            }
        }
        _ => std::path::PathBuf::from("/tmp/icon_dump"),
    }
}

/// Whether any mostly-opaque pixel sits within `tol` LOGICAL units of the
/// logical-space point `pt` (the tests speak the icons' 24×24 language;
/// the helper maps it through the raster's scale).
fn has_opaque_near(pm: &tiny_skia::Pixmap, pt: (f32, f32), tol: f32) -> bool {
    let k = pm.width() as f32 / 24.0;
    let (x, y) = (pt.0 * k, pt.1 * k);
    pm.pixels().iter().enumerate().any(|(i, p)| {
        p.alpha() > 200
            && ((i as f32 % pm.width() as f32) - x).abs() <= tol * k
            && ((i as f32 / pm.width() as f32) - y).abs() <= tol * k
    })
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
