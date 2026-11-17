//! The snapshot codec of the layers: lossless PNG roundtrips of a layer's
//! raw premultiplied bytes plus the top-left crop/pad helper
//! ([`fit_copy`]). Split from `layers.rs` 2026-11-16 (pure code motion, the
//! 500-line file guard): the pixel plumbing lives here, the layer state
//! ([`super::Layer`], [`super::LayerStack`]) stays in the parent module.
//!
//! Snapshot format note: the PNGs are an INTERNAL RAM container, never
//! shown or saved, so the premultiplied bytes go in and come out untouched
//! — `decode(encode(x)) == x` bit-for-bit. A straight-alpha PNG of
//! premultiplied data could NOT roundtrip losslessly (the 8-bit
//! premultiply/demultiply roundtrip is lossy) — exactly what a snapshot
//! must not be.
//!
//! The public functions are re-exported from [`super`], so every consumer
//! keeps the stable `layers::` paths (`crate::layers::encode_snapshot` and
//! friends, referenced from `history_tree.rs` and `app.rs`).

use tiny_skia::Pixmap;

/// Lossless fast PNG of a pixmap's raw premultiplied bytes (see the module
/// note on why no premultiply conversion happens here). Uses the `image`
/// crate's FAST compression (already a dependency — no new crates); an
/// encoding failure (practically impossible for in-memory RGBA8) yields an
/// empty snapshot that decodes into a clear error instead of corruption.
pub fn encode_snapshot(pixmap: &Pixmap) -> Vec<u8> {
    let mut png = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut png,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::Up,
        );
        let encoded = image::ImageEncoder::write_image(
            encoder,
            pixmap.data(),
            pixmap.width(),
            pixmap.height(),
            image::ExtendedColorType::Rgba8,
        );
        if encoded.is_err() {
            png.clear();
        }
    }
    png
}

/// Decodes a node snapshot back into a pixmap. The bytes come back exactly
/// as they went in ([`encode_snapshot`] is lossless), so they can be
/// treated as premultiplied right away.
pub fn decode_snapshot(bytes: &[u8]) -> Result<Pixmap, String> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|err| format!("Corrupt history snapshot: {err}"))?;
    let rgba = image.to_rgba8();
    let Some(mut pixmap) = Pixmap::new(rgba.width(), rgba.height()) else {
        return Err("Invalid snapshot size".to_string());
    };
    pixmap.data_mut().copy_from_slice(rgba.as_raw());
    Ok(pixmap)
}

/// Fits a pixmap onto a target size, TOP-LEFT anchored: same size —
/// unchanged; larger — the top-left crop; smaller — padded with
/// transparency at the bottom/right. This is the "keep the canvas size"
/// branch of the base-choice dialog (the new bottom layer must match the
/// canvas without shifting the composite) and the zero-offset case of
/// `resize::fit_copy_at`.
pub fn fit_copy(src: &Pixmap, width: u32, height: u32) -> Pixmap {
    if src.width() == width && src.height() == height {
        return src.clone();
    }
    let Some(mut out) = Pixmap::new(width, height) else {
        return src.clone();
    };
    let copy_width = src.width().min(width) as usize * 4;
    let copy_height = src.height().min(height) as usize;
    let src_stride = src.width() as usize * 4;
    let dst_stride = width as usize * 4;
    let src_data = src.data();
    let dst = out.data_mut();
    for row in 0..copy_height {
        let src_start = row * src_stride;
        let dst_start = row * dst_stride;
        dst[dst_start..dst_start + copy_width]
            .copy_from_slice(&src_data[src_start..src_start + copy_width]);
    }
    out
}

/// An empty (fully transparent) pixmap with the Canvas clamps; falls back
/// to 1×1 like `Canvas::new` when the allocation is refused.
pub(super) fn pixmap_of(width: u32, height: u32) -> Pixmap {
    let width = width.clamp(1, 4096);
    let height = height.clamp(1, 4096);
    // Pixmap::new yields a fully transparent buffer.
    Pixmap::new(width, height).unwrap_or_else(|| unreachable!("1x1 pixmap"))
}
