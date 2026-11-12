use super::*;

use eframe::egui::Color32;

fn stroke(tree: &mut HistoryTree) -> NodeId {
    tree.record_stroke(Color32::BLACK, Color32::WHITE);
    tree.head().expect("stroke appends a node")
}

#[test]
fn strokes_chain_linearly_on_one_branch() {
    let mut tree = HistoryTree::new();
    let a = stroke(&mut tree);
    let b = stroke(&mut tree);
    let c = stroke(&mut tree);

    let layout = tree.layout();
    assert_eq!(layout.rows.len(), 3);
    assert_eq!(layout.rows[0].id, a);
    assert_eq!(layout.rows[1].id, b);
    assert_eq!(layout.rows[2].id, c);
    assert!(layout.rows.iter().all(|row| row.branch == 0 && !row.fork));
    assert_eq!(layout.spans.len(), 1);
    assert_eq!((layout.spans[0].start_row, layout.spans[0].end_row), (0, 2));
    assert!(layout.rows[2].is_head);
}

#[test]
fn undo_moves_head_back_and_redo_returns_along_the_same_branch() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    let b = stroke(&mut tree);
    let c = stroke(&mut tree);

    tree.on_undo();
    assert_eq!(tree.head(), Some(b));

    tree.on_redo();
    assert_eq!(tree.head(), Some(c));
}

#[test]
fn stroke_after_undo_forks_and_keeps_the_old_tail() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    let fork_point = stroke(&mut tree);
    let old_tail = stroke(&mut tree);

    tree.on_undo();
    let fork = stroke(&mut tree);

    let layout = tree.layout();
    let fork_row = layout
        .rows
        .iter()
        .find(|row| row.id == fork)
        .expect("fork row");
    assert!(fork_row.fork);
    assert_eq!(fork_row.parent, Some(fork_point));
    assert_eq!(fork_row.branch, 1);

    let tail_row = layout
        .rows
        .iter()
        .find(|row| row.id == old_tail)
        .expect("old tail stays in the graph");
    assert_eq!(tail_row.branch, 0);
    assert!(!tail_row.is_head, "the head sits on the new branch");

    // The forked branch's rail starts at the parent's row so the curve
    // joins the parent circle.
    let span = layout
        .spans
        .iter()
        .find(|s| s.branch == 1)
        .expect("fork span");
    let parent_row = layout
        .rows
        .iter()
        .position(|row| row.id == fork_point)
        .expect("parent row");
    assert_eq!(span.start_row, parent_row);
}

#[test]
fn redo_prefers_the_branch_left_by_the_last_undo() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    let fork_point = stroke(&mut tree);
    stroke(&mut tree);

    tree.on_undo();
    let fork = stroke(&mut tree);
    tree.on_undo();

    // The head left the fork branch on the way down: redo must return
    // there, not into the older tail branch.
    tree.on_redo();
    assert_eq!(tree.head(), Some(fork));
    assert_eq!(tree.node(fork).expect("known").parent, Some(fork_point));
}

#[test]
fn undo_at_the_start_is_a_no_op() {
    let mut tree = HistoryTree::new();
    let a = stroke(&mut tree);

    tree.on_undo();
    assert_eq!(tree.head(), Some(a));

    tree.on_redo();
    assert_eq!(tree.head(), Some(a));
}

#[test]
fn remove_last_retargets_forward_to_the_first_child() {
    let mut tree = HistoryTree::new();
    let a = stroke(&mut tree);
    let mid = stroke(&mut tree);
    let old_tail = stroke(&mut tree);

    tree.on_undo();
    stroke(&mut tree); // fork from `mid`
    tree.on_undo(); // head back on `mid`, which now has two children

    let removal = tree.remove_last().expect("head exists");
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
    let mut tree = HistoryTree::new();
    let a = stroke(&mut tree);
    let b = stroke(&mut tree);

    let removal = tree.remove_last().expect("head exists");
    assert_eq!(removal.target, b);
    assert_eq!(removal.forward, None);
    assert_eq!(removal.backward, Some(a));
    assert_eq!(tree.head(), Some(a));
    assert_eq!(tree.layout().rows.len(), 1);
}

#[test]
fn remove_last_reattaches_children_to_the_grandparent() {
    let mut tree = HistoryTree::new();
    let a = stroke(&mut tree);
    let b = stroke(&mut tree);
    let c = stroke(&mut tree);

    let removal = tree.remove_last().expect("head exists");
    assert_eq!(removal.target, c);
    assert_eq!(tree.node(b).expect("known").parent, Some(a));
    assert_eq!(tree.head(), Some(b));
}

#[test]
fn remove_root_with_children_reattaches_them_as_roots() {
    let mut tree = HistoryTree::new();
    let a = stroke(&mut tree);
    let b = stroke(&mut tree);
    tree.on_undo(); // head back on the root `a`

    let removal = tree.remove_last().expect("head exists");
    assert_eq!(removal.target, a);
    // The nearest action forward from the removed root IS its child.
    assert_eq!(removal.forward, Some(b));
    assert_eq!(removal.backward, None);
    assert_eq!(tree.head(), None);
    assert_eq!(tree.layout().rows.len(), 1, "the child survives as a root");
    assert_eq!(
        tree.layout().rows[0].parent,
        None,
        "reattached child becomes a root"
    );
}

#[test]
fn layers_keep_independent_trees_with_globally_unique_ids() {
    let mut tree = HistoryTree::new();
    let layer0_node = stroke(&mut tree);

    tree.set_active_layer(1);
    assert_eq!(tree.head(), None, "each layer starts empty");
    let layer1_node = stroke(&mut tree);
    assert_ne!(layer0_node, layer1_node);

    tree.set_active_layer(0);
    assert_eq!(tree.head(), Some(layer0_node));
    assert_eq!(tree.layout().rows.len(), 1);
}

#[test]
fn reset_clears_every_layer() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    tree.set_active_layer(2);
    stroke(&mut tree);
    let generation_before = tree.generation();

    tree.reset();
    assert_eq!(tree.head(), None);
    assert_eq!(tree.layout().rows.len(), 0);
    assert!(tree.generation() > generation_before);
    assert_eq!(tree.last_change(), Change::Reset);

    tree.set_active_layer(0);
    assert_eq!(tree.layout().rows.len(), 0, "all layers were cleared");
}

#[test]
fn generation_advances_on_every_mutation() {
    let mut tree = HistoryTree::new();
    let before = tree.generation();
    stroke(&mut tree);
    stroke(&mut tree);
    tree.on_undo();
    tree.on_redo();
    assert_eq!(tree.generation(), before + 4);
    assert_eq!(tree.last_change(), Change::Moved);
}
