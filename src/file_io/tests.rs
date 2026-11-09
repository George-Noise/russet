//! Unit tests for the platform-independent image plumbing (SPEC §3.7, §5.1):
//! premultiplied alpha on the way in, demultiplied bytes on the way out, and
//! the white background JPEG expects.

use tiny_skia::{ColorU8, Pixmap};

use super::{encode_jpeg, encode_png, pixmap_from_image};

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

    let jpeg = encode_jpeg(&pixmap).expect("JPEG encodes");
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
