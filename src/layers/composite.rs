//! The display composite of the layers: premultiplied src-over blending of
//! the visible layers (bottom first) into the display buffer, region-based
//! so strokes stay cheap. Split from `layers.rs` 2026-11-16 (pure code
//! motion, the 500-line file guard).
//!
//! [`composite`] is re-exported from [`super`], so every consumer keeps the
//! stable `layers::composite` path (`app.rs` is the only caller — the full
//! `refresh_composite` and the per-stroke `composite_dirty` hot path). The
//! layer state lives in the parent module, the snapshot codec in its
//! [`super::snapshot`] child.

use tiny_skia::Pixmap;

use super::Layer;

/// Blends the visible layers (bottom first) over the region `x, y, w, h`
/// of `dst` (the display buffer): clears the region, then premultiplied
/// src-over per layer. Layers composite at (0,0) — a layer smaller than
/// `dst` only contributes its overlap, a larger one is clipped to the
/// region (owner request 2026-11-17: mixed-size layers are legal — the
/// BOTTOM layer owns the canvas size).
pub fn composite(layers: &[Layer], dst: &mut Pixmap, x: u32, y: u32, w: u32, h: u32) {
    let dst_w = dst.width();
    let dst_h = dst.height();
    let x0 = x.min(dst_w);
    let y0 = y.min(dst_h);
    let x1 = x.saturating_add(w).min(dst_w);
    let y1 = y.saturating_add(h).min(dst_h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    clear_region(dst, x0, y0, x1 - x0, y1 - y0);
    for layer in layers {
        if !layer.visible() {
            continue;
        }
        blend_region(layer.pixmap(), dst, x0, y0, x1 - x0, y1 - y0);
    }
}

/// Zeros the region's bytes (fully transparent).
fn clear_region(dst: &mut Pixmap, x: u32, y: u32, w: u32, h: u32) {
    let stride = dst.width() as usize * 4;
    let data = dst.data_mut();
    for row in 0..h as usize {
        let start = (y as usize + row) * stride + x as usize * 4;
        let end = start + w as usize * 4;
        data[start..end].fill(0);
    }
}

/// Premultiplied src-over of ONE layer's region into `dst`: for every
/// channel `out = src + dst * (255 - src_alpha) / 255` (rounded). With
/// premultiplied data this is exact over-blending for all four channels at
/// once; the alpha-0 and alpha-255 shortcuts keep flat areas bit-exact.
/// The layer's pixmap may be ANY size (mixed sizes are legal since the
/// 2026-11-17 resize model): the region is clipped to the layer's bounds —
/// rows and columns outside it are skipped, never read.
fn blend_region(src_pixmap: &Pixmap, dst: &mut Pixmap, x: u32, y: u32, w: u32, h: u32) {
    // Clip the region to BOTH pixmaps: `dst` is the canvas (already
    // clamped by `composite`), the layer may be smaller (or larger — then
    // the region is inside its bounds by definition of the clip below).
    let src_w = src_pixmap.width();
    let src_h = src_pixmap.height();
    if x >= src_w || y >= src_h {
        return; // the layer does not even reach the region
    }
    let w = w.min(src_w - x);
    let h = h.min(src_h - y);
    let src_stride = src_pixmap.width() as usize * 4;
    let dst_stride = dst.width() as usize * 4;
    let copy_width = w as usize;
    let copy_height = h as usize;
    let src = src_pixmap.data();
    let dst_data = dst.data_mut();
    for row in 0..copy_height {
        let src_row = y as usize + row;
        let src_start = src_row * src_stride + x as usize * 4;
        let dst_start = src_row * dst_stride + x as usize * 4;
        for px in 0..copy_width {
            let s = src_start + px * 4;
            let d = dst_start + px * 4;
            let sa = src[s + 3] as u32;
            if sa == 0 {
                continue;
            }
            if sa == 255 {
                dst_data[d..d + 4].copy_from_slice(&src[s..s + 4]);
            } else {
                let inv = 255 - sa;
                for k in 0..4 {
                    let kept = (dst_data[d + k] as u32 * inv + 127) / 255;
                    dst_data[d + k] = src[s + k].saturating_add(kept as u8);
                }
            }
        }
    }
}
