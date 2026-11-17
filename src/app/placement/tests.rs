use super::{
    AxisAnchor, TOOL_SETTINGS_GAP, WINDOW_SNAP_DISTANCE, WindowAnchor, anchor_axis,
    first_open_ratio_pos, magnet_settings_pos, reposition_axis, reposition_window_pos, resize_step,
    window_anchor,
};
use eframe::egui::{Rect, pos2, vec2};

fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(pos2(x, y), vec2(w, h))
}

fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn anchor_axis_covers_both_sides_and_dead_zone() {
    assert_eq!(
        anchor_axis(10.0, 40.0, 32.0, AxisAnchor::Min),
        AxisAnchor::Min
    );
    assert_eq!(
        anchor_axis(40.0, 4.0, 32.0, AxisAnchor::Min),
        AxisAnchor::Max
    );
    assert_eq!(
        anchor_axis(40.0, 40.0, 32.0, AxisAnchor::Min),
        AxisAnchor::None
    );
    // Exactly at the distance: "closer than" is strict, so no anchor.
    assert_eq!(
        anchor_axis(32.0, 40.0, 32.0, AxisAnchor::Min),
        AxisAnchor::None
    );
}

#[test]
fn anchor_axis_conflict_takes_the_requested_side() {
    // Both edges within reach: the horizontal axis must pick LEFT,
    // the vertical one BOTTOM (§3.11).
    assert_eq!(
        anchor_axis(10.0, 20.0, 32.0, AxisAnchor::Min),
        AxisAnchor::Min
    );
    assert_eq!(
        anchor_axis(10.0, 20.0, 32.0, AxisAnchor::Max),
        AxisAnchor::Max
    );
}

#[test]
fn dropped_window_anchors_to_each_close_edge() {
    let area = rect(0.0, 0.0, 800.0, 600.0);
    // The magnet only decides the ANCHOR; the window stays exactly
    // where the user dropped it — never pulled flush (§3.11).
    assert_eq!(
        window_anchor(rect(10.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
        WindowAnchor {
            x: AxisAnchor::Min,
            y: AxisAnchor::None
        }
    );
    assert_eq!(
        window_anchor(rect(690.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
        WindowAnchor {
            x: AxisAnchor::Max,
            y: AxisAnchor::None
        }
    );
    assert_eq!(
        window_anchor(rect(300.0, 8.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
        WindowAnchor {
            x: AxisAnchor::None,
            y: AxisAnchor::Min
        }
    );
    assert_eq!(
        window_anchor(rect(300.0, 542.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
        WindowAnchor {
            x: AxisAnchor::None,
            y: AxisAnchor::Max
        }
    );
}

#[test]
fn dropped_window_middle_exact_distance_and_flush() {
    let area = rect(0.0, 0.0, 800.0, 600.0);
    // Mid-screen: no edge within reach.
    assert_eq!(
        window_anchor(rect(300.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
        WindowAnchor::default()
    );
    // Exactly 2rem from the left edge: not "closer than".
    assert_eq!(
        window_anchor(rect(32.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE).x,
        AxisAnchor::None
    );
    // Dropped flush: gap 0 is a valid dropped gap — anchored left, so
    // the window stays flush through resizes.
    assert_eq!(
        window_anchor(rect(0.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE).x,
        AxisAnchor::Min
    );
}

#[test]
fn anchor_conflict_prefers_left_and_bottom() {
    // 150 pt window in a 200 pt main window: both horizontal gaps (20/30)
    // and both vertical gaps (20/30) are within reach → left and bottom.
    let area = rect(0.0, 0.0, 200.0, 200.0);
    assert_eq!(
        window_anchor(rect(20.0, 20.0, 150.0, 150.0), area, WINDOW_SNAP_DISTANCE),
        WindowAnchor {
            x: AxisAnchor::Min,
            y: AxisAnchor::Max
        }
    );
}

#[test]
fn reposition_free_axis_keeps_anchors_and_center() {
    let old = (0.0, 1000.0);
    let new = (0.0, 500.0);
    assert_close(
        reposition_axis(old, new, (0.0, 200.0), AxisAnchor::None),
        0.0,
    ); // left-anchored
    assert_close(
        reposition_axis(old, new, (800.0, 1000.0), AxisAnchor::None),
        300.0,
    ); // right-anchored
    assert_close(
        reposition_axis(old, new, (400.0, 600.0), AxisAnchor::None),
        150.0,
    ); // centered
}

#[test]
fn reposition_free_axis_keeps_gap_ratio_when_growing_and_shrinking() {
    let old = (0.0, 1000.0);
    // gaps 240/560 → ratio 0.3
    assert_close(
        reposition_axis(old, (0.0, 2000.0), (240.0, 440.0), AxisAnchor::None),
        540.0,
    );
    assert_close(
        reposition_axis(old, (0.0, 500.0), (240.0, 440.0), AxisAnchor::None),
        90.0,
    );
}

#[test]
fn reposition_free_axis_clamps_window_into_new_range() {
    // Window wider than the new range: clamped flush to the lower edge.
    assert_close(
        reposition_axis(
            (0.0, 1000.0),
            (0.0, 100.0),
            (100.0, 300.0),
            AxisAnchor::None,
        ),
        0.0,
    );
    // Window fills the old range exactly (no gaps): keep the lower edge.
    assert_close(
        reposition_axis((0.0, 200.0), (0.0, 600.0), (0.0, 200.0), AxisAnchor::None),
        0.0,
    );
}

#[test]
fn anchored_axis_keeps_the_dropped_gap_on_resize() {
    let old = (0.0, 1000.0);
    // Dropped with a 30 pt left gap: the gap survives grow and shrink.
    assert_close(
        reposition_axis(old, (0.0, 2000.0), (30.0, 230.0), AxisAnchor::Min),
        30.0,
    );
    assert_close(
        reposition_axis(old, (0.0, 500.0), (30.0, 230.0), AxisAnchor::Min),
        30.0,
    );
    // Dropped with a 30 pt right gap: slides with the right edge.
    assert_close(
        reposition_axis(old, (0.0, 2000.0), (770.0, 970.0), AxisAnchor::Max),
        1770.0,
    );
    assert_close(
        reposition_axis(old, (0.0, 500.0), (770.0, 970.0), AxisAnchor::Max),
        270.0,
    );
}

#[test]
fn anchored_axis_clamps_into_the_new_range() {
    let old = (0.0, 1000.0);
    // Left gap 300 no longer fits (free space 200): the window is kept
    // inside the range instead of overflowing the right edge.
    assert_close(
        reposition_axis(old, (0.0, 400.0), (300.0, 500.0), AxisAnchor::Min),
        200.0,
    );
    // Right gap 300 no longer fits: clamped against the left edge.
    assert_close(
        reposition_axis(old, (0.0, 400.0), (500.0, 700.0), AxisAnchor::Max),
        0.0,
    );
}

#[test]
fn reposition_bottom_right_free_window_stays_bottom_right() {
    let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
    let new_area = rect(0.0, 0.0, 500.0, 600.0);
    let prev = rect(800.0, 650.0, 200.0, 150.0); // flush right-bottom
    assert_eq!(
        reposition_window_pos(prev, prev_area, new_area, WindowAnchor::default()),
        pos2(300.0, 450.0)
    );
}

#[test]
fn reposition_does_not_magnetize_free_windows() {
    // 100 pt off both edges: after a resize the window is only
    // re-proportioned, never pulled flush to an edge.
    let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
    let new_area = rect(0.0, 0.0, 400.0, 300.0);
    let result = reposition_window_pos(
        rect(100.0, 100.0, 200.0, 200.0),
        prev_area,
        new_area,
        WindowAnchor::default(),
    );
    assert_close(result.x, 25.0); // x ratio 100/800 = 0.125, free_new 200
    assert_close(result.y, 16.667); // y ratio 100/600 = 1/6, free_new 100
}

#[test]
fn anchored_window_keeps_dropped_gaps_through_resize() {
    let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
    let new_area = rect(0.0, 0.0, 500.0, 600.0);
    // Dropped 25 pt from the left and 20 pt from the top: BOTH gaps are
    // kept exactly, on any resize.
    let anchor = window_anchor(
        rect(25.0, 20.0, 200.0, 150.0),
        prev_area,
        WINDOW_SNAP_DISTANCE,
    );
    assert_eq!(
        anchor,
        WindowAnchor {
            x: AxisAnchor::Min,
            y: AxisAnchor::Min
        }
    );
    assert_eq!(
        reposition_window_pos(rect(25.0, 20.0, 200.0, 150.0), prev_area, new_area, anchor),
        pos2(25.0, 20.0)
    );
    // Dropped flush right-bottom: stays flush right-bottom.
    let flush = window_anchor(
        rect(800.0, 650.0, 200.0, 150.0),
        prev_area,
        WINDOW_SNAP_DISTANCE,
    );
    assert_eq!(
        flush,
        WindowAnchor {
            x: AxisAnchor::Max,
            y: AxisAnchor::Max
        }
    );
    assert_eq!(
        reposition_window_pos(rect(800.0, 650.0, 200.0, 150.0), prev_area, new_area, flush),
        pos2(300.0, 450.0)
    );
}

#[test]
fn first_open_ratio_splits_the_free_vertical_space() {
    let area = rect(0.0, 0.0, 800.0, 600.0);
    // 50 pt window: free space 550 → 220 above / 330 below = the §3.2
    // 40%/60% first-open split; the left edge stays where it was.
    assert_eq!(
        first_open_ratio_pos(area, rect(8.0, 275.0, 100.0, 50.0), 0.4),
        pos2(8.0, 220.0)
    );
    // A window taller than the main window is clamped flush to the top.
    assert_eq!(
        first_open_ratio_pos(area, rect(8.0, 0.0, 100.0, 700.0), 0.4),
        pos2(8.0, 0.0)
    );
}

#[test]
fn resize_pins_the_reference_on_the_first_changed_frame() {
    let stable_area = rect(0.0, 0.0, 1000.0, 800.0);
    let win = rect(100.0, 100.0, 200.0, 200.0);
    let grown = rect(0.0, 0.0, 1200.0, 800.0);
    // First changed frame: the PRE-resize state is pinned and the free
    // axis keeps its gap ratio (100/800 = 0.125 of the new 1000 pt free run).
    let (pin, pos) = resize_step(
        Some(win),
        Some(stable_area),
        grown,
        None,
        WindowAnchor::default(),
    );
    assert_eq!(pin, Some((win, stable_area)));
    assert_eq!(pos, Some(pos2(125.0, 100.0)));
    // The series turns back (non-monotonic Wayland/Web configure events) and
    // the drawn rect is stale — the target is re-derived FROM THE PIN, so the
    // content returning to the pre-resize size lands the window EXACTLY at
    // its pre-resize position instead of drifting (the «колбасит» fix).
    let (_, pos_back) = resize_step(
        Some(rect(300.0, 100.0, 200.0, 200.0)),
        Some(grown),
        stable_area,
        pin,
        WindowAnchor::default(),
    );
    assert_eq!(pos_back, Some(pos2(100.0, 100.0)));
}

#[test]
fn resize_pin_clears_when_stable_and_repins_fresh() {
    let a = rect(0.0, 0.0, 1000.0, 800.0);
    let win = rect(100.0, 100.0, 200.0, 200.0);
    // Stable frame: the pin clears, nothing moves — the last computed
    // position persists until the NEXT resize.
    assert_eq!(
        resize_step(
            Some(win),
            Some(a),
            a,
            Some((win, a)),
            WindowAnchor::default()
        ),
        (None, None)
    );
    // The next resize pins the CURRENT state fresh (the window was moved
    // between resizes): the new reference, not the stale one.
    let b = rect(0.0, 0.0, 500.0, 800.0);
    let moved = rect(200.0, 100.0, 200.0, 200.0);
    let (pin, pos) = resize_step(Some(moved), Some(a), b, None, WindowAnchor::default());
    assert_eq!(pin, Some((moved, a)));
    assert_eq!(pos, Some(pos2(75.0, 100.0)));
}

#[test]
fn resize_step_needs_history_on_both_sides() {
    let a = rect(0.0, 0.0, 1000.0, 800.0);
    let b = rect(0.0, 0.0, 500.0, 800.0);
    // First frame ever (no previous content): nothing to compare.
    assert_eq!(
        resize_step(None, None, b, None, WindowAnchor::default()),
        (None, None)
    );
    // First frame of a just-shown window (no drawn rect yet): no pin
    // material either — the next changed frame pins normally.
    assert_eq!(
        resize_step(None, Some(a), b, None, WindowAnchor::default()),
        (None, None)
    );
}

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

// Regression tests for the owner bug report 2026-11-17: the FIRST-OPEN
// positions of the floating windows sit at fixed gaps to the main-window
// edges, but a never-dragged window used to have FREE axes (anchors were
// decided only on drag-end), so its first-open gaps scaled with the free
// space and the windows drifted off their magnet boundaries on the very
// first main-window resize. The fix gives every first-open position
// explicit anchors (see `WindowPlacement::with_first_open_anchors`); these
// tests pin the math the fix relies on.

#[test]
fn first_open_anchors_keep_first_open_gaps_through_resize() {
    // Layers window first open: 0.5rem from the right edge, 2.25rem above
    // the bottom edge — content 1200×800, window 240×300.
    let prev_area = rect(0.0, 0.0, 1200.0, 800.0);
    let first_open = rect(1200.0 - 8.0 - 240.0, 800.0 - 36.0 - 300.0, 240.0, 300.0);
    // Raw drop-style detection (what a drag-end would compute) does NOT
    // cover this position: the 36 pt bottom gap is outside the 2rem snap
    // zone. The first-open anchors are explicit, not re-detected.
    assert_eq!(
        window_anchor(first_open, prev_area, WINDOW_SNAP_DISTANCE),
        WindowAnchor {
            x: AxisAnchor::Max,
            y: AxisAnchor::None,
        }
    );
    // With the explicit first-open anchors the gaps survive grow and shrink.
    let anchor = WindowAnchor {
        x: AxisAnchor::Max,
        y: AxisAnchor::Max,
    };
    let grown = reposition_window_pos(first_open, prev_area, rect(0.0, 0.0, 1300.0, 850.0), anchor);
    assert_close(grown.x + 240.0, 1300.0 - 8.0); // right gap 8 kept
    assert_close(grown.y + 300.0, 850.0 - 36.0); // bottom gap 36 kept
    let shrunk =
        reposition_window_pos(first_open, prev_area, rect(0.0, 0.0, 1000.0, 700.0), anchor);
    assert_close(shrunk.x + 240.0, 1000.0 - 8.0);
    assert_close(shrunk.y + 300.0, 700.0 - 36.0);
}

#[test]
fn without_first_open_anchors_the_gaps_drift_with_the_free_space() {
    // The BUG, pinned: the same Layers first-open rect with FREE axes (the
    // old `WindowPlacement::default()`) scales the gaps proportionally to
    // the free space — the window visibly slides off its magnet boundaries
    // on the first resize, which is exactly what the owner reported.
    let prev_area = rect(0.0, 0.0, 1200.0, 800.0);
    let first_open = rect(1200.0 - 8.0 - 240.0, 800.0 - 36.0 - 300.0, 240.0, 300.0);
    let grown = reposition_window_pos(
        first_open,
        prev_area,
        rect(0.0, 0.0, 1300.0, 850.0),
        WindowAnchor::default(),
    );
    // Right gap 8 → 8 × (1300−240)/(1200−240): drifted off the edge.
    assert_close(grown.x + 240.0, 1300.0 - 8.0 * (1060.0 / 960.0));
    // Bottom gap 36 → 36 × (850−300)/(800−300): drifted too.
    assert_close(grown.y + 300.0, 850.0 - 36.0 * (550.0 / 500.0));
}

#[test]
fn tools_first_open_keeps_left_gap_and_free_vertical_ratio() {
    // Tools window (§3.2): 0.5rem from the left edge (an anchored axis);
    // vertically 40% of the free space above / 60% below — a FREE-axis
    // placement that must keep its ratio while the left gap stays fixed.
    let prev_area = rect(0.0, 0.0, 1200.0, 800.0);
    // Window 200×300 at left gap 8: free vertical space 500 → 200 above.
    let first_open = rect(8.0, 200.0, 200.0, 300.0);
    let anchor = WindowAnchor {
        x: AxisAnchor::Min,
        y: AxisAnchor::None,
    };
    let grown = reposition_window_pos(first_open, prev_area, rect(0.0, 0.0, 1300.0, 900.0), anchor);
    assert_close(grown.x, 8.0); // left gap kept
    assert_close(grown.y, 240.0); // 40% of the new free space 600
    let shrunk =
        reposition_window_pos(first_open, prev_area, rect(0.0, 0.0, 1100.0, 700.0), anchor);
    assert_close(shrunk.x, 8.0);
    assert_close(shrunk.y, 160.0); // 40% of the new free space 400
}
