//! The pixel/byte codecs of the `file_io` module (SPEC §3.7): the decode
//! conversion ([`pixmap_from_image`] — the premultiplied-alpha canvas
//! invariant), the PNG encoders (the encoder-default one shared with the
//! MCP snapshot tool, the slider-driven one behind the "Save Image"
//! dialog's Compression knob), the JPEG encoder (transparency flattened
//! onto white, §5.1), the BMP encoder (owner request 2026-11-26, F17's
//! decision — the `bmp` feature flag on the pinned crate, no new
//! dependencies), plus the two encoder defaults the dialog prefill and
//! the path-based MCP export start from. Pure plumbing — no platform
//! branches, no dialogs; the platform glue stays in the root's
//! `native`/`web` mods and the root re-exports everything bin code
//! consumes.
//!
//! Split-off note (owner request 2026-11-21): moved verbatim from
//! `file_io.rs` (the ~500-line guard trip) — pure code motion.

use tiny_skia::Pixmap;

/// The JPEG quality the path-based export (the MCP `export_png` tool) uses
/// and the "Save Image" dialog starts from (the value the old hardcoded
/// encoder used — SPEC §3.7). The dialog keeps the user's last choice
/// across opens.
pub(crate) const DEFAULT_JPEG_QUALITY: u8 = 90;

/// The PNG compression effort the path-based exports use and the "Save
/// Image" dialog starts from (6 = the pinned encoder's balanced deflate
/// level — flate2's own default, SPEC §3.7). The dialog keeps the user's
/// last choice across opens.
pub(crate) const DEFAULT_PNG_COMPRESSION: u8 = 6;

pub(crate) fn pixmap_from_image(image: image::DynamicImage) -> Result<Pixmap, String> {
    let rgba = image.to_rgba8();
    let (width, height) = rgba.dimensions();
    let mut pixmap = Pixmap::new(width, height).ok_or_else(|| "Invalid image size".to_string())?;
    for (dst, src) in pixmap.pixels_mut().iter_mut().zip(rgba.pixels()) {
        let [r, g, b, a] = src.0;
        *dst = tiny_skia::ColorU8::from_rgba(r, g, b, a).premultiply();
    }
    Ok(pixmap)
}

/// The slider's 0..=9 value mapped onto the pinned encoder's compression
/// efforts (SPEC §3.7): `CompressionType::Level` — png maps Level(0) to
/// NoCompression (the rows are stored raw) and Level(1..) to the EXACT
/// deflate level 1..=9, so every slider stop is a real, distinct setting:
/// higher = a smaller file, a slower encode, THE SAME pixels (PNG is
/// lossless at every level — the promise the dialog's Lossless row
/// displays). Out-of-range values clamp to 9 — no panic can reach the
/// user-data path. Filters stay Adaptive at every level.
pub(crate) fn png_compression_type(compression: u8) -> image::codecs::png::CompressionType {
    image::codecs::png::CompressionType::Level(compression.min(9))
}

/// The pixmap's premultiplied bytes demultiplied into an `RgbaImage` — the
/// source both PNG encoders (default and slider-driven) build from.
fn demultiplied_rgba(pixmap: &Pixmap) -> Result<image::RgbaImage, String> {
    let mut bytes = Vec::with_capacity(pixmap.width() as usize * pixmap.height() as usize * 4);
    for pixel in pixmap.pixels() {
        let color = pixel.demultiply();
        bytes.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
    }
    image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), bytes)
        .ok_or_else(|| "Failed to build PNG image".to_string())
}

/// Shared with the MCP server (feature `mcp`): the snapshot tool returns
/// the canvas as PNG bytes. The settings are the encoder's OWN defaults
/// (`PngEncoder::new` = the Fast fdeflate effort + Adaptive filter) — the
/// settings every PNG save used before the dialog's Compression slider
/// existed; the MCP export has no dialog, so it deliberately stays on
/// them, off the slider's 0..=9 scale.
#[cfg_attr(not(feature = "mcp"), allow(dead_code))]
pub(crate) fn encode_png(pixmap: &Pixmap) -> Result<Vec<u8>, String> {
    let image = demultiplied_rgba(pixmap)?;
    let mut png = Vec::new();
    image
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .map_err(|err| format!("Failed to encode PNG: {err}"))?;
    Ok(png)
}

/// PNG encode with the "Save Image" dialog's compression choice (owner
/// request 2026-11-20 — the slider). The value rides
/// [`png_compression_type`]: any `u8` maps onto a legal effort, so no
/// panic ever reaches the user-data path.
pub(crate) fn encode_png_with_compression(
    pixmap: &Pixmap,
    compression: u8,
) -> Result<Vec<u8>, String> {
    let image = demultiplied_rgba(pixmap)?;
    let mut png = Vec::new();
    use image::ImageEncoder;
    let mut cursor = std::io::Cursor::new(&mut png);
    let encoder = image::codecs::png::PngEncoder::new_with_quality(
        &mut cursor,
        png_compression_type(compression),
        image::codecs::png::FilterType::Adaptive,
    );
    encoder
        .write_image(
            image.as_raw(),
            image.width(),
            image.height(),
            image::ExtendedColorType::Rgba8,
        )
        .map_err(|err| format!("Failed to encode PNG: {err}"))?;
    Ok(png)
}

/// JPEG encode with the "Save Image" dialog's quality choice (owner
/// request 2026-11-19 — the quality used to be hardcoded 90). Transparent
/// pixels flatten onto white (§5.1). The quality is CLAMPED 1..=100: the
/// slider is not the only writer and the encoder rejects out-of-range
/// values — no panic ever reaches the user-data path.
pub(crate) fn encode_jpeg_with_quality(pixmap: &Pixmap, quality: u8) -> Result<Vec<u8>, String> {
    let quality = quality.clamp(1, 100);
    let mut bytes = Vec::with_capacity(pixmap.width() as usize * pixmap.height() as usize * 3);
    for pixel in pixmap.pixels() {
        let alpha = u32::from(pixel.alpha());
        let red = (u32::from(pixel.red()) + 255 - alpha).min(255) as u8;
        let green = (u32::from(pixel.green()) + 255 - alpha).min(255) as u8;
        let blue = (u32::from(pixel.blue()) + 255 - alpha).min(255) as u8;
        bytes.extend_from_slice(&[red, green, blue]);
    }
    let image = image::RgbImage::from_raw(pixmap.width(), pixmap.height(), bytes)
        .ok_or_else(|| "Failed to build JPEG image".to_string())?;
    let mut jpeg = Vec::new();
    {
        let mut cursor = std::io::Cursor::new(&mut jpeg);
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut cursor, quality);
        encoder
            .encode_image(&image)
            .map_err(|err| format!("Failed to encode JPEG: {err}"))?;
    }
    Ok(jpeg)
}

/// BMP encode (owner request 2026-11-26 — F17's decision, the `bmp` feature
/// flag on the pinned crate): RGBA, the same demultiplied source the PNG
/// encoders build from. The pinned encoder writes a 32-bit BITMAPV4HEADER
/// file for `Rgba8` — the ONE BMP flavor that carries alpha; 24-bit files
/// would need the JPEG's white-flatten instead. There is no knob: the
/// format stores its rows UNCOMPRESSED, so a quality/effort slider has
/// nothing to trade (the F17 note — no options row), and the encode is
/// lossless. The alpha channel is preserved in the bytes; viewers that
/// predate the V4 header may render fully-opaque (the format's own quirk,
/// recorded in F17).
pub(crate) fn encode_bmp(pixmap: &Pixmap) -> Result<Vec<u8>, String> {
    let image = demultiplied_rgba(pixmap)?;
    let mut bmp = Vec::new();
    {
        use image::ImageEncoder;
        let mut cursor = std::io::Cursor::new(&mut bmp);
        let encoder = image::codecs::bmp::BmpEncoder::new(&mut cursor);
        encoder
            .write_image(
                image.as_raw(),
                image.width(),
                image.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|err| format!("Failed to encode BMP: {err}"))?;
    }
    Ok(bmp)
}
