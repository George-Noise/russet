//! The row-label helpers of the graph painter (owner request 2026-11-21):
//! every row drops the node id, and only the pencil row swaps its word for
//! the tool's icon. Pure functions over [`TreeNode`] — no UI.

use super::{row_icon, row_label};
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
fn only_the_pencil_row_draws_an_icon() {
    // Owner request 2026-11-21: «пока что сделаем только для pencil» —
    // every other kind stays a word row (new tools join one at a time).
    assert_eq!(row_icon(&node(ActionKind::Pencil)), Some(Tool::Pencil));
    for kind in ALL_KINDS {
        if kind == ActionKind::Pencil {
            continue;
        }
        assert_eq!(row_icon(&node(kind)), None, "{kind:?} stays a word row");
    }
}
