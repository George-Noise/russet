//! The magnetized tool-settings panel's position math (owner request
//! 2026-11-17): below/above flip around the Tools window, edge alignment
//! by the main-window center half, and clamping when the panel fits
//! nowhere.

use super::common::rect;
use crate::app::placement::{TOOL_SETTINGS_GAP, magnet_settings_pos};
use eframe::egui::{pos2, vec2};

#[test]
fn settings_gap_is_a_quarter_rem() {
    assert_eq!(TOOL_SETTINGS_GAP, 4.0);
}

#[test]
fn settings_panel_sits_below_the_tools_window_at_the_gap() {
    let content = rect(0.0, 0.0, 800.0, 600.0);
    // Tools window on the left half with plenty of room below.
    let tools = rect(8.0, 220.0, 100.0, 150.0);
    assert_eq!(
        magnet_settings_pos(tools, vec2(240.0, 80.0), content, 4.0),
        pos2(8.0, 374.0) // left edges align; 370 + gap 4
    );
}

#[test]
fn settings_panel_flips_above_when_no_room_below() {
    let content = rect(0.0, 0.0, 800.0, 600.0);
    // Below: 500 + 4 + 120 = 624 > 600 — does not fit → above, and the
    // Tools window (center x 450) is right of the center (400): right edges.
    let tools = rect(400.0, 500.0, 100.0, 90.0);
    assert_eq!(
        magnet_settings_pos(tools, vec2(240.0, 120.0), content, 4.0),
        pos2(260.0, 376.0) // 500 - 240; 500 - 4 - 120
    );
}

#[test]
fn settings_panel_aligns_by_the_screen_half_of_the_tools_window() {
    let content = rect(0.0, 0.0, 800.0, 600.0); // center x 400
    let size = vec2(240.0, 80.0);
    let left_tools = rect(8.0, 100.0, 100.0, 150.0); // center 58 < 400
    let right_tools = rect(500.0, 100.0, 100.0, 150.0); // center 550 > 400
    assert_eq!(
        magnet_settings_pos(left_tools, size, content, 4.0).x,
        left_tools.min.x
    );
    assert_eq!(
        magnet_settings_pos(right_tools, size, content, 4.0).x,
        right_tools.max.x - size.x
    );
    // Exactly on the center counts as the right half.
    let center_tools = rect(350.0, 100.0, 100.0, 150.0); // center 400
    assert_eq!(
        magnet_settings_pos(center_tools, size, content, 4.0).x,
        center_tools.max.x - size.x
    );
}

#[test]
fn settings_panel_clamps_into_the_main_window_when_it_fits_nowhere() {
    let content = rect(0.0, 0.0, 800.0, 600.0);
    // Tools near the bottom, the panel TALLER than the space above: the
    // flipped position would start above the main window — clamped to the
    // top edge (overlapping the Tools window, but on-screen).
    let tools = rect(8.0, 550.0, 100.0, 45.0);
    assert_eq!(
        magnet_settings_pos(tools, vec2(240.0, 600.0), content, 4.0),
        pos2(8.0, 0.0)
    );
    // A panel wider than the main window is clamped to the left edge
    // (the left-aligned Tools window would start it off-screen).
    let wide = rect(8.0, 100.0, 100.0, 150.0);
    assert_eq!(
        magnet_settings_pos(wide, vec2(900.0, 60.0), content, 4.0),
        pos2(0.0, 254.0)
    );
}
