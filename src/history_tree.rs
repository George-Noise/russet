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
//!
//! The branch/lane LAYOUT (rows, rail spans, lane assignment) lives in the
//! child module [`layout`] and is re-exported here, so every consumer keeps
//! the stable `history_tree::` paths.

use std::collections::HashMap;

use eframe::egui::Color32;

mod layout;

pub use layout::{BranchSpan, Row, TreeLayout};

use layout::layout_layer;

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
        self.layer_layout(self.active).unwrap_or_default()
    }

    /// Graph layout of ONE layer, independent of the active layer
    /// (read-only: the MCP server dumps it, tests assert on it).
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn layer_layout(&self, layer: usize) -> Option<TreeLayout> {
        self.layers
            .get(layer)
            .map(|l| layout_layer(&l.nodes, l.head, l.redo_hint))
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
