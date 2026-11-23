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
fn a_plain_ctrl_c_key_event_runs_the_copy() {
    // The egui-winit clipboard-shortcuts conversion is DISABLED (owner
    // decision 2026-11-20 — NativeOptions::clipboard_shortcuts = false),
    // so the native combos arrive as plain Key events and consume_key
    // must see them (this used to be impossible while the conversion was
    // ON: egui-winit swallowed the Key event after pushing Event::Copy).
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
    assert!(app.clipboard.is_none(), "nothing copied yet");

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

    assert!(app.clipboard.is_some(), "the Key combo ran the copy op");
}

#[test]
fn a_key_op_wins_over_its_dedicated_twin_in_one_pass() {
    // The web backend can carry the same gesture twice in one frame (the
    // keydown's Key event AND the browser copy event's dedicated
    // Event::Copy) — at most ONE op must run. The signal is a TRANSLUCENT
    // paste: one stamp over the transparent layer leaves the source's
    // alpha (128), a double stamp would compound it (~192).
    let mut app = App::new_headless();
    if let Some(half) = tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 128) {
        let layer = app.layers.active_mut().pixmap_mut();
        let stride = layer.width() as usize;
        for y in 10..30 {
            for x in 10..30 {
                layer.pixels_mut()[y as usize * stride + x as usize] = half;
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
    app.copy_selection();
    assert!(app.clipboard.is_some(), "the clipboard seeded");

    let ctx = egui::Context::default();
    let mut output = ctx.run_pass(egui::RawInput::default(), |ctx| {
        ctx.input_mut(|i| {
            i.events.push(egui::Event::Key {
                key: egui::Key::V,
                physical_key: Some(egui::Key::V),
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::COMMAND,
            });
            i.events.push(egui::Event::Paste("os text".into()));
        });
        app.handle_hotkeys(ctx);
    });
    output.textures_delta.clear();

    let bounds = app.selection.bounds().expect("the paste selected its rect");
    let pixel = app
        .layers
        .active()
        .pixmap()
        .pixel(bounds.x as u32 + 1, bounds.y as u32 + 1)
        .expect("in bounds");
    assert_eq!(
        pixel.red(),
        127,
        "exactly ONE paste ran despite both channels carrying the gesture (the headless layer is WHITE — the half-alpha black halves it once; a second stamp would blend it to ~190)"
    );
}

// -----------------------------------------------------------------------
// The selection-clipped freehand commit (owner request 2026-11-21): the
// gesture snapshots the mask at press (`stroke_clip`) and both paint
// paths transfer only masked pixels — the press dot and the frame's
// smoothed pieces alike.
// -----------------------------------------------------------------------

/// An app with a half-canvas selection and a brush stroke in flight at
/// (20, 20) — the state the press sets up, minus the egui plumbing.
fn stroking_app() -> (App, crate::selection::Mask) {
    let mut app = App::new_headless();
    let (w, h) = (app.canvas.width, app.canvas.height);
    app.selection.replace(
        Some(Region::Rect {
            a: pos2(0.0, 0.0),
            b: pos2(20.0, h as f32),
        }),
        w,
        h,
    );
    let mask = app.selection.build_mask(w, h).expect("non-empty selection");
    app.tools.tool = Tool::Brush;
    app.tools.size = 4.0;
    app.tools.primary = Color32::BLACK;
    app.stroke_paint = Some(app.tools.primary);
    app.stroke_trail = PencilTrail::starting_from(pencil_pixel(pos2(20.0, 20.0)));
    (app, mask)
}

#[test]
fn the_press_dot_and_smoothed_pieces_confine_to_the_selection() {
    let (mut app, mask) = stroking_app();
    app.stroke_clip = Some(mask);

    // The press dot OUTSIDE the selection: nothing changes...
    let slate = app
        .layers
        .active()
        .pixmap()
        .pixel(30, 20)
        .expect("in bounds");
    app.paint_segment(pos2(30.0, 20.0), pos2(30.0, 20.0));
    assert_eq!(
        app.layers
            .active()
            .pixmap()
            .pixel(30, 20)
            .expect("in bounds"),
        slate,
        "the press dot outside the selection must not paint"
    );

    // ...and a frame's smoothed pieces crossing the edge paint only inside.
    let pieces = [SmoothPiece::Line {
        from: pos2(6.0, 20.0),
        to: pos2(34.0, 20.0),
    }];
    app.paint_smooth_pieces(&pieces, None);
    for x in [21, 27, 33] {
        assert_eq!(
            app.layers
                .active()
                .pixmap()
                .pixel(x, 20)
                .expect("in bounds"),
            slate,
            "outside the selection must stay untouched at {x}"
        );
    }
    assert!(
        app.layers
            .active()
            .pixmap()
            .pixel(10, 20)
            .expect("in bounds")
            .alpha()
            > 0,
        "inside the selection the stroke paints"
    );
}

/// The owner's bug report 2026-11-23: with a semi-transparent brush color
/// the stroke's joint circles turned LESS transparent than the stroke body.
/// Every frame rasterizes its pieces as an independent tiny-skia stroke —
/// the new frame's start cap and the press dot sat exactly on the previous
/// frame's end cap, and premultiplied source-over stacked the alpha at
/// every overlap (α 0.5 twice ≈ 0.75). The gesture must composite the whole
/// stroke ONCE, so a joint pixel is indistinguishable from the body.
#[test]
fn translucent_brush_joints_match_the_stroke_body() {
    let mut app = App::new_headless();
    let color = Color32::from_rgba_unmultiplied(10, 20, 30, 128);
    app.tools.tool = Tool::Brush;
    app.tools.size = 8.0;
    app.tools.primary = color;
    app.stroke_button = Some(PointerButton::Primary);
    app.is_stroking = true;
    app.stroke_paint = Some(color);

    // The gesture: the press dot, then two drag frames whose pieces share
    // the midpoint (60, 50) — the second frame's start cap disc overlaps
    // the first frame's end cap disc (and the dot) exactly.
    app.paint_segment(pos2(50.0, 50.0), pos2(50.0, 50.0));
    app.paint_smooth_pieces(
        &[SmoothPiece::Line {
            from: pos2(50.0, 50.0),
            to: pos2(60.0, 50.0),
        }],
        None,
    );
    app.paint_smooth_pieces(
        &[SmoothPiece::Line {
            from: pos2(60.0, 50.0),
            to: pos2(70.0, 50.0),
        }],
        None,
    );

    let layer = app.layers.active().pixmap();
    let pixel = |x: i32, y: i32| {
        layer
            .pixel(x as u32, y as u32)
            .expect("in bounds")
            .demultiply()
    };
    let body = pixel(55, 50);
    let joint = pixel(60, 50);
    let press = pixel(50, 50);
    assert_eq!(
        (joint.red(), joint.green(), joint.blue(), joint.alpha()),
        (body.red(), body.green(), body.blue(), body.alpha()),
        "the joint circle must be indistinguishable from the stroke body"
    );
    assert_eq!(
        (press.red(), press.green(), press.blue(), press.alpha()),
        (body.red(), body.green(), body.blue(), body.alpha()),
        "the press dot must not darken under the first frame's cap"
    );
}

#[test]
fn a_stroke_without_a_selection_paints_everywhere() {
    let (mut app, _) = stroking_app();
    // No selection: `stroke_clip` stays None — the direct fast path.
    app.paint_segment(pos2(30.0, 20.0), pos2(30.0, 20.0));
    assert!(
        app.layers
            .active()
            .pixmap()
            .pixel(30, 20)
            .expect("in bounds")
            .alpha()
            > 0,
        "without a selection the press dot paints anywhere"
    );
}
