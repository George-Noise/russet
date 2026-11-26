//! The PNG compression KNOB tests (owner request 2026-11-20: the "Save
//! Image" dialog's Compression slider): 0..=9 are the encoder's real,
//! distinct deflate stops, every level is lossless — the slider trades
//! file size vs encode time, never pixels — and the stops are ordered on
//! a compressible canvas. Split out of `tests.rs` (the ~300-line guard)
//! when BMP joined the codecs (owner request 2026-11-26).

use tiny_skia::{ColorU8, Pixmap};

use super::super::codecs::{
    DEFAULT_PNG_COMPRESSION, encode_png_with_compression, png_compression_type,
};

/// A compressible 64×64 canvas: flat quadrants over a smooth horizontal
/// gradient (the shape deflate's effort levels actually act on).
fn structured_test_pixmap() -> Pixmap {
    let mut pixmap = Pixmap::new(64, 64).expect("64x64 pixmap");
    for (i, pixel) in pixmap.pixels_mut().iter_mut().enumerate() {
        let x = (i % 64) as u8;
        let y = (i / 64) as u8;
        let (r, g, b) = if y < 16 {
            (255, 0, 0)
        } else if y < 32 {
            (0, 255, 0)
        } else if y < 48 {
            (x, x, x)
        } else {
            (255 - x, 255 - x, 255 - x)
        };
        *pixel = ColorU8::from_rgba(r, g, b, 255).premultiply();
    }
    pixmap
}

#[test]
fn the_png_compression_slider_maps_onto_the_deflate_levels() {
    // 0..=9 are the encoder's real, distinct stops: Level(0) stores the
    // rows uncompressed, Level(1..=9) set the EXACT deflate level; any
    // out-of-range value clamps to 9 — no panic can reach the save path.
    use image::codecs::png::CompressionType;
    assert_eq!(png_compression_type(0), CompressionType::Level(0));
    assert_eq!(png_compression_type(1), CompressionType::Level(1));
    assert_eq!(png_compression_type(6), CompressionType::Level(6));
    assert_eq!(png_compression_type(9), CompressionType::Level(9));
    assert_eq!(png_compression_type(10), CompressionType::Level(9));
    assert_eq!(png_compression_type(255), CompressionType::Level(9));
    // The dialog's default rides the balanced middle (flate2's own default
    // level).
    assert_eq!(
        png_compression_type(DEFAULT_PNG_COMPRESSION),
        CompressionType::Level(6)
    );
}

#[test]
fn every_png_compression_level_saves_the_same_pixels() {
    // THE promise the dialog's Lossless row displays (owner request
    // 2026-11-20): the slider trades file size, never fidelity — decoding
    // the saved bytes at every effort returns the exact RGBA pixels.
    let pixmap = structured_test_pixmap();
    let source = image::RgbaImage::from_raw(
        64,
        64,
        pixmap
            .pixels()
            .iter()
            .flat_map(|p| {
                let c = p.demultiply();
                [c.red(), c.green(), c.blue(), c.alpha()]
            })
            .collect::<Vec<u8>>(),
    )
    .expect("64x64 RGBA image");
    for compression in [0, 6, 9] {
        let png = encode_png_with_compression(&pixmap, compression).expect("PNG encodes");
        let decoded = image::load_from_memory(&png)
            .expect("PNG decodes")
            .to_rgba8();
        assert_eq!(decoded.as_raw(), source.as_raw());
    }
}

#[test]
fn higher_png_compression_encodes_smaller_files() {
    // The slider must not be a placebo: level 9 packs the structured
    // canvas tighter than level 0's uncompressed store.
    let pixmap = structured_test_pixmap();
    let stored = encode_png_with_compression(&pixmap, 0).expect("store encodes");
    let best = encode_png_with_compression(&pixmap, 9).expect("best encodes");
    assert!(
        best.len() < stored.len(),
        "compression 9 ({} bytes) should encode smaller than 0 ({} bytes)",
        best.len(),
        stored.len()
    );
}

#[test]
fn intermediate_png_compression_levels_encode_intermediate_files() {
    // The slider's stops are ordered: level 3 must land BETWEEN the store
    // and level 9 on this compressible canvas.
    let pixmap = structured_test_pixmap();
    let stored = encode_png_with_compression(&pixmap, 0).expect("store encodes");
    let mid = encode_png_with_compression(&pixmap, 3).expect("level 3 encodes");
    let best = encode_png_with_compression(&pixmap, 9).expect("level 9 encodes");
    assert!(
        mid.len() < stored.len() && best.len() < mid.len(),
        "expected store ({} bytes) > level 3 ({} bytes) > level 9 ({} bytes)",
        stored.len(),
        mid.len(),
        best.len()
    );
}
