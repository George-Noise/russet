use super::*;

use eframe::egui::Color32;

fn stroke(tree: &mut HistoryTree) -> NodeId {
    tree.record_stroke(Color32::BLACK);
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
fn stroke_after_undo_forks_and_the_head_path_takes_lane_zero() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    let fork_point = stroke(&mut tree);
    let old_tail_tip = stroke(&mut tree);

    tree.on_undo(); // head back on `fork_point`; its old tail becomes a side branch
    let fork = stroke(&mut tree);

    let layout = tree.layout();
    // The CURRENT timeline (root..fork) is the straight left edge; the
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
    let tail_first = &layout.rows[2];
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
        .find(|s| s.branch == 1)
        .expect("side branch span");
    assert_eq!((span1.start_row, span1.end_row), (2, 2));
    assert_eq!(
        span1.parent_row,
        Some(1),
        "the connector starts at the parent"
    );

    let span0 = &layout.spans[0];
    assert_eq!(span0.branch, 0);
    assert_eq!((span0.start_row, span0.end_row), (0, 3), "root..head rail");
    assert_eq!(span0.parent_row, None);
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

#[test]
fn head_path_rides_lane_zero_through_side_rows() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    let b = stroke(&mut tree);
    stroke(&mut tree);
    stroke(&mut tree);
    tree.on_undo();
    tree.on_undo(); // head on `b`; c..d become a side branch
    let e = stroke(&mut tree); // the head path continues below the side rows

    let layout = tree.layout();
    let head_row = layout.rows.iter().find(|row| row.id == e).expect("head");
    assert_eq!(head_row.branch, 0, "the head path stays on the left edge");
    assert_eq!(head_row.parent, Some(b));

    // Lane 0's rail runs unbroken from the root to the head, straight
    // THROUGH the rows the side branch occupies.
    let span0 = &layout.spans[0];
    assert_eq!(span0.branch, 0);
    assert_eq!((span0.start_row, span0.end_row), (0, 4));
    assert_eq!(span0.parent_row, None);

    // The side branch peels off `b` (creation row 1) onto lane 1.
    let span1 = &layout.spans[1];
    assert_eq!(span1.branch, 1);
    assert_eq!((span1.start_row, span1.end_row), (2, 3));
    assert_eq!(span1.parent_row, Some(1));
    assert!(layout.rows[2].fork && layout.rows[2].parent == Some(b));
    assert!(!layout.rows[3].fork, "the side branch continues its rail");
    assert_eq!(layout.rows[3].branch, 1);
}

#[test]
fn second_side_branch_of_a_parent_takes_the_next_lane() {
    let mut tree = HistoryTree::new();
    stroke(&mut tree);
    stroke(&mut tree);
    stroke(&mut tree);
    tree.on_undo();
    tree.on_undo(); // head on the root; 2..3 become side branch 1
    stroke(&mut tree); // side branch 2
    tree.on_undo();
    stroke(&mut tree); // side branch 3
    tree.on_undo();
    stroke(&mut tree); // the head path rides lane 0 to the newest head

    let layout = tree.layout();
    let branches: Vec<usize> = layout.rows.iter().map(|row| row.branch).collect();
    assert_eq!(branches, vec![0, 1, 1, 2, 3, 0], "lanes in creation order");
    let forks: Vec<bool> = layout.rows.iter().map(|row| row.fork).collect();
    assert_eq!(forks, vec![false, true, false, true, true, false]);
    assert_eq!(layout.spans.len(), 4, "one rail per branch, no merges");
    // Every forked span remembers its parent row for the connector.
    assert_eq!(layout.spans[1].parent_row, Some(0));
    assert_eq!(layout.spans[2].parent_row, Some(0));
    assert_eq!(layout.spans[3].parent_row, Some(0));
}

#[test]
fn nodes_carry_one_color_without_primary_secondary() {
    let mut tree = HistoryTree::new();
    tree.record_stroke(Color32::from_rgb(0x12, 0x34, 0x56));
    let id = tree.head().expect("stroke appended");
    let node = tree.node(id).expect("known node");
    assert_eq!(node.color, Color32::from_rgb(0x12, 0x34, 0x56));
}
