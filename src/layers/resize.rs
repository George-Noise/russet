//! The Resize-dialog pixel plumbing (owner request 2026-11-17): the
//! Canvas-mode crop/pad around a 3×3 anchor, the Image-mode resampling and
//! the per-layer proportional target size. Pure functions of a pixmap —
//! the orchestration (which layers, which nodes, which
//! [`ResizeInfo`](crate::history_tree::ResizeInfo)) lives in
//! [`super::LayerStack::resize_all`]. Split from `layers.rs` as a child
//! module (the same pattern as [`super::snapshot`] and [`super::composite`],
//! the 500-line file guard); re-exported from [`super`], so every consumer
//! keeps the stable `layers::` paths.
//!
//! Coordinate convention of [`fit_copy_at`]: the destination pixel `(x, y)`
//! shows the source pixel `(x - ox, y - oy)`; a positive offset means the
//! content moves RIGHT/DOWN (padding on the left/top), a negative one means
//! it moves LEFT/UP (crop from the left/top). [`anchor_offset`] derives
//! `(ox, oy)` from a [`ResizeAnchor`](crate::history_tree::ResizeAnchor).
//!
//! Resampling note: [`rescale`] filters the PREMULTIPLIED bytes. Linear
//! filtering commutes with premultiplication (every channel including alpha
//! is filtered with the same weights), so the result is the correct
//! straight-alpha downscale/upscale — filtering straight RGBA instead would
//! bleed color out of transparent pixels.

use tiny_skia::Pixmap;

use crate::history_tree::{ResizeAnchor, ResizeFilter};

/// Where the old content lands in the new buffer for an anchor: growing
/// adds the new space on the sides AWAY from the anchor, shrinking crops
/// them. Center uses FLOOR semantics — an odd crop removes the extra pixel
/// from the left/top, an odd pad adds it on the right/bottom (deterministic;
/// pinned by unit tests).
pub fn anchor_offset(
    anchor: ResizeAnchor,
    old_w: u32,
    old_h: u32,
    new_w: u32,
    new_h: u32,
) -> (i64, i64) {
    let dx = new_w as i64 - old_w as i64;
    let dy = new_h as i64 - old_h as i64;
    let ox = match anchor {
        ResizeAnchor::TopLeft | ResizeAnchor::MiddleLeft | ResizeAnchor::BottomLeft => 0,
        ResizeAnchor::TopCenter | ResizeAnchor::Center | ResizeAnchor::BottomCenter => {
            dx.div_euclid(2)
        }
        ResizeAnchor::TopRight | ResizeAnchor::MiddleRight | ResizeAnchor::BottomRight => dx,
    };
    let oy = match anchor {
        ResizeAnchor::TopLeft | ResizeAnchor::TopCenter | ResizeAnchor::TopRight => 0,
        ResizeAnchor::MiddleLeft | ResizeAnchor::Center | ResizeAnchor::MiddleRight => {
            dy.div_euclid(2)
        }
        ResizeAnchor::BottomLeft | ResizeAnchor::BottomCenter | ResizeAnchor::BottomRight => dy,
    };
    (ox, oy)
}

/// Generalized top-left crop/pad: the destination pixel `(x, y)` shows the
/// source pixel `(x - ox, y - oy)` when that source pixel exists; the rest
/// stays transparent (premultiplied zero). The zero-offset case is
/// [`super::snapshot::fit_copy`]'s exact behavior.
pub(super) fn fit_copy_at(src: &Pixmap, width: u32, height: u32, ox: i64, oy: i64) -> Pixmap {
    if src.width() == width && src.height() == height && ox == 0 && oy == 0 {
        return src.clone();
    }
    let Some(mut out) = Pixmap::new(width, height) else {
        return src.clone();
    };
    let src_w = src.width() as i64;
    let src_h = src.height() as i64;
    let src_stride = src.width() as usize * 4;
    let dst_stride = width as usize * 4;
    let src_data = src.data();
    let dst = out.data_mut();
    for dst_y in 0..height as i64 {
        let src_y = dst_y - oy;
        if src_y < 0 || src_y >= src_h {
            continue; // the row stays transparent
        }
        // Destination columns that map into the source row: dx in
        // [ox, ox + src_w) intersected with [0, width).
        let dx_start = ox.max(0);
        let dx_end = (ox + src_w).min(width as i64);
        if dx_end <= dx_start {
            continue;
        }
        let src_start = (src_y as usize) * src_stride + (dx_start - ox) as usize * 4;
        let dst_start = (dst_y as usize) * dst_stride + dx_start as usize * 4;
        let count = (dx_end - dx_start) as usize * 4;
        dst[dst_start..dst_start + count].copy_from_slice(&src_data[src_start..src_start + count]);
    }
    out
}

/// Rescales a pixmap to `width × height` with the chosen
/// [`ResizeFilter`](crate::history_tree::ResizeFilter), premultiplied-correct
/// (see the module note). Same-size requests return a clone unchanged, so a
/// degenerate ×1.0 Image resize is bit-exact.
pub(super) fn rescale(src: &Pixmap, filter: ResizeFilter, width: u32, height: u32) -> Pixmap {
    if (src.width() == width && src.height() == height) || width == 0 || height == 0 {
        return src.clone();
    }
    let Some(buffer) = image::RgbaImage::from_raw(src.width(), src.height(), src.data().to_vec())
    else {
        return src.clone(); // unreachable for a real pixmap; stay safe anyway
    };
    let resized = image::imageops::resize(&buffer, width, height, filter.image_filter());
    let mut out = match Pixmap::new(width, height) {
        Some(out) => out,
        None => return src.clone(),
    };
    out.data_mut().copy_from_slice(resized.as_raw());
    out
}

/// A layer's proportional target in an Image-mode resize: every layer scales
/// by the CANVAS factor (new canvas / old canvas), so the composite shrinks
/// and grows as one picture; the bottom layer (canvas-sized) lands exactly
/// on the target. Rounded with integer math (no float drift), minimum 1.
pub fn scaled_layer_size(
    layer_w: u32,
    layer_h: u32,
    old_canvas_w: u32,
    old_canvas_h: u32,
    new_canvas_w: u32,
    new_canvas_h: u32,
) -> (u32, u32) {
    let scale = |size: u32, old: u32, new: u32| -> u32 {
        if old == 0 {
            return size.max(1);
        }
        // round(size * new / old) without floats: (2*size*new + old) / (2*old)
        let scaled =
            ((u64::from(size) * 2 * u64::from(new)) + u64::from(old)) / (u64::from(old) * 2);
        scaled.max(1).min(u64::from(u32::MAX)) as u32
    };
    (
        scale(layer_w, old_canvas_w, new_canvas_w),
        scale(layer_h, old_canvas_h, new_canvas_h),
    )
}
