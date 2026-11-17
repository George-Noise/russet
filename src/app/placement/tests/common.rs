//! Shared helpers of the placement test suites: a rect builder from
//! min-corner + size and a float assert that survives f32 jitter.

use eframe::egui::{Rect, pos2, vec2};

pub(super) fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
    Rect::from_min_size(pos2(x, y), vec2(w, h))
}

pub(super) fn assert_close(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() < 1e-3,
        "expected {expected}, got {actual}"
    );
}
