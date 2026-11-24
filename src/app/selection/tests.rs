//! App-level tests of the selection ops (owner request 2026-11-20): the
//! copy → cut → paste round trip, the crop's resize-machinery coupling
//! (every layer, one crop node per layer, the display buffer), and the
//! move gesture's commit — all through the REAL build path
//! ([`App::new_headless`], the scenarios-suite constructor).

use super::*;
use crate::history_tree::ActionKind;
use crate::selection::{IRect, Region};
use eframe::egui::{Rect, pos2};

/// Fills a rect of the app's active layer with a premultiplied color
/// (the direct pixel write the raster helpers would produce).
fn fill_layer_rect(app: &mut App, rect: IRect, pixel: tiny_skia::PremultipliedColorU8) {
    let layer = app.layers.active_mut().pixmap_mut();
    let stride = layer.width() as usize;
    for y in rect.y..rect.y + rect.h as i64 {
        for x in rect.x..rect.x + rect.w as i64 {
            layer.pixels_mut()[y as usize * stride + x as usize] = pixel;
        }
    }
}

fn black() -> Option<tiny_skia::PremultipliedColorU8> {
    tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 255)
}

fn half_black() -> Option<tiny_skia::PremultipliedColorU8> {
    tiny_skia::PremultipliedColorU8::from_rgba(0, 0, 0, 128)
}

/// A headless app whose active layer carries a black 20×20 square at
/// (10, 10) on a transparent 100×100 canvas, selected as a rect region.
fn app_with_square() -> App {
    let mut app = App::new_headless();
    if let Some(black) = black() {
        fill_layer_rect(
            &mut app,
            IRect {
                x: 10,
                y: 10,
                w: 20,
                h: 20,
            },
            black,
        );
    }
    app.selection.replace(
        Some(Region::Rect {
            a: pos2(10.0, 10.0),
            b: pos2(30.0, 30.0),
        }),
        100,
        100,
    );
    app
}

fn head_kind(app: &App) -> ActionKind {
    let head = app.layers.head().expect("a recorded head");
    app.layers.node(head).expect("the head node").kind
}

fn alpha_at(app: &App, x: u32, y: u32) -> u8 {
    app.layers
        .active()
        .pixmap()
        .pixel(x, y)
        .expect("in bounds")
        .alpha()
}

#[test]
fn copy_cut_paste_round_trip_moves_the_selected_pixels() {
    let mut app = app_with_square();

    app.copy_selection();
    assert!(app.clipboard.is_some(), "the clipboard holds the extract");

    app.cut_selection();
    assert_eq!(alpha_at(&app, 15, 15), 0, "the cut erased the square");
    assert_eq!(head_kind(&app), ActionKind::Cut, "ONE cut node");
    assert!(
        app.selection.bounds().is_some(),
        "the selection survives the cut"
    );

    app.paste_clipboard();
    let bounds = app.selection.bounds().expect("the paste selects its rect");
    assert_eq!(head_kind(&app), ActionKind::Paste, "ONE paste node");
    let px = (bounds.x as u32).min(99);
    let py = (bounds.y as u32).min(99);
    assert!(
        alpha_at(&app, px, py) == 255,
        "the pasted square is at the selection the paste set"
    );
}

#[test]
fn crop_to_selection_resizes_every_layer_and_records_crop_nodes() {
    let mut app = app_with_square();

    app.crop_to_selection();

    assert_eq!(
        (app.canvas.width, app.canvas.height),
        (20, 20),
        "the display buffer takes the selection bounds"
    );
    let layer = app.layers.active().pixmap();
    assert_eq!((layer.width(), layer.height()), (20, 20));
    // The square's top-left pixel was at canvas (10, 10) — now at (0, 0).
    assert_eq!(alpha_at(&app, 0, 0), 255);
    assert_eq!(alpha_at(&app, 19, 19), 255);
    assert_eq!(head_kind(&app), ActionKind::Crop, "ONE crop node");
    assert!(app.selection.is_empty(), "the crop clears the selection");
}

#[test]
fn the_move_gesture_commits_one_move_node_and_translates() {
    let mut app = app_with_square();

    // Lift (the erase runs over the bounds) + drag (the overlay carries
    // the pixels) + release commit (the layer's second and last write).
    app.start_move(pos2(15.0, 15.0));
    app.update_move((5, 3));
    app.commit_move((5, 3));

    assert_eq!(alpha_at(&app, 15, 13), 255, "the square's new top-left");
    assert_eq!(alpha_at(&app, 34, 32), 255, "the square's new bottom-right");
    assert_eq!(alpha_at(&app, 12, 12), 0, "the old spot is empty");
    assert_eq!(head_kind(&app), ActionKind::Move, "ONE move node");
    assert_eq!(
        app.selection.bounds(),
        Some(IRect {
            x: 15,
            y: 13,
            w: 20,
            h: 20
        }),
        "the selection translated with the pixels"
    );
}

#[test]
fn cancel_select_gesture_restores_the_lifted_pixels() {
    let mut app = app_with_square();

    app.start_move(pos2(15.0, 15.0));
    app.update_move((5, 3));
    app.cancel_select_gesture();

    assert_eq!(alpha_at(&app, 15, 15), 255, "the square is back");
    assert_eq!(alpha_at(&app, 29, 29), 255, "the whole square is back");
    assert_eq!(
        app.selection.bounds(),
        Some(IRect {
            x: 10,
            y: 10,
            w: 20,
            h: 20
        }),
        "the selection stays at the origin"
    );
}

#[test]
fn a_translucent_paste_blends_with_its_alpha() {
    let mut app = App::new_headless();
    if let Some(half) = half_black() {
        fill_layer_rect(
            &mut app,
            IRect {
                x: 10,
                y: 10,
                w: 20,
                h: 20,
            },
            half,
        );
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
    // A fresh TRANSPARENT canvas — the in-app clipboard survives the New
    // reset (owner request 2026-11-20), so the paste lands on clean ground
    // and the result's alpha is exactly the source's.
    app.reset_canvas(100, 100);
    assert!(app.clipboard.is_some(), "the clipboard survives the reset");

    app.paste_clipboard();

    let bounds = app.selection.bounds().expect("the paste selected");
    // The fresh canvas's layer is WHITE (owner request 2026-11-13): the
    // 128-alpha black blends over it into an OPAQUE half-gray pixel —
    // premultiplied src-over: out_rgb = 0 + 255·(1−128/255) = 127.
    let pixel = app
        .layers
        .active()
        .pixmap()
        .pixel(bounds.x as u32 + 1, bounds.y as u32 + 1)
        .expect("in bounds");
    assert_eq!(
        pixel.alpha(),
        255,
        "the blend over an opaque layer stays opaque"
    );
    assert_eq!(pixel.red(), 127, "the black half-alpha halves the white");
}

#[test]
fn the_move_drag_writes_nothing_to_the_layer_until_release() {
    // Owner redesign 2026-11-20: the dragged part must be a separate
    // element from the canvas — the layer sees exactly TWO writes, the
    // lift's erase and the release's stamp; the drag itself paints
    // nothing (the old per-frame restore/erase/stamp printed unclear
    // artifacts).
    let mut app = app_with_square();

    app.start_move(pos2(15.0, 15.0));
    assert_eq!(alpha_at(&app, 12, 12), 0, "the lift erased the square");

    app.update_move((5, 3));
    app.update_move((10, 0));
    assert_eq!(
        alpha_at(&app, 16, 14),
        0,
        "the drag painted nothing at the intermediate offset"
    );
    assert_eq!(
        alpha_at(&app, 25, 20),
        0,
        "the drag painted nothing anywhere"
    );

    app.commit_move((10, 0));
    assert_eq!(
        alpha_at(&app, 20, 10),
        255,
        "the square at the final offset"
    );
    assert_eq!(alpha_at(&app, 39, 29), 255, "the whole square stamped");
    assert_eq!(alpha_at(&app, 12, 12), 0, "the origin stays erased");
    assert_eq!(head_kind(&app), ActionKind::Move, "ONE move node");
}

#[test]
fn the_move_gesture_carries_a_live_overlay_texture() {
    let mut app = app_with_square();

    app.start_move(pos2(15.0, 15.0));
    let SelectGesture::MoveDrag {
        overlay, origin, ..
    } = &app.select_gesture
    else {
        panic!("the move gesture must carry the overlay texture");
    };
    assert_eq!(
        overlay.size(),
        [origin.w as usize, origin.h as usize],
        "the texture is the floating pixels, sized to the selection bounds"
    );

    app.commit_move((0, 0));
    assert!(
        !matches!(app.select_gesture, SelectGesture::MoveDrag { .. }),
        "the commit drops the gesture and with it the texture handle"
    );
}

#[test]
fn the_move_overlay_rect_extends_past_the_canvas_bounds() {
    // Owner redesign 2026-11-20: the dragged part is a separate element
    // from the canvas, VISIBLE PAST THE CANVAS BOUNDS until release — the
    // overlay rect keeps its full pixel size wherever the drag goes (the
    // Foreground painter clips to the whole viewport, not the canvas).
    let mut app = app_with_square();

    app.start_move(pos2(15.0, 15.0));
    app.update_move((95, 95));
    let canvas_rect = Rect::from_min_size(pos2(0.0, 0.0), egui::vec2(100.0, 100.0));
    let rect = app.move_overlay_rect(canvas_rect);

    assert_eq!(rect.left(), 105.0, "origin.x 10 + offset 95, zoom 1");
    assert_eq!(rect.top(), 105.0);
    assert!(
        rect.left() >= canvas_rect.right(),
        "the floating piece sits fully past the canvas and is still drawn"
    );
    assert_eq!(rect.width(), 20.0, "the pixel size is preserved");
}

#[test]
fn a_move_partially_off_canvas_clips_the_selection() {
    // Owner bug report 2026-11-20: drag the selection partially off the
    // canvas, then move it back toward the center — the selection must
    // STAY the clipped shape (the off-canvas pixels are gone; the
    // un-clipped region list would heal the full 20×20 shape and claim
    // pixels that no longer exist). The headless canvas is 800×600.
    let mut app = app_with_square();

    // Move 1: +785 on x — the square lands at (795..815, 10..30), 15px
    // past the right edge; the stamp keeps only the on-canvas strip.
    app.start_move(pos2(15.0, 15.0));
    app.update_move((785, 0));
    app.commit_move((785, 0));
    assert_eq!(
        app.selection.bounds(),
        Some(IRect {
            x: 795,
            y: 10,
            w: 5,
            h: 20
        }),
        "the selection shrinks to the on-canvas strip"
    );
    assert_eq!(alpha_at(&app, 799, 15), 255, "the surviving strip");

    // Move 2: back toward the center — the clipped shape must ride along,
    // NOT grow back to 20×20.
    app.start_move(pos2(797.0, 15.0));
    app.update_move((-760, 0));
    app.commit_move((-760, 0));
    assert_eq!(
        app.selection.bounds(),
        Some(IRect {
            x: 35,
            y: 10,
            w: 5,
            h: 20
        }),
        "the clipped shape, not the healed 20×20"
    );
    // The strip (BLACK pixels) landed at (35..40); the headless canvas is
    // WHITE everywhere else, so "no pixels" reads as red == 255 there.
    let strip = app
        .layers
        .active()
        .pixmap()
        .pixel(36, 15)
        .expect("in bounds");
    assert_eq!(strip.red(), 0, "the black strip at the new spot");
    let healed_corner = app
        .layers
        .active()
        .pixmap()
        .pixel(50, 15)
        .expect("in bounds");
    assert_eq!(
        healed_corner.red(),
        255,
        "the canvas beyond the strip is untouched — no healed stamp"
    );
    assert!(
        !app.selection.contains(egui::pos2(50.0, 15.0)),
        "the healed part must NOT be selectable — there are no pixels there"
    );
}

#[test]
fn a_move_fully_off_canvas_deselects() {
    let mut app = app_with_square();

    app.start_move(pos2(15.0, 15.0));
    app.update_move((900, 700));
    app.commit_move((900, 700));

    assert!(
        app.selection.is_empty(),
        "nothing of the region survives on the canvas — no selection"
    );
    assert_eq!(
        head_kind(&app),
        ActionKind::Move,
        "the move node still records"
    );
}

#[test]
fn select_all_selects_the_whole_canvas() {
    let mut app = App::new_headless();
    let (width, height) = (app.canvas.width, app.canvas.height);
    assert!(app.selection.is_empty(), "nothing selected at startup");

    app.select_all();

    let bounds = app.selection.bounds().expect("the whole canvas selected");
    assert_eq!(
        bounds,
        IRect {
            x: 0,
            y: 0,
            w: width,
            h: height
        },
        "every pixel of the current layer's canvas is inside"
    );
    let mask = app
        .selection
        .build_mask(width, height)
        .expect("the full-canvas mask builds");
    let (last_x, last_y) = (width as i64 - 1, height as i64 - 1);
    for (x, y) in [
        (0, 0),
        (last_x, 0),
        (0, last_y),
        (last_x, last_y),
        (width as i64 / 2, height as i64 / 2),
    ] {
        assert!(
            mask.get(x, y),
            "the corner/center pixel ({x},{y}) is selected"
        );
    }
    assert!(
        !app.selection.ants().is_empty(),
        "the boundary loops exist for the ants"
    );
    assert_eq!(
        app.action_log.items().last().map(|e| e.label.as_str()),
        Some("Select all"),
        "the Tool Log names the action"
    );
}

#[test]
fn select_all_replaces_the_previous_selection() {
    // The replace rule matters: with an ADD the second Select All would
    // only union an identical rect over the old ops, and a subtract cut
    // in between would keep its hole forever (the ops list never
    // resets).
    let mut app = app_with_square();
    let (width, height) = (app.canvas.width, app.canvas.height);

    app.select_all();
    let bounds = app.selection.bounds().expect("the full selection");
    assert_eq!(
        bounds,
        IRect {
            x: 0,
            y: 0,
            w: width,
            h: height
        },
        "the old square is gone — the whole canvas is the selection"
    );

    app.selection.subtract(
        Region::Rect {
            a: pos2(width as f32 / 2.0, height as f32 / 2.0),
            b: pos2(width as f32 / 2.0 + 20.0, height as f32 / 2.0 + 20.0),
        },
        width,
        height,
    );
    app.select_all();
    assert!(
        app.selection
            .contains(pos2(width as f32 / 2.0 + 10.0, height as f32 / 2.0 + 10.0)),
        "a fresh Select All heals the subtract hole — a REPLACE, not an add"
    );
}
