//! Unit tests of the "Save Image" dialog's pure helpers (SPEC §3.7; owner
//! request 2026-11-19): the composed file name (the format choice owns the
//! extension) and the prefill base name. The dialog's modal plumbing is
//! the shared `dialogs.rs` helpers — covered in `dialogs/tests.rs`.

use super::{
    FALLBACK_NAME, JPEG_QUALITY_TAIL, PNG_COMPRESSION_TAIL, base_name_of, compose_file_name,
    jpeg_quality_hint, png_compression_hint,
};
use crate::file_io::SaveFormat;

#[test]
fn the_composed_name_carries_the_chosen_formats_extension() {
    assert_eq!(compose_file_name("photo", SaveFormat::Png), "photo.png");
    assert_eq!(compose_file_name("photo", SaveFormat::Jpeg), "photo.jpg");
    assert_eq!(compose_file_name("photo", SaveFormat::Bmp), "photo.bmp");
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
    assert_eq!(compose_file_name("photo.bmp", SaveFormat::Bmp), "photo.bmp");
}

#[test]
fn a_conflicting_typed_extension_is_replaced() {
    // The dropdown owns the format — a stale extension must not win.
    assert_eq!(
        compose_file_name("photo.png", SaveFormat::Jpeg),
        "photo.jpg"
    );
    assert_eq!(compose_file_name("photo.jpg", SaveFormat::Png), "photo.png");
    assert_eq!(compose_file_name("photo.png", SaveFormat::Bmp), "photo.bmp");
    assert_eq!(compose_file_name("photo.bmp", SaveFormat::Png), "photo.png");
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
    assert_eq!(base_name_of("photo.bmp"), "photo");
    assert_eq!(base_name_of("photo.old.png"), "photo.old");
    assert_eq!(base_name_of("photo.txt"), "photo.txt");
    assert_eq!(base_name_of("Untitled"), "Untitled");
    // A multi-byte name: `base_name_of` slices by byte length, and the
    // ASCII-only lowercasing keeps every UTF-8 boundary intact.
    assert_eq!(base_name_of("grüße.png"), "grüße");
}

// ---- the dialog's rendered rows (owner request 2026-11-20: the PNG
// Compression slider + the per-format Lossless row; the same day's layout
// pass — the dropdown, the preview/name/folder cluster, the encoding group) ----

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
    // The live meaning caption under the slider (owner request
    // 2026-11-20): the current level's effect + the never-changes tail.
    assert!(
        rendered.contains("6 — balanced packing"),
        "rows: {rendered}"
    );
    assert!(rendered.contains(PNG_COMPRESSION_TAIL), "rows: {rendered}");
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
    // The live meaning caption under the slider (owner request
    // 2026-11-20): the current quality's effect + the scale direction.
    assert!(rendered.contains("90 — high quality"), "rows: {rendered}");
    assert!(rendered.contains(JPEG_QUALITY_TAIL), "rows: {rendered}");
}

#[test]
fn the_png_caption_names_every_compression_level() {
    // Each stop lands in the bucket whose sentence names its packing
    // trade, and the caption always names the CURRENT number (the owner's
    // whole point: the bare 0..=9 told nothing).
    assert!(png_compression_hint(0).starts_with("0 — stored raw"));
    assert!(png_compression_hint(2).starts_with("2 — light packing"));
    assert!(png_compression_hint(6).starts_with("6 — balanced packing"));
    assert!(png_compression_hint(9).starts_with("9 — tight packing"));
    for level in 0..=9 {
        assert!(
            png_compression_hint(level).starts_with(&format!("{level} — ")),
            "level {level} is not named in its caption"
        );
    }
}

#[test]
fn the_jpeg_caption_names_every_quality_value() {
    // The four buckets cover the whole 1..=100 scale, and the caption
    // always names the CURRENT number.
    assert!(jpeg_quality_hint(100).starts_with("100 — near-perfect picture"));
    assert!(jpeg_quality_hint(90).starts_with("90 — high quality"));
    assert!(jpeg_quality_hint(60).starts_with("60 — medium quality"));
    assert!(jpeg_quality_hint(30).starts_with("30 — low quality"));
    for quality in 1..=100 {
        assert!(
            jpeg_quality_hint(quality).starts_with(&format!("{quality} — ")),
            "quality {quality} is not named in its caption"
        );
    }
}

// ---- the layout pass (owner request 2026-11-20: the preview above the
// name field, the folder row under it, the format dropdown, the encoding
// group — Lossless ABOVE the knob — separated by distance) ----

#[test]
fn the_layout_reads_what_where_format_then_the_encoding_group() {
    let rendered = rendered_save_texts(SaveFormat::Png);
    // First paint of each row's text, in dialog order: the composed-name
    // preview ABOVE the file name (owner 2026-11-20: the result reads
    // first), the folder row directly under the name field, the format
    // dropdown after them.
    let saving_as = rendered.find("Saving as:").expect("preview row rendered");
    let file_name = rendered.find("File name").expect("name field rendered");
    let folder = rendered
        .find("Folder")
        .expect("folder row rendered (native test build)");
    let format = rendered.find("Format").expect("format row rendered");
    assert!(saving_as < file_name, "rows: {rendered}");
    assert!(file_name < folder, "rows: {rendered}");
    assert!(folder < format, "rows: {rendered}");
    // The encoding group: the Lossless row ABOVE the Compression slider.
    let lossless = rendered.find("Lossless").expect("lossless row rendered");
    let compression = rendered.find("Compression").expect("slider rendered");
    assert!(lossless < compression, "rows: {rendered}");
    // The dropdown paints the SELECTED format's label (a radio's active
    // label painted the same text — the choice row is still recognizable).
    assert!(rendered.contains("PNG image"), "rows: {rendered}");
}

#[test]
fn the_jpeg_group_keeps_lossless_above_quality() {
    let rendered = rendered_save_texts(SaveFormat::Jpeg);
    let lossless = rendered.find("Lossless").expect("lossless row rendered");
    let quality = rendered.find("Quality").expect("quality slider rendered");
    assert!(lossless < quality, "rows: {rendered}");
    // The dropdown's selected text is the JPEG label.
    assert!(rendered.contains("JPEG image"), "rows: {rendered}");
}
