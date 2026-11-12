//! The visual history tree (owner request 2026-11-12, codename "history
//! tree"): a READ-ONLY mirror of pencil strokes shaped like the git-style
//! DAG from the owner's design concept, one independent tree per
//! pseudo-layer. This is the data model; the window that renders it lives
//! in `app/tree_window.rs`.
//!
//! Deliberate prototype limits (owner request 2026-11-12):
//! - Nodes carry the action KIND and ONE stroke color only (owner request
//!   2026-11-13: no primary/secondary pair — a stroke is painted with one
//!   color, so one is displayed and stored) — no geometry, no pixels,
//!   nothing the real undo can replay. The real snapshot
//!   [`crate::history::History`] is untouched and unaware of this tree.
//! - The mirror follows REAL events: a completed pencil gesture appends a
//!   node, and a successful real undo/redo moves the head back/forward
//!   (unconditionally — see the caveat in `App::undo`).
//! - A stroke made after undo forks the tree git-style: the head's parent
//!   gets a second child, the old tail stays in the graph.
//! - Branches within one layer NEVER merge (owner request 2026-11-13:
//!   paint is not a text file — two timelines cannot converge), so every
//!   branch is a plain rail hanging off the head path and the renderer
//!   needs no join/convergence handling.
//! - `remove_last` exists only as a debug hook to exercise the scroll
//!   fallback; it reattaches the removed node's children to its parent.
//!
//! Pseudo-layers (owner request 2026-11-12): [`PSEUDO_LAYERS`] placeholder
//! layers with one GLOBAL active layer, each with its own tree — the
//! interface the real layer system will take over later.

use std::collections::{HashMap, HashSet};

use eframe::egui::Color32;

/// Identity of one visual action node. Ids come from ONE global counter, so
/// they stay unique across all pseudo-layers and a scroll anchor from any
/// window can always be resolved unambiguously.
pub type NodeId = u64;

/// Kind of the recorded action. The prototype records pencil strokes only;
/// more kinds arrive together with the real action model.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    Pencil,
}

/// Pseudo-layers prepared for the real layer system: three placeholders
/// with one global active layer, each holding its own tree. Switching the
/// active layer switches the graph every window shows.
pub const PSEUDO_LAYERS: usize = 3;
pub const LAYER_NAMES: [&str; PSEUDO_LAYERS] = ["Layer 1", "Layer 2", "Layer 3"];

/// One node of the visual tree: what was done and with which color.
#[derive(Clone, Copy, Debug)]
pub struct TreeNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub kind: ActionKind,
    /// The ONE color the gesture painted with (owner request 2026-11-13:
    /// no primary/secondary distinction in the tree).
    pub color: Color32,
}

/// What mutated a layer since the previous generation. The windows use it
/// to decide whether their scroll anchor needs re-pinning: only changes
/// that SHIFT earlier rows (removals, resets, prepends) move an anchored
/// row, plain appends in the top-down order do not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// A node was appended (a completed pencil gesture).
    Added,
    /// The head moved without changing the row set (undo/redo mirror).
    Moved,
    /// The head node was removed (debug button) — rows may shift.
    Removed,
    /// Everything was cleared (New / Open).
    Reset,
}

/// Result of [`HistoryTree::remove_last`]: the removed node plus the
/// nearest surviving actions forward and backward from its position, so
/// every window can retarget a scroll anchor pinned to the removed node
/// (owner request 2026-11-12: jump to the nearest action FORWARD).
#[derive(Clone, Copy, Debug)]
pub struct Removal {
    pub target: NodeId,
    /// First child (the next action on the forward path), else the next
    /// later sibling, else the nearest later sibling of an ancestor.
    pub forward: Option<NodeId>,
    /// The removed node's parent (the previous action), if any.
    pub backward: Option<NodeId>,
}

/// One row of the graph, in creation order. The windows reverse the list
/// for the bottom-up direction.
#[derive(Clone, Copy, Debug)]
pub struct Row {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    /// Branch index == graph lane (column). The head path owns lane 0
    /// (owner request 2026-11-13: the CURRENT timeline is the straight
    /// left edge); every side branch hangs off it to the right. A node
    /// continues its parent's branch unless it opens a new side branch.
    pub branch: usize,
    /// True when this node opens a NEW side branch and needs a connector
    /// from its parent's lane to its own.
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

struct LayerTree {
    nodes: Vec<TreeNode>,
    index: HashMap<NodeId, usize>,
    head: Option<NodeId>,
    /// The child the head was left from by the last undo (the "last left
    /// branch" rule of the design concept): a later redo prefers it.
    redo_hint: Option<NodeId>,
    generation: u64,
    last_change: Change,
}

impl LayerTree {
    fn new() -> Self {
        Self {
            nodes: Vec::new(),
            index: HashMap::new(),
            head: None,
            redo_hint: None,
            generation: 0,
            last_change: Change::Reset,
        }
    }
}

/// The visual history tree of all pseudo-layers; only the ACTIVE layer's
/// tree is exposed to the windows at a time.
pub struct HistoryTree {
    layers: Vec<LayerTree>,
    active: usize,
    next_id: NodeId,
}

impl HistoryTree {
    pub fn new() -> Self {
        Self {
            layers: (0..PSEUDO_LAYERS).map(|_| LayerTree::new()).collect(),
            active: 0,
            next_id: 1,
        }
    }

    pub fn active_layer(&self) -> usize {
        self.active
    }

    /// Clamps out-of-range indices instead of panicking — the switcher only
    /// ever passes valid ones.
    pub fn set_active_layer(&mut self, layer: usize) {
        self.active = layer.min(PSEUDO_LAYERS - 1);
    }

    /// Mutation counter of the active layer; windows compare it frame to
    /// frame to notice content changes.
    pub fn generation(&self) -> u64 {
        self.layers[self.active].generation
    }

    pub fn last_change(&self) -> Change {
        self.layers[self.active].last_change
    }

    /// The node the canvas state currently corresponds to (None = the start
    /// of the layer's history).
    pub fn head(&self) -> Option<NodeId> {
        self.layers[self.active].head
    }

    pub fn node(&self, id: NodeId) -> Option<&TreeNode> {
        let layer = &self.layers[self.active];
        layer.index.get(&id).map(|&i| &layer.nodes[i])
    }

    /// All nodes of one layer in creation order, independent of the active
    /// layer (read-only: the MCP server dumps them, tests assert on them).
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn layer_nodes(&self, layer: usize) -> Option<&[TreeNode]> {
        self.layers.get(layer).map(|l| l.nodes.as_slice())
    }

    /// Head of one layer, independent of the active layer (read-only MCP).
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn layer_head(&self, layer: usize) -> Option<NodeId> {
        self.layers.get(layer).and_then(|l| l.head)
    }

    /// Graph layout of the active layer (see [`TreeLayout`]).
    pub fn layout(&self) -> TreeLayout {
        let layer = &self.layers[self.active];
        layout_layer(&layer.nodes, layer.head)
    }

    /// One completed pencil gesture = one node appended at the head. If the
    /// head already has a child (the user drew after an undo), the new node
    /// becomes a second child — the fork is implicit in the data.
    pub fn record_stroke(&mut self, color: Color32) {
        let layer = &mut self.layers[self.active];
        let id = self.next_id;
        self.next_id += 1;
        layer.nodes.push(TreeNode {
            id,
            parent: layer.head,
            kind: ActionKind::Pencil,
            color,
        });
        layer.index.insert(id, layer.nodes.len() - 1);
        layer.head = Some(id);
        layer.redo_hint = None;
        layer.generation += 1;
        layer.last_change = Change::Added;
    }

    /// Mirror of a successful real undo: the head steps back to the parent
    /// and the child it left becomes the redo hint. No-op when the head is
    /// already at the layer's start (the real undo would have failed too).
    pub fn on_undo(&mut self) {
        let layer = &mut self.layers[self.active];
        let head = layer.head;
        let parent = head
            .and_then(|id| layer.index.get(&id).copied())
            .and_then(|idx| layer.nodes[idx].parent);
        if let (Some(head), Some(parent)) = (head, parent) {
            layer.redo_hint = Some(head);
            layer.head = Some(parent);
            layer.generation += 1;
            layer.last_change = Change::Moved;
        }
    }

    /// Mirror of a successful real redo: prefer the branch the head was left
    /// from (the redo hint), else the first-created child. No-op without a
    /// child to move to.
    pub fn on_redo(&mut self) {
        let layer = &mut self.layers[self.active];
        let Some(head) = layer.head else { return };
        let children: Vec<NodeId> = layer
            .nodes
            .iter()
            .filter(|n| n.parent == Some(head))
            .map(|n| n.id)
            .collect();
        let pick = layer
            .redo_hint
            .filter(|hint| children.contains(hint))
            .or_else(|| children.first().copied());
        if let Some(picked) = pick {
            layer.head = Some(picked);
            layer.redo_hint = None;
            layer.generation += 1;
            layer.last_change = Change::Moved;
        }
    }

    /// DEBUG hook (owner request 2026-11-12): removes the LAST action (the
    /// head) of the active layer so the windows' scroll fallback can be
    /// exercised. The removed node's children are reattached to its parent
    /// (or become roots); the returned [`Removal`] tells the windows where
    /// a pinned scroll anchor should jump. No-op on an empty layer.
    pub fn remove_last(&mut self) -> Option<Removal> {
        let head_id = self.layers[self.active].head?;
        let layer = &mut self.layers[self.active];
        let idx = *layer.index.get(&head_id)?;
        let parent = layer.nodes[idx].parent;
        let forward = next_forward(&layer.nodes, &layer.index, head_id);
        let removal = Removal {
            target: head_id,
            forward,
            backward: parent,
        };

        for node in layer.nodes.iter_mut() {
            if node.parent == Some(head_id) {
                node.parent = parent;
            }
        }
        layer.nodes.remove(idx);
        layer.index.clear();
        for (i, node) in layer.nodes.iter().enumerate() {
            layer.index.insert(node.id, i);
        }
        if layer.redo_hint == Some(head_id) {
            layer.redo_hint = None;
        }
        layer.head = parent;
        layer.generation += 1;
        layer.last_change = Change::Removed;
        Some(removal)
    }

    /// New / Open: every pseudo-layer's tree is dropped (the real history
    /// resets the same way).
    pub fn reset(&mut self) {
        for layer in &mut self.layers {
            layer.nodes.clear();
            layer.index.clear();
            layer.head = None;
            layer.redo_hint = None;
            layer.generation += 1;
            layer.last_change = Change::Reset;
        }
    }
}

impl Default for HistoryTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Nearest action FORWARD from `from`: its first child, else the next later
/// sibling, else (walking up) the nearest later sibling of the nearest
/// ancestor. Creation order is the time order, so "later" is well defined.
fn next_forward(
    nodes: &[TreeNode],
    index: &HashMap<NodeId, usize>,
    from: NodeId,
) -> Option<NodeId> {
    // Forward along this node's own path: its first child.
    if let Some(child) = nodes.iter().find(|n| n.parent == Some(from)) {
        return Some(child.id);
    }
    // Otherwise climb: at each level take the next later sibling; a level
    // without one hands the search to the parent. Climbed ancestors are
    // never re-entered through their children.
    let mut current = from;
    loop {
        let idx = *index.get(&current)?;
        let parent = nodes[idx].parent?;
        let next_sibling = nodes
            .iter()
            .filter(|n| n.parent == Some(parent) && index.get(&n.id).is_some_and(|i| *i > idx))
            .map(|n| n.id)
            .next();
        if let Some(sibling) = next_sibling {
            return Some(sibling);
        }
        current = parent;
    }
}

/// Branch/lane assignment and rail spans for one layer (see the struct
/// docs). Parents are always processed before their children (creation
/// order), so the branch of a parent is always known when needed.
///
/// Lane rule (owner request 2026-11-13): the CURRENT timeline (root..head)
/// owns lane 0 and reads as the straight left edge; side branches hang off
/// it to the right in the order they were created. Because branches never
/// merge (paint, not text), a branch is a plain rail: one lane, one span,
/// one optional connector to its fork parent.
fn layout_layer(nodes: &[TreeNode], head: Option<NodeId>) -> TreeLayout {
    let index: HashMap<NodeId, usize> = nodes
        .iter()
        .enumerate()
        .map(|(i, node)| (node.id, i))
        .collect();

    // The head path: walk from the head to the root through parents.
    let mut on_head_path: HashSet<NodeId> = HashSet::new();
    let mut cursor = head;
    while let Some(id) = cursor {
        on_head_path.insert(id);
        cursor = index.get(&id).and_then(|&i| nodes[i].parent);
    }

    // First NON-head child of each parent in creation order: on a side
    // branch it continues its parent's branch; every later child of the
    // same parent opens yet another branch.
    let mut first_side_child: HashMap<NodeId, NodeId> = HashMap::new();
    for node in nodes {
        if !on_head_path.contains(&node.id)
            && let Some(parent) = node.parent
        {
            first_side_child.entry(parent).or_insert(node.id);
        }
    }

    let mut rows = Vec::with_capacity(nodes.len());
    let mut branch_of: HashMap<NodeId, usize> = HashMap::new();
    // Lane 0 is reserved for the head path when it exists.
    let mut next_lane = if head.is_some() { 1 } else { 0 };
    for node in nodes {
        let (branch, fork) = match node.parent {
            None => {
                if on_head_path.contains(&node.id) {
                    (0, false)
                } else {
                    let branch = next_lane;
                    next_lane += 1;
                    (branch, false)
                }
            }
            Some(parent) => {
                let parent_branch = branch_of.get(&parent).copied().unwrap_or(0);
                if on_head_path.contains(&node.id) {
                    // The current timeline rides its parent's lane: the
                    // left rail stays perfectly straight.
                    (parent_branch, false)
                } else if !on_head_path.contains(&parent)
                    && first_side_child.get(&parent) == Some(&node.id)
                {
                    // First child of a side branch: continue it.
                    (parent_branch, false)
                } else {
                    // A new side branch hanging off an existing lane.
                    let branch = next_lane;
                    next_lane += 1;
                    (branch, true)
                }
            }
        };
        branch_of.insert(node.id, branch);
        rows.push(Row {
            id: node.id,
            parent: node.parent,
            branch,
            fork,
            is_head: Some(node.id) == head,
        });
    }

    let row_index: HashMap<NodeId, usize> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| (row.id, i))
        .collect();
    let mut spans: Vec<BranchSpan> = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        match spans.iter_mut().find(|span| span.branch == row.branch) {
            Some(span) => span.end_row = i,
            None => spans.push(BranchSpan {
                branch: row.branch,
                start_row: i,
                end_row: i,
                // A forked branch remembers its parent's row so the
                // renderer can draw the peel-off connector.
                parent_row: if row.fork {
                    row.parent
                        .and_then(|parent| row_index.get(&parent).copied())
                } else {
                    None
                },
            }),
        }
    }

    TreeLayout { rows, spans }
}

#[cfg(test)]
mod tests;
