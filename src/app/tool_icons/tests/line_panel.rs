//! The Line panel's icon tests (owner request 2026-11-20): the word "Mode"
//! radio became icon buttons in the Tools window's style — the same pass
//! that reshaped the Shape panel. The pair draws the LINE TOOL's emblem
//! (one geometry, the Shape MODE trio's principle): the Curve button
//! strokes it EXACTLY as the Tools window's Line button does, the Straight
//! button draws the same stroke UNBENT. The topic pins both.

use super::*;
use crate::app::tool_icons::{LineIcon, line_icon_label};
use crate::tools::Tool;

/// The one-geometry rule: the panel's Curve button and the Tools window's
/// Line button draw the SAME mark — the rasterized pixmaps are
/// byte-identical (the Shape topic's "pixel-for-pixel the tool button"
/// assertion, now for Line).
#[test]
fn the_curve_button_draws_the_tools_own_mark() {
    let panel = rasterize_line_icon(LineIcon::Curve, 24, TOOL_INK).expect("px is in 1..=256");
    let tool = rasterize_tool_icon(Tool::Line, 24, TOOL_INK).expect("px is in 1..=256");
    assert_eq!(panel.data(), tool.data());
}

/// The Straight button is the tool's mark UNBENT: the stroke runs STRAIGHT
/// between the emblem curve's endpoints (4.8, 18.6) → (19.2, 4.8) — so it
/// has ink at the straight segment's midpoint (12.0, 11.7) and NONE in the
/// curve's flat bottom run (the box around (8.7, 18.2), where the emblem's
/// first arc passes and a straight line between the same endpoints never
/// can). The Curve mark hits that box, and the two pixmaps differ.
#[test]
fn the_straight_button_is_the_mark_unbent() {
    let straight = rasterize_line_icon(LineIcon::Straight, 24, TOOL_INK).expect("px is in 1..=256");
    let curve = rasterize_line_icon(LineIcon::Curve, 24, TOOL_INK).expect("px is in 1..=256");
    assert!(has_opaque_near(&straight, (12.0, 11.7), 1.5));
    assert_eq!(
        count_opaque_in_box(&straight, (7.5, 17.3, 11.0, 19.5)),
        0,
        "a straight stroke between the endpoints cannot dip into the curve's flat run"
    );
    assert!(count_opaque_in_box(&curve, (7.5, 17.3, 11.0, 19.5)) > 0);
    assert_ne!(straight.data(), curve.data());
}

/// The tooltips carry EXACTLY the words the old word radio showed.
#[test]
fn the_line_icons_label_the_old_radio_words() {
    for (icon, label) in [(LineIcon::Straight, "Straight"), (LineIcon::Curve, "Curve")] {
        assert_eq!(line_icon_label(icon), label, "tooltip of {icon:?}");
    }
}
