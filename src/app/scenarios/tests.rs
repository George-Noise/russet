//! WHOLE-APP SCENARIO SUITE (owner request 2026-11-17): the automated
//! counterpart of the owner's manual walkthrough scenarios (chat, 2026-11-17)
//! — layers + canvas-mode Resize + per-layer history trees + undo/redo/node
//! jumps in FULL runs. One scenario = one test = one continuous run, so the
//! resize/history coupling is exercised the way a user actually chains it.
//!
//! OWNER-LOCKED FILE (owner request 2026-11-17):
//! "Editing this file without the owner's permission is forbidden" — the agent may
//! only APPEND new scenarios on an explicit owner request; the existing steps
//! and expectations are changed by the owner alone. The file is exempt from
//! BOTH size guards (the tests' ~300-line guard and the 500-line file guard,
//! AGENTS.md "Tests" rule 7 + "Repo facts") and splitting it is NOT a debt —
//! `scripts/rules.sh` skips it in the size-guard loop.
//!
//! SCENARIO CONFIRMATION LEDGER (owner request 2026-11-17: "mark every
//! current scenario as not yet confirmed by me"): every scenario
//! below is the AGENT'S coding of the chat walkthrough — the owner approves
//! them ONE BY ONE, and only his word flips the per-scenario marker to
//! CONFIRMED <date>. Status as of 2026-11-17 — 0 of 8 approved, all NOT YET
//! CONFIRMED: scenario_1 (resize in a chain), scenario_2 (bottom layer owns
//! the canvas size), scenario_3 (two resizes fork), scenario_4 (node jumps
//! keep branch pixels), scenario_5 (chained resizes step by step),
//! scenario_6 (same-size bottom deletion), scenario_7 (use layer as base),
//! scenario_8 (keep canvas size + undo re-expand). Before approving, note
//! the three CODE-CORRECTED steps of the chat round (the BR crop lands the
//! dab at (100,100); scenario 5 needs THREE undos to reach 800×600;
//! scenario 3's fork hangs off the mark's node) — see the 2026-11-17
//! PROGRESS entry.
//!
//! Automation boundary (read before extending): the suite drives a REAL `App`
//! built by `App::new_headless` — no renderer and NO egui passes (`Window::
//! show` panics outside a pass). Everything a dialog does through `App`
//! methods is automated; the two CLICKS a dialog needs (the Delete-layer
//! confirmation and the "Canvas Size Source" choice) have no headless
//! equivalent, so scenarios 7/8 assert the dialog TRIGGER (the successor-size
//! condition `panels` checks) and then run the exact branch bodies the two
//! buttons call (`fit_bottom_to` + refresh / `sync_canvas_to_bottom` +
//! refresh). A stroke is stand-in by ONE committed pixel (`dot`): the
//! scenarios' subject is the resize/history coupling, not segment
//! rasterization. Scenario numbering follows the owner's chat list of
//! 2026-11-17; image-mode scenarios land here on the owner's request.

use super::App;

use crate::history_tree::{ActionKind, ResizeAnchor, ResizeFilter, ResizeMode};

use eframe::egui::Color32;

const RED: Color32 = Color32::from_rgb(255, 0, 0);
const BLUE: Color32 = Color32::from_rgb(0, 0, 255);
const GREEN: Color32 = Color32::from_rgb(0, 255, 0);
const BLACK: Color32 = Color32::from_rgb(0, 0, 0);

/// Straight (demultiplied) RGBA expectations for the COMPOSITE buffer.
const RED_PX: [u8; 4] = [255, 0, 0, 255];
const BLUE_PX: [u8; 4] = [0, 0, 255, 255];
const GREEN_PX: [u8; 4] = [0, 255, 0, 255];
const BLACK_PX: [u8; 4] = [0, 0, 0, 255];
const WHITE_PX: [u8; 4] = [255, 255, 255, 255];
const CLEAR_PX: [u8; 4] = [0, 0, 0, 0];

/// A REAL `App` with a fresh default egui `Context` — the same constructor
/// the native entry point runs, headlessly.
fn app() -> App {
    App::new_headless()
}

/// Commits ONE opaque pixel as a gesture node on the ACTIVE layer — the
/// model-level stand-in for a pointer stroke: paint, record, then RE-BLEND
/// the display buffer (the real stroke path updates the composite too —
/// `composite_dirty` — and the assertions read the COMPOSITE, not buffers).
fn dot(app: &mut App, x: u32, y: u32, color: Color32, kind: ActionKind) {
    let width = app.layers.active_mut().pixmap().width();
    let index = (y * width + x) as usize;
    app.layers.active_mut().pixmap_mut().pixels_mut()[index] =
        tiny_skia::ColorU8::from_rgba(color.r(), color.g(), color.b(), color.a()).premultiply();
    app.record_active_action(kind, color, None);
    app.refresh_composite();
}

/// An ERASER commit: the pixel fully cleared, the node kind `Eraser`.
fn erased_dot(app: &mut App, x: u32, y: u32, erased: Color32) {
    let width = app.layers.active_mut().pixmap().width();
    let index = (y * width + x) as usize;
    app.layers.active_mut().pixmap_mut().pixels_mut()[index] =
        tiny_skia::ColorU8::from_rgba(0, 0, 0, 0).premultiply();
    // The node's color mirrors the stroke color; the scenarios never read it.
    app.record_active_action(ActionKind::Eraser, erased, None);
    app.refresh_composite();
}

/// The COMPOSITE (display buffer) pixel at `(x, y)` as straight RGBA — what
/// the user sees on the canvas.
fn canvas_pixel(app: &App, x: u32, y: u32) -> [u8; 4] {
    let pixel = app
        .canvas
        .pixmap()
        .pixel(x, y)
        .expect("canvas pixel in bounds");
    let straight = pixel.demultiply();
    [
        straight.red(),
        straight.green(),
        straight.blue(),
        straight.alpha(),
    ]
}

/// A layer's own buffer size.
fn layer_size(app: &App, index: usize) -> (u32, u32) {
    let layer = app.layers.layer(index).expect("layer exists");
    (layer.pixmap().width(), layer.pixmap().height())
}

/// The tree-row summary of one node (`"canvas 800×600→1000×800 (TL)"`) —
/// the "orientation" text the owner asked to record on every resize node.
fn resize_summary(app: &App, layer: usize, node_index: usize) -> String {
    let nodes = app.layers.layer_nodes(layer).expect("layer nodes");
    nodes[node_index]
        .resize
        .expect("a resize node carries its ResizeInfo")
        .summary()
}

// ---------------------------------------------------------------------------
// Chat scenario 1 (owner walkthrough 2026-11-17): a resize node in the middle
// of ONE layer's chain — undo/redo across it moves the canvas size, the mark
// positions and the tree head together.
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_1_resize_in_a_chain_steps_back_and_forward_with_the_canvas() {
    let mut app = app();

    dot(&mut app, 10, 10, RED, ActionKind::Pencil);
    app.apply_resize(
        ResizeMode::Canvas,
        1000,
        800,
        ResizeAnchor::Center,
        ResizeFilter::Bicubic,
    );

    // The canvas grew around the CENTER: the mark shifted (+100, +100), the
    // new frame is transparent padding, the tree holds the resize node.
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 110, 110), RED_PX);
    assert_eq!(
        canvas_pixel(&app, 5, 5),
        CLEAR_PX,
        "the pad stays transparent"
    );
    let nodes = app.layers.layer_nodes(0).expect("layer 0 nodes");
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[2].kind, ActionKind::Resize);
    assert_eq!(resize_summary(&app, 0, 2), "canvas 800×600→1000×800 (C)");

    dot(&mut app, 900, 700, BLUE, ActionKind::Pencil);
    assert_eq!(canvas_pixel(&app, 900, 700), BLUE_PX);

    // One undo lands ON the resize node: the blue stroke is gone, the resize
    // itself still stands (canvas 1000×800, red at the shifted spot).
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(
        canvas_pixel(&app, 900, 700),
        CLEAR_PX,
        "the stroke was undone"
    );
    assert_eq!(canvas_pixel(&app, 110, 110), RED_PX);

    // Undo PAST the resize: the canvas shrinks back and the red mark sits at
    // its original spot — the node's native snapshot rules.
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(canvas_pixel(&app, 10, 10), RED_PX);
    assert_eq!(
        canvas_pixel(&app, 110, 110),
        WHITE_PX,
        "the restored 800×600 canvas has no shifted copy"
    );

    // Redo walks forward through the resize...
    app.redo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 110, 110), RED_PX);

    // ...and restores the blue stroke.
    app.redo();
    assert_eq!(canvas_pixel(&app, 900, 700), BLUE_PX);
}

// ---------------------------------------------------------------------------
// Chat scenario 2 (owner walkthrough 2026-11-17): two layers — a TOP-layer
// undo/jump never moves the canvas; only a step of the BOTTOM layer's history
// does (the bottom layer owns the canvas size).
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_2_only_a_bottom_layer_history_step_changes_the_canvas_size() {
    let mut app = app();

    let top = app.layers.add_layer(800, 600);
    assert_eq!(
        app.layers.active_layer(),
        top,
        "Add layer activates the new layer"
    );
    dot(&mut app, 400, 300, RED, ActionKind::Pencil);
    app.layers.set_active_layer(0);
    dot(&mut app, 100, 100, BLUE, ActionKind::Pencil);

    app.apply_resize(
        ResizeMode::Canvas,
        1000,
        800,
        ResizeAnchor::Center,
        ResizeFilter::Bicubic,
    );

    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 500, 400), RED_PX);
    assert_eq!(canvas_pixel(&app, 200, 200), BLUE_PX);
    assert_eq!(layer_size(&app, 0), (1000, 800));
    assert_eq!(layer_size(&app, 1), (1000, 800));

    // Undo on the TOP layer: its buffer returns to the native 800×600 (the
    // composite clips it at (0,0)), the canvas STAYS 1000×800.
    app.layers.set_active_layer(1);
    app.undo();
    assert_eq!(
        (app.canvas.width, app.canvas.height),
        (1000, 800),
        "a top-layer undo leaves the canvas alone"
    );
    assert_eq!(layer_size(&app, 1), (800, 600));
    assert_eq!(
        canvas_pixel(&app, 400, 300),
        RED_PX,
        "the mark is back at its native spot"
    );
    assert_eq!(canvas_pixel(&app, 200, 200), BLUE_PX);

    // Jump the top layer to its Start root: the red mark disappears
    // entirely; still no canvas change.
    let start = app.layers.layer_nodes(1).expect("layer 1 nodes")[0].id;
    app.jump_to_history_node(start);
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(
        canvas_pixel(&app, 400, 300),
        WHITE_PX,
        "the mark is gone with the jump"
    );

    // Undo on the BOTTOM layer: THE canvas follows — down to 800×600.
    app.layers.set_active_layer(0);
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(canvas_pixel(&app, 100, 100), BLUE_PX);
    assert_eq!(
        canvas_pixel(&app, 400, 300),
        WHITE_PX,
        "the top layer sits at its Start root"
    );

    // Redo on the bottom layer: the canvas grows back with it.
    app.redo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 200, 200), BLUE_PX);
}

// ---------------------------------------------------------------------------
// Chat scenario 3 (owner walkthrough 2026-11-17): two different resizes as
// two BRANCHES of one tree (undo between them forks the Start root) — the
// branch dots switch the canvas between the branches like slides.
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_3_two_resizes_fork_into_branches_and_jumps_switch_the_canvas() {
    let mut app = app();

    dot(&mut app, 400, 300, BLACK, ActionKind::Pencil);

    // First resize on the head, then undo and a SECOND resize.
    app.apply_resize(
        ResizeMode::Canvas,
        1000,
        800,
        ResizeAnchor::TopLeft,
        ResizeFilter::Bicubic,
    );
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(
        canvas_pixel(&app, 400, 300),
        BLACK_PX,
        "TopLeft keeps the mark in place"
    );

    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));

    app.apply_resize(
        ResizeMode::Canvas,
        600,
        500,
        ResizeAnchor::BottomRight,
        ResizeFilter::Bicubic,
    );
    assert_eq!((app.canvas.width, app.canvas.height), (600, 500));
    assert_eq!(
        canvas_pixel(&app, 200, 200),
        BLACK_PX,
        "BottomRight shift: 200 px cropped left, 100 px top"
    );

    // The fork hangs off the MARK's node (the head the undo landed on):
    // Start → dot → { resize(TL), resize(BR) } — each resize child keeps
    // its own readable summary, the orientation info lives on the node.
    let nodes = app.layers.layer_nodes(0).expect("layer 0 nodes");
    assert_eq!(nodes.len(), 4, "Start + the dot + two resize branches");
    let fork = nodes[1].id;
    let top_left_id = nodes[2].id;
    let bottom_right_id = nodes[3].id;
    assert_eq!(nodes[2].kind, ActionKind::Resize);
    assert_eq!(nodes[3].kind, ActionKind::Resize);
    assert_eq!(nodes[2].parent, Some(fork));
    assert_eq!(nodes[3].parent, Some(fork));
    assert_eq!(resize_summary(&app, 0, 2), "canvas 800×600→1000×800 (TL)");
    assert_eq!(resize_summary(&app, 0, 3), "canvas 800×600→600×500 (BR)");

    // Clicking the branch dots switches the canvas between the branches.
    app.jump_to_history_node(top_left_id);
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 400, 300), BLACK_PX);

    app.jump_to_history_node(bottom_right_id);
    assert_eq!((app.canvas.width, app.canvas.height), (600, 500));
    assert_eq!(canvas_pixel(&app, 200, 200), BLACK_PX);

    // Drawing on the BR branch forks from THERE; the TL branch keeps its
    // own pixels.
    dot(&mut app, 500, 450, GREEN, ActionKind::Pencil);
    assert_eq!(canvas_pixel(&app, 500, 450), GREEN_PX);

    app.jump_to_history_node(top_left_id);
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(
        canvas_pixel(&app, 500, 450),
        WHITE_PX,
        "the green mark lives on the BR branch"
    );
    assert_eq!(canvas_pixel(&app, 400, 300), BLACK_PX);
}

// ---------------------------------------------------------------------------
// Chat scenario 4 (owner walkthrough 2026-11-17): jumping between nodes of a
// ready chain `Start → dot → resize → dab` — a fork drawn after the jump
// keeps the other branch's pixels untouched.
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_4_jumping_between_nodes_keeps_each_branch_pixels() {
    let mut app = app();

    dot(&mut app, 100, 100, RED, ActionKind::Pencil);
    app.apply_resize(
        ResizeMode::Canvas,
        1000,
        800,
        ResizeAnchor::Center,
        ResizeFilter::Bicubic,
    );
    dot(&mut app, 900, 700, BLUE, ActionKind::Brush);
    let nodes = app.layers.layer_nodes(0).expect("layer 0 nodes");
    assert_eq!(nodes.len(), 4);
    let red_node = nodes[1].id;
    let resize_node = nodes[2].id;
    let dab_node = nodes[3].id;

    // Jump back to the red dot: canvas 800×600, the resize and the dab are
    // ahead of the head.
    app.jump_to_history_node(red_node);
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(canvas_pixel(&app, 100, 100), RED_PX);
    assert_eq!(
        layer_size(&app, 0),
        (800, 600),
        "the dab's 1000×800 snapshot is ahead of the head"
    );

    // Erase right there: the eraser node forks the tree at the red dot.
    erased_dot(&mut app, 100, 100, RED);
    let nodes = app.layers.layer_nodes(0).expect("layer 0 nodes");
    assert_eq!(nodes.len(), 5);
    assert_eq!(
        nodes[4].parent,
        Some(red_node),
        "the eraser forks at the red dot"
    );
    assert_eq!(nodes[4].kind, ActionKind::Eraser);
    assert_eq!(
        canvas_pixel(&app, 100, 100),
        CLEAR_PX,
        "the eraser cleared the mark"
    );

    // Jump to the resize node: canvas 1000×800, the red mark is INTACT (the
    // eraser lives on the other branch) and the dab is still ahead.
    app.jump_to_history_node(resize_node);
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(
        canvas_pixel(&app, 200, 200),
        RED_PX,
        "the eraser never touched this branch"
    );
    assert_eq!(
        canvas_pixel(&app, 900, 700),
        CLEAR_PX,
        "the dab is ahead of the head"
    );

    // Jump FORWARD to the old tail (the dab)...
    app.jump_to_history_node(dab_node);
    assert_eq!(canvas_pixel(&app, 200, 200), RED_PX);
    assert_eq!(canvas_pixel(&app, 900, 700), BLUE_PX);

    // ...and undo back onto the resize node.
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 900, 700), CLEAR_PX, "the dab is undone");
}

// ---------------------------------------------------------------------------
// Chat scenario 5 (owner walkthrough 2026-11-17): two chained resizes with a
// mark before each — undo/redo steps ONE node at a time; every step moves the
// canvas with it; both resize rows stay readable (the orientation info).
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_5_chained_resizes_undo_and_redo_one_step_at_a_time() {
    let mut app = app();

    dot(&mut app, 100, 100, RED, ActionKind::Pencil);
    app.apply_resize(
        ResizeMode::Canvas,
        1000,
        800,
        ResizeAnchor::Center,
        ResizeFilter::Bicubic,
    );
    dot(&mut app, 500, 400, BLUE, ActionKind::Brush);
    app.apply_resize(
        ResizeMode::Canvas,
        600,
        500,
        ResizeAnchor::BottomRight,
        ResizeFilter::Bicubic,
    );

    // The chain: Start → dot → resize(C) → dab → resize(BR).
    let nodes = app.layers.layer_nodes(0).expect("layer 0 nodes");
    assert_eq!(nodes.len(), 5);
    assert_eq!(resize_summary(&app, 0, 2), "canvas 800×600→1000×800 (C)");
    assert_eq!(resize_summary(&app, 0, 4), "canvas 1000×800→600×500 (BR)");

    // The BottomRight crop takes 400 px from the left and 300 from the top:
    // the red mark (now at 200,200) falls off the left edge, the dab lands
    // at (100,100).
    assert_eq!((app.canvas.width, app.canvas.height), (600, 500));
    assert_eq!(canvas_pixel(&app, 100, 100), BLUE_PX);
    assert_eq!(
        canvas_pixel(&app, 0, 100),
        WHITE_PX,
        "the red mark fell off the left edge"
    );

    // One undo: back onto the DAB — the second resize is undone, the canvas
    // is back at 1000×800 with both marks.
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 200, 200), RED_PX);
    assert_eq!(canvas_pixel(&app, 500, 400), BLUE_PX);

    // Another undo: back onto the FIRST resize — still 1000×800, but the
    // dab is not in this node's snapshot yet: the shifted WHITE canvas
    // shows through at (500,400).
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 200, 200), RED_PX);
    assert_eq!(
        canvas_pixel(&app, 500, 400),
        WHITE_PX,
        "the dab is ahead of the head"
    );

    // A third undo: back onto the red mark — 800×600.
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(canvas_pixel(&app, 100, 100), RED_PX);

    // Redo replays every step: resize → dab → second resize.
    app.redo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    app.redo();
    assert_eq!(canvas_pixel(&app, 500, 400), BLUE_PX);
    app.redo();
    assert_eq!((app.canvas.width, app.canvas.height), (600, 500));
    assert_eq!(canvas_pixel(&app, 100, 100), BLUE_PX);
    assert_eq!(
        canvas_pixel(&app, 0, 100),
        WHITE_PX,
        "the red mark stays cropped"
    );
}

// ---------------------------------------------------------------------------
// Chat scenario 6 (owner walkthrough 2026-11-17): resize on TWO layers, then
// the bottom layer is deleted — a SAME-size successor becomes the base
// silently (no dialog), the survivor keeps its whole tree and inherits the
// size duty; the deletion itself is NOT undoable.
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_6_deleting_the_bottom_layer_hands_the_size_duty_to_the_survivor() {
    let mut app = app();

    dot(&mut app, 100, 100, BLUE, ActionKind::Pencil);
    app.layers.add_layer(800, 600);
    dot(&mut app, 400, 300, RED, ActionKind::Pencil);
    app.apply_resize(
        ResizeMode::Canvas,
        1000,
        800,
        ResizeAnchor::Center,
        ResizeFilter::Bicubic,
    );

    // Same-size successor: `panels` deletes SILENTLY — the exact condition
    // it checks before opening the "Canvas Size Source" dialog.
    let successor = layer_size(&app, 1);
    assert_eq!(
        successor,
        (app.canvas.width, app.canvas.height),
        "same size → no base-choice dialog"
    );

    assert!(app.layers.remove_layer(0), "the bottom layer is removable");
    app.refresh_composite();

    assert_eq!(app.layers.count(), 1);
    assert_eq!(
        (app.canvas.width, app.canvas.height),
        (1000, 800),
        "the canvas keeps its size"
    );

    // The survivor carries its whole tree — Start, dot, resize — and its
    // head now OWNS the canvas size.
    let nodes = app.layers.layer_nodes(0).expect("survivor nodes");
    assert_eq!(nodes.len(), 3);
    assert_eq!(nodes[2].kind, ActionKind::Resize);

    app.undo();
    assert_eq!(
        (app.canvas.width, app.canvas.height),
        (800, 600),
        "the survivor's undo now moves the canvas"
    );
    assert_eq!(canvas_pixel(&app, 400, 300), RED_PX);

    app.redo();
    assert_eq!((app.canvas.width, app.canvas.height), (1000, 800));
    assert_eq!(canvas_pixel(&app, 500, 400), RED_PX);
}

// ---------------------------------------------------------------------------
// Chat scenario 7 (owner walkthrough 2026-11-17): a DIFFERENT-size successor
// opens the owner's "Canvas Size Source" dialog (the click itself stays
// owner-manual); the "Use layer as base" button resizes the canvas WITHOUT
// recording any history node, and a later undo cannot bring the mark back
// once its layer stands at the Start root.
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_7_use_layer_as_base_resizes_the_canvas_without_a_node() {
    let mut app = app();

    app.layers.add_layer(800, 600);
    app.apply_resize(
        ResizeMode::Canvas,
        400,
        300,
        ResizeAnchor::TopLeft,
        ResizeFilter::Bicubic,
    );
    assert_eq!(layer_size(&app, 0), (400, 300));
    assert_eq!(layer_size(&app, 1), (400, 300));

    // Undo on the TOP layer only: its buffer returns to the native 800×600
    // while the canvas stays 400×300 — mixed sizes are legal.
    app.undo();
    assert_eq!(layer_size(&app, 1), (800, 600));
    assert_eq!(
        (app.canvas.width, app.canvas.height),
        (400, 300),
        "a top-layer undo leaves the canvas alone"
    );

    dot(&mut app, 100, 100, RED, ActionKind::Pencil);
    // Layer 2's tree forked at Start: the undone resize on one lane, the
    // fresh mark on the other.
    let nodes = app.layers.layer_nodes(1).expect("layer 1 nodes");
    assert_eq!(nodes.len(), 3);
    assert_eq!(resize_summary(&app, 1, 1), "canvas 800×600→400×300 (TL)");
    assert_eq!(nodes[2].parent, Some(nodes[0].id));

    // Delete the bottom layer: the successor is BIGGER than the canvas —
    // exactly the condition that opens the dialog.
    let successor = layer_size(&app, 1);
    assert_ne!(
        successor,
        (app.canvas.width, app.canvas.height),
        "different size → the dialog opens"
    );
    assert!(app.layers.remove_layer(0));
    app.refresh_composite();

    // "Use layer as base" (the dialog branch body): the canvas takes the
    // layer's size — NO new history node, the layer's own pixels never
    // changed.
    app.sync_canvas_to_bottom();
    app.refresh_composite();
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(canvas_pixel(&app, 100, 100), RED_PX);
    assert_eq!(
        app.layers.layer_nodes(0).expect("survivor nodes").len(),
        3,
        "no node for the canvas resize"
    );

    // Undo on the new base: back to its Start root — the mark disappears,
    // the canvas stays 800×600 (the Start snapshot IS that size).
    app.undo();
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(
        canvas_pixel(&app, 100, 100),
        CLEAR_PX,
        "the mark is gone with the undo"
    );
}

// ---------------------------------------------------------------------------
// Chat scenario 8 (owner walkthrough 2026-11-17): the same delete with the
// "Keep canvas size" button — the new base CROPS to the canvas top-left and
// the adaptation records its OWN resize node; undoing that node steps onto
// the mark's node whose NATIVE snapshot is 800×600, so the canvas RE-EXPANDS
// with it (the surprise the owner asked about).
// ---------------------------------------------------------------------------
// OWNER STATUS: NOT YET CONFIRMED — the owner approves this scenario
// individually; only his word flips this marker to CONFIRMED <date>.
#[test]
fn scenario_8_keep_canvas_size_adapts_the_layer_and_undo_reexpands() {
    let mut app = app();

    app.layers.add_layer(800, 600);
    app.apply_resize(
        ResizeMode::Canvas,
        400,
        300,
        ResizeAnchor::TopLeft,
        ResizeFilter::Bicubic,
    );
    app.undo();
    dot(&mut app, 100, 100, RED, ActionKind::Pencil);

    let successor = layer_size(&app, 1);
    assert_ne!(
        successor,
        (app.canvas.width, app.canvas.height),
        "different size → the dialog opens"
    );
    assert!(app.layers.remove_layer(0));
    app.refresh_composite();

    // "Keep canvas size" (the dialog branch body): the new bottom layer
    // crops to 400×300 TOP-LEFT and the adaptation records its own node.
    app.layers
        .fit_bottom_to(app.canvas.width, app.canvas.height);
    app.refresh_composite();
    assert_eq!(layer_size(&app, 0), (400, 300));
    assert_eq!(
        (app.canvas.width, app.canvas.height),
        (400, 300),
        "the canvas keeps its size"
    );
    assert_eq!(
        canvas_pixel(&app, 100, 100),
        RED_PX,
        "the mark keeps its top-left spot"
    );
    assert_eq!(
        resize_summary(&app, 0, 3),
        "canvas 800×600→400×300 (TL)",
        "the adaptation is a real pixel change → its own node"
    );

    // The surprise: undo on the new base steps onto the mark's node — whose
    // NATIVE snapshot is 800×600 — and the canvas re-expands with it.
    app.undo();
    assert_eq!(layer_size(&app, 0), (800, 600));
    assert_eq!((app.canvas.width, app.canvas.height), (800, 600));
    assert_eq!(canvas_pixel(&app, 100, 100), RED_PX);

    // Redo re-crops the layer and the canvas to 400×300.
    app.redo();
    assert_eq!((app.canvas.width, app.canvas.height), (400, 300));
    assert_eq!(canvas_pixel(&app, 100, 100), RED_PX);
}
