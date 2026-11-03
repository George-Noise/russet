use eframe::egui::Color32;
use tiny_skia::Pixmap;

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pixmap: Pixmap,
}

impl Canvas {
    pub fn new(width: u32, height: u32, color: Color32) -> Self {
        let width = width.clamp(1, 4096);
        let height = height.clamp(1, 4096);
        let mut pixmap = match Pixmap::new(width, height) {
            Some(pixmap) => pixmap,
            None => Pixmap::new(1, 1).unwrap_or_else(|| unreachable!("1x1 pixmap")),
        };
        pixmap.fill(tiny_skia::Color::from_rgba8(
            color.r(),
            color.g(),
            color.b(),
            color.a(),
        ));
        Self {
            width,
            height,
            pixmap,
        }
    }

    pub fn data(&self) -> &[u8] {
        self.pixmap.data()
    }
}
