use super::*;

use crate::app::App;
use crate::selection::Region;
use eframe::egui::pos2;

/// The alpha of the active layer's pixel (the paste/cut probes read it).
fn alpha_at(app: &App, x: u32, y: u32) -> u8 {
    app.layers
        .active()
        .pixmap()
        .pixel(x, y)
        .expect("in bounds")
        .alpha()
}

#[test]
fn pointer_moves_extract_only_move_events_in_order() {
    let events = vec![
        egui::Event::PointerMoved(Pos2::new(1.0, 2.0)),
        egui::Event::PointerGone,
        egui::Event::PointerMoved(Pos2::new(3.0, 4.0)),
        egui::Event::Text("x".into()),
    ];
    assert_eq!(
        pointer_move_positions(&events),
        vec![Pos2::new(1.0, 2.0), Pos2::new(3.0, 4.0)]
    );
}

#[test]
fn a_frame_without_moves_yields_an_empty_stream() {
    assert!(pointer_move_positions(&[egui::Event::PointerGone]).is_empty());
    assert!(pointer_move_positions(&[]).is_empty());
}

#[test]
fn the_platform_clipboard_combos_arrive_as_dedicated_events() {
    // egui-winit (clipboard_shortcuts, ON by default) converts Ctrl+C/X/V
    // into `Event::Copy` / `Event::Cut` / `Event::Paste` and swallows the
    // Key events — the ops must run off the dedicated events (owner bug
    // report 2026-11-20: the combos never fired through consume_key).
    let mut app = App::new_headless();
    if let Some(black) = tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 255) {
        let layer = app.layers.active_mut().pixmap_mut();
        let stride = layer.width() as usize;
        for y in 10..30 {
            for x in 10..30 {
                layer.pixels_mut()[y as usize * stride + x as usize] = black;
            }
        }
    }
    app.selection.replace(
        Some(Region::Rect {
            a: pos2(10.0, 10.0),
            b: pos2(30.0, 30.0),
        }),
        100,
        100,
    );

    let ctx = egui::Context::default();
    let input = egui::RawInput::default();
    let mut output = ctx.run_pass(input.clone(), |ctx| {
        ctx.input_mut(|i| i.events.push(egui::Event::Copy));
        app.handle_hotkeys(ctx);
    });
    output.textures_delta.clear();
    assert!(app.clipboard.is_some(), "Event::Copy runs the copy op");

    let mut output = ctx.run_pass(input.clone(), |ctx| {
        ctx.input_mut(|i| i.events.push(egui::Event::Cut));
        app.handle_hotkeys(ctx);
    });
    output.textures_delta.clear();
    assert_eq!(alpha_at(&app, 15, 15), 0, "Event::Cut erased the square");

    let mut output = ctx.run_pass(input, |ctx| {
        ctx.input_mut(|i| i.events.push(egui::Event::Paste("os text".into())));
        app.handle_hotkeys(ctx);
    });
    output.textures_delta.clear();
    assert!(
        app.layers.active().pixmap().data().iter().any(|&b| b != 0),
        "Event::Paste stamped the clipboard back"
    );
}

#[test]
fn a_raw_ctrl_c_key_event_does_not_run_the_copy() {
    // The conversion is the ONLY channel: a bare Key event (what
    // consume_key would have matched) must not trigger the op — the
    // Key event never exists for these combos in a real egui-winit frame.
    let mut app = App::new_headless();
    let ctx = egui::Context::default();
    let mut output = ctx.run_pass(egui::RawInput::default(), |ctx| {
        ctx.input_mut(|i| {
            i.events.push(egui::Event::Key {
                key: egui::Key::C,
                physical_key: Some(egui::Key::C),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            });
        });
        app.handle_hotkeys(ctx);
    });
    output.textures_delta.clear();
    assert!(app.clipboard.is_none());
}
