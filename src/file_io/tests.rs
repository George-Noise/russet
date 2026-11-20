//! Unit tests for the platform-independent image plumbing (SPEC §3.7, §5.1):
//! premultiplied alpha on the way in, demultiplied bytes on the way out, and
//! the white background JPEG expects — plus the "Save Image" dialog's
//! `SaveFormat` mapping and quality clamp (owner request 2026-11-19).

use tiny_skia::{ColorU8, Pixmap};

use super::{
    DEFAULT_JPEG_QUALITY, SaveFormat, encode_jpeg_with_quality, encode_png, pixmap_from_image,
};

#[test]
fn png_round_trip_preserves_rgba_bytes() {
    let mut pixmap = Pixmap::new(2, 1).expect("2x1 pixmap");
    pixmap.pixels_mut()[0] = ColorU8::from_rgba(10, 20, 30, 255).premultiply();
    pixmap.pixels_mut()[1] = ColorU8::from_rgba(0, 0, 0, 0).premultiply();

    let png = encode_png(&pixmap).expect("PNG encodes");
    let decoded = image::load_from_memory(&png)
        .expect("PNG decodes")
        .to_rgba8();
    assert_eq!(decoded.dimensions(), (2, 1));
    assert_eq!(decoded.get_pixel(0, 0).0, [10, 20, 30, 255]);
    assert_eq!(decoded.get_pixel(1, 0).0[3], 0);
}

#[test]
fn loading_an_image_premultiplies_semi_transparent_pixels() {
    let source = image::RgbaImage::from_raw(1, 1, vec![255, 0, 0, 128]).expect("1x1 RGBA image");
    let pixmap = pixmap_from_image(image::DynamicImage::ImageRgba8(source))
        .expect("image converts to a pixmap");

    let pixel = pixmap.pixel(0, 0).expect("pixel in bounds");
    assert_eq!(pixel.alpha(), 128);
    // Premultiplied red: 255 * 128 / 255 = 128 (tiny-skia rounds).
    assert!(
        (127..=129).contains(&pixel.red()),
        "expected premultiplied red ~= 128, got {}",
        pixel.red()
    );
}

#[test]
fn jpeg_flattens_transparency_onto_white() {
    let pixmap = Pixmap::new(4, 4).expect("4x4 pixmap"); // zeroed = transparent

    let jpeg = encode_jpeg_with_quality(&pixmap, DEFAULT_JPEG_QUALITY).expect("JPEG encodes");
    let decoded = image::load_from_memory(&jpeg)
        .expect("JPEG decodes")
        .to_rgb8();
    for pixel in decoded.pixels() {
        assert!(
            pixel.0.iter().all(|&channel| channel >= 250),
            "transparent JPEG pixel must be white, got {pixel:?}"
        );
    }
}

// ---- the Save Image dialog's format mapping + quality (owner request 2026-11-19) ----

#[test]
fn the_save_format_maps_extensions_both_ways() {
    assert_eq!(SaveFormat::from_extension("png"), Some(SaveFormat::Png));
    assert_eq!(SaveFormat::from_extension("jpg"), Some(SaveFormat::Jpeg));
    assert_eq!(SaveFormat::from_extension("JPEG"), Some(SaveFormat::Jpeg));
    assert_eq!(SaveFormat::from_extension("gif"), None);
    assert_eq!(SaveFormat::from_extension(""), None);
    assert_eq!(SaveFormat::Png.extension(), "png");
    assert_eq!(SaveFormat::Jpeg.extension(), "jpg");
}

#[test]
fn the_jpeg_quality_clamps_instead_of_panicking() {
    // The dialog slider is not the only writer; the encoder rejects
    // out-of-range qualities, so the clamp must absorb every input.
    let pixmap = Pixmap::new(2, 2).expect("2x2 pixmap");
    for quality in [0, 1, 100, 255] {
        let jpeg = encode_jpeg_with_quality(&pixmap, quality).expect("JPEG encodes");
        image::load_from_memory(&jpeg).expect("JPEG decodes");
    }
}

#[test]
fn different_jpeg_qualities_encode_different_bytes() {
    // Quality 1 vs 100 must not silently collapse to the same encoder
    // settings — the dialog's Quality slider would be a placebo.
    let mut pixmap = Pixmap::new(64, 64).expect("64x64 pixmap");
    for (i, pixel) in pixmap.pixels_mut().iter_mut().enumerate() {
        let value = (i % 256) as u8;
        *pixel = ColorU8::from_rgba(value, value.wrapping_mul(7), value.wrapping_mul(13), 255)
            .premultiply();
    }
    let low = encode_jpeg_with_quality(&pixmap, 1).expect("low quality encodes");
    let high = encode_jpeg_with_quality(&pixmap, 100).expect("high quality encodes");
    assert!(
        low.len() < high.len(),
        "quality 1 ({} bytes) should encode smaller than quality 100 ({} bytes)",
        low.len(),
        high.len()
    );
}

#[test]
fn the_web_download_blob_carries_the_format_mime_type() {
    // The web save hands the bytes to the browser as a Blob whose `type`
    // comes straight from the dialog's format choice (owner request
    // 2026-11-20: the download starts at once, no fallback window) — the
    // mapping must stay in lockstep with SaveFormat.
    assert_eq!(SaveFormat::Png.mime_type(), "image/png");
    assert_eq!(SaveFormat::Jpeg.mime_type(), "image/jpeg");
}
