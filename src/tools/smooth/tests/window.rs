//! Tests of [`SmoothStroke`] — the midpoint-quadratic emission window:
//! what each fed point emits, the duplicate filter and the release
//! closer (pure geometry, no rasterization).

use super::super::{SmoothPiece, SmoothStroke};
use eframe::egui::pos2;

#[test]
fn first_push_emits_the_line_to_the_first_midpoint() {
    let mut stroke = SmoothStroke::new(pos2(10.0, 10.0));
    assert_eq!(
        stroke.push(pos2(14.0, 12.0)),
        Some(SmoothPiece::Line {
            from: pos2(10.0, 10.0),
            to: pos2(12.0, 11.0)
        })
    );
}

#[test]
fn later_pushes_emit_quads_through_the_midpoints() {
    let mut stroke = SmoothStroke::new(pos2(0.0, 0.0));
    stroke.push(pos2(4.0, 0.0));
    assert_eq!(
        stroke.push(pos2(4.0, 4.0)),
        Some(SmoothPiece::Quad {
            from: pos2(2.0, 0.0),
            control: pos2(4.0, 0.0),
            to: pos2(4.0, 2.0)
        })
    );
    // The window slides: the new quad starts where the previous one ended.
    assert_eq!(
        stroke.push(pos2(0.0, 8.0)),
        Some(SmoothPiece::Quad {
            from: pos2(4.0, 2.0),
            control: pos2(4.0, 4.0),
            to: pos2(2.0, 6.0)
        })
    );
}

#[test]
fn duplicate_point_emits_nothing_and_keeps_the_window() {
    let mut stroke = SmoothStroke::new(pos2(5.0, 5.0));
    stroke.push(pos2(9.0, 5.0));
    assert_eq!(stroke.push(pos2(9.0, 5.0)), None);
    assert!(
        stroke.push(pos2(13.0, 5.0)).is_some(),
        "the next real point still emits"
    );
}

#[test]
fn close_emits_the_line_from_the_last_midpoint_to_the_last_point() {
    let mut stroke = SmoothStroke::new(pos2(0.0, 0.0));
    stroke.push(pos2(4.0, 0.0));
    stroke.push(pos2(4.0, 4.0));
    assert_eq!(
        stroke.close(),
        Some(SmoothPiece::Line {
            from: pos2(4.0, 2.0),
            to: pos2(4.0, 4.0)
        })
    );
}

#[test]
fn close_after_a_single_push_closes_the_first_segment() {
    let mut stroke = SmoothStroke::new(pos2(0.0, 0.0));
    stroke.push(pos2(4.0, 0.0));
    assert_eq!(
        stroke.close(),
        Some(SmoothPiece::Line {
            from: pos2(2.0, 0.0),
            to: pos2(4.0, 0.0)
        })
    );
}

#[test]
fn close_without_movement_emits_nothing() {
    let mut stroke = SmoothStroke::new(pos2(7.0, 7.0));
    assert_eq!(
        stroke.close(),
        None,
        "the press dot already covers the start"
    );
}

#[test]
fn close_is_consumed_once() {
    let mut stroke = SmoothStroke::new(pos2(0.0, 0.0));
    stroke.push(pos2(4.0, 0.0));
    assert!(stroke.close().is_some());
    assert_eq!(stroke.close(), None);
}
