//! Layout half of the history tree tests: branch/lane assignment. The
//! model tests live in the parent `tests.rs`; these assert what the
//! windows and the MCP layout dump see. Every trace includes the Start
//! root as creation row 0.

use super::*;

/// A fresh tree: the Start root takes id 1, so strokes start at 2.
fn fresh_tree() -> LayerTree {
    LayerTree::new(1, Vec::new())
}

fn stroke(tree: &mut LayerTree, next_id: NodeId) -> NodeId {
    tree.record_action(
        next_id,
        super::ActionKind::Pencil,
        Color32::BLACK,
        Vec::new(),
    );
    next_id
}

#[test]
fn head_path_rides_lane_zero_through_side_rows() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let b = stroke(&mut tree, 3);
    stroke(&mut tree, 4);
    stroke(&mut tree, 5);
    tree.undo();
    tree.undo(); // head on `b`; c..d become a side branch
    let e = stroke(&mut tree, 6); // the head path continues below the side rows

    let layout = tree.layout();
    let head_row = layout.rows.iter().find(|row| row.id == e).expect("head");
    assert_eq!(head_row.branch, 0, "the head path stays on the left edge");
    assert_eq!(head_row.parent, Some(b));

    // Lane 0's rail runs unbroken from the Start root to the head, straight
    // THROUGH the rows the side branch occupies.
    let span0 = &layout.spans[0];
    assert_eq!(span0.branch, 0);
    assert_eq!((span0.start_row, span0.end_row), (0, 5));
    assert_eq!(span0.parent_row, None);

    // The side branch peels off `b` (creation row 2) onto lane 1.
    let span1 = &layout.spans[1];
    assert_eq!(span1.branch, 1);
    assert_eq!((span1.start_row, span1.end_row), (3, 4));
    assert_eq!(span1.parent_row, Some(2));
    assert!(layout.rows[3].fork && layout.rows[3].parent == Some(b));
    assert!(!layout.rows[4].fork, "the side branch continues its rail");
    assert_eq!(layout.rows[4].branch, 1);
}

#[test]
fn side_branches_stack_by_recency_with_the_newest_on_the_innermost_lane() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    stroke(&mut tree, 3);
    stroke(&mut tree, 4);
    tree.undo();
    tree.undo(); // head on the first stroke; 3..4 become a side branch
    stroke(&mut tree, 5); // side branch 2
    tree.undo();
    stroke(&mut tree, 6); // side branch 3
    tree.undo();
    stroke(&mut tree, 7); // the head path rides lane 0 to the newest head

    let layout = tree.layout();
    let branches: Vec<usize> = layout.rows.iter().map(|row| row.branch).collect();
    // NEWEST branch first (owner request 2026-11-13): the latest fork hugs
    // the current timeline on lane 1, older ones stack to the right.
    assert_eq!(branches, vec![0, 0, 3, 3, 2, 1, 0]);
    let forks: Vec<bool> = layout.rows.iter().map(|row| row.fork).collect();
    assert_eq!(forks, vec![false, false, true, false, true, true, false]);
    assert_eq!(layout.spans.len(), 4, "one rail per branch, no merges");
    // Every forked span remembers its parent row for the connector.
    assert_eq!(layout.spans[1].parent_row, Some(1));
    assert_eq!(layout.spans[2].parent_row, Some(1));
    assert_eq!(layout.spans[3].parent_row, Some(1));
}

#[test]
fn plain_undo_keeps_the_tail_on_lane_zero() {
    // Owner request 2026-11-13: undo alone must NOT divert a branch to the
    // side — the undone tail is still the timeline redo walks back into.
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    stroke(&mut tree, 3);
    stroke(&mut tree, 4);
    tree.undo();
    tree.undo(); // head on the first stroke; the tail is NOT a branch

    let layout = tree.layout();
    assert!(layout.rows.iter().all(|row| row.branch == 0 && !row.fork));
    assert_eq!(layout.spans.len(), 1, "one straight rail, no fork");
    assert_eq!(
        (layout.spans[0].start_row, layout.spans[0].end_row),
        (0, 3),
        "the rail covers the undone tail"
    );
    assert!(layout.rows[1].is_head, "the head sits mid-rail now");

    // Redo returns along the same rail — still nothing forked.
    tree.redo();
    let layout = tree.layout();
    assert!(layout.rows.iter().all(|row| row.branch == 0));
    assert_eq!(layout.spans.len(), 1);
}

#[test]
fn drawing_after_undo_is_what_forks_the_tree() {
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let _b = stroke(&mut tree, 3);
    let c = stroke(&mut tree, 4);
    tree.undo(); // plain undo: the tail stays on the main line
    assert_eq!(tree.layout().spans.len(), 1);
    let d = stroke(&mut tree, 5); // now the user diverges from `b`

    let layout = tree.layout();
    let branches: Vec<usize> = layout.rows.iter().map(|row| row.branch).collect();
    assert_eq!(
        branches,
        vec![0, 0, 0, 1, 0],
        "only the abandoned tail forks"
    );
    let forks: Vec<bool> = layout.rows.iter().map(|row| row.fork).collect();
    assert_eq!(forks, vec![false, false, false, true, false]);
    assert!(
        layout
            .rows
            .iter()
            .find(|row| row.id == c)
            .expect("known row")
            .fork
    );
    let _ = d;
}

#[test]
fn a_late_fork_reuses_the_lane_freed_above_it() {
    // Owner request 2026-11-13: a fork far below earlier branches must not
    // land to the right of ALL of them when its vertical range is free.
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    stroke(&mut tree, 3); // b: the early branch
    tree.undo();
    stroke(&mut tree, 4); // c forks the first stroke: children {b, c}
    stroke(&mut tree, 5);
    stroke(&mut tree, 6); // main line continues below the early branch
    tree.undo();
    stroke(&mut tree, 7); // a late fork off the main line, BELOW the early branch

    let layout = tree.layout();
    let branches: Vec<usize> = layout.rows.iter().map(|row| row.branch).collect();
    // rows: start, a, b(branch), c, d, e, f(branch) — the late fork shares
    // lane 1 with the early branch: their ranges don't intersect.
    assert_eq!(branches, vec![0, 0, 1, 0, 0, 1, 0]);
    assert_eq!(layout.spans.len(), 3);
    let span1 = &layout.spans[1];
    assert_eq!(span1.branch, 1);
    assert_eq!(span1.parent_row, Some(1), "the early branch forks node a");
    let span2 = &layout.spans[2];
    assert_eq!(span2.branch, 1, "the late fork reuses lane 1");
    assert_eq!(
        span2.parent_row,
        Some(4),
        "the late fork hangs off the main line"
    );
}

#[test]
fn the_redo_chain_follows_first_created_children_through_a_forked_tail() {
    // Undoing past a tail that already contains a real fork: the main line
    // beyond the head walks the redo chain — the hint first, then always
    // the FIRST-CREATED child (what repeated redo would actually visit).
    // The newer fork at `b` hangs off lane 0 as a side branch.
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    let _b = stroke(&mut tree, 3);
    stroke(&mut tree, 4);
    stroke(&mut tree, 5);
    tree.undo();
    tree.undo(); // head on `b`
    stroke(&mut tree, 6); // e: `b`'s second child — a real fork at `b`
    tree.undo();
    tree.undo(); // head on the first stroke; the tail holds the fork at `b`

    let layout = tree.layout();
    let branches: Vec<usize> = layout.rows.iter().map(|row| row.branch).collect();
    // rows: start, a, b, c, d — the OLD path (first-created children) stays
    // on lane 0; e (the newer fork) peels off to lane 1.
    assert_eq!(branches, vec![0, 0, 0, 0, 0, 1]);
    assert!(layout.rows[5].fork);
    assert_eq!(layout.spans.len(), 2);
}

#[test]
fn a_fork_on_a_side_node_continues_then_forks_to_the_right() {
    // A side node with two children: the first side child continues the
    // branch, the second opens a NEW branch to the RIGHT of its parent's
    // lane — never back across it.
    let mut tree = fresh_tree();
    stroke(&mut tree, 2);
    stroke(&mut tree, 3);
    stroke(&mut tree, 4); // c: will become the side fork parent
    stroke(&mut tree, 5); // v: c's first child
    tree.undo(); // head back on c
    stroke(&mut tree, 6); // w: c's second child (a real fork at c)
    stroke(&mut tree, 7); // u: continues w's branch
    tree.undo(); // head back on w
    stroke(&mut tree, 8); // u2: w's second child — a fork ON a side node
    tree.undo();
    tree.undo();
    tree.undo(); // head on b; the redo chain runs c -> v

    let layout = tree.layout();
    // rows: start, a, b, c, v (the redo chain rides lane 0), w+u on their
    // branch, u2 on a branch to the RIGHT of w's branch.
    let branches: Vec<usize> = layout.rows.iter().map(|row| row.branch).collect();
    assert_eq!(branches, vec![0, 0, 0, 0, 0, 1, 1, 2]);
    let forks: Vec<bool> = layout.rows.iter().map(|row| row.fork).collect();
    assert_eq!(
        forks,
        vec![false, false, false, false, false, true, false, true]
    );
    // u2's span hangs off w (creation row 5) on lane 2 — right of lane 1.
    let span_u2 = &layout.spans[2];
    assert_eq!(span_u2.branch, 2);
    assert_eq!(span_u2.parent_row, Some(5));
}
