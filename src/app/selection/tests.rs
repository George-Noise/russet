//! App-level tests of the selection ops (owner request 2026-11-20): the
//! copy → cut → paste round trip, the crop's resize-machinery coupling
//! (every layer, one crop node per layer, the display buffer), and the
//! move gesture's commit — all through the REAL build path
//! ([`App::new_headless`], the scenarios-suite constructor).

use super::*;
use crate::history_tree::ActionKind;
use crate::selection::{IRect, Region};
use eframe::egui::pos2;

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

    // Lift (the erase runs over the bounds) + drag + commit.
    app.start_move(pos2(15.0, 15.0));
    app.redraw_moved((5, 3));
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
    app.redraw_moved((5, 3));
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
