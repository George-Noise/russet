//! The visual history tree (owner request 2026-11-12, codename "history
//! tree"): a READ-ONLY mirror of pencil strokes shaped like the git-style
//! DAG from the owner's design concept, one independent tree per
//! pseudo-layer. This is the data model; the window that renders it lives
//! in `app/tree_window.rs`.
//!
//! Deliberate prototype limits (owner request 2026-11-12):
//! - Nodes carry the action KIND and the tool's primary/secondary colors
//!   only — no geometry, no pixels, nothing the real undo can replay. The
//!   real snapshot [`crate::history::History`] is untouched and unaware of
//!   this tree.
//! - The mirror follows REAL events: a completed pencil gesture appends a
//!   node, and a successful real undo/redo moves the head back/forward
//!   (unconditionally — see the caveat in `App::undo`).
//! - A stroke made after undo forks the tree git-style: the head's parent
//!   gets a second child, the old tail stays in the graph.
//! - `remove_last` exists only as a debug hook to exercise the scroll
//!   fallback; it reattaches the removed node's children to its parent.
//!
//! Pseudo-layers (owner request 2026-11-12): [`PSEUDO_LAYERS`] placeholder
//! layers with one GLOBAL active layer, each with its own tree — the
//! interface the real layer system will take over later.

use std::collections::HashMap;

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

/// One node of the visual tree: what was done and with which colors.
#[derive(Clone, Copy, Debug)]
pub struct TreeNode {
    pub id: NodeId,
    pub parent: Option<NodeId>,
    pub kind: ActionKind,
    /// Primary tool color at the moment of the stroke (upper square).
    pub primary: Color32,
    /// Secondary tool color at the moment of the stroke (lower square).
    pub secondary: Color32,
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
    /// Branch index == graph lane (column). The first child continues its
    /// parent's branch; every later child of the same parent starts a new
    /// branch — the git-style fork.
    pub branch: usize,
    /// True when this node opens a NEW branch (its parent already has an
    /// earlier child).
    pub fork: bool,
    pub is_head: bool,
}

/// Vertical extent of one branch's rail, in row indices (creation order):
/// the rail starts at the fork's parent row (or the root's own row) and
/// ends at the branch's tip.
#[derive(Clone, Copy, Debug)]
pub struct BranchSpan {
    pub branch: usize,
    pub start_row: usize,
    pub end_row: usize,
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

    /// Graph layout of the active layer (see [`TreeLayout`]).
    pub fn layout(&self) -> TreeLayout {
        let layer = &self.layers[self.active];
        layout_layer(&layer.nodes, layer.head)
    }

    /// One completed pencil gesture = one node appended at the head. If the
    /// head already has a child (the user drew after an undo), the new node
    /// becomes a second child — the fork is implicit in the data.
    pub fn record_stroke(&mut self, primary: Color32, secondary: Color32) {
        let layer = &mut self.layers[self.active];
        let id = self.next_id;
        self.next_id += 1;
        layer.nodes.push(TreeNode {
            id,
            parent: layer.head,
            kind: ActionKind::Pencil,
            primary,
            secondary,
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
fn layout_layer(nodes: &[TreeNode], head: Option<NodeId>) -> TreeLayout {
    // First child of each parent in creation order: it continues the
    // parent's branch, every later child forks.
    let mut first_child: HashMap<NodeId, NodeId> = HashMap::new();
    for node in nodes {
        if let Some(parent) = node.parent {
            first_child.entry(parent).or_insert(node.id);
        }
    }

    let mut rows = Vec::with_capacity(nodes.len());
    let mut branch_of: HashMap<NodeId, usize> = HashMap::new();
    let mut next_branch = 0usize;
    for node in nodes {
        let (branch, fork) = match node.parent {
            None => {
                let branch = next_branch;
                next_branch += 1;
                (branch, false)
            }
            Some(parent) => {
                if first_child.get(&parent) == Some(&node.id) {
                    (branch_of.get(&parent).copied().unwrap_or(0), false)
                } else {
                    let branch = next_branch;
                    next_branch += 1;
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
            None => {
                // A forked branch's rail starts at its PARENT's row so the
                // curve joins the parent circle; a root starts at itself.
                let start_row = if row.fork {
                    row.parent
                        .and_then(|parent| row_index.get(&parent).copied())
                        .unwrap_or(i)
                } else {
                    i
                };
                spans.push(BranchSpan {
                    branch: row.branch,
                    start_row,
                    end_row: i,
                });
            }
        }
    }

    TreeLayout { rows, spans }
}

#[cfg(test)]
mod tests;
