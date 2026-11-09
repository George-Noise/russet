use eframe::egui::{Color32, Pos2};
use tiny_skia::{
    BlendMode, FillRule, LineCap, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Stroke,
    Transform,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Brush,
    Eraser,
    /// Samples the canvas pixel color into the primary/secondary well.
    Eyedropper,
    /// Straight lines (drag) and polylines/curves (click points).
    Line,
    /// Rectangle and ellipse outlines (or fills), dragged as a preview.
    Shape,
    /// Places text at the click point through a small input window.
    Text,
}

/// Sub-mode of the [`Tool::Line`] tool (owner request 2026-11-10).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LineMode {
    /// Press–drag–release draws one straight segment.
    Straight,
    /// Click to add vertices; Enter/double-click commits, Esc cancels.
    Curve,
}

/// Sub-mode of the [`Tool::Shape`] tool (owner request 2026-11-10).
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
}

pub struct ToolState {
    pub tool: Tool,
    pub size: f32,
    pub primary: Color32,
    pub secondary: Color32,
    pub line_mode: LineMode,
    pub shape_kind: ShapeKind,
    /// Fill shapes with the fill color (the secondary well).
    pub shape_fill: bool,
    /// Outline shapes with a `size`-wide stroke (the primary well).
    /// The Tools window keeps at least one of `shape_fill`/`shape_outline`
    /// enabled, so a shape always paints something.
    pub shape_outline: bool,
    /// Font size (canvas pixels) used by the Text tool.
    pub font_size: f32,
}

impl ToolState {
    pub fn new() -> Self {
        Self {
            tool: Tool::Pencil,
            size: 8.0,
            primary: Color32::BLACK,
            secondary: Color32::WHITE,
            line_mode: LineMode::Straight,
            shape_kind: ShapeKind::Rectangle,
            shape_fill: false,
            shape_outline: true,
            font_size: 24.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn draw_segment(
    pixmap: &mut Pixmap,
    tool: Tool,
    size: f32,
    color: Color32,
    from: Pos2,
    to: Pos2,
) -> Option<DirtyRect> {
    let width = match tool {
        Tool::Pencil => 1.0,
        Tool::Brush | Tool::Eraser => size.clamp(1.0, 100.0),
        // Unreachable: the preview tools commit through `rasterize_polyline`
        // / `rasterize_shape`, not through this freehand path.
        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text => size.clamp(1.0, 100.0),
    };

    let pad = (width * 0.5).ceil() + 2.0;
    let min_x = from.x.min(to.x) - pad;
    let min_y = from.y.min(to.y) - pad;
    let max_x = from.x.max(to.x) + pad;
    let max_y = from.y.max(to.y) + pad;
    let x0 = min_x.floor().max(0.0) as u32;
    let y0 = min_y.floor().max(0.0) as u32;
    let x1 = max_x.ceil().min(pixmap.width() as f32) as u32;
    let y1 = max_y.ceil().min(pixmap.height() as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    if matches!(tool, Tool::Pencil) {
        // Classic integer Bresenham between the rounded endpoints: exactly one
        // pixel per step, like the pencil in Paint / Paint.NET. tiny-skia's
        // non-AA scanline stroke crosses its 50% coverage threshold differently
        // depending on the subpixel position, which covers two pixels in some
        // places of a diagonal and one in others (ragged 1px/2px lines).
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        // Premultiplied source components (the Pixmap stores premultiplied RGBA).
        let src = (
            r as u32 * a as u32 / 255,
            g as u32 * a as u32 / 255,
            b as u32 * a as u32 / 255,
        );
        let pw = pixmap.width() as i64;
        let ph = pixmap.height() as i64;
        let pixels = pixmap.pixels_mut();
        bresenham_line(
            from.x.round() as i64,
            from.y.round() as i64,
            to.x.round() as i64,
            to.y.round() as i64,
            |x, y| {
                if x < 0 || y < 0 || x >= pw || y >= ph {
                    return;
                }
                let dst = pixels[(y * pw + x) as usize];
                // src-over: out = src + dst * (1 - src_alpha), rounded.
                let ia = 255 - a as u32;
                let mix = |s: u32, d: u8| (s + (d as u32 * ia + 127) / 255) as u8;
                let out = PremultipliedColorU8::from_rgba(
                    mix(src.0, dst.red()),
                    mix(src.1, dst.green()),
                    mix(src.2, dst.blue()),
                    mix(a as u32, dst.alpha()),
                )
                .unwrap_or(dst);
                pixels[(y * pw + x) as usize] = out;
            },
        );
        return Some(DirtyRect {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        });
    }

    let mut builder = PathBuilder::new();
    builder.move_to(from.x, from.y);
    let (end_x, end_y) = if (to.x - from.x).abs() < 0.01 && (to.y - from.y).abs() < 0.01 {
        (from.x + 0.01, from.y)
    } else {
        (to.x, to.y)
    };
    builder.line_to(end_x, end_y);
    let path = builder.finish()?;

    let mut paint = Paint::default();
    let [r, g, b, a] = match tool {
        Tool::Eraser => [0, 0, 0, 255],
        Tool::Pencil | Tool::Brush => color.to_srgba_unmultiplied(),
        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text => color.to_srgba_unmultiplied(),
    };
    paint.set_color_rgba8(r, g, b, a);
    paint.anti_alias = !matches!(tool, Tool::Pencil);
    if matches!(tool, Tool::Eraser) {
        paint.blend_mode = BlendMode::DestinationOut;
    }

    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

    Some(DirtyRect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

/// Blends one premultiplied source pixel over the pixmap pixel at `(x, y)`
/// (premultiplied source-over, rounded). Shared by the pencil plotter and
/// the text rasterizer. Out-of-bounds coordinates are ignored.
pub(crate) fn blend_pixel(
    pixels: &mut [PremultipliedColorU8],
    width: i64,
    height: i64,
    x: i64,
    y: i64,
    src: [u32; 4],
) {
    if x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    let dst = pixels[(y * width + x) as usize];
    // src-over: out = src + dst * (1 - src_alpha), rounded.
    let ia = 255 - src[3];
    let mix = |s: u32, d: u8| (s + (d as u32 * ia + 127) / 255) as u8;
    if let Some(out) = PremultipliedColorU8::from_rgba(
        mix(src[0], dst.red()),
        mix(src[1], dst.green()),
        mix(src[2], dst.blue()),
        mix(src[3], dst.alpha()),
    ) {
        pixels[(y * width + x) as usize] = out;
    }
}

/// Padded, canvas-clipped dirty rectangle for a shape whose bounding box
/// (canvas coordinates, any orientation) was painted with `width`-wide
/// strokes — same padding rule as [`draw_segment`].
fn dirty_rect_for(
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
) -> Option<DirtyRect> {
    let width = width.clamp(1.0, 100.0);
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
        let mut paint = Paint::default();
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        paint.set_color_rgba8(r, g, b, a);
        paint.anti_alias = true;
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
        return Some(dirty);
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

    let mut paint = Paint::default();
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    paint.set_color_rgba8(r, g, b, a);
    paint.anti_alias = true;
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);

    Some(dirty)
}

/// Rasterizes a rectangle or an ellipse between two opposite corners
/// (either orientation). `fill` fills the interior (the secondary color);
/// `outline` strokes the boundary with a `width`-wide antialiased stroke
/// (the primary color) — both may be enabled at once, then the fill goes
/// first and the stroke covers it. The Tools window never lets both be
/// `None`; if they are, nothing is drawn. Returns the dirty rectangle.
pub fn rasterize_shape(
    pixmap: &mut Pixmap,
    kind: ShapeKind,
    from: Pos2,
    to: Pos2,
    width: f32,
    outline: Option<Color32>,
    fill: Option<Color32>,
) -> Option<DirtyRect> {
    let width = width.clamp(1.0, 100.0);
    let (min_x, max_x) = (from.x.min(to.x), from.x.max(to.x));
    let (min_y, max_y) = (from.y.min(to.y), from.y.max(to.y));
    if max_x - min_x < 0.01 || max_y - min_y < 0.01 {
        return None;
    }
    if outline.is_none() && fill.is_none() {
        return None;
    }
    let bounds = tiny_skia::Rect::from_xywh(min_x, min_y, max_x - min_x, max_y - min_y)?;

    let mut builder = PathBuilder::new();
    match kind {
        ShapeKind::Rectangle => builder.push_rect(bounds),
        ShapeKind::Ellipse => builder.push_oval(bounds),
    }
    let path = builder.finish()?;
    let dirty = dirty_rect_for(pixmap, min_x, min_y, max_x, max_y, width)?;

    if let Some(fill) = fill {
        let mut paint = Paint::default();
        let [r, g, b, a] = fill.to_srgba_unmultiplied();
        paint.set_color_rgba8(r, g, b, a);
        paint.anti_alias = true;
        pixmap.fill_path(
            &path,
            &paint,
            FillRule::Winding,
            Transform::identity(),
            None,
        );
    }
    if let Some(outline) = outline {
        let mut paint = Paint::default();
        let [r, g, b, a] = outline.to_srgba_unmultiplied();
        paint.set_color_rgba8(r, g, b, a);
        paint.anti_alias = true;
        let stroke = Stroke {
            width,
            line_cap: LineCap::Round,
            ..Default::default()
        };
        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
    }

    Some(dirty)
}

/// Samples the canvas pixel at integer coordinates and returns it as a
/// straight-alpha color for the color wells (the eyedropper tool).
pub fn pick_color(pixmap: &Pixmap, x: u32, y: u32) -> Option<Color32> {
    let pixel = pixmap.pixel(x, y)?.demultiply();
    Some(Color32::from_rgba_unmultiplied(
        pixel.red(),
        pixel.green(),
        pixel.blue(),
        pixel.alpha(),
    ))
}

/// Integer Bresenham line, all quadrants, both endpoints inclusive; `plot`
/// receives every pixel of the line, one pixel per step.
fn bresenham_line(x0: i64, y0: i64, x1: i64, y1: i64, mut plot: impl FnMut(i64, i64)) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        plot(x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests;
