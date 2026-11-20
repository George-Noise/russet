//! The ATLAS/CURSOR PIPELINE tests (owner request 2026-11-18): the

//! lifecycle (rebuild keys), the texture ids, the pipette cursor bitmap,

//! the shared transform, the fixed ink — plus the permanent icon-dump

//! diagnostic.

use super::super::{
    LINE_ICONS, PIPETTE_TIP, SHAPE_ICONS, TOOL_ANGLE, TOOL_INK, TOOLS, cursor_bitmap, ensure,
    icon_transform, rasterize_line_icon, rasterize_shape_icon, rasterize_tool_icon,
};
use crate::tools::Tool;
use eframe::egui::{Color32, Context, RawInput, TextureHandle, Theme, Visuals};

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
    let mut tip = tiny_skia::Point::from_xy(PIPETTE_TIP.0, PIPETTE_TIP.1);
    icon_transform(w, TOOL_ANGLE).map_point(&mut tip);
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
    let ts45 = icon_transform(24, TOOL_ANGLE);
    // The rotation about the center keeps the center fixed.
    let mut c = tiny_skia::Point::from_xy(12.0, 12.0);
    ts45.map_point(&mut c);
    assert!((c.x - 12.0).abs() < 1e-3 && (c.y - 12.0).abs() < 1e-3);
}

/// The atlas lifecycle: built on the first frame, untouched while the key
/// matches, rebuilt when the scale changes. The key is the SCALE ALONE —
/// theme-independent since the v4 pass, and since owner request 2026-11-20
/// kind-independent too (the Shape panel's MODE radios draw the Shape
/// tool's emblem, fixed drawings all).

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
    assert_eq!(
        pass(&ctx),
        25,
        "8 tool + 8 shape + 2 line + 6 select icons + egui's font atlas"
    );
    let first = atlas.as_ref().expect("the first ensure builds the atlas");
    assert_eq!(first.key, 1.0);
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
        24,
        "rebuilt after the scale change (the font atlas only uploads once)"
    );
    let scaled = atlas.as_ref().expect("rebuilt after the scale change");
    assert_eq!(scaled.key, 2.0);
    let cursor1 = scaled
        .pipette_cursor
        .as_ref()
        .expect("cursor re-rasterized");
    assert_eq!(cursor1.size, [48, 48]);

    // Theme change: NOT a rebuild — the fixed ink and the fixed rims make
    // the icons theme-independent (owner v4: one drawing reads well in both
    // themes; the old theme re-raster died with its reason). The pass still
    // uploads ONE texture: egui's font atlas follows the theme too. OUR
    // twenty-four icons would make it 25 — the count stays the "did the
    // atlas rebuild?" discriminator.
    ctx.set_theme(Theme::Light);
    ensure(&mut atlas, &ctx);
    assert_eq!(pass(&ctx), 1, "only egui's font atlas, not our 24 icons");
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

/// All atlas textures exist and are distinct: one per tool PLUS the Shape
/// panel's eight (owner requests 2026-11-20).

#[test]
fn atlas_ids_are_one_per_tool() {
    let ctx = Context::default();
    let mut atlas = None;
    ensure(&mut atlas, &ctx);
    let atlas = atlas.expect("built");
    let mut ids: Vec<_> = atlas
        .handles
        .iter()
        .chain(atlas.shape_icons.iter())
        .chain(atlas.line_icons.iter())
        .map(TextureHandle::id)
        .collect();
    let n = ids.len();
    ids.sort_by_key(|id| format!("{id:?}"));
    ids.dedup();
    assert_eq!(ids.len(), n, "every icon owns its own texture");
    assert_eq!(
        n,
        TOOLS.len() + SHAPE_ICONS.len() + LINE_ICONS.len(),
        "one texture per tool, per Shape-panel icon and per Line-panel icon"
    );
}

/// The icon contact sheet, kept PERMANENTLY as an ignored test (owner
/// request 2026-11-18). A plain `cargo test` — the done-gate — SKIPS it,
/// so it costs verify.sh nothing; run it explicitly when redraw/adjusting
/// an icon to SEE it instead of judging coordinates:
///
///     cargo test --profile gates -- --ignored icons_dump
///
/// It rasterizes every tool at 256 px (the design view) and at the real
/// button size 24 px (cleared first), and since owner request 2026-11-20
/// also the Shape panel's eight icons (the kind-independent MODE radios and
/// the OUTLINE-PLACEMENT trio — owner requests 2026-11-20) and the Line
/// panel's two icons (the emblem per mode — the same day's Line-panel
/// round). The raster
/// color is BLACK so the monochrome
/// icons
/// (Line/Shape/Text and the Shape panel take the raster color) stay
/// legible; the colored icons paint their own palette. The output folder
/// is `RUSSET_ICON_DUMP_DIR` (a leading `~/` expands to HOME/USERPROFILE —
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
    for icon in SHAPE_ICONS {
        for px in [256, 24] {
            let pm = rasterize_shape_icon(icon, px, Color32::BLACK).expect("px is in 1..=256");
            let path = dir.join(format!("{px}px_shape_{icon:?}.png"));
            pm.save_png(&path)
                .expect("tiny-skia's default features include png");
        }
    }
    for icon in LINE_ICONS {
        for px in [256, 24] {
            let pm = rasterize_line_icon(icon, px, Color32::BLACK).expect("px is in 1..=256");
            let path = dir.join(format!("{px}px_line_{icon:?}.png"));
            pm.save_png(&path)
                .expect("tiny-skia's default features include png");
        }
    }
    let n = std::fs::read_dir(&dir)
        .expect("the dir was just created")
        .count();
    assert_eq!(
        n,
        TOOLS.len() * 2 + SHAPE_ICONS.len() * 2 + LINE_ICONS.len() * 2,
        "one 256 px + one 24 px PNG per tool, shape icon and line icon"
    );
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
