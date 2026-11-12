//! Branch/lane assignment for the visual history tree: turns one layer's
//! flat node list into the [`TreeLayout`] that the windows
//! ([`crate::app::tree_window`]) and the MCP server consume — rows in
//! creation order plus one rail span per branch.
//!
//! This is the LAYOUT half of [`crate::history_tree`]; the node/history
//! model (recording, undo/redo mirroring, removal) stays in the parent
//! module. The entry point is pure — it takes the layer's nodes, head and
//! redo hint and touches no state — so it is trivially testable from
//! `history_tree/tests.rs` through `HistoryTree::layout()`.
//!
//! The types here are re-exported from [`crate::history_tree`], so every
//! consumer keeps using the stable `history_tree::` paths.

use std::collections::{HashMap, HashSet};

use super::{NodeId, TreeNode};

/// One row of the graph, in creation order. The windows reverse the list
/// for the bottom-up direction.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub id: NodeId,
    /// The node's parent; read by the MCP layout dump and the tests.
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub parent: Option<NodeId>,
    /// Branch index == graph lane (column). The MAIN LINE (root..head plus
    /// the redo chain beyond the head) owns lane 0 — the straight left
    /// edge (owner request 2026-11-13: a plain undo must NOT push the
    /// undone tail sideways; only a real fork does). Side branches share
    /// the lowest free lane, newest branch placed first, so newer branches
    /// hug the current timeline and a late fork reuses a lane freed above
    /// it instead of creeping right (owner request 2026-11-13).
    pub branch: usize,
    /// True when this node opens a NEW side branch and needs a connector
    /// from its parent's lane to its own. Read by the MCP layout dump and
    /// the tests; the renderer draws connectors from the SPANS.
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fork: bool,
    pub is_head: bool,
}

/// Vertical extent of one branch's rail, in row indices (creation order):
/// the rail spans the branch's own nodes; a forked branch also knows its
/// parent's row so the renderer can build the peel-off connector. The
/// parent row is NOT part of the rail (owner request 2026-11-13: the old
/// per-row stub through the parent's line was a rendering bug).
#[derive(Clone, Copy, Debug)]
pub struct BranchSpan {
    pub branch: usize,
    pub start_row: usize,
    pub end_row: usize,
    /// Creation row of the fork's parent (forked branches only).
    pub parent_row: Option<usize>,
}

/// Graph layout of one layer: rows in creation order plus one rail span
/// per branch. Pure data — the window paints curves and rings from it.
#[derive(Clone, Debug, Default)]
pub struct TreeLayout {
    pub rows: Vec<Row>,
    pub spans: Vec<BranchSpan>,
}

/// Branch/lane assignment and rail spans for one layer (see the struct
/// docs). Parents are always processed before their children (creation
/// order), so a branch's parent branch is always known when needed.
///
/// Lane rules (owner requests 2026-11-13):
/// - The MAIN LINE — root..head plus the redo chain beyond the head, i.e.
///   exactly what repeated redo would visit — owns lane 0, the straight
///   left edge. A PLAIN UNDO forks nothing: while the user has not drawn
///   past the head, the undone tail is still the current timeline and
///   stays on lane 0.
/// - A branch opened from the main line (or a detached root) takes the
///   LOWEST free lane, branches placed NEWEST first: newer branches hug
///   the current timeline (less visual noise) and a late fork reuses a
///   lane freed above it instead of creeping right. Two branches conflict
///   when their vertical ranges (fork parent's row..= branch tip row)
///   intersect; overlapping horizontal CONNECTORS are fine.
/// - A branch opened from a side branch always lands to the RIGHT of its
///   parent branch's lane (the peel-off connector never runs backwards),
///   on the lowest free lane above that bound.
pub(super) fn layout_layer(
    nodes: &[TreeNode],
    head: Option<NodeId>,
    redo_hint: Option<NodeId>,
) -> TreeLayout {
    let index: HashMap<NodeId, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, i))
        .collect();

    // Children in creation order (the order `on_redo` picks from).
    let mut children: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
    for node in nodes {
        if let Some(parent) = node.parent {
            children.entry(parent).or_default().push(node.id);
        }
    }

    // The MAIN LINE: the head path (root..head) ...
    let mut main_line: HashSet<NodeId> = HashSet::new();
    let mut cursor = head;
    while let Some(id) = cursor {
        main_line.insert(id);
        cursor = index.get(&id).and_then(|&i| nodes[i].parent);
    }
    // ... plus the redo chain beyond the head: the redo hint first (the
    // child the last undo left, validated like `on_redo`), then always the
    // FIRST-CREATED child — the walk strictly advances in creation order,
    // so it terminates.
    let mut prefer_hint = redo_hint;
    cursor = head;
    while let Some(id) = cursor {
        let siblings = children.get(&id).map(Vec::as_slice).unwrap_or(&[]);
        let next = prefer_hint
            .take()
            .filter(|hint| siblings.contains(hint))
            .or_else(|| siblings.first().copied());
        let Some(next) = next else { break };
        main_line.insert(next);
        cursor = Some(next);
    }

    // First side child of every node (creation order): on a side branch it
    // continues its parent's branch; every later side child of the same
    // parent opens yet another branch.
    let mut first_side_child: HashMap<NodeId, NodeId> = HashMap::new();
    for node in nodes {
        if !main_line.contains(&node.id)
            && let Some(parent) = node.parent
        {
            first_side_child.entry(parent).or_insert(node.id);
        }
    }

    // Group the nodes into branches. Index 0 = the main line; side
    // branches are discovered in creation order (parents before children).
    struct Branch {
        /// Graph lane (column); assigned below.
        lane: usize,
        /// Opens with a peel-off connector from `parent` (a detached root
        /// has none).
        fork: bool,
        parent: Option<NodeId>,
        nodes: Vec<NodeId>,
    }
    let mut branches: Vec<Branch> = vec![Branch {
        lane: 0,
        fork: false,
        parent: None,
        nodes: Vec::new(),
    }];
    let mut branch_of: HashMap<NodeId, usize> = HashMap::new();
    for node in nodes {
        if main_line.contains(&node.id) {
            branch_of.insert(node.id, 0);
            branches[0].nodes.push(node.id);
            continue;
        }
        // A side node whose parent is a side node AND whose parent has no
        // earlier side child continues that branch; everything else opens
        // a new one.
        let continues = match node.parent {
            Some(parent)
                if !main_line.contains(&parent)
                    && first_side_child.get(&parent) == Some(&node.id) =>
            {
                branch_of.get(&parent).copied()
            }
            _ => None,
        };
        let branch_index = match continues {
            Some(existing) => existing,
            None => {
                branches.push(Branch {
                    lane: 0,
                    fork: node.parent.is_some(),
                    parent: node.parent,
                    nodes: Vec::new(),
                });
                branches.len() - 1
            }
        };
        branches[branch_index].nodes.push(node.id);
        branch_of.insert(node.id, branch_index);
    }

    // Vertical range of a branch in creation rows: from its fork parent's
    // row (where the connector starts) to its tip's row.
    let range_of = |branch: &Branch| -> (usize, usize) {
        let start = branch
            .parent
            .map_or(index[&branch.nodes[0]], |parent| index[&parent]);
        // Unreachable fallback: every side branch holds at least one node.
        let end = branch.nodes.last().map_or(start, |last| index[last]);
        (start, end)
    };
    // Lane occupancy per lane; lane 0 is occupied by the main rail's
    // extent (completely free when the main line is empty — detached
    // roots may then sit on the left edge, like before this rule).
    let main_extent = match (branches[0].nodes.first(), branches[0].nodes.last()) {
        (Some(first), Some(last)) => vec![(index[first], index[last])],
        _ => Vec::new(),
    };
    let mut lane_ranges: Vec<Vec<(usize, usize)>> = vec![main_extent];

    // Placement order: round A — branches forked from the main line (and
    // detached roots), NEWEST first; round B — branches forked off side
    // branches, in discovery order (parents before children, so the
    // parent's lane is final when the child is placed).
    let mut order: Vec<usize> = (1..branches.len())
        .filter(|&i| {
            branches[i]
                .parent
                .is_none_or(|parent| main_line.contains(&parent))
        })
        .collect();
    order.sort_by_key(|&i| std::cmp::Reverse(index[&branches[i].nodes[0]]));
    order.extend((1..branches.len()).filter(|&i| {
        branches[i]
            .parent
            .is_some_and(|parent| !main_line.contains(&parent))
    }));

    let fork_lane_floor = if head.is_some() { 1 } else { 0 };
    for i in order {
        let (start, end) = range_of(&branches[i]);
        let floor = match branches[i]
            .parent
            .filter(|parent| !main_line.contains(parent))
            .and_then(|parent| branch_of.get(&parent).copied())
        {
            Some(parent_branch) => branches[parent_branch].lane + 1,
            None => fork_lane_floor,
        };
        let mut lane = floor;
        while lane_ranges
            .get(lane)
            .is_some_and(|ranges| ranges.iter().any(|&(s, e)| start <= e && s <= end))
        {
            lane += 1;
        }
        if lane >= lane_ranges.len() {
            lane_ranges.push(Vec::new());
        }
        lane_ranges[lane].push((start, end));
        branches[i].lane = lane;
    }

    let mut rows = Vec::with_capacity(nodes.len());
    for node in nodes {
        let branch = &branches[branch_of[&node.id]];
        rows.push(Row {
            id: node.id,
            parent: node.parent,
            branch: branch.lane,
            fork: branch.fork && branch.nodes.first() == Some(&node.id),
            is_head: Some(node.id) == head,
        });
    }

    let mut spans = Vec::with_capacity(branches.len());
    for branch in &branches {
        // The main line may be empty (no head) — an empty branch has no
        // rail to span.
        let Some(last) = branch.nodes.last() else {
            continue;
        };
        spans.push(BranchSpan {
            branch: branch.lane,
            start_row: index[&branch.nodes[0]],
            end_row: index[last],
            parent_row: branch.parent.map(|parent| index[&parent]),
        });
    }

    TreeLayout { rows, spans }
}
