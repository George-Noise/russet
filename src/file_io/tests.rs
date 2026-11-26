//! Unit tests for the platform-independent image plumbing (SPEC §3.7,
//! §5.1): premultiplied alpha on the way in, demultiplied bytes on the
//! way out, the white background the JPEG expects — and the BMP's
//! 32-bit RGBA round trip (owner request 2026-11-26, F17's decision) —
//! plus the "Save Image" dialog's `SaveFormat` mapping and quality clamp
//! (owner request 2026-11-19). The PNG compression knob tests live in
//! `tests/png_knob.rs` (the ~300-line split, 2026-11-26).

use tiny_skia::{ColorU8, Pixmap};

use super::SaveFormat;
use super::codecs::{
    DEFAULT_JPEG_QUALITY, encode_bmp, encode_jpeg_with_quality, encode_png, pixmap_from_image,
};

mod png_knob;

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
    assert_eq!(SaveFormat::from_extension("bmp"), Some(SaveFormat::Bmp));
    assert_eq!(SaveFormat::from_extension("BMP"), Some(SaveFormat::Bmp));
    assert_eq!(SaveFormat::from_extension("gif"), None);
    assert_eq!(SaveFormat::from_extension(""), None);
    assert_eq!(SaveFormat::Png.extension(), "png");
    assert_eq!(SaveFormat::Jpeg.extension(), "jpg");
    assert_eq!(SaveFormat::Bmp.extension(), "bmp");
    assert_eq!(SaveFormat::Png.label(), "PNG image");
    assert_eq!(SaveFormat::Jpeg.label(), "JPEG image");
    assert_eq!(SaveFormat::Bmp.label(), "BMP image");
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
    assert_eq!(SaveFormat::Bmp.mime_type(), "image/bmp");
}

// ---- the BMP codec (owner request 2026-11-26, F17's decision) ----

#[test]
fn bmp_round_trip_preserves_rgba_bytes_including_alpha() {
    // BMP saves ride the 32-bit BITMAPV4HEADER flavor — the ONE BMP form
    // that carries alpha (F17's note), so the round trip returns the exact
    // RGBA bytes, a fully transparent pixel included.
    let mut pixmap = Pixmap::new(2, 1).expect("2x1 pixmap");
    pixmap.pixels_mut()[0] = ColorU8::from_rgba(10, 20, 30, 255).premultiply();
    pixmap.pixels_mut()[1] = ColorU8::from_rgba(0, 0, 0, 0).premultiply();

    let bmp = encode_bmp(&pixmap).expect("BMP encodes");
    let decoded = image::load_from_memory(&bmp)
        .expect("BMP decodes")
        .to_rgba8();
    assert_eq!(decoded.dimensions(), (2, 1));
    assert_eq!(decoded.get_pixel(0, 0).0, [10, 20, 30, 255]);
    assert_eq!(decoded.get_pixel(1, 0).0[3], 0);
}

#[test]
fn bmp_files_open_through_the_shared_decode_path() {
    // The Open path (native `image::open`, web `load_from_memory`) needs
    // only the crate's `bmp` feature — the bytes above decode through
    // the same conversion every opened file takes.
    let mut pixmap = Pixmap::new(3, 3).expect("3x3 pixmap");
    pixmap
        .pixels_mut()
        .iter_mut()
        .enumerate()
        .for_each(|(i, pixel)| {
            *pixel = ColorU8::from_rgba(
                (i * 7 % 256) as u8,
                (i * 13 % 256) as u8,
                (i * 29 % 256) as u8,
                200,
            )
            .premultiply();
        });

    let bmp = encode_bmp(&pixmap).expect("BMP encodes");
    let reloaded = pixmap_from_image(image::load_from_memory(&bmp).expect("BMP decodes"))
        .expect("BMP converts to a pixmap");
    assert_eq!((reloaded.width(), reloaded.height()), (3, 3));
    assert_eq!(reloaded.pixel(1, 2).expect("pixel in bounds").alpha(), 200);
}

// ---- the FileTask handles (F21) ----

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
