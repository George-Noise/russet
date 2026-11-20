//! The Shape panel's icon tests (owner request 2026-11-20): the KIND

//! buttons' gray previews, the MODE toggles' solid/hollow current-kind

//! drawings and the old-word tooltips.

use super::super::{SHAPE_ICONS, ShapeIcon, TOOL_INK, rasterize_shape_icon, shape_icon_label};
use super::*;
use crate::tools::ShapeKind;

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
            },
            "tooltip of {icon:?} — the words the old widgets showed"
        );
        for kind in [ShapeKind::Rectangle, ShapeKind::Ellipse] {
            let pm = rasterize_shape_icon(icon, kind, 24, TOOL_INK).expect("px is in 1..=256");
            assert!(
                count_opaque(&pm) >= MIN_OPAQUE_PIXELS,
                "{icon:?}/{kind:?} must draw more than {MIN_OPAQUE_PIXELS} opaque pixels"
            );
        }
    }
}

#[test]
fn kind_buttons_are_gray_filled_previews_with_an_ink_rim() {
    for icon in [ShapeIcon::Rectangle, ShapeIcon::Ellipse] {
        let pm = rasterize_shape_icon(icon, ShapeKind::Rectangle, 48, TOOL_INK)
            .expect("px is in 1..=256");
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

#[test]
fn mode_toggles_are_the_solid_and_hollow_current_kind() {
    // The mode icons are INK ONLY (no gray preview fill), and the solid/
    // hollow pair differs exactly in the interior: the Fill toggle fills it,
    // the Outline toggle keeps it empty.
    for kind in [ShapeKind::Rectangle, ShapeKind::Ellipse] {
        let fill =
            rasterize_shape_icon(ShapeIcon::Fill, kind, 48, TOOL_INK).expect("px is in 1..=256");
        let outline =
            rasterize_shape_icon(ShapeIcon::Outline, kind, 48, TOOL_INK).expect("px is in 1..=256");
        assert!(
            !has_color(&fill, (160, 160, 160), 6) && !has_color(&outline, (160, 160, 160), 6),
            "{kind:?}: the mode toggles must carry no preview gray"
        );
        assert!(
            has_opaque_near(&fill, (12.0, 12.0), 2.0),
            "{kind:?}: the Fill toggle must fill the interior"
        );
        assert!(
            !has_opaque_near(&outline, (12.0, 12.0), 2.0),
            "{kind:?}: the Outline toggle must keep the interior empty"
        );
        // The crisp solid-vs-hollow discriminator over the central box
        // (7.5..16.5 — the rim's inner edge runs at ≈5.45, so the box holds
        // no rim ink): the solid icon fills it, the hollow one has ZERO
        // opaque pixels there.
        const CENTER: (f32, f32, f32, f32) = (7.5, 7.5, 16.5, 16.5);
        assert!(
            count_opaque_in_box(&fill, CENTER) > 200,
            "{kind:?}: the Fill toggle must ink the central box"
        );
        assert!(
            count_opaque_in_box(&outline, CENTER) == 0,
            "{kind:?}: the Outline toggle must keep the central box empty"
        );
    }
    // The toggles preview the CURRENT kind: with Ellipse selected they are
    // ROUND — the bounding-box corner the square fills stays empty, while
    // the Rectangle kind fills it.
    for icon in [ShapeIcon::Fill, ShapeIcon::Outline] {
        let square = rasterize_shape_icon(icon, ShapeKind::Rectangle, 48, TOOL_INK)
            .expect("px is in 1..=256");
        let round =
            rasterize_shape_icon(icon, ShapeKind::Ellipse, 48, TOOL_INK).expect("px is in 1..=256");
        // (5.5, 5.5) sits inside the 15×15 square (its rim's inner edge runs
        // at ≈5.45) but 1.7 logical units clear of the circle's outer ink.
        assert!(
            has_opaque_near(&square, (5.5, 5.5), 0.5),
            "{icon:?}: the rectangle kind must ink the corner"
        );
        assert!(
            !has_opaque_near(&round, (5.5, 5.5), 0.5),
            "{icon:?}: the ellipse kind must keep the corner empty"
        );
    }
}
