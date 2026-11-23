//! The ATLAS/CURSOR PIPELINE tests (owner request 2026-11-18): the

//! lifecycle (rebuild keys), the texture ids, the pipette cursor bitmap,

//! the shared transform, the fixed ink — plus the permanent icon-dump

//! diagnostic.

use super::super::{
    LINE_ICONS, PIPETTE_TIP, SHAPE_ICONS, TOOL_ANGLE, TOOL_INK, TOOLS, TREE_ICON_PT, cursor_bitmap,
    ensure, icon_transform, paint_debug_border, rasterize_line_icon, rasterize_shape_icon,
    rasterize_tool_icon, tree_texture_id,
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
    ensure(&mut atlas, &ctx, false);
    assert_eq!(
        pass(&ctx),
        36,
        "9 tool + 8 shape + 2 line + 7 select + 9 tree icons + egui's font atlas"
    );
    let first = atlas.as_ref().expect("the first ensure builds the atlas");
    assert_eq!(first.key, 1.0);
    let cursor0 = first.pipette_cursor.as_ref().expect("the pipette cursor");
    assert_eq!(cursor0.size, [24, 24]);

    // Same conditions: a cheap no-op (the key match must keep the atlas).
    ensure(&mut atlas, &ctx, false);
    assert_eq!(pass(&ctx), 0);
    assert_eq!(atlas.as_ref().expect("kept").key, 1.0);

    // Scale change: the WHOLE atlas re-rasterizes (24 pt × 2.0 = 48 px).
    // set_pixels_per_point becomes active at the START of the next pass,
    // so a pass runs before ensure() can observe the new scale.
    ctx.set_pixels_per_point(2.0);
    pass(&ctx);
    ensure(&mut atlas, &ctx, false);
    assert_eq!(
        pass(&ctx),
        35,
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
    // thirty-five icons would make it 36 — the count stays the "did the
    // atlas rebuild?" discriminator.
    ctx.set_theme(Theme::Light);
    ensure(&mut atlas, &ctx, false);
    assert_eq!(pass(&ctx), 1, "only egui's font atlas, not our 35 icons");
    assert_eq!(atlas.as_ref().expect("kept").key, 2.0);
}

/// The SECOND atlas (owner request 2026-11-21): the History Tree row icons
/// rasterize at [`TREE_ICON_PT`] — one per tool, at the ROW size (16 px at
/// ppp 1), not a downscaled 24 pt texture (the thin strokes would blur).
#[test]
fn the_tree_atlas_rasterizes_one_row_icon_per_tool() {
    let ctx = Context::default();
    let mut atlas = None;
    ensure(&mut atlas, &ctx, false);
    let built = atlas.as_ref().expect("built");
    assert_eq!(
        built.tree_icons.len(),
        TOOLS.len(),
        "one row-size icon per tool"
    );
    for (i, tool) in TOOLS.iter().enumerate() {
        let id = tree_texture_id(&atlas, *tool).expect("the small texture");
        assert_eq!(id, built.tree_icons[i].id());
        assert_eq!(
            built.tree_icons[i].size(),
            [TREE_ICON_PT as usize; 2],
            "the row size at ppp 1, not the 24 pt button raster"
        );
    }
    // The 12 px pencil still carries ink — the emblem does not vanish at
    // the row size (its legibility is the owner's V-check).
    let pm = rasterize_tool_icon(Tool::Pencil, TREE_ICON_PT as usize, TOOL_INK)
        .expect("px is in 1..=256");
    assert!(
        pm.pixels().iter().filter(|p| p.alpha() > 0).count() > 8,
        "the small pencil raster is not empty"
    );
}

/// The `--atlas-test` border (owner request 2026-11-21): a STATIC 1 px
/// black-and-white speckle on the raster's outer ring — one physical pixel
/// at any size, deterministic, the interior untouched.
#[test]
fn the_atlas_test_border_is_a_static_one_pixel_speckle_ring() {
    let size = 24u32;
    let mut pm =
        rasterize_tool_icon(Tool::Pencil, size as usize, TOOL_INK).expect("px is in 1..=256");
    let before = pm.clone();
    paint_debug_border(&mut pm);

    // The ring is ONLY fully-opaque black or white — and it IS a speckle
    // (both colors appear), not a solid frame.
    let mut blacks = 0;
    let mut whites = 0;
    for i in 0..size {
        for (x, y) in [(i, 0), (i, size - 1), (0, i), (size - 1, i)] {
            let c = pm.pixel(x, y).expect("inside");
            assert_eq!(c.alpha(), 255, "the ring pixel ({x},{y}) is opaque");
            match (c.red(), c.green(), c.blue()) {
                (0, 0, 0) => blacks += 1,
                (255, 255, 255) => whites += 1,
                other => panic!("the ring is black/white only at ({x},{y}): {other:?}"),
            }
        }
    }
    assert!(
        blacks > 0 && whites > 0,
        "a speckle has BOTH colors ({blacks} black, {whites} white)"
    );

    // Exactly ONE pixel: the interior keeps whatever the emblem painted
    // there — the ring never leaks inward.
    for y in 1..size - 1 {
        for x in 1..size - 1 {
            assert_eq!(
                pm.pixel(x, y),
                before.pixel(x, y),
                "the interior pixel ({x},{y}) is untouched"
            );
        }
    }

    // Deterministic: repainting the ALREADY bordered raster replays the
    // same ring byte-for-byte (no animation between frames).
    let first = pm.data().to_vec();
    paint_debug_border(&mut pm);
    assert_eq!(pm.data(), &first[..], "the speckle is static");
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
/// panel's eight, the Line panel's two AND the tree atlas's row-size set
/// (owner requests 2026-11-20, 2026-11-21).

#[test]
fn atlas_ids_are_one_per_tool() {
    let ctx = Context::default();
    let mut atlas = None;
    ensure(&mut atlas, &ctx, false);
    let atlas = atlas.expect("built");
    let mut ids: Vec<_> = atlas
        .handles
        .iter()
        .chain(atlas.shape_icons.iter())
        .chain(atlas.line_icons.iter())
        .chain(atlas.tree_icons.iter())
        .map(TextureHandle::id)
        .collect();
    let n = ids.len();
    ids.sort_by_key(|id| format!("{id:?}"));
    ids.dedup();
    assert_eq!(ids.len(), n, "every icon owns its own texture");
    assert_eq!(
        n,
        TOOLS.len() * 2 + SHAPE_ICONS.len() + LINE_ICONS.len(),
        "one texture per tool + its row-size twin, per Shape-panel icon and per Line-panel icon"
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
        // 256 = the design view, 24 = the real button, TREE_ICON_PT = the
        // History Tree row size (owner request 2026-11-21).
        for px in [256, 24, TREE_ICON_PT as usize] {
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
        TOOLS.len() * 3 + SHAPE_ICONS.len() * 2 + LINE_ICONS.len() * 2,
        "one 256 px + one 24 px + one 12 px PNG per tool (the tree row size), \
         256 px + 24 px for the shape and line icons"
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
