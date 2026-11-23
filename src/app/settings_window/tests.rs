//! The Settings window's headless tests (owner request 2026-11-23, SPEC
//! §3.14): the REAL `App` window code rendered through real egui passes —
//! the same trick `dialogs/tests.rs` uses, driven through `App` because
//! the dropdown writes back into the app's own state.

use eframe::egui::{self, Context, Rect, Vec2};

use super::*;
use crate::app::App;

/// The screen every test runs on (the same convention `dialogs/tests.rs`
/// uses).
const SCREEN: Vec2 = Vec2::new(1280.0, 800.0);

/// Renders the REAL window for one pass on the REAL app and joins every
/// painted text run, so the assertions see exactly what the user sees.
fn render_settings_texts(app: &mut App, ctx: &Context) -> String {
    let input = egui::RawInput {
        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, SCREEN)),
        ..egui::RawInput::default()
    };
    let mut output = ctx.run_pass(input, |ctx| {
        app.settings_window(ctx);
    });
    // No renderer here: the font-atlas deltas each pass hands back must be
    // explicitly discarded (the sanctioned path, see dialogs/tests.rs).
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
    runs.join("\n")
}

#[test]
fn the_settings_window_renders_the_setting_with_its_description() {
    let mut app = App::new_headless();
    app.settings_window_open = true;
    let ctx = Context::default();
    // egui sizes a fresh window on the frame after its first layout; run
    // three passes so the text exists (the first paints nothing).
    let mut rendered = String::new();
    for _ in 0..3 {
        rendered = render_settings_texts(&mut app, &ctx);
    }

    for promised in [
        "Mouse side buttons (Undo / Redo)",
        MouseSideMode::Ask.label(),
        MouseSideMode::Ask.description(),
        "The choice is saved between sessions.",
        "Close",
    ] {
        assert!(
            rendered.contains(promised),
            "the Settings window must render {promised:?}; painted text: {rendered:?}"
        );
    }
}

#[test]
fn the_window_stays_hidden_until_the_menu_opens_it() {
    let mut app = App::new_headless();
    let ctx = Context::default();

    let rendered = render_settings_texts(&mut app, &ctx);

    assert!(
        rendered.is_empty(),
        "no settings flag - no window: painted {rendered:?}"
    );
}

/// The Tool Log labels this app has recorded so far.
fn log_labels(app: &App) -> Vec<String> {
    app.action_log
        .items()
        .iter()
        .map(|entry| entry.label.clone())
        .collect()
}

#[test]
fn a_settled_choice_logs_itself_once() {
    let mut app = App::new_headless();

    app.set_mouse_side_mode(MouseSideMode::Never);
    assert_eq!(app.mouse_side.mode(), MouseSideMode::Never);
    assert_eq!(
        log_labels(&app),
        vec!["Mouse side buttons: off".to_owned()],
        "the pick lands in the Tool Log once"
    );

    app.set_mouse_side_mode(MouseSideMode::Never);
    assert_eq!(
        log_labels(&app).len(),
        1,
        "re-picking the same mode logs nothing"
    );
}
