//! Tests of the Palette window's color-well popup logic (owner request
//! 2026-11-19): the hex field's parse-and-write-back, the canonical display
//! form and the shared syntax with the `--primary`/`--secondary` launch
//! flags. The popup wiring itself follows egui's own color button; the
//! testable behavior is the pure helpers this module feeds it with. The
//! file lives HERE — not in the parent's `tests/` — since the owner-approved
//! 2026-11-19 split moved the palette code into this child module, and the
//! test follows its behavior's module (AGENTS.md "Tests" rule 2).

use super::{format_color_hex, hex_edited_color};
use crate::app::launch::{parse_color, parse_hex_components};
use eframe::egui::Color32;

/// The fallback alpha of the field's short forms: the SEED snapshot — the
/// alpha the well had when the edit session started (see
/// [`color_swell_button`]'s re-seed branch).
const SEED_ALPHA: u8 = 0xC8;

#[test]
fn a_short_hex_keeps_the_seeded_alpha() {
    // 6 digits: only the RGB is replaced, the session's alpha survives.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "#FF7F00"),
        Some(Color32::from_rgba_unmultiplied(
            0xFF, 0x7F, 0x00, SEED_ALPHA
        )),
        "a 6-digit hex must keep the seeded alpha"
    );
    // Junk-tolerant (owner request 2026-11-19): the leading #, spaces, even
    // a doubled hash or an 0x prefix are all stripped — the exact launch
    // flag syntax, one parser for both consumers.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "ff7f00"),
        hex_edited_color(SEED_ALPHA, "#FF7F00")
    );
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "  ##ff7f00  "),
        hex_edited_color(SEED_ALPHA, "#FF7F00")
    );
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "0xff7f00"),
        hex_edited_color(SEED_ALPHA, "#FF7F00")
    );
    // The 3-digit doubled form: f70 = ff 77 00, alpha still kept.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "#f70"),
        Some(Color32::from_rgba_unmultiplied(
            0xFF, 0x77, 0x00, SEED_ALPHA
        ))
    );
}

#[test]
fn an_alpha_carrying_form_sets_the_alpha() {
    // 8 digits: RRGGBBAA.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "#FF7F0032"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, 0x32)),
        "the written alpha byte wins over the seeded one"
    );
    // 4 digits: the RGBA shorthand, every digit doubled.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "f70a"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x77, 0x00, 0xAA))
    );
    // 5 digits: the shorthand RGB + a FULL alpha byte.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "f70ab"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x77, 0x00, 0xAB))
    );
    // 7 digits: the full RGB + a DOUBLED alpha digit.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "ff7f00c"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, 0xCC))
    );
    // Fully transparent is a real choice, not a fallback.
    assert_eq!(
        hex_edited_color(SEED_ALPHA, "ff7f0000"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, 0x00))
    );
}

#[test]
fn a_typed_prefix_does_not_leak_its_alpha_into_the_final_color() {
    // THE live-typing regression (owner round 2026-11-19): typing `FF7F00`
    // passes THROUGH the 5-digit `RGBAA` prefix `FF7F0`, whose own alpha
    // (F0) briefly applies. The final 6-digit form falls back to the SEED
    // alpha — the one the well had when the edit session started — never
    // to the prefix's leak.
    let seed = 0xFF;
    let through_prefix = hex_edited_color(seed, "FF7F0").expect("the prefix parses");
    let [_, _, _, prefix_alpha] = through_prefix.to_srgba_unmultiplied();
    assert_eq!(
        prefix_alpha, 0xF0,
        "the prefix form does carry its own alpha"
    );
    assert_eq!(
        hex_edited_color(seed, "FF7F00"),
        Some(Color32::from_rgba_unmultiplied(0xFF, 0x7F, 0x00, seed)),
        "the final 6-digit color must keep the SEED alpha"
    );
}

#[test]
fn invalid_hex_text_applies_nothing() {
    // Fewer than 3 or more than 8 digits left after the junk strip, empty
    // or digit-less — all None, so the field simply does not touch the
    // color while the text is partial.
    assert_eq!(hex_edited_color(SEED_ALPHA, "#F"), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, "#FF"), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, "FF7F000321"), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, "zzzzzz"), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, "#GGGGGG"), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, "0xzz"), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, ""), None);
    assert_eq!(hex_edited_color(SEED_ALPHA, "   "), None);
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
    let formatted = format_color_hex(Color32::from_rgba_unmultiplied(
        0x12, 0x34, 0x56, SEED_ALPHA,
    ));
    assert_eq!(formatted.len(), 9, "# plus 8 hex digits: {formatted}");
    assert!(formatted.starts_with('#'));
    assert!(formatted[1..].bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_eq!(&formatted[7..], "C8", "the alpha tail is the seeded one");
}

#[test]
fn the_launch_flags_and_the_hex_field_parse_one_syntax() {
    // One primitive (`launch.rs::parse_hex_components`) feeds BOTH the
    // `--primary`/`--secondary` flags and the hex field — the syntax cannot
    // drift apart. The only difference is the alpha fallback: the flags
    // default a missing alpha to opaque, the field keeps the seeded one.
    let ([r, g, b], alpha) = parse_hex_components("#ff7f00").expect("parses");
    assert_eq!((r, g, b), (0xFF, 0x7F, 0x00));
    assert_eq!(alpha, None, "a short form carries no alpha of its own");
    assert_eq!(
        parse_color("#ff7f00"),
        Some(Color32::from_rgb(0xFF, 0x7F, 0x00)),
        "the launch form defaults the missing alpha to opaque"
    );
    let edited = hex_edited_color(SEED_ALPHA, "#ff7f00").expect("parses");
    let [r, g, b, a] = edited.to_srgba_unmultiplied();
    assert_eq!(
        (r, b, a),
        (0xFF, 0x00, SEED_ALPHA),
        "the text's RGB + the seeded alpha"
    );
    assert!(
        g.abs_diff(0x7F) <= 1,
        "egui's premultiplied Color32 storage may nudge a channel ±1, got {g}"
    );
}
