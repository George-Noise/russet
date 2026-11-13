use super::*;

use eframe::egui::Color32;

// The branch/lane assertions live in their own topic file (the ~300-line
// test guard).
mod layout;

/// A fresh tree: the Start root takes id 1, so strokes start at 2.
fn fresh_tree() -> LayerTree {
    LayerTree::new(1, Vec::new())
}

fn stroke(tree: &mut LayerTree, next_id: NodeId) -> NodeId {
    tree.record_action(next_id, ActionKind::Pencil, Color32::BLACK, Vec::new());
    next_id
}

#[test]
fn the_start_root_is_the_first_head() {
    let tree = fresh_tree();

    assert_eq!(tree.head(), Some(1));
    assert_eq!(tree.node(1).map(|node| node.kind), Some(ActionKind::Start));
    assert!(tree.nodes().len() == 1);
    assert!(!tree.can_undo(), "nothing to undo at the Start root");
    assert!(!tree.can_redo(), "no children yet");
    assert!(!tree.can_remove_last(), "the Start root is protected");
}

#[test]
fn strokes_chain_linearly_on_one_branch() {
    let mut tree = fresh_tree();
    let a = stroke(&mut tree, 2);
    let b = stroke(&mut tree, 3);
    let c = stroke(&mut tree, 4);

    let layout = tree.layout();
    assert_eq!(layout.rows.len(), 4, "Start plus three strokes");
    assert_eq!(
        layout.rows.iter().map(|row| row.id).collect::<Vec<_>>(),
        vec![1, a, b, c]
    );
    assert!(layout.rows.iter().all(|row| row.branch == 0 && !row.fork));
    assert_eq!(layout.spans.len(), 1);
    assert_eq!((layout.spans[0].start_row, layout.spans[0].end_row), (0, 3));
    assert!(layout.rows[3].is_head);
    assert_eq!(tree.node(c).map(|node| node.parent), Some(Some(b)));
}

#[test]
fn undo_moves_head_back_and_redo_returns_along_the_same_branch() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let b = stroke(&mut tree, 3);
    let c = stroke(&mut tree, 4);

    assert!(tree.undo());
    assert_eq!(tree.head(), Some(b));

    assert!(tree.redo());
    assert_eq!(tree.head(), Some(c));
}

#[test]
fn undo_past_the_first_stroke_lands_on_start_and_stops_there() {
    let mut tree = fresh_tree();
    let a = stroke(&mut tree, 2);

    assert!(tree.undo(), "undo from the first stroke reaches Start");
    assert_eq!(tree.head(), Some(1));
    assert!(!tree.undo(), "Start has no parent — undo stops");
    assert!(tree.redo(), "redo returns to the stroke");
    assert_eq!(tree.head(), Some(a));
}

#[test]
fn stroke_after_undo_forks_and_the_head_path_takes_lane_zero() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let fork_point = stroke(&mut tree, 3);
    let old_tail_tip = stroke(&mut tree, 4);

    tree.undo(); // head back on `fork_point`; its old tail becomes a side branch
    let fork = stroke(&mut tree, 5);

    let layout = tree.layout();
    // The CURRENT timeline (Start..fork) is the straight left edge; the
    // abandoned tail hangs off it on its own lane (owner 2026-11-13).
    let fork_row = layout
        .rows
        .iter()
        .find(|row| row.id == fork)
        .expect("fork row");
    assert_eq!(fork_row.branch, 0, "the head path stays on lane 0");
    assert!(!fork_row.fork, "no connector along its own rail");
    assert!(fork_row.is_head, "the head sits on the new branch");
    assert_eq!(fork_row.parent, Some(fork_point));

    // The abandoned tail's FIRST node peels off lane 0 with a connector.
    let tail_first = &layout.rows[3];
    assert_eq!(tail_first.parent, Some(fork_point));
    assert!(tail_first.fork, "the tail's first node peels off lane 0");

    let tail_tip = layout
        .rows
        .iter()
        .find(|row| row.id == old_tail_tip)
        .expect("old tail stays in the graph");
    assert_eq!(tail_tip.branch, 1, "the abandoned tail moves right");
    assert!(!tail_tip.is_head);

    // The tail's span covers only its OWN nodes, plus the parent row ref
    // for the connector (never the parent's row itself — that stub was a
    // rendering bug).
    let span1 = layout
        .spans
        .iter()
        .find(|span| span.branch == 1)
        .expect("side branch span");
    assert_eq!((span1.start_row, span1.end_row), (3, 3));
    assert_eq!(
        span1.parent_row,
        Some(2),
        "the connector starts at the parent"
    );

    let span0 = &layout.spans[0];
    assert_eq!(span0.branch, 0);
    assert_eq!((span0.start_row, span0.end_row), (0, 4), "Start..head rail");
    assert_eq!(span0.parent_row, None);
}

#[test]
fn redo_prefers_the_branch_left_by_the_last_undo() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let fork_point = stroke(&mut tree, 3);
    stroke(&mut tree, 4);

    tree.undo();
    let fork = stroke(&mut tree, 5);
    tree.undo();

    // The head left the fork branch on the way down: redo must return
    // there, not into the older tail branch.
    tree.redo();
    assert_eq!(tree.head(), Some(fork));
    assert_eq!(tree.node(fork).expect("known").parent, Some(fork_point));
}

#[test]
fn jump_moves_the_head_anywhere_and_resets_the_redo_hint() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let b = stroke(&mut tree, 3);
    stroke(&mut tree, 4);
    tree.undo();
    let _fork = stroke(&mut tree, 5);

    // Jump straight to the old tail tip (creation order), then to the
    // second stroke, then back onto the same head (no-op), then to an
    // unknown id. Only a REAL move returns true.
    assert!(tree.jump_to(4), "jump to the old tail tip");
    assert_eq!(tree.head(), Some(4));
    assert!(tree.jump_to(b), "jump back to the second stroke");
    assert_eq!(tree.head(), Some(b));
    assert!(!tree.jump_to(b), "jumping to the current head is a no-op");
    assert!(!tree.jump_to(99), "unknown ids are refused");
}

#[test]
fn remove_last_retargets_forward_to_the_first_child() {
    let mut tree = fresh_tree();
    let a = stroke(&mut tree, 2);
    let mid = stroke(&mut tree, 3);
    let old_tail = stroke(&mut tree, 4);

    tree.undo();
    stroke(&mut tree, 5); // fork from `mid`
    tree.undo(); // head back on `mid`, which now has two children

    let removal = tree.remove_last().expect("head is not the root");
    assert_eq!(removal.target, mid);
    assert_eq!(removal.backward, Some(a));
    assert_eq!(
        removal.forward,
        Some(old_tail),
        "first child in creation order"
    );
    assert_eq!(tree.head(), Some(a));
}

#[test]
fn remove_last_without_children_falls_back_to_the_parent() {
    let mut tree = fresh_tree();
    let a = stroke(&mut tree, 2);
    let b = stroke(&mut tree, 3);

    let removal = tree.remove_last().expect("head is not the root");
    assert_eq!(removal.target, b);
    assert_eq!(removal.forward, None);
    assert_eq!(removal.backward, Some(a));
    assert_eq!(tree.head(), Some(a));
    assert_eq!(tree.layout().rows.len(), 2);
}

#[test]
fn remove_last_reattaches_children_to_the_grandparent() {
    let mut tree = fresh_tree();
    let a = stroke(&mut tree, 2);
    let b = stroke(&mut tree, 3);
    let c = stroke(&mut tree, 4);

    let removal = tree.remove_last().expect("head is not the root");
    assert_eq!(removal.target, c);
    assert_eq!(tree.node(b).expect("known").parent, Some(a));
    assert_eq!(tree.head(), Some(b));
    // The reattached child keeps its own snapshot.
    assert!(tree.node(b).is_some_and(|node| node.snapshot.is_empty()));
}

#[test]
fn remove_last_is_refused_at_the_start_root() {
    let mut tree = fresh_tree();
    assert!(tree.remove_last().is_none(), "Start is permanent");
    stroke(&mut tree, 2);
    tree.undo(); // head back on Start
    assert!(tree.remove_last().is_none(), "still the Start root");
}

#[test]
fn generation_advances_on_every_mutation() {
    let mut tree = fresh_tree();
    let before = tree.generation();
    stroke(&mut tree, 2);
    stroke(&mut tree, 3);
    tree.undo();
    tree.redo();
    assert_eq!(tree.generation(), before + 4);
    assert_eq!(tree.last_change(), Change::Moved);
}

#[test]
fn nodes_carry_one_color_and_kind_without_primary_secondary() {
    let mut tree = fresh_tree();
    tree.record_action(
        2,
        ActionKind::Brush,
        Color32::from_rgb(0x12, 0x34, 0x56),
        Vec::new(),
    );
    let node = tree.node(2).expect("recorded node");
    assert_eq!(node.color, Color32::from_rgb(0x12, 0x34, 0x56));
    assert_eq!(node.kind, ActionKind::Brush);
    assert_eq!(node.kind.label(), "brush");
    assert_eq!(ActionKind::Start.label(), "start");
}
