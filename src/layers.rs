//! Real layers (owner request 2026-11-13, paying functional debts F1+F4):
//! a [`LayerStack`] of pixel buffers, each with its own history tree
//! ([`LayerTree`] from [`crate::history_tree`]) whose nodes carry full PNG
//! snapshots. One canvas per node, held in RAM as lossless fast-compressed
//! PNG — the compromise between the raw-pixel prototype and the action
//! replay of the design concept (actions arrive later).
//!
//! What this module owns:
//! - [`Layer`]: name, visibility, the pixel buffer (`tiny_skia::Pixmap`,
//!   premultiplied RGBA — always the CURRENT canvas size) and its tree.
//! - [`LayerStack`]: the ordered stack (index 0 = bottom of the composite),
//!   the ACTIVE layer, the document-wide node-id counter, the layer
//!   management ops (add / remove / rename / visibility / reorder /
//!   resize) and the undo/redo/jump drivers for the active layer.
//! - Child modules (split 2026-11-16, pure code motion — the 500-line
//!   file guard): [`snapshot`] (the lossless PNG snapshot codec —
//!   `encode_snapshot` / `decode_snapshot` — and the canvas-fit policy —
//!   `fit_to_canvas`) and [`composite`] (premultiplied src-over blending
//!   of the visible layers into the display buffer, region-based so
//!   strokes stay cheap). Both are re-exported here, so every consumer
//!   keeps the stable `layers::` paths.
//!
//! Neighbours: `canvas.rs` holds the display buffer the composite writes
//! into; `app.rs` orchestrates gestures and texture sync; the DAG model
//! lives in `history_tree.rs`.

mod composite;
mod snapshot;

pub use composite::composite;
pub use snapshot::{decode_snapshot, encode_snapshot, fit_to_canvas};

use snapshot::{fit_copy, pixmap_of};

use eframe::egui::Color32;
use tiny_skia::Pixmap;

use crate::history_tree::{ActionKind, Change, LayerTree, NodeId, Removal, TreeLayout, TreeNode};

/// One layer: a pixel buffer plus its own history. `pixmap` is always the
/// CURRENT canvas size; node snapshots may differ after a Resize (they are
/// fitted back on restore).
pub struct Layer {
    name: String,
    visible: bool,
    pixmap: Pixmap,
    tree: LayerTree,
}

impl Layer {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn visible(&self) -> bool {
        self.visible
    }

    pub fn pixmap(&self) -> &Pixmap {
        &self.pixmap
    }

    pub fn pixmap_mut(&mut self) -> &mut Pixmap {
        &mut self.pixmap
    }

    /// Replaces the buffer (used by the snapshot-restore path; the caller
    /// passes a canvas-sized pixmap).
    pub fn set_pixmap(&mut self, pixmap: Pixmap) {
        self.pixmap = pixmap;
    }
}

/// The document's layer stack with one GLOBAL active layer: the history
/// windows show the active layer's tree and every gesture lands on it.
pub struct LayerStack {
    layers: Vec<Layer>,
    active: usize,
    /// Document-wide node-id counter; the Start roots take ids first.
    next_node_id: NodeId,
    /// Names new layers "Layer N" without reusing numbers after renames.
    next_layer_number: usize,
}

impl LayerStack {
    /// A new document: ONE layer ("Layer 1") filled WHITE, whose Start root
    /// holds that white canvas. The initial layer is white AGAIN as of the
    /// owner request 2026-11-13 — it restores the §3.8 white New canvas
    /// that the layers rewrite had turned transparent; every layer ADDED
    /// later is fully transparent (see [`LayerStack::add_layer`]).
    pub fn new(width: u32, height: u32) -> Self {
        let mut pixmap = pixmap_of(width, height);
        // Opaque white premultiplies to itself, so the fill is bit-exact.
        pixmap.fill(tiny_skia::Color::WHITE);
        Self::with_root(pixmap)
    }

    /// A single layer from a ready buffer (Open Image): the loaded picture
    /// becomes the layer's canvas AND its Start snapshot.
    pub fn with_root(pixmap: Pixmap) -> Self {
        let snapshot = encode_snapshot(&pixmap);
        let tree = LayerTree::new(1, snapshot);
        Self {
            layers: vec![Layer {
                name: "Layer 1".to_string(),
                visible: true,
                pixmap,
                tree,
            }],
            active: 0,
            next_node_id: 2,
            next_layer_number: 2,
        }
    }

    pub fn count(&self) -> usize {
        self.layers.len()
    }

    pub fn layer(&self, index: usize) -> Option<&Layer> {
        self.layers.get(index)
    }

    /// The whole stack, bottom first — the composite input.
    pub fn stack(&self) -> &[Layer] {
        &self.layers
    }

    pub fn active_layer(&self) -> usize {
        self.active
    }

    /// Clamps out-of-range indices instead of panicking — the switcher
    /// only ever passes valid ones.
    pub fn set_active_layer(&mut self, layer: usize) {
        self.active = layer.min(self.layers.len().saturating_sub(1));
    }

    /// The active layer (tests exercise it frame-free; the app goes
    /// through `active_mut` and the passthroughs).
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn active(&self) -> &Layer {
        &self.layers[self.active]
    }

    pub fn active_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.active]
    }

    /// Split borrow of the active layer's buffer and tree (the MCP stroke
    /// helper needs both at once).
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn active_parts(&mut self) -> (&mut Pixmap, &mut LayerTree) {
        let layer = &mut self.layers[self.active];
        (&mut layer.pixmap, &mut layer.tree)
    }

    pub fn active_layer_name(&self) -> &str {
        self.layers[self.active].name()
    }

    // ---- active-layer history passthroughs (the windows' view) ----

    pub fn head(&self) -> Option<NodeId> {
        self.layers[self.active].tree.head()
    }

    pub fn node(&self, id: NodeId) -> Option<&TreeNode> {
        self.layers[self.active].tree.node(id)
    }

    pub fn generation(&self) -> u64 {
        self.layers[self.active].tree.generation()
    }

    pub fn last_change(&self) -> Change {
        self.layers[self.active].tree.last_change()
    }

    pub fn layout(&self) -> TreeLayout {
        self.layers[self.active].tree.layout()
    }

    pub fn can_undo(&self) -> bool {
        self.layers[self.active].tree.can_undo()
    }

    pub fn can_redo(&self) -> bool {
        self.layers[self.active].tree.can_redo()
    }

    pub fn can_remove_last(&self) -> bool {
        self.layers[self.active].tree.can_remove_last()
    }

    /// Undo on the ACTIVE layer: the head steps to the parent. The caller
    /// restores the pixels from the new head's snapshot.
    pub fn undo(&mut self) -> bool {
        self.layers[self.active].tree.undo()
    }

    /// Redo on the ACTIVE layer (the "last left branch" rule). The caller
    /// restores the pixels from the new head's snapshot.
    pub fn redo(&mut self) -> bool {
        self.layers[self.active].tree.redo()
    }

    /// Direct head jump (the tree windows' node-dot click). No-op when the
    /// id is unknown or already the head.
    pub fn jump_to(&mut self, id: NodeId) -> bool {
        self.layers[self.active].tree.jump_to(id)
    }

    /// Debug removal of the active layer's head (refused at Start). The
    /// caller restores the pixels from the parent snapshot.
    pub fn remove_last(&mut self) -> Option<Removal> {
        self.layers[self.active].tree.remove_last()
    }

    /// Records one completed gesture on the ACTIVE layer: encodes the
    /// layer's post-action buffer as its PNG snapshot and appends the node
    /// at the head. `color` is the gesture's main color, `color2` the
    /// second one of a two-color gesture (a Shape commit with both the
    /// outline and the fill — owner request 2026-11-16). Returns the new
    /// node id.
    pub fn record_action(
        &mut self,
        kind: ActionKind,
        color: Color32,
        color2: Option<Color32>,
    ) -> Result<NodeId, String> {
        let snapshot = encode_snapshot(&self.layers[self.active].pixmap);
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.layers[self.active]
            .tree
            .record_action(id, kind, color, color2, snapshot);
        Ok(id)
    }

    // ---- document-wide node ids (the MCP frame-free stroke path) ----

    /// The id the next recorded node will take.
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn next_node_id(&self) -> NodeId {
        self.next_node_id
    }

    /// Consumes one id after a caller recorded a node itself
    /// (`apply_stroke` in the MCP tools).
    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn advance_next_node_id(&mut self) {
        self.next_node_id += 1;
    }

    // ---- per-layer read-only views (the MCP dump) ----

    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn layer_nodes(&self, layer: usize) -> Option<&[TreeNode]> {
        self.layers.get(layer).map(|layer| layer.tree.nodes())
    }

    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn layer_head(&self, layer: usize) -> Option<NodeId> {
        self.layers.get(layer).and_then(|layer| layer.tree.head())
    }

    #[cfg_attr(not(feature = "mcp"), allow(dead_code))]
    pub fn layer_layout(&self, layer: usize) -> Option<TreeLayout> {
        self.layers.get(layer).map(|layer| layer.tree.layout())
    }

    // ---- layer management ----

    /// Appends a transparent layer on top, names it "Layer N" and makes it
    /// active. Returns the new active index.
    pub fn add_layer(&mut self, width: u32, height: u32) -> usize {
        let pixmap = pixmap_of(width, height);
        let snapshot = encode_snapshot(&pixmap);
        let name = format!("Layer {}", self.next_layer_number);
        self.next_layer_number += 1;
        let tree = LayerTree::new(self.next_node_id, snapshot);
        self.next_node_id += 1;
        self.layers.push(Layer {
            name,
            visible: true,
            pixmap,
            tree,
        });
        self.active = self.layers.len() - 1;
        self.active
    }

    /// Removes a layer with its whole history. The LAST layer is refused
    /// (a document keeps at least one). The active index follows: below
    /// the removed one it shifts up, on it it clamps.
    pub fn remove_layer(&mut self, index: usize) -> bool {
        if self.layers.len() <= 1 || index >= self.layers.len() {
            return false;
        }
        self.layers.remove(index);
        if self.active >= self.layers.len() {
            self.active = self.layers.len() - 1;
        } else if self.active > index {
            self.active -= 1;
        }
        true
    }

    pub fn rename_layer(&mut self, index: usize, name: String) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.name = name;
        }
    }

    pub fn set_visible(&mut self, index: usize, visible: bool) {
        if let Some(layer) = self.layers.get_mut(index) {
            layer.visible = visible;
        }
    }

    /// Moves a layer `delta` positions (display "up" = +1 = later in the
    /// composite). The active index follows its layer.
    pub fn move_layer(&mut self, index: usize, delta: isize) {
        if delta == 0 || index >= self.layers.len() {
            return;
        }
        let target = index as isize + delta;
        if target < 0 || target >= self.layers.len() as isize {
            return;
        }
        let target = target as usize;
        self.layers.swap(index, target);
        if self.active == index {
            self.active = target;
        } else if self.active == target {
            self.active = index;
        }
    }

    /// Resize Canvas policy (owner decision 2026-11-13, "keep the nodes"):
    /// every layer's buffer crops/pads to the new size (top-left anchored)
    /// while the node PNGs keep their original sizes — switching to an old
    /// node fits it back through [`fit_to_canvas`].
    pub fn resize_all(&mut self, width: u32, height: u32) {
        for layer in &mut self.layers {
            let fitted = fit_copy(&layer.pixmap, width, height);
            layer.pixmap = fitted;
        }
    }
}

#[cfg(test)]
mod tests;
