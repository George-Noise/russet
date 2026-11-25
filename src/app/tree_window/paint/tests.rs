//! The row-label helpers of the graph painter (owner request 2026-11-21):
//! every row drops the node id, and the pencil/brush/eraser rows swap
//! their words for the tool's icons. Pure functions over [`TreeNode`] —
//! no UI.

use super::{row_icon, row_label, row_swatch};
use crate::history_tree::{ActionKind, CropInfo, ResizeAnchor, ResizeInfo, ResizeMode, TreeNode};
use crate::tools::Tool;
use eframe::egui::Color32;

/// A bare node of `kind`: no snapshot bytes (the helpers never touch them),
/// WHITE color (the Start root's convention), id 12 — its digits must not
/// leak into any label.
fn node(kind: ActionKind) -> TreeNode {
    TreeNode {
        id: 12,
        parent: None,
        kind,
        color: Color32::WHITE,
        color2: None,
        resize: None,
        crop: None,
        snapshot: Vec::new(),
    }
}

const ALL_KINDS: [ActionKind; 12] = [
    ActionKind::Start,
    ActionKind::Pencil,
    ActionKind::Brush,
    ActionKind::Eraser,
    ActionKind::Line,
    ActionKind::Shape,
    ActionKind::Text,
    ActionKind::Resize,
    ActionKind::Crop,
    ActionKind::Cut,
    ActionKind::Paste,
    ActionKind::Move,
];

#[test]
fn row_labels_carry_no_node_id() {
    // Owner request 2026-11-21: the digit is gone from EVERY row — the
    // reading order is the id's context and the dot's hover carries the
    // node; a bare row prints ONLY the kind word.
    for kind in ALL_KINDS {
        let label = row_label(&node(kind));
        assert_eq!(label, kind.label(), "a bare {kind:?} row prints one word");
        assert!(
            !label.contains('1') && !label.contains('2'),
            "no digit of the node id 12 may survive in '{label}'"
        );
    }
}

#[test]
fn resize_and_crop_rows_keep_their_summary() {
    let mut resize = node(ActionKind::Resize);
    resize.resize = Some(ResizeInfo {
        mode: ResizeMode::Canvas,
        from_w: 800,
        from_h: 600,
        to_w: 1000,
        to_h: 800,
        anchor: Some(ResizeAnchor::TopLeft),
        filter: None,
    });
    assert_eq!(row_label(&resize), "resize canvas 800×600→1000×800 (TL)");

    let mut crop = node(ActionKind::Crop);
    crop.crop = Some(CropInfo {
        x: 100,
        y: 50,
        from_w: 800,
        from_h: 600,
        to_w: 300,
        to_h: 200,
    });
    assert_eq!(row_label(&crop), "crop 800×600→300×200 @(100,50)");
}

#[test]
fn pencil_brush_eraser_rows_draw_an_icon() {
    // Owner request 2026-11-21: "for now we'll do it only for pencil", then
    // the same-day follow-up "like pencil, let's have brush and eraser
    // in the history tree too" — the three painting tools are icon rows, every
    // other kind stays a word row (the next tool joins on the owner's
    // word).
    assert_eq!(row_icon(&node(ActionKind::Pencil)), Some(Tool::Pencil));
    assert_eq!(row_icon(&node(ActionKind::Brush)), Some(Tool::Brush));
    assert_eq!(row_icon(&node(ActionKind::Eraser)), Some(Tool::Eraser));
    for kind in ALL_KINDS {
        if matches!(
            kind,
            ActionKind::Pencil | ActionKind::Brush | ActionKind::Eraser
        ) {
            continue;
        }
        assert_eq!(row_icon(&node(kind)), None, "{kind:?} stays a word row");
    }
}

#[test]
fn the_eraser_row_paints_no_swatch() {
    // Owner request 2026-11-21: "for some reason the eraser has a color in the
    // history tree — that must not be" — the eraser erases, it does not
    // paint. The Start root and the resize rows carry no color either
    // (the resize choices live in the label); the painting tools do.
    assert!(!row_swatch(&node(ActionKind::Eraser)));
    assert!(!row_swatch(&node(ActionKind::Start)));
    assert!(row_swatch(&node(ActionKind::Pencil)));
    assert!(row_swatch(&node(ActionKind::Brush)));
    assert!(row_swatch(&node(ActionKind::Text)));

    let mut resize = node(ActionKind::Resize);
    resize.resize = Some(ResizeInfo {
        mode: ResizeMode::Canvas,
        from_w: 800,
        from_h: 600,
        to_w: 1000,
        to_h: 800,
        anchor: None,
        filter: None,
    });
    assert!(!row_swatch(&resize));
}
