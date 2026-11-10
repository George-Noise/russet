use super::{App, key_pan_dir};
use eframe::egui::Vec2;

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
