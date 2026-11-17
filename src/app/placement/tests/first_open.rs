//! The §3.2 first-open placement of the Tools window (the one-shot 40/60
//! free-space split) and the regression tests for the owner bug report
//! 2026-11-17: the FIRST-OPEN positions of the floating windows sit at
//! fixed gaps to the main-window edges, but a never-dragged window used to
//! have FREE axes (anchors were decided only on drag-end), so its
//! first-open gaps scaled with the free space and the windows drifted off
//! their magnet boundaries on the very first main-window resize. The fix
//! gives every first-open position explicit anchors (see
//! `WindowPlacement::with_first_open_anchors`); these tests pin the math
//! the fix relies on.

use super::common::{assert_close, rect};
use crate::app::placement::{
    AxisAnchor, WINDOW_SNAP_DISTANCE, WindowAnchor, first_open_ratio_pos, reposition_window_pos,
    window_anchor,
};
use eframe::egui::pos2;

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
