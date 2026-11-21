//! The polygon/shape rasterizers behind the Line and Shape tools' commits
//! (pure code motion, owner-approved 2026-11-21 file-guard split): the
//! polyline stoker ([`rasterize_polyline`]), the rectangle/ellipse painter
//! ([`rasterize_shape`]) and the outline-placement math
//! ([`outline_stroke_bounds`]) shared with the Shape panel's placement
//! tests. Everything draws through the parent's [`super::paint_clipped`]
//! pipeline, so an active selection clips these commits the same way it
//! clips the direct strokes; the dirty-rect helper
//! ([`dirty_rect_for`]) is shared with [`super::smooth`].

use super::{DirtyRect, MAX_STROKE_SIZE, ShapeKind, ShapeOutline, paint_clipped};
use crate::selection::Mask;
use eframe::egui::{Color32, Pos2};
use tiny_skia::{FillRule, LineCap, Paint, PathBuilder, Pixmap, Stroke};

/// Padded, canvas-clipped dirty rectangle for a shape whose bounding box
/// (canvas coordinates, any orientation) was painted with `width`-wide
/// strokes — same padding rule as [`draw_segment`].
pub(crate) fn dirty_rect_for(
    pixmap: &Pixmap,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    width: f32,
) -> Option<DirtyRect> {
    let pad = (width * 0.5).ceil() + 2.0;
    let x0 = (min_x - pad).floor().max(0.0) as u32;
    let y0 = (min_y - pad).floor().max(0.0) as u32;
    let x1 = (max_x + pad).ceil().min(pixmap.width() as f32) as u32;
    let y1 = (max_y + pad).ceil().min(pixmap.height() as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

/// Rasterizes a polyline with round caps and antialiasing (the same look as
/// the brush). One point draws a dot; two points draw a straight line —
/// both used by the Line tool to commit its preview (owner request
/// 2026-11-10). Returns the dirty rectangle.
pub fn rasterize_polyline(
    pixmap: &mut Pixmap,
    points: &[Pos2],
    width: f32,
    color: Color32,
    clip: Option<&Mask>,
) -> Option<DirtyRect> {
    let width = width.clamp(1.0, MAX_STROKE_SIZE);
    let (&first, rest) = points.split_first()?;

    let mut min_x = first.x;
    let mut min_y = first.y;
    let mut max_x = first.x;
    let mut max_y = first.y;
    for p in rest {
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x);
        max_y = max_y.max(p.y);
    }
    let dirty = dirty_rect_for(pixmap, min_x, min_y, max_x, max_y, width)?;

    // A single point (or a fully degenerate bbox) cannot be stroked:
    // tiny-skia rejects paths whose bounds have zero width/height. Draw a
    // round dot instead — a filled circle of the brush diameter, which is
    // exactly what a round cap leaves.
    if (max_x - min_x) < 0.01 && (max_y - min_y) < 0.01 {
        let mut builder = PathBuilder::new();
        builder.push_circle(first.x, first.y, width * 0.5);
        let path = builder.finish()?;
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        return paint_clipped(pixmap, dirty, clip, false, |pm, ts, _| {
            let mut paint = Paint::default();
            paint.set_color_rgba8(r, g, b, a);
            paint.anti_alias = true;
            pm.fill_path(&path, &paint, FillRule::Winding, ts, None);
        });
    }

    let mut builder = PathBuilder::new();
    builder.move_to(first.x, first.y);
    let last_index = rest.len().saturating_sub(1);
    for (i, p) in rest.iter().enumerate() {
        // A perfectly horizontal or vertical polyline has zero-height/width
        // path bounds, which tiny-skia rejects — nudge the final point by a
        // subpixel epsilon (far below one pixel, invisible at any zoom).
        let (x, y) = if i == last_index && (max_y - min_y) < 0.01 {
            (p.x, p.y + 0.01)
        } else if i == last_index && (max_x - min_x) < 0.01 {
            (p.x + 0.01, p.y)
        } else {
            (p.x, p.y)
        };
        builder.line_to(x, y);
    }
    let path = builder.finish()?;

    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    paint_clipped(pixmap, dirty, clip, false, |pm, ts, _| {
        let mut paint = Paint::default();
        paint.set_color_rgba8(r, g, b, a);
        paint.anti_alias = true;
        pm.stroke_path(&path, &paint, &stroke, ts, None);
    })
}

/// The bounds of the path the shape's outline stroke runs along. The stroke
/// straddles its path (half the width on each side), so each
/// [`ShapeOutline`] mode shifts the path accordingly: Outside shifts it half
/// a width outward (the stroke's inner edge lands exactly on the boundary),
/// Inside half a width inward (the outer edge lands on the boundary), Center
/// keeps the boundary itself. The inward shift is CLAMPED so a shape at most
/// a stroke wide keeps a valid non-degenerate path — the stroke then simply
/// covers the whole tiny shape. Pure — the panel's three modes and their
/// tests share this math.
pub(crate) fn outline_stroke_bounds(
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    width: f32,
    pos: ShapeOutline,
) -> (f32, f32, f32, f32) {
    match pos {
        ShapeOutline::Center => (min_x, min_y, max_x, max_y),
        ShapeOutline::Outside => (
            min_x - width * 0.5,
            min_y - width * 0.5,
            max_x + width * 0.5,
            max_y + width * 0.5,
        ),
        ShapeOutline::Inside => {
            let ix = (width * 0.5).min((max_x - min_x - 0.1) * 0.5).max(0.0);
            let iy = (width * 0.5).min((max_y - min_y - 0.1) * 0.5).max(0.0);
            (min_x + ix, min_y + iy, max_x - ix, max_y - iy)
        }
    }
}

/// Rasterizes a rectangle or an ellipse between two opposite corners
/// (either orientation). `fill` fills the interior (the secondary color);
/// `outline` strokes the boundary with a `width`-wide antialiased stroke
/// (the primary color) — both may be enabled at once, then the fill goes
/// first and the stroke covers it. `outline_pos` decides where that stroke
/// band lies relative to the two mouse points (see [`ShapeOutline`]); the
/// fill always covers exactly the two points. The Tools window never lets
/// both colors be `None`; if they are, nothing is drawn. Returns the dirty
/// rectangle.
#[allow(clippy::too_many_arguments)] // the flat paint list the gesture carries (same as layers_window.rs)
pub fn rasterize_shape(
    pixmap: &mut Pixmap,
    kind: ShapeKind,
    from: Pos2,
    to: Pos2,
    width: f32,
    outline: Option<Color32>,
    fill: Option<Color32>,
    outline_pos: ShapeOutline,
    clip: Option<&Mask>,
) -> Option<DirtyRect> {
    let width = width.clamp(1.0, MAX_STROKE_SIZE);
    let (min_x, max_x) = (from.x.min(to.x), from.x.max(to.x));
    let (min_y, max_y) = (from.y.min(to.y), from.y.max(to.y));
    if max_x - min_x < 0.01 || max_y - min_y < 0.01 {
        return None;
    }
    if outline.is_none() && fill.is_none() {
        return None;
    }
    let bounds = tiny_skia::Rect::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)?;

    // An Outside outline reaches a FULL width beyond the two points (the
    // shared pad rule adds half a width + 2 for it); the other modes stay
    // within the usual half-width reach.
    let reach = if outline.is_some() && outline_pos == ShapeOutline::Outside {
        width * 2.0
    } else {
        width
    };
    let dirty = dirty_rect_for(pixmap, min_x, min_y, max_x, max_y, reach)?;

    // Fill first, outline over it — the same order on the direct surface
    // and on the scratch (the scratch transfers the composited result, and
    // src-over's associativity keeps it pixel-identical).
    paint_clipped(pixmap, dirty, clip, false, |pm, ts, _| {
        if let Some(fill) = fill {
            let mut builder = PathBuilder::new();
            match kind {
                ShapeKind::Rectangle => builder.push_rect(bounds),
                ShapeKind::Ellipse => builder.push_oval(bounds),
            }
            let Some(path) = builder.finish() else {
                return;
            };
            let mut paint = Paint::default();
            let [r, g, b, a] = fill.to_srgba_unmultiplied();
            paint.set_color_rgba8(r, g, b, a);
            paint.anti_alias = true;
            pm.fill_path(&path, &paint, FillRule::Winding, ts, None);
        }
        if let Some(outline) = outline {
            let (sx0, sy0, sx1, sy1) =
                outline_stroke_bounds(min_x, min_y, max_x, max_y, width, outline_pos);
            // Unreachable None: the bounds were just built from valid extents
            // and `outline_stroke_bounds` never shrinks a span below 0.1.
            let Some(stroke_bounds) = tiny_skia::Rect::from_xywh(sx0, sy0, sx1 - sx0, sy1 - sy0)
            else {
                return;
            };
            let mut builder = PathBuilder::new();
            match kind {
                ShapeKind::Rectangle => builder.push_rect(stroke_bounds),
                ShapeKind::Ellipse => builder.push_oval(stroke_bounds),
            }
            let Some(path) = builder.finish() else {
                return;
            };
            let mut paint = Paint::default();
            let [r, g, b, a] = outline.to_srgba_unmultiplied();
            paint.set_color_rgba8(r, g, b, a);
            paint.anti_alias = true;
            let stroke = Stroke {
                width,
                line_cap: LineCap::Round,
                ..Default::default()
            };
            pm.stroke_path(&path, &paint, &stroke, ts, None);
        }
    })
}
