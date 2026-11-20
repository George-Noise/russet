//! Smoothed freehand strokes (owner request 2026-11-19): the raw
//! frame-boundary segments are gone. Every coalesced `Event::PointerMoved`
//! of a frame feeds the smoothing window ([`SmoothStroke`]) — the classic
//! midpoint-quadratic scheme: the drawn path runs `start → mid(start, p1)
//! → (quad, control p1) → mid(p1, p2) → (quad, control p2) → …`, which is
//! tangent-continuous (C1) at every joint and never overshoots (a
//! quadratic stays inside its control triangle). [`rasterize_smooth_pieces`]
//! paints one frame's pieces in ONE tiny-skia stroke (brush/eraser — the
//! pieces join at their shared midpoints, so no per-segment cap seams) or
//! as the pencil's classic hard 1 px trail following the flattened
//! centerline. The pencil walks a drifting pen ([`PencilTrail`], owner
//! request 2026-11-20): unit steps along the chord-dominant axis, the
//! minor coordinate carried as an EXACT float across the whole gesture —
//! the same fractional phase a single long Bresenham run carries — so a
//! straight stroke renders the same even staircase no matter how the
//! pointer stream is batched per frame (the old per-run Bresenham restarts
//! re-rounded every ~1 px endpoint and left double-covered columns and
//! holes on every diagonal, differently on each computer). The freehand
//! gesture lives in [`crate::app::input`]; [`super::draw_segment`] remains
//! for the press-time touch dot and the MCP paths.

use eframe::egui::{Color32, Pos2};
use tiny_skia::{BlendMode, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

use super::{DirtyRect, MAX_STROKE_SIZE, Tool, blend_pixel, dirty_rect_for};

/// Subdivision cap for the pencil's flattened quadratic centerline: the
/// quads are cut into ~1 px chords so the drifting pen follows the curve's
/// shape, not its control polygon. The cap is a sanity bound for a
/// pathological pointer jump; real pieces are a few pixels long.
const PENCIL_MAX_SUBSTEPS: usize = 4096;

/// Walk-step guard: a canvas-sized jump takes a few thousand unit steps;
/// anything beyond this is a bug and the walk bails out.
const WALK_STEP_GUARD: usize = 8192;

/// The pencil's drifting pen (owner request 2026-11-20): the gesture-level
/// rasterization state carried across pieces AND frames. The pen holds an
/// EXACT float position — its dominant-axis coordinate rides the integer
/// pixel grid, its minor coordinate keeps the fractional phase of the
/// smoothed centerline (never rounded into the state), which is what makes
/// the 1 px staircase even and frame-batch-independent. `last_pixel` is
/// the dedupe state: premultiplied source-over is not idempotent, so a
/// translucent color must not blend into the same pixel twice at the
/// subdivision/frame joints.
#[derive(Clone, Copy, Default, PartialEq, Eq, Debug)]
pub struct PencilTrail {
    pen: Option<Pos2>,
    last_pixel: Option<(i64, i64)>,
}

impl PencilTrail {
    /// The gesture-start state: the press dot's pixel seeds the dedupe so
    /// the first piece does not re-blend over the touch point.
    pub fn starting_from(press_pixel: (i64, i64)) -> Self {
        Self {
            pen: None,
            last_pixel: Some(press_pixel),
        }
    }
}

/// Walks the pen toward `target` in unit steps of the chord-dominant axis,
/// plotting one pixel per step (the pen's rounded position). The minor
/// coordinate advances proportionally and is carried EXACTLY — the same
/// error accumulation a single long Bresenham run performs, with no
/// per-run restarts. A `None` pen initializes on the target (the gesture's
/// first sample; the press dot already covers its pixel).
fn walk_pen(pen: &mut Option<Pos2>, target: Pos2, plot: &mut impl FnMut(i64, i64)) {
    let Some(p) = pen.as_mut() else {
        *pen = Some(target);
        let (x, y) = pencil_pixel(target);
        plot(x, y);
        return;
    };
    for _ in 0..WALK_STEP_GUARD {
        let dx = target.x - p.x;
        let dy = target.y - p.y;
        let (adx, ady) = (dx.abs(), dy.abs());
        if adx < 1.0 && ady < 1.0 {
            return;
        }
        if adx >= ady {
            p.x += if dx > 0.0 { 1.0 } else { -1.0 };
            // adx >= 1 here (the dominant axis carries the remaining step).
            p.y += dy / adx;
        } else {
            p.y += if dy > 0.0 { 1.0 } else { -1.0 };
            p.x += dx / ady;
        }
        plot(p.x.round() as i64, p.y.round() as i64);
    }
}

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

/// Rasterizes one frame's smoothed pieces into the pixmap in a single
/// pass and returns their UNION dirty rectangle. Brush/eraser: one
/// continuous tiny-skia stroke (round caps, AA; the eraser
/// `DestinationOut`) — `trail` and `pin` are ignored. Pencil: the 1 px
/// drifting-pen trail along the flattened centerline — quads subdivide
/// into ~1 px steps, the pen walks unit grid steps with its exact minor
/// phase carried across the whole gesture, consecutive duplicate pixels
/// are skipped (the dedupe in [`PencilTrail`]). `pin` is the RELEASE
/// hook: the closing call passes the raw release point so the trail ends
/// exactly on its pixel (the walk itself stops within one pixel of its
/// target and catches up on the next sample).
pub fn rasterize_smooth_pieces(
    pixmap: &mut Pixmap,
    tool: Tool,
    size: f32,
    color: Color32,
    pieces: &[SmoothPiece],
    trail: &mut PencilTrail,
    pin: Option<Pos2>,
) -> Option<DirtyRect> {
    let width = match tool {
        Tool::Pencil => 1.0,
        Tool::Brush | Tool::Eraser => size.clamp(1.0, MAX_STROKE_SIZE),
        // Unreachable: the preview tools commit through `rasterize_polyline`
        // / `rasterize_shape`, not through this freehand path.
        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text => {
            size.clamp(1.0, MAX_STROKE_SIZE)
        }
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
        let mut pen = trail.pen;
        let mut last_pixel = trail.last_pixel;
        {
            let mut plot = |x: i64, y: i64| {
                if last_pixel == Some((x, y)) {
                    return;
                }
                blend_pixel(pixels, pw, ph, x, y, src);
                last_pixel = Some((x, y));
            };
            for piece in pieces {
                match *piece {
                    SmoothPiece::Line { from, to } => {
                        walk_pen(&mut pen, from, &mut plot);
                        walk_pen(&mut pen, to, &mut plot);
                    }
                    SmoothPiece::Quad { from, control, to } => {
                        walk_pen(&mut pen, from, &mut plot);
                        let steps = ((from.distance(control) + control.distance(to)).ceil()
                            as usize)
                            .clamp(1, PENCIL_MAX_SUBSTEPS);
                        for step in 1..=steps {
                            let t = step as f32 / steps as f32;
                            let p = SmoothPiece::quad_at(from, control, to, t);
                            walk_pen(&mut pen, p, &mut plot);
                        }
                    }
                }
            }
            if let Some(final_point) = pin {
                walk_pen(&mut pen, final_point, &mut plot);
                // The walk stops within one pixel of the target: plot the
                // release pixel itself so the stroke ends exactly where the
                // pointer released (the classic Bresenham end behavior).
                let (x, y) = pencil_pixel(final_point);
                plot(x, y);
                pen = Some(final_point);
            }
        }
        trail.pen = pen;
        trail.last_pixel = last_pixel;
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
