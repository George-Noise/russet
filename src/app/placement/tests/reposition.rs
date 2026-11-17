//! The §3.11 resize repositioning math: an anchored axis keeps its dropped
//! gap (clamped into the new range), a free axis keeps its gap ratio —
//! re-proportioned, never magnetized, never pulled flush.

use super::common::{assert_close, rect};
use crate::app::placement::{
    AxisAnchor, WINDOW_SNAP_DISTANCE, WindowAnchor, reposition_axis, reposition_window_pos,
    window_anchor,
};
use eframe::egui::pos2;

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
