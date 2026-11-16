use super::{App, key_pan_dir, wheel_notches};
use eframe::egui::{MouseWheelUnit, Vec2};

fn percent(zoom: f32) -> i32 {
    (zoom * 100.0).round() as i32
}

#[test]
fn zoom_in_sequence_matches_paint_net() {
    let mut zoom = 1.0;
    for expected in [112, 125, 140, 157, 176, 200, 224, 251, 281, 300] {
        zoom = App::next_zoom(zoom, true);
        assert_eq!(percent(zoom), expected);
    }
}

#[test]
fn zoom_out_returns_to_100_and_clamps() {
    assert_eq!(percent(App::next_zoom(1.12, false)), 100);
    assert_eq!(percent(App::next_zoom(8.0, true)), 800);
    assert_eq!(percent(App::next_zoom(0.1, false)), 10);
}

#[test]
fn wasd_directions_move_the_viewport() {
    // W = view up = pan.y grows (canvas content moves DOWN the screen,
    // same as a wheel-up scroll); A = view left = pan.x grows.
    assert_eq!(key_pan_dir(true, false, false, false), Vec2::new(0.0, 1.0));
    assert_eq!(key_pan_dir(false, false, true, false), Vec2::new(0.0, -1.0));
    assert_eq!(key_pan_dir(false, true, false, false), Vec2::new(1.0, 0.0));
    assert_eq!(key_pan_dir(false, false, false, true), Vec2::new(-1.0, 0.0));
}

#[test]
fn wasd_diagonals_are_normalized() {
    let diag = key_pan_dir(true, false, false, true);
    assert!((diag.length() - 1.0).abs() < 1e-4);
    assert!((diag.x + 1.0 / 2.0f32.sqrt()).abs() < 1e-4);
    // No keys — no movement.
    assert_eq!(key_pan_dir(false, false, false, false), Vec2::ZERO);
}

#[test]
fn wheel_notches_are_unit_aware() {
    // F12 variant B: the notch, not the event unit, defines the pan step.
    // Line (Windows/X11, Wayland discrete): ±1 per physical notch — pass-through.
    assert_eq!(
        wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, 1.0)),
        Vec2::new(0.0, 1.0)
    );
    // Fractional lines (high-resolution wheels/touchpads) stay proportional.
    assert_eq!(
        wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, 0.25)),
        Vec2::new(0.0, 0.25)
    );
    // Point (eframe-web DOM px, Wayland pixel deltas): 100 pt per notch —
    // Chrome's default wheel notch, the INTERIM F12-B calibration.
    assert_eq!(
        wheel_notches(MouseWheelUnit::Point, Vec2::new(0.0, 100.0)),
        Vec2::new(0.0, 1.0)
    );
    assert_eq!(
        wheel_notches(MouseWheelUnit::Point, Vec2::new(0.0, -100.0)),
        Vec2::new(0.0, -1.0)
    );
    // Page: capped at 3 notches per page (INTERIM, never observed in the wild).
    assert_eq!(
        wheel_notches(MouseWheelUnit::Page, Vec2::new(0.0, 1.0)),
        Vec2::new(0.0, 3.0)
    );
}
