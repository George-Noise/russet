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

    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    pub fn pixmap_mut(&mut self) -> &mut Pixmap {
        &mut self.pixmap
    }

    pub fn set_pixmap(&mut self, pixmap: Pixmap) {
        self.width = pixmap.width();
        self.height = pixmap.height();
        self.pixmap = pixmap;
    }

    pub fn resize_canvas(&mut self, width: u32, height: u32) {
        let width = width.clamp(1, 4096);
        let height = height.clamp(1, 4096);
        if width == self.width && height == self.height {
            return;
        }
        let Some(mut next) = Pixmap::new(width, height) else {
            return;
        };
        let copy_width = width.min(self.width) as usize * 4;
        let copy_height = height.min(self.height) as usize;
        let src = self.pixmap.data();
        let dst = next.data_mut();
        let src_stride = self.width as usize * 4;
        let dst_stride = width as usize * 4;
        for row in 0..copy_height {
            let src_start = row * src_stride;
            let dst_start = row * dst_stride;
            dst[dst_start..dst_start + copy_width]
                .copy_from_slice(&src[src_start..src_start + copy_width]);
        }
        self.width = width;
        self.height = height;
        self.pixmap = next;
    }

    pub fn region_bytes(&self, x: u32, y: u32, w: u32, h: u32) -> Vec<u8> {
        let x0 = x.min(self.width);
        let y0 = y.min(self.height);
        let x1 = x.saturating_add(w).min(self.width);
        let y1 = y.saturating_add(h).min(self.height);
        if x1 <= x0 || y1 <= y0 {
            return Vec::new();
        }
        let region_w = (x1 - x0) as usize;
        let region_h = (y1 - y0) as usize;
        let mut out = Vec::with_capacity(region_w * region_h * 4);
        let src = self.pixmap.data();
        let stride = self.width as usize * 4;
        for row in 0..region_h {
            let start = (y0 as usize + row) * stride + x0 as usize * 4;
            out.extend_from_slice(&src[start..start + region_w * 4]);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn region_bytes_has_expected_size_and_corners() {
        let mut canvas = Canvas::new(4, 4, Color32::WHITE);
        let stride = canvas.width as usize * 4;
        // offset(row, col) = row * stride + col * 4 — byte offset of a pixel.
        let offset = |row: usize, col: usize| row * stride + col * 4;
        {
            let data = canvas.pixmap_mut().data_mut();
            let top_left = offset(1, 1);
            data[top_left..top_left + 4].copy_from_slice(&[10, 20, 30, 255]);
            let bottom_right = offset(2, 2);
            data[bottom_right..bottom_right + 4].copy_from_slice(&[40, 50, 60, 255]);
        }

        let region = canvas.region_bytes(1, 1, 2, 2);
        assert_eq!(region.len(), 2 * 2 * 4);
        assert_eq!(&region[0..4], &[10, 20, 30, 255]);
        assert_eq!(&region[12..16], &[40, 50, 60, 255]);
    }

    #[test]
    fn region_bytes_clips_out_of_bounds() {
        let canvas = Canvas::new(4, 4, Color32::WHITE);
        assert!(canvas.region_bytes(3, 3, 10, 10).len() == 4);
        assert!(canvas.region_bytes(10, 10, 2, 2).is_empty());
    }
}
