use super::*;
use eframe::egui::pos2;

/// A context with fonts installed. Fonts are normally set up by the
/// first `Context::run` pass; tests have no pass, so the default set is
/// installed explicitly.
fn test_context() -> Context {
    let ctx = Context::default();
    ctx.set_fonts(egui::FontDefinitions::default());
    // `set_fonts` only schedules the installation: fonts are actually
    // created at the start of a pass, so run one headless pass.
    ctx.begin_pass(egui::RawInput::default());
    ctx
}

#[test]
fn rasterizes_non_empty_text_into_pixels() {
    let ctx = test_context();
    let mut pixmap = Pixmap::new(300, 120).expect("300x120 pixmap");
    let dirty = rasterize_text(
        &ctx,
        &mut pixmap,
        "Ab",
        pos2(10.0, 30.0),
        32.0,
        Color32::BLACK,
        None,
    );
    let dirty = dirty.expect("text should rasterize");
    assert!(dirty.w > 0 && dirty.h > 0);
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 200).count();
    assert!(painted > 20, "glyphs have solid pixels: {painted}");
    // Nothing outside the dirty rect was touched: pixels far from the
    // anchor stay transparent.
    let at = |x: usize, y: usize| pixmap.pixels()[y * 300 + x];
    assert_eq!(at(290, 110).alpha(), 0);
}

#[test]
fn blank_text_draws_nothing() {
    let ctx = test_context();
    let mut pixmap = Pixmap::new(100, 100).expect("100x100 pixmap");
    assert!(
        rasterize_text(
            &ctx,
            &mut pixmap,
            "  ",
            pos2(10.0, 10.0),
            24.0,
            Color32::BLACK,
            None
        )
        .is_none()
    );
    assert!(pixmap.pixels().iter().all(|c| c.alpha() == 0));
}

#[test]
fn multiline_text_advances_rows() {
    let ctx = test_context();
    let mut pixmap = Pixmap::new(200, 200).expect("200x200 pixmap");
    let dirty = rasterize_text(
        &ctx,
        &mut pixmap,
        "A\nB",
        pos2(20.0, 20.0),
        32.0,
        Color32::BLACK,
        None,
    );
    assert!(dirty.is_some());
    // Both rows painted something in the left column region.
    let painted_rows: Vec<usize> = (0..200usize)
        .filter(|&y| (0..200).any(|x| pixmap.pixels()[y * 200 + x].alpha() > 200))
        .collect();
    assert!(painted_rows.len() >= 2, "two rows of glyphs");
}

/// The selection confines the text commit (owner request 2026-11-21): with
/// a mask covering the LEFT half of the canvas, glyphs landing on the right
/// half change nothing and extend no dirty rect; painted pixels exist only
/// inside the mask.
#[test]
fn clipped_text_rasterizes_only_masked_pixels() {
    let ctx = test_context();
    let mut sel = crate::selection::Selection::default();
    sel.replace(
        Some(crate::selection::Region::Rect {
            a: pos2(0.0, 0.0),
            b: pos2(20.0, 120.0),
        }),
        300,
        120,
    );
    let mask = sel.build_mask(300, 120).expect("non-empty selection");
    let mut pixmap = Pixmap::new(300, 120).expect("300x120 pixmap");
    let dirty = rasterize_text(
        &ctx,
        &mut pixmap,
        "Ab",
        pos2(10.0, 30.0),
        32.0,
        Color32::BLACK,
        Some(&mask),
    );
    let dirty = dirty.expect("the left half of the glyphs must paint");
    assert!(
        dirty.x + dirty.w <= 22,
        "the dirty rect must stop at the mask's right edge (+2 pad): {dirty:?}"
    );
    // No opaque pixel beyond the mask.
    for y in 0..120usize {
        for x in 20..300usize {
            assert_eq!(
                pixmap.pixels()[y * 300 + x].alpha(),
                0,
                "pixel ({x}, {y}) is outside the selection"
            );
        }
    }
    // And something DID paint inside.
    let painted = pixmap.pixels().iter().filter(|c| c.alpha() > 200).count();
    assert!(painted > 20, "the masked glyphs still paint: {painted}");
}
