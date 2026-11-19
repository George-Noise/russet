//! Tests of the Palette window's color-well popup logic (owner request
//! 2026-11-19): the hex field's parse-and-write-back, the canonical display
//! form and the shared syntax with the `--primary`/`--secondary` launch
//! flags. The popup wiring itself follows egui's own color button; the
//! testable behavior is the pure helpers `windows.rs` feeds it with.

use super::super::{format_color_hex, hex_edited_color};
use crate::app::launch::{parse_color, parse_hex_components};
use eframe::egui::Color32;

/// A translucent current color, so the tests can SEE which alpha survives.
fn current() -> Color32 {
    Color32::from_rgba_unmultiplied(0x12, 0x34, 0x56, 0xC8)
}

#[test]
fn a_short_hex_keeps_the_wells_current_alpha() {
    // 6 digits: only the RGB is replaced, the well's alpha survives.
    assert_eq!(
        hex_edited_color(current(), "#FF7F00"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, 0xC8)),
        "a 6-digit hex must keep the current alpha"
    );
    // The leading # and the surrounding spaces are optional — the exact
    // launch-flag syntax, one parser for both consumers.
    assert_eq!(
        hex_edited_color(current(), "ff7f00"),
        hex_edited_color(current(), "#FF7F00")
    );
    assert_eq!(
        hex_edited_color(current(), "  #ff7f00  "),
        hex_edited_color(current(), "#FF7F00")
    );
    // The 3-digit doubled form: f70 = ff 77 00, alpha still kept.
    assert_eq!(
        hex_edited_color(current(), "#f70"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x77, 0x00, 0xC8))
    );
}

#[test]
fn an_eight_digit_hex_sets_the_alpha() {
    assert_eq!(
        hex_edited_color(current(), "#FF7F0032"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, 0x32)),
        "the written alpha byte wins over the well's current one"
    );
    assert_eq!(
        hex_edited_color(current(), "ff7f0000"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, 0x00)),
        "fully transparent is a real choice, not a fallback"
    );
}

#[test]
fn invalid_hex_text_applies_nothing() {
    // Too short (mid-typing), too long, non-hex, empty — all None, so the
    // field simply does not touch the color while the text is partial.
    assert_eq!(hex_edited_color(current(), "#FF7F"), None);
    assert_eq!(hex_edited_color(current(), "FF7F000"), None);
    assert_eq!(hex_edited_color(current(), "zzzzzz"), None);
    assert_eq!(hex_edited_color(current(), "#GGGGGG"), None);
    assert_eq!(hex_edited_color(current(), ""), None);
    assert_eq!(hex_edited_color(current(), "   "), None);
}

#[test]
fn the_hex_field_shows_the_full_rrggbbaa_form() {
    // Opaque colors roundtrip egui's PREMULTIPLIED Color32 storage exactly,
    // so their bytes can be pinned.
    assert_eq!(
        format_color_hex(Color32::from_rgb(0, 255, 0)),
        "#00FF00FF",
        "the alpha is always spelled out"
    );
    // Translucent colors may be nudged ±1 per channel by the
    // straight→premultiplied→straight roundtrip (an egui storage property,
    // shared by the launch flags and the picker) — the format test pins the
    // SHAPE there and the exact alpha tail (alpha never premultiplies).
    let formatted = format_color_hex(current());
    assert_eq!(formatted.len(), 9, "# plus 8 hex digits: {formatted}");
    assert!(formatted.starts_with('#'));
    assert!(formatted[1..].bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(&formatted[7..], "C8", "the alpha tail is the well's one");
}

#[test]
fn the_launch_flags_and_the_hex_field_parse_one_syntax() {
    // One primitive (`launch.rs::parse_hex_components`) feeds BOTH the
    // `--primary`/`--secondary` flags and the hex field — the syntax cannot
    // drift apart. The only difference is the alpha fallback: the flags
    // default a missing alpha to opaque, the field keeps the well's.
    let ([r, g, b], alpha) = parse_hex_components("#ff7f00").expect("parses");
    assert_eq!((r, g, b), (0xFF, 0x7F, 0x00));
    assert_eq!(alpha, None, "a short form carries no alpha of its own");
    assert_eq!(
        parse_color("#ff7f00"),
        Some(Color32::from_rgb(0xFF, 0x7F, 0x00)),
        "the launch form defaults the missing alpha to opaque"
    );
    let edited = hex_edited_color(current(), "#ff7f00").expect("parses");
    let [r, g, b, a] = edited.to_srgba_unmultiplied();
    assert_eq!(
        (r, b, a),
        (0xFF, 0x00, 0xC8),
        "the text's RGB + the well's current alpha"
    );
    assert!(
        g.abs_diff(0x7F) <= 1,
        "egui's premultiplied Color32 storage may nudge a channel ±1, got {g}"
    );
}
