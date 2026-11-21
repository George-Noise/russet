//! Unit tests for the platform-independent image plumbing (SPEC §3.7, §5.1):
//! premultiplied alpha on the way in, demultiplied bytes on the way out, and
//! the white background JPEG expects — plus the "Save Image" dialog's
//! `SaveFormat` mapping and quality clamp (owner request 2026-11-19) and the
//! PNG compression knob (owner request 2026-11-20: the slider trades file
//! size vs encode time, NEVER pixels — every level is lossless).

use tiny_skia::{ColorU8, Pixmap};

use super::SaveFormat;
use super::codecs::{
    DEFAULT_JPEG_QUALITY, DEFAULT_PNG_COMPRESSION, encode_jpeg_with_quality, encode_png,
    encode_png_with_compression, pixmap_from_image, png_compression_type,
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

// ---- the PNG compression knob (owner request 2026-11-20: the slider) ----

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

/// F21 (owner pick of variant A, 2026-11-21): the file tasks are POLL
/// handles — the blocking picker runs on a worker thread and the UI loop
/// consumes the result per frame. These tests drive the handle itself
/// (channel-fed tasks — no dialog opens, no thread spawns).
#[cfg(not(target_arch = "wasm32"))]
mod file_task {
    use std::path::PathBuf;
    use std::sync::mpsc;

    use super::super::{FileOutcome, FileTask};

    #[test]
    fn a_file_task_yields_its_result_exactly_once() {
        let (tx, rx) = mpsc::channel();
        tx.send(Ok(FileOutcome::Folder(Some(PathBuf::from("/picked")))))
            .expect("the channel is alive");

        let mut task = FileTask { rx };
        let outcome = task.poll().expect("the sent result arrives");
        assert!(
            matches!(outcome, Ok(FileOutcome::Folder(Some(ref picked))) if picked == &PathBuf::from("/picked")),
            "expected the picked folder, got a different outcome"
        );

        assert!(task.poll().is_none(), "the result is consumed once");
    }

    #[test]
    fn a_running_file_task_polls_as_pending() {
        let (_tx, rx) = mpsc::channel::<Result<FileOutcome, String>>();
        // `_tx` stays alive: no verdict has been sent yet.

        let mut task = FileTask { rx };
        assert!(task.poll().is_none(), "no result yet — the task is pending");
        assert!(task.poll().is_none(), "pending again on the next poll");
    }

    #[test]
    fn a_worker_dying_without_a_verdict_resolves_the_task_with_an_error() {
        // The no-wedge rule: the single-dialog slot must never stay shut
        // because a worker vanished — the poll resolves with an error
        // instead and the slot frees (the slot consumes the task on its
        // first verdict — poll_file_task's take-then-drop).
        let (tx, rx) = mpsc::channel::<Result<FileOutcome, String>>();
        drop(tx);

        let mut task = FileTask { rx };
        let outcome = task.poll().expect("a dropped sender resolves the task");
        assert!(outcome.is_err(), "expected an error verdict");
    }

    #[test]
    fn a_cancelled_open_resolves_to_no_image() {
        let (tx, rx) = mpsc::channel();
        tx.send(Ok(FileOutcome::Opened(None)))
            .expect("the channel is alive");

        let mut task = FileTask { rx };
        assert!(
            matches!(task.poll(), Some(Ok(FileOutcome::Opened(None)))),
            "a cancelled pick must resolve to no image"
        );
    }
}
