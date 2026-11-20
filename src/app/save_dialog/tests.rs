//! Unit tests of the "Save Image" dialog's pure helpers (SPEC §3.7; owner
//! request 2026-11-19): the composed file name (the format radio owns the
//! extension) and the prefill base name. The dialog's modal plumbing is
//! the shared `dialogs.rs` helpers — covered in `dialogs/tests.rs`.

use super::{FALLBACK_NAME, base_name_of, compose_file_name};
use crate::file_io::SaveFormat;

#[test]
fn the_composed_name_carries_the_chosen_formats_extension() {
    assert_eq!(compose_file_name("photo", SaveFormat::Png), "photo.png");
    assert_eq!(compose_file_name("photo", SaveFormat::Jpeg), "photo.jpg");
}

#[test]
fn a_typed_extension_matching_the_format_survives() {
    assert_eq!(compose_file_name("photo.png", SaveFormat::Png), "photo.png");
    // A matching extension is canonicalized to the format's spelling.
    assert_eq!(
        compose_file_name("photo.jpeg", SaveFormat::Jpeg),
        "photo.jpg"
    );
    assert_eq!(
        compose_file_name("photo.jpg", SaveFormat::Jpeg),
        "photo.jpg"
    );
}

#[test]
fn a_conflicting_typed_extension_is_replaced() {
    // The radio owns the format — a stale extension must not win.
    assert_eq!(
        compose_file_name("photo.png", SaveFormat::Jpeg),
        "photo.jpg"
    );
    assert_eq!(compose_file_name("photo.jpg", SaveFormat::Png), "photo.png");
    // Only the IMAGE extension is the format's; the rest of the base stays.
    assert_eq!(
        compose_file_name("photo.old.png", SaveFormat::Png),
        "photo.old.png"
    );
    assert_eq!(
        compose_file_name("photo.old.png", SaveFormat::Jpeg),
        "photo.old.jpg"
    );
}

#[test]
fn separators_and_forbidden_characters_never_reach_the_file_system() {
    // A separator would make `Path::join` write outside the chosen folder;
    // the rest is the Windows-forbidden set.
    assert_eq!(compose_file_name("a/b\\c", SaveFormat::Png), "abc.png");
    assert_eq!(compose_file_name("a<b>*c?:d", SaveFormat::Png), "abcd.png");
    assert_eq!(
        compose_file_name("im\"g|1.png", SaveFormat::Png),
        "img1.png"
    );
    assert_eq!(
        compose_file_name("  spaced  ", SaveFormat::Png),
        "spaced.png"
    );
}

#[test]
fn an_empty_or_extension_only_name_falls_back_to_untitled() {
    assert_eq!(compose_file_name("", SaveFormat::Png), "Untitled.png");
    assert_eq!(compose_file_name("   ", SaveFormat::Jpeg), "Untitled.jpg");
    assert_eq!(compose_file_name(".png", SaveFormat::Jpeg), "Untitled.jpg");
    assert_eq!(base_name_of(".png"), FALLBACK_NAME);
    assert_eq!(base_name_of("   "), FALLBACK_NAME);
}

#[test]
fn the_prefill_strips_a_known_extension_only() {
    assert_eq!(base_name_of("photo.png"), "photo");
    assert_eq!(base_name_of("PHOTO.JPG"), "PHOTO");
    assert_eq!(base_name_of("photo.jpeg"), "photo");
    assert_eq!(base_name_of("photo.old.png"), "photo.old");
    assert_eq!(base_name_of("photo.txt"), "photo.txt");
    assert_eq!(base_name_of("Untitled"), "Untitled");
    // A multi-byte name: `base_name_of` slices by byte length, and the
    // ASCII-only lowercasing keeps every UTF-8 boundary intact.
    assert_eq!(base_name_of("grüße.png"), "grüße");
}

// ---- the dialog's rendered rows (owner request 2026-11-20: the PNG
// Compression slider + the per-format Lossless row) ----

use crate::app::App;
use eframe::egui::{self, Pos2, Rect};

/// Renders the REAL save dialog through the REAL `App::save_dialog` method
/// for the given format and joins every painted text run — the same trick
/// `dialogs/tests.rs` uses on the shared plumbing. THREE passes: egui sizes
/// a fresh window on the frame after its first layout, so the first pass
/// paints nothing (every shape culled to `Noop`).
fn rendered_save_texts(format: SaveFormat) -> String {
    let mut app = App::new_headless();
    app.show_save_dialog = true;
    app.save_format = format;
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(
            Pos2::ZERO,
            egui::Vec2::new(1280.0, 800.0),
        )),
        ..egui::RawInput::default()
    };
    let mut rendered = String::new();
    for _ in 0..3 {
        let mut output = ctx.run_pass(input.clone(), |ctx| {
            app.save_dialog(ctx);
        });
        // No renderer here: the font-atlas deltas each pass hands back must
        // be explicitly discarded (see dialogs/tests.rs show_once).
        output.textures_delta.clear();
        // egui nests paint commands (`Shape::Vec`) — collect recursively.
        fn texts<'a>(shape: &'a egui::Shape, out: &mut Vec<&'a str>) {
            match shape {
                egui::Shape::Text(text) => out.push(text.galley.job.text.as_str()),
                egui::Shape::Vec(children) => children.iter().for_each(|c| texts(c, out)),
                _ => {}
            }
        }
        let mut runs = Vec::new();
        for clipped in &output.shapes {
            texts(&clipped.shape, &mut runs);
        }
        rendered = runs.join("\n");
    }
    rendered
}

#[test]
fn the_png_rows_show_the_compression_slider_and_the_locked_lossless_promise() {
    let rendered = rendered_save_texts(SaveFormat::Png);
    // The per-format knob and the guarantee row, both PNG-flavored...
    assert!(rendered.contains("Compression"), "rows: {rendered}");
    assert!(rendered.contains("Lossless"), "rows: {rendered}");
    assert!(
        rendered.contains("(always — PNG is lossless)"),
        "rows: {rendered}"
    );
    // ...and nothing of the JPEG-only rows.
    assert!(!rendered.contains("Quality"), "rows: {rendered}");
    assert!(
        !rendered.contains("(never — JPEG is always lossy)"),
        "rows: {rendered}"
    );
}

#[test]
fn the_jpeg_rows_show_quality_and_no_lossless_promise() {
    let rendered = rendered_save_texts(SaveFormat::Jpeg);
    // JPEG keeps its Quality knob; the Lossless row flips to the locked-OFF
    // display (a lossless JPEG is neither writable by the pinned encoder
    // nor decodable by browsers).
    assert!(rendered.contains("Quality"), "rows: {rendered}");
    assert!(rendered.contains("Lossless"), "rows: {rendered}");
    assert!(
        rendered.contains("(never — JPEG is always lossy)"),
        "rows: {rendered}"
    );
    // ...and nothing of the PNG-only rows.
    assert!(!rendered.contains("Compression"), "rows: {rendered}");
    assert!(
        !rendered.contains("(always — PNG is lossless)"),
        "rows: {rendered}"
    );
}
