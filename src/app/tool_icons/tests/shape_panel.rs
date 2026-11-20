//! The Shape panel's icon tests (owner request 2026-11-20): the KIND

//! buttons' gray previews and the old-word tooltips, plus the MODE trio's

//! emblem drawings — the follow-up the same day made the MODE row a

//! THREE-way radio over the Shape TOOL's icon ("не повторение выбранной

//! фигуры, а иконка инструмента Shape"): solid = Fill, hollow = Outline

//! (pixel-for-pixel the tool button), gray fill under the rim = both.

use super::super::{SHAPE_ICONS, ShapeIcon, TOOL_INK, rasterize_shape_icon, shape_icon_label};
use super::*;
use crate::tools::Tool;

#[test]
fn shape_panel_icons_rasterize_and_keep_the_old_labels() {
    for icon in SHAPE_ICONS {
        assert_eq!(
            shape_icon_label(icon),
            match icon {
                ShapeIcon::Rectangle => "Rectangle",
                ShapeIcon::Ellipse => "Ellipse",
                ShapeIcon::Fill => "Fill",
                ShapeIcon::Outline => "Outline",
                ShapeIcon::FillOutline => "Fill + Outline",
            },
            "tooltip of {icon:?} — the words the old widgets showed"
        );
        let pm = rasterize_shape_icon(icon, 24, TOOL_INK).expect("px is in 1..=256");
        assert!(
            count_opaque(&pm) >= MIN_OPAQUE_PIXELS,
            "{icon:?} must draw more than {MIN_OPAQUE_PIXELS} opaque pixels"
        );
    }
}

#[test]
fn kind_buttons_are_gray_filled_previews_with_an_ink_rim() {
    for icon in [ShapeIcon::Rectangle, ShapeIcon::Ellipse] {
        let pm = rasterize_shape_icon(icon, 48, TOOL_INK).expect("px is in 1..=256");
        assert!(
            has_color(&pm, (160, 160, 160), 6),
            "{icon:?}: the gray preview fill is missing"
        );
        assert!(
            has_color(&pm, (64, 64, 64), 10),
            "{icon:?}: the ink rim is missing"
        );
    }
}

/// The MODE radios are the TOOL emblem (the Tools window's Shape button's
/// rectangle + circle — owner request 2026-11-20), not a shape preview: the
/// hollow radio must be PIXEL-FOR-PIXEL the tool's icon, and the trio's
/// interiors tell the three paints apart — Fill inks the interior, Outline
/// keeps it empty, Fill + Outline puts the gray under the rim.
#[test]
fn the_outline_radio_is_pixel_identical_to_the_shape_tool_icon() {
    for px in [24, 48, 256] {
        let outline =
            rasterize_shape_icon(ShapeIcon::Outline, px, TOOL_INK).expect("px is in 1..=256");
        let tool = rasterize_tool_icon(Tool::Shape, px, TOOL_INK).expect("px is in 1..=256");
        assert_eq!(
            outline.data(),
            tool.data(),
            "{px} px: the Outline radio must BE the Shape tool's icon"
        );
    }
}

#[test]
fn the_mode_radios_paint_the_emblem_three_ways() {
    // The emblem's two interiors in the icons' logical space: the
    // rectangle's body point (clear of the circle: dist from (15.6, 10.6)
    // is ~11.2 > 5.95 = r + rim half) and the circle's center (clear of the
    // rectangle: x 15.6 > 13.8 + rim half). The rim's inner edge sits at
    // ~0.95 from each boundary, so both probe points hold no rim ink.
    const RECT_BODY: (f32, f32) = (6.0, 16.0);
    const CIRCLE_BODY: (f32, f32) = (15.6, 10.6);
    let fill = rasterize_shape_icon(ShapeIcon::Fill, 48, TOOL_INK).expect("px is in 1..=256");
    let outline = rasterize_shape_icon(ShapeIcon::Outline, 48, TOOL_INK).expect("px is in 1..=256");
    let both =
        rasterize_shape_icon(ShapeIcon::FillOutline, 48, TOOL_INK).expect("px is in 1..=256");

    // No gray preview anywhere on Fill/Outline — the gray belongs to the
    // kind previews and the explicit Fill+Outline choice only.
    assert!(
        !has_color(&fill, (160, 160, 160), 6) && !has_color(&outline, (160, 160, 160), 6),
        "Fill/Outline must carry no preview gray"
    );

    for pt in [RECT_BODY, CIRCLE_BODY] {
        assert!(
            has_opaque_near(&fill, pt, 0.5),
            "{pt:?}: the Fill radio must ink the emblem's interior"
        );
        assert!(
            !has_opaque_near(&outline, pt, 0.5),
            "{pt:?}: the Outline radio must keep the interior empty"
        );
        assert!(
            has_color(&both, (160, 160, 160), 6),
            "{pt:?}: the Fill+Outline radio must gray-fill the interior"
        );
    }

    // The crisp discriminator over a rectangle-body-only box: x/y clear of
    // the rim's inner edges (4.55/12.85 on x, 10.15/18.45 on y) AND farther
    // than the circle's outer ink radius (r 5.0 + the 0.95 rim half ≈ 6.2)
    // from the circle's center (15.6, 10.6) — the emblem's shapes overlap,
    // so the box must avoid the circle's rim too: solid ink for Fill, ZERO
    // opaque pixels for Outline, gray for Fill + Outline.
    const RECT_BOX: (f32, f32, f32, f32) = (4.9, 11.0, 9.0, 18.2);
    assert!(
        count_opaque_in_box(&fill, RECT_BOX) > 100,
        "the Fill radio must ink the rectangle's body"
    );
    assert!(
        count_opaque_in_box(&outline, RECT_BOX) == 0,
        "the Outline radio must keep the rectangle's body empty"
    );
    let gray_in_box = count_opaque_in_box(&both, RECT_BOX);
    assert!(
        gray_in_box > 100,
        "the Fill+Outline radio must gray-fill the rectangle's body"
    );
}

/// The three radios are visually DISTINCT — the owner's three choices must
/// not be confusable: Fill ≠ Fill+Outline even though both fill the
/// interior (solid ink vs the gray under the rim), and each differs from
/// Outline by its interior alone.
#[test]
fn the_three_mode_radios_are_mutually_distinct() {
    let fill = rasterize_shape_icon(ShapeIcon::Fill, 48, TOOL_INK).expect("px is in 1..=256");
    let outline = rasterize_shape_icon(ShapeIcon::Outline, 48, TOOL_INK).expect("px is in 1..=256");
    let both =
        rasterize_shape_icon(ShapeIcon::FillOutline, 48, TOOL_INK).expect("px is in 1..=256");
    assert_ne!(fill.data(), outline.data());
    assert_ne!(fill.data(), both.data());
    assert_ne!(outline.data(), both.data());
    // Fill and Fill+Outline share the FILLED silhouette (the ink rim of
    // Fill+Outline sits over the same silhouette), but differ INSIDE: Fill
    // is ink-dark there, Fill+Outline is the lighter gray.
    assert!(
        !has_color(&fill, (160, 160, 160), 6) && has_color(&both, (160, 160, 160), 6),
        "Fill is solid ink, Fill+Outline is the gray under the rim"
    );
}
