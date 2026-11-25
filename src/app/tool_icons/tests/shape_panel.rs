//! The Shape panel's icon tests (owner request 2026-11-20): the KIND

//! buttons' gray previews, the old-word tooltips, the MODE trio's

//! emblem drawings (the follow-up the same day made the MODE row a

//! THREE-way radio over the Shape TOOL's icon — "not a repetition of the selected shape but the

//! Shape tool's icon": solid = Fill, hollow = Outline

//! (pixel-for-pixel the tool button), gray fill under the rim = both) and

//! the OUTLINE-PLACEMENT trio's corner compositions (the third pass the

//! same day: outside / centered / inside — always a SQUARE's corner,

//! whatever the selected kind is, with the gray fill quadrant, the click

//! dot and the outline bands in the mode's position).

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
                ShapeIcon::OutlineOutside => "Outline outside",
                ShapeIcon::OutlineCenter => "Outline centered",
                ShapeIcon::OutlineInside => "Outline inside",
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

// -----------------------------------------------------------------------
// The OUTLINE-PLACEMENT trio (owner request 2026-11-20): one corner of a
// square — whatever the selected kind is — with the gray fill quadrant,
// the click dot and the outline bands. The three icons differ ONLY in
// where the bands sit relative to the dot: beyond it (Outside), straddling
// it (Center), inside the fill (Inside).
// -----------------------------------------------------------------------

/// The composition's landmarks in the icons' logical space (draw.rs):
/// the click dot at (10, 10), the outline bands 6.5 thick (the owner's
/// 2026-11-21 pass), each running to 27 — past the icon's 24-pt edge — and
/// the gray fill quadrant running from the dot to the icon's edges.
const DOT: (f32, f32) = (10.0, 10.0);

/// The strip INSIDE the Outside band's left segment (x 3.5..10), LEFT of
/// the Center band's inner edge (6.75) and clear of the dot's ring —
/// inked ONLY by the Outside radio.
const OUTSIDE_STRIP: (f32, f32, f32, f32) = (4.0, 14.0, 6.0, 20.0);

/// The strip INSIDE the Inside band's left segment (x 10..16.5), RIGHT of
/// the Center band's outer edge (13.25) and clear of the dot — inked
/// ONLY by the Inside radio.
const INSIDE_STRIP: (f32, f32, f32, f32) = (14.0, 14.0, 15.8, 20.0);

/// A box ON the boundary below the dot (x ≥ 10.3): inked by Center and
/// Inside, but NOT by Outside (its band ends at 10).
const CENTER_BOX: (f32, f32, f32, f32) = (10.3, 12.6, 11.3, 13.4);

/// The placement trio: ALWAYS a square corner (no circle anywhere — the
/// owner: "as the corner of the square regardless of the selected shape"),
/// the gray fill quadrant marks the shape, the click dot sits at the
/// corner, and the bands' position tells the three modes apart.
#[test]
fn the_placement_radios_draw_the_corner_composition() {
    let outside =
        rasterize_shape_icon(ShapeIcon::OutlineOutside, 48, TOOL_INK).expect("px is in 1..=256");
    let center =
        rasterize_shape_icon(ShapeIcon::OutlineCenter, 48, TOOL_INK).expect("px is in 1..=256");
    let inside =
        rasterize_shape_icon(ShapeIcon::OutlineInside, 48, TOOL_INK).expect("px is in 1..=256");
    // The bands' ink is the same fixed mono ink the whole set draws with.
    let ink = (TOOL_INK.r(), TOOL_INK.g(), TOOL_INK.b());

    // The gray fill quadrant — the shape itself — in ALL three.
    for (name, pm) in [
        ("Outside", &outside),
        ("Center", &center),
        ("Inside", &inside),
    ] {
        assert!(
            has_color(pm, (160, 160, 160), 6),
            "{name}: the gray fill quadrant (the shape) is missing"
        );
        // The click dot: a white core over everything at the corner.
        assert!(
            has_opaque_near(pm, DOT, 1.5),
            "{name}: the click dot is missing at the corner"
        );
    }

    // The bands' positions tell the modes apart. The probes count the INK
    // color (not mere opacity): the bands sit over the opaque gray fill,
    // which would otherwise count as "inked" everywhere. The Outside-only
    // strip is inked ONLY by Outside; the Inside-only strip ONLY by
    // Inside; the straddle box by Center (and Inside), never by Outside.
    assert!(
        count_color_in_box(&outside, ink, 10, OUTSIDE_STRIP) > 10,
        "Outside must ink the band beyond the click dot"
    );
    assert_eq!(
        count_color_in_box(&center, ink, 10, OUTSIDE_STRIP),
        0,
        "Center must not ink beyond the click dot"
    );
    assert_eq!(
        count_color_in_box(&inside, ink, 10, OUTSIDE_STRIP),
        0,
        "Inside must not ink beyond the click dot"
    );
    assert!(
        count_color_in_box(&inside, ink, 10, INSIDE_STRIP) > 10,
        "Inside must ink the band within the fill"
    );
    assert_eq!(
        count_color_in_box(&outside, ink, 10, INSIDE_STRIP),
        0,
        "Outside must not ink within the fill"
    );
    assert_eq!(
        count_color_in_box(&center, ink, 10, INSIDE_STRIP),
        0,
        "Center must not ink past its straddle half-width"
    );
    assert!(
        has_color_in_box(&center, ink, 10, CENTER_BOX),
        "Center must ink the straddle band on the boundary"
    );
    assert!(
        !has_color_in_box(&outside, ink, 10, CENTER_BOX),
        "Outside must keep the boundary itself unpainted"
    );

    // And the three bitmaps are pairwise distinct.
    assert_ne!(outside.data(), center.data());
    assert_ne!(outside.data(), inside.data());
    assert_ne!(center.data(), inside.data());
}

/// The click dot is a CONTRASTING mark (owner: "a circle of the right
/// contrast, size and shape so it is visible"): a white core over
/// the gray fill AND over the ink bands — the Center radio's dot sits
/// exactly on the band corner, so its white core must survive there.
#[test]
fn the_click_dot_is_a_white_core_that_survives_every_background() {
    for icon in [
        ShapeIcon::OutlineOutside,
        ShapeIcon::OutlineCenter,
        ShapeIcon::OutlineInside,
    ] {
        let pm = rasterize_shape_icon(icon, 48, TOOL_INK).expect("px is in 1..=256");
        assert!(
            has_color(&pm, (255, 255, 255), 8),
            "{icon:?}: the dot's white core is missing"
        );
        // The core sits at the dot's center, over fill and bands alike.
        assert!(
            has_color_in_box(&pm, (255, 255, 255), 8, (9.2, 9.2, 10.8, 10.8)),
            "{icon:?}: the dot's white core must sit AT the click corner"
        );
    }
}
