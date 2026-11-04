use eframe::egui::{Color32, Pos2};
use tiny_skia::{
    BlendMode, LineCap, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Stroke, Transform,
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Brush,
    Eraser,
}

pub struct ToolState {
    pub tool: Tool,
    pub size: f32,
    pub primary: Color32,
    pub secondary: Color32,
}

impl ToolState {
    pub fn new() -> Self {
        Self {
            tool: Tool::Pencil,
            size: 8.0,
            primary: Color32::BLACK,
            secondary: Color32::WHITE,
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
mod tests {
    use super::*;
    use eframe::egui::pos2;

    fn line_points(x0: i64, y0: i64, x1: i64, y1: i64) -> Vec<(i64, i64)> {
        let mut pts = Vec::new();
        bresenham_line(x0, y0, x1, y1, |x, y| pts.push((x, y)));
        pts
    }

    #[test]
    fn diagonal_is_a_one_pixel_staircase() {
        assert_eq!(
            line_points(0, 0, 3, 3),
            vec![(0, 0), (1, 1), (2, 2), (3, 3)]
        );
    }

    #[test]
    fn shallow_line_has_exactly_one_pixel_per_column() {
        let pts = line_points(0, 0, 7, 3);
        assert_eq!(pts.len(), 8);
        for col in 0..=7 {
            assert_eq!(pts.iter().filter(|&&(x, _)| x == col).count(), 1);
        }
    }

    #[test]
    fn steep_line_has_exactly_one_pixel_per_row() {
        let pts = line_points(0, 0, 3, 7);
        assert_eq!(pts.len(), 8);
        for row in 0..=7 {
            assert_eq!(pts.iter().filter(|&&(_, y)| y == row).count(), 1);
        }
    }

    #[test]
    fn single_point_plots_one_pixel() {
        assert_eq!(line_points(5, 5, 5, 5), vec![(5, 5)]);
    }

    #[test]
    fn pencil_draws_hard_one_pixel_line_on_pixmap() {
        let mut pixmap = Pixmap::new(16, 16).expect("16x16 pixmap");
        let from = pos2(0.4, 0.6); // rounds to (0, 1)
        let to = pos2(7.6, 3.4); // rounds to (8, 3)
        assert!(draw_segment(&mut pixmap, Tool::Pencil, 1.0, Color32::BLACK, from, to).is_some());
        let black: Vec<(usize, usize)> = pixmap
            .pixels()
            .iter()
            .enumerate()
            .filter(|(_, c)| c.alpha() == 255 && c.red() == 0)
            .map(|(i, _)| (i % 16, i / 16))
            .collect();
        assert_eq!(black.len(), 9); // shallow line: one pixel per column
        for col in 0..16 {
            assert!(black.iter().filter(|&&(x, _)| x == col).count() <= 1);
        }
    }
}
