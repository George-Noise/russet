//! The per-layer history tree (owner request 2026-11-13, evolution of the
//! 2026-11-12 visual mirror): the REAL history, one independent tree per
//! layer, shaped like the git-style DAG from the owner's design concept.
//! Every node carries a full PNG snapshot of its layer's canvas AFTER the
//! action (lossless fast-compressed PNG held in RAM, see
//! [`crate::layers::encode_snapshot`]) — "one canvas per node" without the
//! action-replay machinery, which arrives later. The pixel buffers and the
//! PNG helpers live in `layers.rs` (a [`LayerTree`] is owned by
//! [`crate::layers::Layer`]); this module is the DAG model. The window that
//! renders it lives in `app/tree_window.rs`.
//!
//! Behavior (owner's answers 2026-11-13):
//! - Every drawing gesture is a node: pencil, brush, eraser strokes and the
//!   Line/Shape/Text commits. The old 20-snapshot stack is gone.
//! - [`ActionKind::Start`] is each layer's permanent root node: clicking it
//!   restores the layer's initial canvas, and undo past the first stroke
//!   lands on it.
//! - A stroke made while the head is not at a tip forks the tree git-style:
//!   the node gets a second child, the old tail stays in the graph.
//! - Branches within one layer NEVER merge (owner request 2026-11-13:
//!   paint is not a text file — two timelines cannot converge), so every
//!   branch is a plain rail hanging off the head path and the renderer
//!   needs no join/convergence handling.
//! - `remove_last` (the windows' debug "-") refuses to remove the Start
//!   root; it reattaches the removed node's children to its parent. The
//!   reattached children keep their own snapshots, so nothing is lost.
//! - Redo follows the "last left branch" rule: the child the head was left
//!   from by the last undo, else the first-created child.
//!
//! The branch/lane LAYOUT (rows, rail spans, lane assignment) lives in the
//! child module [`layout`] and is re-exported here, so every consumer keeps
//! the stable `history_tree::` paths.

use std::collections::HashMap;

use eframe::egui::Color32;

mod layout;

pub use layout::{BranchSpan, Row, TreeLayout};

use layout::layout_layer;

/// Identity of one action node. Ids come from ONE document-wide counter
/// (owned by [`crate::layers::LayerStack`]), so they stay unique across all
/// layers and a scroll anchor from any window can always be resolved
/// unambiguously.
pub type NodeId = u64;

/// Kind of the recorded action. One completed gesture = one node; the kind
/// is what the tree windows and the MCP dump display next to the dot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionKind {
    /// The layer's permanent root: its snapshot is the initial canvas.
    Start,
    Pencil,
    Brush,
    Eraser,
    /// Straight-line drags and curve polylines (one tool, two gestures).
    Line,
    /// Rectangle / ellipse commits.
    Shape,
    /// Text commits.
    Text,
}

impl ActionKind {
    /// Lowercase label shown in the graph row ("pencil 7") and dumped by
    /// the MCP tools.
    pub fn label(self) -> &'static str {
        match self {
            ActionKind::Start => "start",
            ActionKind::Pencil => "pencil",
            ActionKind::Brush => "brush",
            ActionKind::Eraser => "eraser",
            ActionKind::Line => "line",
            ActionKind::Shape => "shape",
            ActionKind::Text => "text",
        }
    }
}

/// One node of the history tree: what was done, with which color, and the
/// full PNG snapshot of the layer's canvas AFTER the action.
#[derive(Clone, Debug)]
pub struct TreeNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub kind: ActionKind,
    /// The ONE color the gesture painted with (owner request 2026-11-13:
    /// no primary/secondary distinction in the tree). Unused by Start.
    pub color: Color32,
    /// Lossless PNG of the layer's full canvas after this action, encoded
    /// with the fast settings of [`crate::layers::encode_snapshot`]. Node
    /// PNGs may carry a different size than the current canvas after a
    /// Resize Canvas — restoring one crops/pads it back (owner decision
    /// 2026-11-13: "Resive keeps the nodes").
    pub snapshot: Vec<u8>,
}

/// What mutated a layer since the previous generation. The windows use it
/// to decide whether their scroll anchor needs re-pinning: only changes
/// that SHIFT earlier rows (removals, prepends) move an anchored row,
/// plain appends in the top-down order do not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Change {
    /// A node was appended (a completed gesture).
    Added,
    /// The head moved without changing the row set (undo/redo/jump).
    Moved,
    /// The head node was removed (debug button) — rows may shift.
    Removed,
    /// The tree was rebuilt from scratch (a fresh document/layer).
    Reset,
}

/// Result of [`LayerTree::remove_last`]: the removed node plus the
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

/// One layer's history: the node DAG plus the head position. The pixel
/// buffer that the snapshots encode lives next to it in
/// [`crate::layers::Layer`].
pub struct LayerTree {
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
    /// A fresh tree with the permanent Start root carrying `root_snapshot`
    /// (the layer's initial canvas). Ids are allocated by the caller — the
    /// document-wide counter in [`crate::layers::LayerStack`].
    pub fn new(root_id: NodeId, root_snapshot: Vec<u8>) -> Self {
        Self {
            nodes: vec![TreeNode {
                id: root_id,
                parent: None,
                kind: ActionKind::Start,
                color: Color32::WHITE,
                snapshot: root_snapshot,
            }],
            index: HashMap::from([(root_id, 0)]),
            head: Some(root_id),
            redo_hint: None,
            generation: 0,
            last_change: Change::Reset,
        }
    }

    /// The node the canvas state currently corresponds to (never None for
    /// a live tree — the Start root always exists).
    pub fn head(&self) -> Option<NodeId> {
        self.head
    }

    pub fn node(&self, id: NodeId) -> Option<&TreeNode> {
        self.index.get(&id).map(|&i| &self.nodes[i])
    }

    /// All nodes in creation order (the MCP server dumps them, tests
    /// assert on them).
    pub fn nodes(&self) -> &[TreeNode] {
        &self.nodes
    }

    /// Undo is possible while the head is not the Start root.
    pub fn can_undo(&self) -> bool {
        self.head.is_some_and(|id| {
            self.index
                .get(&id)
                .is_some_and(|&i| self.nodes[i].parent.is_some())
        })
    }

    /// Redo is possible while the head has a child to move to.
    pub fn can_redo(&self) -> bool {
        self.head
            .is_some_and(|head| self.nodes.iter().any(|node| node.parent == Some(head)))
    }

    /// The debug "-" button may remove the head while it is NOT the Start
    /// root (the root keeps the layer's initial canvas reachable).
    pub fn can_remove_last(&self) -> bool {
        self.head.is_some_and(|id| {
            self.index
                .get(&id)
                .is_some_and(|&i| self.nodes[i].parent.is_some())
        })
    }

    /// Mutation counter; windows compare it frame to frame to notice
    /// content changes.
    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn last_change(&self) -> Change {
        self.last_change
    }

    /// Graph layout of this layer (see [`TreeLayout`]).
    pub fn layout(&self) -> TreeLayout {
        layout_layer(&self.nodes, self.head, self.redo_hint)
    }

    /// One completed gesture = one node appended at the head, carrying the
    /// PNG of the post-action canvas. If the head already has a child (the
    /// user drew after an undo/jump), the new node becomes a second child
    /// — the fork is implicit in the data.
    pub fn record_action(
        &mut self,
        id: NodeId,
        kind: ActionKind,
        color: Color32,
        snapshot: Vec<u8>,
    ) {
        let parent = self.head;
        self.nodes.push(TreeNode {
            id,
            parent,
            kind,
            color,
            snapshot,
        });
        self.index.insert(id, self.nodes.len() - 1);
        self.head = Some(id);
        self.redo_hint = None;
        self.generation += 1;
        self.last_change = Change::Added;
    }

    /// Undo: the head steps back to the parent and the child it left
    /// becomes the redo hint. No-op when the head is the Start root.
    pub fn undo(&mut self) -> bool {
        let parent = self
            .head
            .and_then(|id| self.index.get(&id).and_then(|&i| self.nodes[i].parent));
        match (self.head, parent) {
            (Some(head), Some(parent)) => {
                self.redo_hint = Some(head);
                self.head = Some(parent);
                self.generation += 1;
                self.last_change = Change::Moved;
                true
            }
            _ => false,
        }
    }

    /// Redo: prefer the branch the head was left from (the redo hint),
    /// else the first-created child. No-op without a child to move to.
    pub fn redo(&mut self) -> bool {
        let Some(head) = self.head else { return false };
        let children: Vec<NodeId> = self
            .nodes
            .iter()
            .filter(|node| node.parent == Some(head))
            .map(|node| node.id)
            .collect();
        let pick = self
            .redo_hint
            .filter(|hint| children.contains(hint))
            .or_else(|| children.first().copied());
        match pick {
            Some(picked) => {
                self.head = Some(picked);
                self.redo_hint = None;
                self.generation += 1;
                self.last_change = Change::Moved;
                true
            }
            None => false,
        }
    }

    /// Direct jump (the windows' node-dot click): the head moves to `id`
    /// and the redo hint resets — the click is explicit navigation, so the
    /// "last left branch" rule restarts from here.
    pub fn jump_to(&mut self, id: NodeId) -> bool {
        if self.head == Some(id) || !self.index.contains_key(&id) {
            return false;
        }
        self.head = Some(id);
        self.redo_hint = None;
        self.generation += 1;
        self.last_change = Change::Moved;
        true
    }

    /// DEBUG hook (owner request 2026-11-12): removes the LAST action (the
    /// head) so the windows' scroll fallback can be exercised. Refused on
    /// the Start root. The removed node's children are reattached to its
    /// parent and KEEP their snapshots (full snapshots stay valid); the
    /// returned [`Removal`] tells the windows where a pinned scroll anchor
    /// should jump.
    pub fn remove_last(&mut self) -> Option<Removal> {
        let head_id = self.head?;
        let idx = *self.index.get(&head_id)?;
        // Refused on the Start root: the parent edge IS the protection.
        let parent = self.nodes[idx].parent?;
        let forward = next_forward(&self.nodes, &self.index, head_id);
        let removal = Removal {
            target: head_id,
            forward,
            backward: Some(parent),
        };

        for node in &mut self.nodes {
            if node.parent == Some(head_id) {
                node.parent = Some(parent);
            }
        }
        self.nodes.remove(idx);
        self.index.clear();
        for (i, node) in self.nodes.iter().enumerate() {
            self.index.insert(node.id, i);
        }
        if self.redo_hint == Some(head_id) {
            self.redo_hint = None;
        }
        self.head = Some(parent);
        self.generation += 1;
        self.last_change = Change::Removed;
        Some(removal)
    }
}

/// Nearest action FORWARD from `from` — its first child, else the next later
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

#[cfg(test)]
mod tests;
