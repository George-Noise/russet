//! Smoothed freehand strokes (owner request 2026-11-19): the raw
//! frame-boundary segments are gone. Every coalesced `Event::PointerMoved`
//! of a frame feeds the smoothing window ([`SmoothStroke`]) — the classic
//! midpoint-quadratic scheme: the drawn path runs `start → mid(start, p1)
//! → (quad, control p1) → mid(p1, p2) → (quad, control p2) → …`, which is
//! tangent-continuous (C1) at every joint and never overshoots (a
//! quadratic stays inside its control triangle). [`rasterize_smooth_pieces`]
//! paints one frame's pieces in ONE tiny-skia stroke (brush/eraser — the
//! pieces join at their shared midpoints, so no per-segment cap seams) or
//! as the pencil's classic 1 px Bresenham chain along the flattened
//! centerline (the same hard 1-pixel trail as before, just following the
//! smoothed path instead of the raw polyline — the fast-stroke polygon
//! look is gone). The freehand gesture lives in [`crate::app::input`];
//! [`super::draw_segment`] remains for the press-time touch dot and the
//! MCP paths.

use eframe::egui::{Color32, Pos2};
use tiny_skia::{
    BlendMode, LineCap, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Stroke, Transform,
};

use super::{DirtyRect, Tool, blend_pixel, bresenham_line, dirty_rect_for};

/// Subdivision cap for the pencil's flattened quadratic centerline: one
/// Bresenham run per ~1 px of control-polygon length, never more than
/// this many per piece.
const PENCIL_MAX_SUBSTEPS: usize = 64;

/// One smoothed piece of a freehand stroke. Pieces chain: a piece's
/// `from` is the previous piece's `to` (a shared midpoint).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SmoothPiece {
    /// A straight segment — the gesture's first filler piece and the
    /// release closer.
    Line { from: Pos2, to: Pos2 },
    /// A quadratic segment with `control` between `from` and `to`.
    Quad { from: Pos2, control: Pos2, to: Pos2 },
}

impl SmoothPiece {
    fn mid(a: Pos2, b: Pos2) -> Pos2 {
        Pos2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
    }

    /// The piece's first point (the pen position before this piece).
    fn start(self) -> Pos2 {
        match self {
            SmoothPiece::Line { from, .. } | SmoothPiece::Quad { from, .. } => from,
        }
    }

    /// Bounding box of the piece's defining points: a quadratic stays
    /// inside its control triangle, so `from`/`control`/`to` cover the
    /// drawn curve.
    fn bounds(self) -> (f32, f32, f32, f32) {
        match self {
            SmoothPiece::Line { from, to } => (
                from.x.min(to.x),
                from.y.min(to.y),
                from.x.max(to.x),
                from.y.max(to.y),
            ),
            SmoothPiece::Quad { from, control, to } => (
                from.x.min(control.x).min(to.x),
                from.y.min(control.y).min(to.y),
                from.x.max(control.x).max(to.x),
                from.y.max(control.y).max(to.y),
            ),
        }
    }

    /// The quadratic Bézier point at parameter `t` (0 = `from`, 1 = `to`).
    fn quad_at(from: Pos2, control: Pos2, to: Pos2, t: f32) -> Pos2 {
        let u = 1.0 - t;
        Pos2::new(
            u * u * from.x + 2.0 * u * t * control.x + t * t * to.x,
            u * u * from.y + 2.0 * u * t * control.y + t * t * to.y,
        )
    }
}

/// The smoothing window of ONE freehand gesture: the last two raw pointer
/// positions. Pure and incremental — `app/input.rs` feeds every coalesced
/// pointer move of a frame and rasterizes the emitted pieces in one batch.
pub struct SmoothStroke {
    second_last: Option<Pos2>,
    last: Option<Pos2>,
}

impl SmoothStroke {
    pub fn new(start: Pos2) -> Self {
        Self {
            second_last: None,
            last: Some(start),
        }
    }

    /// Feeds the next raw pointer position and returns the piece to
    /// rasterize (`None` for an exact duplicate of the previous point).
    pub fn push(&mut self, p: Pos2) -> Option<SmoothPiece> {
        if self.last == Some(p) {
            return None;
        }
        let piece = match (self.second_last, self.last) {
            // The gesture's first move: a straight filler from the press
            // point to the first midpoint (the press dot covers the start
            // point itself).
            (None, Some(first)) => SmoothPiece::Line {
                from: first,
                to: SmoothPiece::mid(first, p),
            },
            // Every later move: the quadratic through the midpoints around
            // the previous point — tangent-continuous with both neighbors.
            (Some(a), Some(b)) => SmoothPiece::Quad {
                from: SmoothPiece::mid(a, b),
                control: b,
                to: SmoothPiece::mid(b, p),
            },
            // Unreachable in practice: `new` starts with `last` set.
            _ => return None,
        };
        self.second_last = self.last;
        self.last = Some(p);
        Some(piece)
    }

    /// The gesture's closer: a straight piece from the last emitted
    /// midpoint to the final raw point. `None` when the gesture never
    /// moved past the press point (the press dot already covers it) or
    /// when the window was already closed — a `close` consumes it.
    pub fn close(&mut self) -> Option<SmoothPiece> {
        let second_last = self.second_last.take();
        let last = self.last.take();
        match (second_last, last) {
            (Some(a), Some(b)) => Some(SmoothPiece::Line {
                from: SmoothPiece::mid(a, b),
                to: b,
            }),
            _ => None,
        }
    }
}

/// The integer pixel a canvas point rounds to — the pencil's plot grid
/// (the same rounding `draw_segment`'s Bresenham endpoints use), exposed
/// so the gesture can seed the dedupe state with the press dot's pixel.
pub fn pencil_pixel(p: Pos2) -> (i64, i64) {
    (p.x.round() as i64, p.y.round() as i64)
}

/// Plots one straight Bresenham run between the rounded endpoints,
/// skipping the pixel plotted immediately before: premultiplied
/// source-over is not idempotent (re-blending the same translucent pixel
/// darkens it), and consecutive runs share their joint pixel — at the
/// subdivision joints, at the frame joints and against the press dot.
fn plot_pencil_run(
    pixels: &mut [PremultipliedColorU8],
    width: i64,
    height: i64,
    src: [u32; 4],
    last_pixel: &mut Option<(i64, i64)>,
    a: Pos2,
    b: Pos2,
) {
    let (x0, y0) = pencil_pixel(a);
    let (x1, y1) = pencil_pixel(b);
    bresenham_line(x0, y0, x1, y1, |x, y| {
        if *last_pixel == Some((x, y)) {
            return;
        }
        blend_pixel(pixels, width, height, x, y, src);
        *last_pixel = Some((x, y));
    });
}

/// Rasterizes one frame's smoothed pieces into the pixmap in a single
/// pass and returns their UNION dirty rectangle. Brush/eraser: one
/// continuous tiny-skia stroke (round caps, AA; the eraser
/// `DestinationOut`). Pencil: the 1 px Bresenham chain along the
/// flattened centerline — quads subdivide into ~1 px steps, consecutive
/// duplicate pixels are skipped through `last_pixel`, the dedupe state
/// carried across the whole gesture (ignored by brush/eraser).
pub fn rasterize_smooth_pieces(
    pixmap: &mut Pixmap,
    tool: Tool,
    size: f32,
    color: Color32,
    pieces: &[SmoothPiece],
    last_pixel: &mut Option<(i64, i64)>,
) -> Option<DirtyRect> {
    let width = match tool {
        Tool::Pencil => 1.0,
        Tool::Brush | Tool::Eraser => size.clamp(1.0, 100.0),
        // Unreachable: the preview tools commit through `rasterize_polyline`
        // / `rasterize_shape`, not through this freehand path.
        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text => size.clamp(1.0, 100.0),
    };
    let (first, rest) = pieces.split_first()?;

    let (mut min_x, mut min_y, mut max_x, mut max_y) = first.bounds();
    for piece in rest {
        let (x0, y0, x1, y1) = piece.bounds();
        min_x = min_x.min(x0);
        min_y = min_y.min(y0);
        max_x = max_x.max(x1);
        max_y = max_y.max(y1);
    }
    let dirty = dirty_rect_for(pixmap, min_x, min_y, max_x, max_y, width)?;

    if matches!(tool, Tool::Pencil) {
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        // Premultiplied source components (the Pixmap stores premultiplied
        // RGBA), the same source `draw_segment`'s pencil plots with.
        let src = [
            r as u32 * a as u32 / 255,
            g as u32 * a as u32 / 255,
            b as u32 * a as u32 / 255,
            a as u32,
        ];
        let (pw, ph) = (pixmap.width() as i64, pixmap.height() as i64);
        let pixels = pixmap.pixels_mut();
        for piece in pieces {
            match *piece {
                SmoothPiece::Line { from, to } => {
                    plot_pencil_run(pixels, pw, ph, src, last_pixel, from, to);
                }
                SmoothPiece::Quad { from, control, to } => {
                    let steps = ((from.distance(control) + control.distance(to)).ceil() as usize)
                        .clamp(1, PENCIL_MAX_SUBSTEPS);
                    let mut prev = from;
                    for step in 1..=steps {
                        let t = step as f32 / steps as f32;
                        let p = SmoothPiece::quad_at(from, control, to, t);
                        plot_pencil_run(pixels, pw, ph, src, last_pixel, prev, p);
                        prev = p;
                    }
                }
            }
        }
        return Some(dirty);
    }

    // A perfectly horizontal or vertical piece chain has zero-height/width
    // path bounds, which tiny-skia rejects — nudge the final point by a
    // subpixel epsilon (far below one pixel, invisible at any zoom; the
    // same trick `rasterize_polyline` uses). A fully sub-pixel chain has
    // nothing to stroke: the press dot already covers it.
    if (max_x - min_x) < 0.01 && (max_y - min_y) < 0.01 {
        return Some(dirty);
    }
    let nudge_y = (max_y - min_y) < 0.01;
    let nudge_x = !nudge_y && (max_x - min_x) < 0.01;
    let nudged = |p: Pos2, is_last: bool| {
        let (dx, dy) = match (is_last, nudge_x, nudge_y) {
            (true, _, true) => (0.0, 0.01),
            (true, true, _) => (0.01, 0.0),
            _ => (0.0, 0.0),
        };
        Pos2::new(p.x + dx, p.y + dy)
    };

    let mut builder = PathBuilder::new();
    builder.move_to(first.start().x, first.start().y);
    let last_index = pieces.len() - 1;
    for (i, piece) in pieces.iter().enumerate() {
        let is_last = i == last_index;
        match *piece {
            SmoothPiece::Line { to, .. } => {
                let p = nudged(to, is_last);
                builder.line_to(p.x, p.y);
            }
            SmoothPiece::Quad { control, to, .. } => {
                let p = nudged(to, is_last);
                builder.quad_to(control.x, control.y, p.x, p.y);
            }
        }
    }
    let path = builder.finish()?;

    let mut paint = Paint::default();
    let [r, g, b, a] = match tool {
        Tool::Eraser => [0, 0, 0, 255],
        _ => color.to_srgba_unmultiplied(),
    };
    paint.set_color_rgba8(r, g, b, a);
    paint.anti_alias = true;
    if matches!(tool, Tool::Eraser) {
        paint.blend_mode = BlendMode::DestinationOut;
    }
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

    Some(dirty)
}

#[cfg(test)]
mod tests;
