use super::{App, key_pan_dir, pan_suppressed_by_modifiers, wheel_notches};
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
    // NATIVE Line (Windows/X11, Wayland discrete): ±1 per physical notch —
    // pass-through.
    assert_eq!(
        wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, 1.0), false),
        Vec2::new(0.0, 1.0)
    );
    // Fractional lines (high-resolution wheels/touchpads) stay proportional.
    assert_eq!(
        wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, 0.25), false),
        Vec2::new(0.0, 0.25)
    );
    // Point (eframe-web DOM px, Wayland pixel deltas): 100 pt per notch —
    // Chrome's default wheel notch, the INTERIM F12-B calibration. The
    // divisor is backend-independent: Chrome reports pixel mode everywhere.
    assert_eq!(
        wheel_notches(MouseWheelUnit::Point, Vec2::new(0.0, 100.0), false),
        Vec2::new(0.0, 1.0)
    );
    assert_eq!(
        wheel_notches(MouseWheelUnit::Point, Vec2::new(0.0, -100.0), true),
        Vec2::new(0.0, -1.0)
    );
    // Page: capped at 3 notches per page (INTERIM, never observed in the wild).
    assert_eq!(
        wheel_notches(MouseWheelUnit::Page, Vec2::new(0.0, 1.0), false),
        Vec2::new(0.0, 3.0)
    );
}

#[test]
fn web_line_mode_divides_by_the_lines_per_notch() {
    // Owner report 2026-11-16: one Firefox wheel click panned ~3× the
    // Windows/Chrome notch. Firefox emits DOM_DELTA_LINE with 3 lines per
    // notch and eframe passes the delta as-is — on the WEB backend the Line
    // delta is divided by 3, so one click is 1 notch again (native wheels
    // keep the pass-through; see wheel_notches_are_unit_aware).
    assert_eq!(
        wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, 3.0), true),
        Vec2::new(0.0, 1.0)
    );
    assert_eq!(
        wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, -3.0), true),
        Vec2::new(0.0, -1.0)
    );
    // Fractional lines stay proportional on the web too. A browser that
    // emitted 1 line per notch would pan 1/3 of a notch (5 pt) — an accepted
    // INTERIM mismatch, retuned from --features diagnostics wheel logs.
    assert!((wheel_notches(MouseWheelUnit::Line, Vec2::new(0.0, 1.5), true).y - 0.5).abs() < 1e-6);
}

#[test]
fn an_accelerator_modifier_suppresses_the_key_pan() {
    // Owner bug report 2026-11-20: Ctrl+S / Ctrl+Shift+S also panned on S
    // — with an accelerator held, the WASD keys belong to the hotkey
    // layer, not the viewport.
    assert!(pan_suppressed_by_modifiers(
        eframe::egui::Modifiers::COMMAND
    ));
    assert!(pan_suppressed_by_modifiers(
        eframe::egui::Modifiers::COMMAND | eframe::egui::Modifiers::SHIFT
    ));
    assert!(pan_suppressed_by_modifiers(eframe::egui::Modifiers::CTRL));
    assert!(pan_suppressed_by_modifiers(eframe::egui::Modifiers::ALT));
    assert!(!pan_suppressed_by_modifiers(eframe::egui::Modifiers::NONE));
    assert!(
        !pan_suppressed_by_modifiers(eframe::egui::Modifiers::SHIFT),
        "Shift alone is no accelerator — the pan stays"
    );
}

// The combo-latch topic lives in its own file (AGENTS.md "Tests" rule 7:
// split past the ~300-line guard — move the tests, not the visibility).
mod latch;
// The multi-touch pan + the restored drag-delta pan (same split rule).
mod drag_pan;
mod touch_gesture;
mod touch_gesture_pass;
