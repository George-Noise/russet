//! The §3.11 drop-time magnetism tests: which edges an axis anchors to
//! (`anchor_axis` on one axis, `window_anchor` on the whole window), the
//! dead zone at exactly the snap distance, the flush-drop gap 0 and the
//! left-over-bottom conflict preference.

use super::common::rect;
use crate::app::placement::{
    AxisAnchor, WINDOW_SNAP_DISTANCE, WindowAnchor, anchor_axis, window_anchor,
};

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
