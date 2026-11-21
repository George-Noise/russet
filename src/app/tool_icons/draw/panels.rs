//! The settings-PANEL icon drawings (pure code motion, owner-approved
//! 2026-11-21 file-guard split): the Shape panel's KIND / MODE /
//! OUTLINE-PLACEMENT sets, the Line panel's Straight mark and the Select
//! family's four op glyphs — with their dispatchers
//! ([`rasterize_shape_icon`] / [`rasterize_line_icon`] /
//! [`rasterize_select_icon`]). The MODE trio and the Select marks reuse the
//! TOOL icons' geometry from the parent and the tools sibling; every icon
//! goes through the pipeline's primitives.

use super::super::{arc_pts, fill, icon_transform, stroke};
use super::{
    EMBLEM_RECT, SHAPE_PREVIEW_FILL, SQUARE_PTS, draw_line, draw_select_lasso, draw_select_rect,
    emblem_circle,
};
use crate::tools::{ShapeKind, ShapeOutline};
use eframe::egui::Color32;
use tiny_skia::{Pixmap, Transform};

/// The kind button: the shape as a NEUTRAL PREVIEW — the mid-gray fill plus
/// the ink rim (the same 1.9 stroke the monochrome tool icons draw).
fn draw_kind_preview(pm: &mut Pixmap, ts: Transform, kind: ShapeKind, mono: Color32) {
    match kind {
        ShapeKind::Rectangle => {
            fill(pm, ts, &SQUARE_PTS, SHAPE_PREVIEW_FILL);
            stroke(pm, ts, &SQUARE_PTS, true, mono, 1.9);
        }
        ShapeKind::Ellipse => {
            let circle = arc_pts(12.0, 12.0, 7.5, 0.0, 360.0, 40);
            fill(pm, ts, &circle, SHAPE_PREVIEW_FILL);
            stroke(pm, ts, &circle, true, mono, 1.9);
        }
    }
}

/// A MODE radio: the Shape TOOL's emblem ([`EMBLEM_RECT`] +
/// [`emblem_circle`]) painted three ways — SOLID ink (`solid`, !`both` —
/// Fill: the whole silhouette in ink, the reading the owner approved in the
/// V51 toggles), HOLLOW (`!solid`, !`both` — Outline: the rim only,
/// pixel-for-pixel the Tools window's Shape button), or the gray fill UNDER
/// the ink rim (`both` — Fill + Outline: both ingredients visible, the
/// same vocabulary the kind previews speak).
fn draw_paint_mode(pm: &mut Pixmap, ts: Transform, both: bool, solid: bool, mono: Color32) {
    let circle = emblem_circle();
    if solid || both {
        let paint = if both { SHAPE_PREVIEW_FILL } else { mono };
        fill(pm, ts, &EMBLEM_RECT, paint);
        fill(pm, ts, &circle, paint);
    }
    if !solid || both {
        stroke(pm, ts, &EMBLEM_RECT, true, mono, 1.9);
        stroke(pm, ts, &circle, false, mono, 1.9);
    }
}

/// The OUTLINE-PLACEMENT radios' composition geometry (owner request
/// 2026-11-20). Each icon shows ONE corner of a square — ALWAYS a square's
/// corner, whatever kind is selected ("в виде угла квадратика вне
/// зависимости от выбранной фигуры") — built from the elements the owner
/// listed: the square's own corner as the gray FILL quadrant ("использовать
/// Fill… чтобы мозгу было понятнее где сама фигура нарисована"), the click
/// point as a contrasting dot at that corner ("точечка… кружочек правильной
/// контрастности, размера и формы"), and two segments — horizontal and
/// vertical — ALONG the outline's lines ("отрезки по горизонтали и по
/// вертикали вдоль линий Outline"). The owner's 2026-11-21 pass made the
/// segments read unmistakably as the OUTLINE: 2–3× thicker (6.5) and each
/// one's far end running PAST the icon's edge (the bitmap crop says the
/// outline line continues beyond the corner window), while the click dot
/// stays exactly as it was ("точечку оставить как есть"). The bands'
/// position relative to the corner IS the mode: fully beyond the dot =
/// [`ShapeOutline::Outside`], straddling the dot = [`ShapeOutline::Center`]
/// (the old look), fully inside the fill = [`ShapeOutline::Inside`].
const OUTLINE_DOT: (f32, f32) = (10.0, 10.0);

/// The outline bands' drawn thickness in the logical space (owner request
/// 2026-11-21: 2–3× the old 2.6 — a heavy band reads as THE outline, the
/// thing the radio places, not as another 1.9 rim).
const OUTLINE_BAND_W: f32 = 6.5;

/// How far each outline segment runs from the corner (owner request
/// 2026-11-21: the FAR end leaves the 24-pt icon space — 10 + 17 = 27,
/// cropped ~3 pt past the edge — so the segment visibly IS the outline
/// line continuing beyond the corner window; the near end stays at the
/// corner).
const OUTLINE_BAND_LEN: f32 = 17.0;

/// The dot's radius (the click point): white core + ink ring — the same
/// anchor vocabulary the curve preview's vertex handles speak.
const OUTLINE_DOT_R: f32 = 2.4;

fn draw_outline_mode(pm: &mut Pixmap, ts: Transform, pos: ShapeOutline, mono: Color32) {
    let (dx, dy) = OUTLINE_DOT;
    let (w, l) = (OUTLINE_BAND_W, OUTLINE_BAND_LEN);
    // The shape's corner as the gray fill quadrant: down-right from the
    // click dot to the icon's edges (a big square's cropped corner).
    fill(
        pm,
        ts,
        &[(dx, dy), (24.0, dy), (24.0, 24.0), (dx, 24.0)],
        SHAPE_PREVIEW_FILL,
    );
    // The two outline segments (the horizontal top-edge band and the
    // vertical left-edge band), joined at the corner the way the stroked
    // shape joins them; their position relative to the dot IS the mode.
    let (top, left) = match pos {
        ShapeOutline::Outside => (
            [
                (dx - w, dy - w),
                (dx + l, dy - w),
                (dx + l, dy),
                (dx - w, dy),
            ],
            [
                (dx - w, dy - w),
                (dx, dy - w),
                (dx, dy + l),
                (dx - w, dy + l),
            ],
        ),
        ShapeOutline::Center => {
            let h = w * 0.5;
            (
                [
                    (dx - h, dy - h),
                    (dx + l, dy - h),
                    (dx + l, dy + h),
                    (dx - h, dy + h),
                ],
                [
                    (dx - h, dy - h),
                    (dx + h, dy - h),
                    (dx + h, dy + l),
                    (dx - h, dy + l),
                ],
            )
        }
        ShapeOutline::Inside => (
            [(dx, dy), (dx + l, dy), (dx + l, dy + w), (dx, dy + w)],
            [(dx, dy), (dx + w, dy), (dx + w, dy + l), (dx, dy + l)],
        ),
    };
    fill(pm, ts, &top, mono);
    fill(pm, ts, &left, mono);
    // The click dot ON TOP (over fill and bands alike): the white core +
    // ink ring reads on every background the icon puts under it.
    let dot = arc_pts(dx, dy, OUTLINE_DOT_R, 0.0, 360.0, 24);
    fill(pm, ts, &dot, Color32::WHITE);
    stroke(pm, ts, &dot, true, mono, 1.0);
}

/// Rasterizes one Shape-panel icon into a `px × px` premultiplied-RGBA
/// pixmap (kind-INDEPENDENT since owner request 2026-11-20 — the MODE
/// radios draw the tool's emblem, the KIND buttons preview their own fixed
/// shape). Same failure contract as [`rasterize_tool_icon`].
pub(in crate::app::tool_icons) fn rasterize_shape_icon(
    icon: super::super::ShapeIcon,
    px: usize,
    text: Color32,
) -> Option<Pixmap> {
    use super::super::ShapeIcon;
    let mut pm = Pixmap::new(px as u32, px as u32)?;
    let ts = icon_transform(px, 0.0);
    match icon {
        ShapeIcon::Rectangle => draw_kind_preview(&mut pm, ts, ShapeKind::Rectangle, text),
        ShapeIcon::Ellipse => draw_kind_preview(&mut pm, ts, ShapeKind::Ellipse, text),
        ShapeIcon::Fill => draw_paint_mode(&mut pm, ts, false, true, text),
        ShapeIcon::Outline => draw_paint_mode(&mut pm, ts, false, false, text),
        ShapeIcon::FillOutline => draw_paint_mode(&mut pm, ts, true, false, text),
        ShapeIcon::OutlineOutside => draw_outline_mode(&mut pm, ts, ShapeOutline::Outside, text),
        ShapeIcon::OutlineCenter => draw_outline_mode(&mut pm, ts, ShapeOutline::Center, text),
        ShapeIcon::OutlineInside => draw_outline_mode(&mut pm, ts, ShapeOutline::Inside, text),
    }
    Some(pm)
}

/// The panel's Straight button: the tool's mark unbent (owner request
/// 2026-11-20) — the straight segment between the emblem curve's two
/// endpoints, the same 1.9 round-cap stroke.
fn draw_line_straight(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(pm, ts, &[(4.8, 18.6), (19.2, 4.8)], false, mono, 1.9);
}

/// Rasterizes one Line-panel icon into a `px × px` premultiplied-RGBA
/// pixmap (owner request 2026-11-20). Same failure contract as
/// [`rasterize_tool_icon`].
pub(in crate::app::tool_icons) fn rasterize_line_icon(
    icon: super::super::LineIcon,
    px: usize,
    text: Color32,
) -> Option<Pixmap> {
    use super::super::LineIcon;
    let mut pm = Pixmap::new(px as u32, px as u32)?;
    let ts = icon_transform(px, 0.0);
    match icon {
        LineIcon::Straight => draw_line_straight(&mut pm, ts, text),
        LineIcon::Curve => draw_line(&mut pm, ts, text),
    }
    Some(pm)
}

/// The Copy op's glyph: two overlapping hollow rectangles.
fn draw_copy(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(
        pm,
        ts,
        &[(3.5, 3.5), (13.5, 3.5), (13.5, 13.5), (3.5, 13.5)],
        true,
        mono,
        1.9,
    );
    stroke(
        pm,
        ts,
        &[(9.5, 9.5), (20.5, 9.5), (20.5, 20.5), (9.5, 20.5)],
        true,
        mono,
        1.9,
    );
}

/// The Cut op's glyph: scissors — two crossing blades with round finger
/// holes.
fn draw_cut(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(
        pm,
        ts,
        &[(4.5, 3.5), (12.0, 12.5), (8.0, 17.5)],
        false,
        mono,
        1.9,
    );
    stroke(
        pm,
        ts,
        &[(19.5, 3.5), (12.0, 12.5), (16.0, 17.5)],
        false,
        mono,
        1.9,
    );
    stroke(
        pm,
        ts,
        &arc_pts(6.5, 18.5, 2.2, 0.0, 360.0, 24),
        false,
        mono,
        1.9,
    );
    stroke(
        pm,
        ts,
        &arc_pts(17.5, 18.5, 2.2, 0.0, 360.0, 24),
        false,
        mono,
        1.9,
    );
}

/// The Paste op's glyph: a clipboard — the board with the filled clip tab.
fn draw_paste(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(
        pm,
        ts,
        &[(4.5, 4.5), (19.5, 4.5), (19.5, 21.0), (4.5, 21.0)],
        true,
        mono,
        1.9,
    );
    fill(
        pm,
        ts,
        &[(9.0, 2.5), (15.0, 2.5), (15.0, 6.5), (9.0, 6.5)],
        mono,
    );
}

/// The Crop op's glyph: the classic two mirrored L-shapes.
fn draw_crop(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(
        pm,
        ts,
        &[(6.5, 2.5), (6.5, 17.5), (21.5, 17.5)],
        false,
        mono,
        1.9,
    );
    stroke(
        pm,
        ts,
        &[(2.5, 6.5), (17.5, 6.5), (17.5, 21.5)],
        false,
        mono,
        1.9,
    );
}

/// Rasterizes one Select-family icon into a `px × px` premultiplied-RGBA
/// pixmap (owner request 2026-11-20). Same failure contract as
/// [`rasterize_tool_icon`].
pub(in crate::app::tool_icons) fn rasterize_select_icon(
    icon: super::super::SelectIcon,
    px: usize,
    text: Color32,
) -> Option<Pixmap> {
    use super::super::SelectIcon;
    let mut pm = Pixmap::new(px as u32, px as u32)?;
    let ts = icon_transform(px, 0.0);
    match icon {
        SelectIcon::Rect => draw_select_rect(&mut pm, ts, text),
        SelectIcon::Lasso => draw_select_lasso(&mut pm, ts, text),
        SelectIcon::Copy => draw_copy(&mut pm, ts, text),
        SelectIcon::Cut => draw_cut(&mut pm, ts, text),
        SelectIcon::Paste => draw_paste(&mut pm, ts, text),
        SelectIcon::Crop => draw_crop(&mut pm, ts, text),
    }
    Some(pm)
}
