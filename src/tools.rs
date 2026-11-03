use eframe::egui::{Color32, Pos2};
use tiny_skia::{BlendMode, LineCap, Paint, PathBuilder, Pixmap, Stroke, Transform};

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
