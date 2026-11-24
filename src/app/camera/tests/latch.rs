//! The W/A/S/D combo-latch tests (owner bug report 2026-11-24): a letter
//! pressed under an accelerator belongs to the hotkey layer — Ctrl+S /
//! Ctrl+A must leave NO panning tail when the modifier lifts first, and a
//! fresh bare press of the same key pans again. Pure latch scans run
//! without an egui context; the two regression shapes run REAL passes
//! (`Context::run_pass`) in the app's frame order — hotkeys consume, then
//! the pan reads the raw events.

use super::super::{App, PanKeyLatch};
use eframe::egui::{self, Key, Modifiers, Vec2};

/// One Key event in the egui-winit shape: physical letter, the modifier
/// state of its own moment, an optional OS key-repeat flag.
fn key_event(key: Key, pressed: bool, repeat: bool, modifiers: Modifiers) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat,
        modifiers,
    }
}

#[test]
fn a_combo_press_latches_the_letter_until_its_release() {
    let mut latch = PanKeyLatch::default();

    assert!(!latch.blocks(Key::S), "a fresh latch blocks nothing");

    latch.update(&[key_event(Key::S, true, false, Modifiers::COMMAND)]);

    assert!(latch.blocks(Key::S), "the Ctrl+S press latches S");

    latch.update(&[key_event(Key::S, false, false, Modifiers::NONE)]);

    assert!(!latch.blocks(Key::S), "the letter's own release clears it");
}

#[test]
fn a_plain_press_never_latches() {
    let mut latch = PanKeyLatch::default();

    latch.update(&[key_event(Key::W, true, false, Modifiers::NONE)]);

    assert!(
        !latch.blocks(Key::W),
        "bare WASD is a pan press, not a combo"
    );

    latch.update(&[key_event(Key::D, true, false, Modifiers::SHIFT)]);

    assert!(
        !latch.blocks(Key::D),
        "Shift is no accelerator (the 2026-11-20 rule) — the press pans"
    );
}

#[test]
fn the_modifier_release_alone_does_not_unlatch() {
    // The owner's exact bug shape: Ctrl lifts FIRST, the letter stays
    // held — the suppression must survive the ModifiersChanged, the OS
    // repeats of the held letter included.
    let mut latch = PanKeyLatch::default();

    latch.update(&[key_event(Key::A, true, false, Modifiers::COMMAND)]);
    latch.update(&[egui::Event::ModifiersChanged(Modifiers::NONE)]);

    assert!(
        latch.blocks(Key::A),
        "only the letter's own release clears the latch"
    );

    latch.update(&[key_event(Key::A, true, true, Modifiers::NONE)]);

    assert!(
        latch.blocks(Key::A),
        "a key-repeat after the modifier lifted carries no accelerator — it must not unlatch"
    );
}

#[test]
fn each_pan_letter_latches_on_its_own() {
    let mut latch = PanKeyLatch::default();

    latch.update(&[
        key_event(Key::W, true, false, Modifiers::CTRL),
        key_event(Key::D, true, false, Modifiers::ALT),
    ]);

    assert!(latch.blocks(Key::W) && latch.blocks(Key::D));
    assert!(
        !latch.blocks(Key::A) && !latch.blocks(Key::S),
        "letters never pressed under an accelerator stay free"
    );

    latch.update(&[key_event(Key::W, false, false, Modifiers::NONE)]);

    assert!(!latch.blocks(Key::W), "W released");
    assert!(latch.blocks(Key::D), "D is still held from its combo");
}

/// Runs ONE headless frame in the app's real order — the hotkeys consume
/// their combos first, then the pan reads the input. A bare window hosts
/// `handle_key_pan` (it needs a `Ui` only for its input reads; it paints
/// nothing).
fn frame_pass(ctx: &egui::Context, input: &egui::RawInput, app: &mut App) {
    let mut output = ctx.run_pass(input.clone(), |ctx| {
        app.handle_hotkeys(ctx);
        egui::Window::new("pan-test").show(ctx, |ui| {
            app.handle_key_pan(ui);
        });
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (`clear` is the sanctioned "drop intentionally"
    // path — epaint's TexturesDelta asserts when unhandled deltas drop).
    output.textures_delta.clear();
}

/// A frame's raw input at wall-clock `time` (seconds) with `events` — the
/// advancing clock gives the pan a real per-frame `stable_dt`.
fn frame_input(time: f64, events: Vec<egui::Event>) -> egui::RawInput {
    egui::RawInput {
        time: Some(time),
        events,
        ..Default::default()
    }
}

#[test]
fn releasing_ctrl_first_keeps_the_still_held_save_letter_from_panning() {
    // Owner bug report 2026-11-24: Ctrl+S, Ctrl released FIRST, S still
    // held — the combo's tail must not pan down; a fresh bare press of S
    // must pan again.
    let mut app = App::new_headless();
    let ctx = egui::Context::default();

    frame_pass(
        &ctx,
        &frame_input(1.0, vec![egui::Event::ModifiersChanged(Modifiers::COMMAND)]),
        &mut app,
    );
    frame_pass(
        &ctx,
        &frame_input(
            2.0,
            vec![key_event(Key::S, true, false, Modifiers::COMMAND)],
        ),
        &mut app,
    );

    assert_eq!(app.pan, Vec2::ZERO, "the combo frame itself must not pan");

    frame_pass(
        &ctx,
        &frame_input(3.0, vec![egui::Event::ModifiersChanged(Modifiers::NONE)]),
        &mut app,
    );
    frame_pass(&ctx, &frame_input(4.0, vec![]), &mut app);

    assert_eq!(
        app.pan,
        Vec2::ZERO,
        "Ctrl lifted first while S stays held — no panning tail"
    );

    frame_pass(
        &ctx,
        &frame_input(5.0, vec![key_event(Key::S, false, false, Modifiers::NONE)]),
        &mut app,
    );
    frame_pass(
        &ctx,
        &frame_input(6.0, vec![key_event(Key::S, true, false, Modifiers::NONE)]),
        &mut app,
    );
    frame_pass(&ctx, &frame_input(7.0, vec![]), &mut app);

    assert!(
        app.pan.y < 0.0,
        "a fresh bare press of the same key pans again (S = view down)"
    );
}

#[test]
fn the_select_all_combo_letter_stays_pan_dead_until_its_release() {
    // Same bug for Ctrl+A: the A press belongs to Select All (the combo
    // still fires), releasing Ctrl first must not start a left pan, and
    // the bare re-press pans again.
    let mut app = App::new_headless();
    let ctx = egui::Context::default();

    frame_pass(
        &ctx,
        &frame_input(1.0, vec![egui::Event::ModifiersChanged(Modifiers::COMMAND)]),
        &mut app,
    );
    frame_pass(
        &ctx,
        &frame_input(
            2.0,
            vec![key_event(Key::A, true, false, Modifiers::COMMAND)],
        ),
        &mut app,
    );

    assert!(
        app.selection.bounds().is_some(),
        "the Ctrl+A combo itself still fires (Select All replaces the selection)"
    );

    frame_pass(
        &ctx,
        &frame_input(3.0, vec![egui::Event::ModifiersChanged(Modifiers::NONE)]),
        &mut app,
    );
    frame_pass(&ctx, &frame_input(4.0, vec![]), &mut app);

    assert_eq!(
        app.pan,
        Vec2::ZERO,
        "Ctrl lifted first while A stays held — no panning tail"
    );

    frame_pass(
        &ctx,
        &frame_input(5.0, vec![key_event(Key::A, false, false, Modifiers::NONE)]),
        &mut app,
    );
    frame_pass(
        &ctx,
        &frame_input(6.0, vec![key_event(Key::A, true, false, Modifiers::NONE)]),
        &mut app,
    );
    frame_pass(&ctx, &frame_input(7.0, vec![]), &mut app);

    assert!(
        app.pan.x > 0.0,
        "a fresh bare press of the same key pans again (A = view left)"
    );
}
