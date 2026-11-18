//! Headless-egui regression tests of the modal dialog plumbing (owner
//! report 2026-11-15): dialogs used to open at egui's default outer size
//! (340×420) — the shared button row inherited the whole remaining
//! available rect, detached from the fields and stretched the window down
//! toward the main-window edge — and they opened at egui's cascade
//! position (slightly right/below of the floating windows) instead of the
//! main window's center, and only on the very FIRST open: after a move the
//! stored area position won forever.
//!
//! These tests run real egui passes through [`egui::Context::run_pass`] —
//! no renderer, no App; the window builder, the position reset and the
//! button row are the same code the dialogs use.

use eframe::egui::{self, AreaState, Pos2, Rect, Vec2};

use super::{dialog_id, dialog_window, forget_dialog_position};

/// The screen every test runs on (bigger than the 500×400 main-window
/// minimum, so a mispositioned dialog cannot accidentally look centered).
const SCREEN: Vec2 = Vec2::new(1280.0, 800.0);

/// Shows the test dialog for one pass and returns its outer rect.
fn show_once(ctx: &egui::Context, input: &egui::RawInput, id: egui::Id) -> Rect {
    let mut rect = Rect::NOTHING;
    let mut output = ctx.run_pass(input.clone(), |ctx| {
        let response = dialog_window("Test", id, ctx)
            .show(ctx, |ui| {
                ui.label("Width");
                super::dialog_button_row(ui, "OK", "Cancel");
            })
            .expect("the dialog window shows");
        rect = response.response.rect;
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (`clear` is the sanctioned "drop intentionally"
    // path — epaint's TexturesDelta asserts when unhandled deltas drop).
    output.textures_delta.clear();
    rect
}

fn screen_center() -> Pos2 {
    Pos2::new(SCREEN.x * 0.5, SCREEN.y * 0.5)
}

fn assert_centered(rect: Rect, what: &str) {
    let center = rect.center();
    assert!(
        (center - screen_center()).length() < 5.0,
        "{what}: dialog center {center:?} vs screen center {:?}",
        screen_center()
    );
}

#[test]
fn the_dialog_window_hugs_its_content() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let id = dialog_id("test_hug");
    // egui sizes a fresh window on the frame after its first layout, so
    // run three passes and assert the settled geometry.
    let mut rect = Rect::NOTHING;
    for _ in 0..3 {
        rect = show_once(&ctx, &input, id);
    }

    // A label + a button row are ~100 pt tall; egui's 420-pt default outer
    // height (or anything near the 800-pt screen) means the window did NOT
    // shrink to its content.
    assert!(rect.height() < 200.0, "dialog height {}", rect.height());
    // The default outer width is 340 pt; anything screen-wide means a
    // first-pass available-width leak.
    assert!(rect.width() < 400.0, "dialog width {}", rect.width());
}

#[test]
fn the_dialog_window_opens_centered_on_the_main_window() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let id = dialog_id("test_center");

    // egui's cascade position (slightly right/below of the floating
    // windows) must never be used: `default_pos` beats it while the area
    // has no stored position, and the first open has none.
    let rect = show_once(&ctx, &input, id);
    assert_centered(rect, "first open");
}

#[test]
fn the_dialog_window_recenters_when_reopened_after_a_move() {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let id = dialog_id("test_epoch");

    // First open is centered...
    let rect = show_once(&ctx, &input, id);
    assert_centered(rect, "first open");

    // ...the user drags it away (the area state now carries a position)
    let moved_pos = Pos2::new(60.0, 70.0);
    // Load OUTSIDE memory_mut: both take the memory lock, nesting them
    // deadlocks the read lock.
    if let Some(mut state) = AreaState::load(&ctx, id) {
        state.set_left_top_pos(moved_pos);
        ctx.memory_mut(|mem| {
            mem.areas_mut()
                .set_state(egui::LayerId::new(egui::Order::Middle, id), state);
        });
    }

    // While it stays open (or is shown again without a close) it stays put
    // — `default_pos` is consumed only while no position is stored.
    let rect = show_once(&ctx, &input, id);
    assert!(
        (rect.center() - moved_pos - rect.size() * 0.5).length() < 5.0,
        "a moved dialog keeps its position (center {:?}, wanted {moved_pos:?}+half size)",
        rect.center()
    );

    // The close path wipes the stored position (forget_dialog_position);
    // the next open is centered AGAIN — not at the moved spot.
    forget_dialog_position(&ctx, id);
    // The wipe produces a fresh state (pivot_pos None — the condition under
    // which `Area::begin` consumes `default_pos`); showing the dialog then
    // consumes it and records the centered position again.
    let state = AreaState::load(&ctx, id).expect("the area was shown before");
    assert!(
        state.pivot_pos.is_none(),
        "pivot_pos must be None after the reset"
    );
    let rect = show_once(&ctx, &input, id);
    assert_centered(rect, "reopened after close");
}

// ---- the Resize dialog's aspect-ratio lock (owner request 2026-11-17) ----

#[test]
fn the_aspect_lock_drives_the_untouched_field() {
    // Canvas 800×600 → ratio 0.75; the user changed only the width.
    let (w, h) = super::aspect_linked(800.0, 600.0, 400.0, 600.0, 0.75);
    assert_eq!((w, h), (400.0, 300.0), "the height follows the width");

    // Only the height changed: the width follows.
    let (w, h) = super::aspect_linked(800.0, 600.0, 800.0, 200.0, 0.75);
    assert_eq!((w, h), (267.0, 200.0), "200 / 0.75 rounds to 267");

    // Both changed (or neither): the fields win as-is.
    assert_eq!(
        super::aspect_linked(800.0, 600.0, 100.0, 50.0, 0.75),
        (100.0, 50.0)
    );
    assert_eq!(
        super::aspect_linked(800.0, 600.0, 800.0, 600.0, 0.75),
        (800.0, 600.0)
    );

    // The driven value never drops below 1.
    let (w, h) = super::aspect_linked(800.0, 600.0, 0.2, 600.0, 0.75);
    assert_eq!((w, h), (0.2, 1.0));
}

// ---- the About window's content (owner request 2026-11-18) ----

/// Renders the REAL About body through the REAL window builder (no `App`)
/// and joins every painted text run, so the assertions see exactly what the
/// user sees — the same trick the plumbing tests use, applied to content.
/// THREE passes, like `the_dialog_window_hugs_its_content`: egui sizes a
/// fresh window on the frame after its first layout — the first pass paints
/// nothing (every shape culled to Noop), the content text exists from the
/// second pass on.
fn render_about_texts() -> String {
    let ctx = egui::Context::default();
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut rendered = String::new();
    for _ in 0..3 {
        let mut output = ctx.run_pass(input.clone(), |ctx| {
            dialog_window("About Russet", dialog_id("about_content"), ctx)
                .show(ctx, |ui| {
                    super::about_window_content(ui);
                })
                .expect("the About window shows");
        });
        // No renderer: drop the font-atlas deltas explicitly (see show_once).
        output.textures_delta.clear();
        // egui nests paint commands (`Shape::Vec`) — collect recursively.
        fn texts<'a>(shape: &'a egui::Shape, out: &mut Vec<&'a str>) {
            match shape {
                egui::Shape::Text(text) => out.push(text.galley.job.text.as_str()),
                egui::Shape::Vec(children) => children.iter().for_each(|c| texts(c, out)),
                _ => {}
            }
        }
        let mut runs = Vec::new();
        for clipped in &output.shapes {
            texts(&clipped.shape, &mut runs);
        }
        rendered = runs.join("\n");
    }
    rendered
}

#[test]
fn the_about_window_renders_the_spec_promise() {
    let rendered = render_about_texts();

    // SPEC §3.1 / §7.20: name, version, the language, the repository link,
    // the platform and renderer lines, the Close row.
    for promised in [
        "Russet",
        "Version",
        super::ABOUT_DESCRIPTION,
        super::ABOUT_REPO_LABEL,
        "Platform:",
        "Renderer:",
        "Close",
    ] {
        assert!(
            rendered.contains(promised),
            "the About window must render {promised:?}; painted text: {rendered:?}"
        );
    }
}

#[test]
fn the_about_hyperlink_targets_the_github_repo() {
    // SPEC §3.1 (owner request 2026-11-18): the window links the project's
    // GitHub repository — locked here so a URL edit is a conscious test
    // update, never an accident.
    assert_eq!(
        super::ABOUT_REPO_URL,
        "https://github.com/George-Noise/russet"
    );
}
