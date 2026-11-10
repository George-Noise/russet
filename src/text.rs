//! Text rasterization through egui's own text stack (owner request
//! 2026-11-10, Text tool). The text is laid out as a galley with the font
//! size in points chosen so that the PHYSICAL glyph size equals the
//! requested canvas-pixel size, and the glyph bitmaps are then copied out
//! of egui's font atlas into the canvas [`Pixmap`]. This keeps the
//! committed pixels identical to the on-canvas preview (both come from the
//! same layout engine and fonts) without adding any dependency.

use eframe::egui::{self, Align, Color32, Context, FontId, Pos2};
use tiny_skia::Pixmap;

use crate::tools::{DirtyRect, blend_pixel};

/// Renders `text` (may contain `\n`) into `pixmap` with the top-left of the
/// text box at `origin` (canvas coordinates). `font_size` is the font size
/// in canvas pixels. Returns the dirty rectangle, or `None` when nothing
/// was drawn (empty/blank text, or all glyphs landed outside the canvas).
pub fn rasterize_text(
    ctx: &Context,
    pixmap: &mut Pixmap,
    text: &str,
    origin: Pos2,
    font_size: f32,
    color: Color32,
) -> Option<DirtyRect> {
    if text.trim().is_empty() {
        return None;
    }
    let font_size = font_size.clamp(4.0, 512.0);

    // The galley is laid out at the context's pixels_per_point; its
    // "physical" pixel scale is `font_size_pts * ppp`, so request a point
    // size that makes the physical glyph size exactly `font_size` canvas
    // pixels. (Layouting at ppp = 1.0 directly is not possible through the
    // public `FontsView` API.)
    let ppp = ctx.pixels_per_point().max(0.1);
    let mut job = egui::text::LayoutJob::simple(
        text.to_string(),
        FontId::proportional(font_size / ppp),
        Color32::WHITE, // the atlas stores coverage; the tint is applied below
        f32::INFINITY,  // no wrapping: lines only break at explicit \n
    );
    job.halign = Align::LEFT;
    let galley = ctx.fonts_mut(|fonts| fonts.layout_job(job));
    debug_assert!((galley.pixels_per_point - ppp).abs() < 1.0);
    // Rasterization happens during layout (atlas allocation), so the atlas
    // image already contains every glyph of this galley.
    let atlas = ctx.fonts(|fonts| fonts.image());
    let [atlas_w, atlas_h] = atlas.size;

    let [cr, cg, cb, _] = color.to_srgba_unmultiplied();
    let pw = pixmap.width() as i64;
    let ph = pixmap.height() as i64;
    let pixels = pixmap.pixels_mut();

    // Bounding box of every canvas pixel actually touched.
    let mut min_x = i64::MAX;
    let mut min_y = i64::MAX;
    let mut max_x = i64::MIN;
    let mut max_y = i64::MIN;

    for row in &galley.rows {
        let row_pos = row.pos;
        for glyph in &row.row.glyphs {
            let uv = glyph.uv_rect;
            if uv.min[0] >= uv.max[0] || uv.min[1] >= uv.max[1] {
                continue; // whitespace / empty glyph
            }
            // Quad top-left on the canvas: row offset + baseline position
            // + the glyph bitmap offset, all in points → scale by ppp into
            // physical pixels (== canvas pixels at this sizing).
            let left =
                ((row_pos.x + glyph.pos.x + uv.offset.x) * ppp).round() as i64 + origin.x as i64;
            let top =
                ((row_pos.y + glyph.pos.y + uv.offset.y) * ppp).round() as i64 + origin.y as i64;

            for ty in uv.min[1]..uv.max[1] {
                for tx in uv.min[0]..uv.max[0] {
                    if tx as usize >= atlas_w || ty as usize >= atlas_h {
                        continue;
                    }
                    let src = atlas.pixels[ty as usize * atlas_w + tx as usize];
                    // Coverage glyphs are white with alpha: use the alpha as
                    // the coverage and tint with the target color. Color
                    // glyphs (emoji) keep their own (premultiplied) pixels.
                    let src_px: [u32; 4] = if glyph.is_color {
                        [
                            src.r() as u32,
                            src.g() as u32,
                            src.b() as u32,
                            src.a() as u32,
                        ]
                    } else {
                        let coverage = src.a() as u32;
                        [
                            cr as u32 * coverage / 255,
                            cg as u32 * coverage / 255,
                            cb as u32 * coverage / 255,
                            coverage,
                        ]
                    };
                    if src_px[3] == 0 {
                        continue;
                    }
                    let dx = left + (tx - uv.min[0]) as i64;
                    let dy = top + (ty - uv.min[1]) as i64;
                    blend_pixel(pixels, pw, ph, dx, dy, src_px);
                    if dx < min_x {
                        min_x = dx;
                    }
                    if dy < min_y {
                        min_y = dy;
                    }
                    if dx >= max_x {
                        max_x = dx + 1;
                    }
                    if dy >= max_y {
                        max_y = dy + 1;
                    }
                }
            }
        }
    }

    if max_x <= min_x || max_y <= min_y {
        return None;
    }
    // Same +2 padding rule as every other tool's dirty rect.
    let x0 = (min_x - 2).max(0) as u32;
    let y0 = (min_y - 2).max(0) as u32;
    let x1 = (max_x + 2).min(pw) as u32;
    let y1 = (max_y + 2).min(ph) as u32;
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

#[cfg(test)]
mod tests;
