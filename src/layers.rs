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
//!   `encode_snapshot` / `decode_snapshot` — and the top-left crop/pad
//!   helper `fit_copy`), [`composite`] (premultiplied src-over blending
//!   of the visible layers into the display buffer, region-based so
//!   strokes stay cheap) and [`resize`] (2026-11-17, owner request: the
//!   Resize-dialog pixel plumbing — the 3×3 anchor crop/pad, the
//!   `image`-backed resampling filters, the per-layer proportional target
//!   size). All are re-exported or imported here, so every consumer keeps
//!   the stable `layers::` paths.
//!
//! SIZE MODEL (owner request 2026-11-17): the BOTTOM layer (index 0) owns
//! the canvas size — its pixmap IS the canvas; every OTHER layer keeps its
//! own pixmap size, and the composite draws them at (0,0), clipped to the
//! canvas (mixed-size layers are legal). A Resize-dialog commit adapts
//! every layer (Canvas mode: the same anchor crop/pad; Image mode: the
//! same proportional rescale) and records ONE `resize` node PER LAYER with
//! the full dialog choices. Restoring an old node restores its NATIVE
//! size — a bottom-layer restore resizes the canvas with it.
//!
//! Neighbours: `canvas.rs` holds the display buffer the composite writes
//! into; `app.rs` orchestrates gestures and texture sync; the DAG model
//! lives in `history_tree.rs`.

mod composite;
mod resize;
mod snapshot;

pub use composite::composite;
pub use snapshot::{decode_snapshot, encode_snapshot};

use resize::{anchor_offset, fit_copy_at, rescale, scaled_layer_size};
use snapshot::{fit_copy, pixmap_of};

use eframe::egui::Color32;
use tiny_skia::Pixmap;

use crate::history_tree::{
    ActionKind, Change, CropInfo, LayerTree, NodeId, Removal, ResizeAnchor, ResizeFilter,
    ResizeInfo, ResizeMode, TreeLayout, TreeNode,
};

/// One layer: a pixel buffer plus its own history. The BOTTOM layer's
/// pixmap is the canvas; other layers keep their own sizes (clipped at
/// composite time). Node snapshots may differ after a Resize — they carry
/// their own size and restore as-is.
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

    /// Replaces the buffer (the snapshot-restore path keeps the node's
    /// native size; the caller syncs the canvas when the BOTTOM layer
    /// changed size).
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

    // ---- resize (owner request 2026-11-17: the Resize dialog) ----

    /// The bottom layer's pixmap size — THE canvas size (the ownership
    /// invariant of the size model).
    pub fn bottom_size(&self) -> (u32, u32) {
        let bottom = &self.layers[0];
        (bottom.pixmap.width(), bottom.pixmap.height())
    }

    /// True when every layer's buffer already has the requested size (the
    /// Resize dialog's no-op check).
    pub fn all_layers_match(&self, width: u32, height: u32) -> bool {
        self.layers
            .iter()
            .all(|layer| layer.pixmap.width() == width && layer.pixmap.height() == height)
    }

    /// One Resize-dialog commit across the WHOLE stack (owner request
    /// 2026-11-17). The BOTTOM layer's current size is the old canvas; the
    /// dialog's target `(width, height)` is the new one.
    ///
    /// Canvas mode: every layer crop/pads by the SAME anchor geometry
    /// (offsets derived from the canvas's old→new mapping), so pixels keep
    /// their positions relative to the anchor corner. Image mode: every
    /// layer rescales proportionally by the canvas factor with the chosen
    /// filter (the bottom layer lands exactly on the target).
    ///
    /// EVERY layer gets ONE `resize` node carrying the layer's before/after
    /// sizes and the dialog choices (`ResizeInfo`) — jumping a layer back
    /// beyond the resize node shows exactly what that resize did to THIS
    /// layer. The caller rebuilds the display buffer and the texture.
    pub fn resize_all(
        &mut self,
        mode: ResizeMode,
        width: u32,
        height: u32,
        anchor: ResizeAnchor,
        filter: ResizeFilter,
    ) {
        let (old_w, old_h) = self.bottom_size();
        let (ox, oy) = anchor_offset(anchor, old_w, old_h, width, height);
        for layer in &mut self.layers {
            let from = (layer.pixmap.width(), layer.pixmap.height());
            let next = match mode {
                ResizeMode::Canvas => fit_copy_at(&layer.pixmap, width, height, ox, oy),
                ResizeMode::Image => {
                    let (new_w, new_h) =
                        scaled_layer_size(from.0, from.1, old_w, old_h, width, height);
                    rescale(&layer.pixmap, filter, new_w, new_h)
                }
            };
            let info = ResizeInfo {
                mode,
                from_w: from.0,
                from_h: from.1,
                to_w: next.width(),
                to_h: next.height(),
                anchor: match mode {
                    ResizeMode::Canvas => Some(anchor),
                    ResizeMode::Image => None,
                },
                filter: match mode {
                    ResizeMode::Canvas => None,
                    ResizeMode::Image => Some(filter),
                },
            };
            let snapshot = encode_snapshot(&next);
            let id = self.next_node_id;
            self.next_node_id += 1;
            layer.tree.record_resize(id, info, snapshot);
            layer.pixmap = next;
        }
    }

    /// One crop-to-selection commit (owner request 2026-11-20): EVERY layer
    /// crops to the same CANVAS-SPACE rect (the selection's pixel bounds) —
    /// the generalized top-left crop/pad of the Resize machinery
    /// (`fit_copy_at`, negative offsets = crop from the left/top) reused
    /// with no dialog: the anchor is the rect's own top-left corner. ONE
    /// `crop` node per layer carries the geometry, exactly like a resize
    /// commit records its choices. The caller rebuilds the display buffer,
    /// resizes the canvas to the rect and clears the selection (its bounds
    /// shift off-canvas).
    pub fn crop_all(&mut self, x: i64, y: i64, width: u32, height: u32) {
        for layer in &mut self.layers {
            let from = (layer.pixmap.width(), layer.pixmap.height());
            let next = fit_copy_at(&layer.pixmap, width, height, -x, -y);
            let info = CropInfo {
                x,
                y,
                from_w: from.0,
                from_h: from.1,
                to_w: next.width(),
                to_h: next.height(),
            };
            let snapshot = encode_snapshot(&next);
            let id = self.next_node_id;
            self.next_node_id += 1;
            layer.tree.record_crop(id, info, snapshot);
            layer.pixmap = next;
        }
    }

    /// The "Keep the canvas size" branch of the base-choice dialog: the new
    /// bottom layer (its size ≠ the canvas) crop/pads to the canvas size,
    /// TOP-LEFT anchored — the composite never shifts, because every layer
    /// sits at (0,0). The adaptation is a real pixel change, so it records
    /// a `resize` node like a Canvas-mode dialog commit would.
    pub fn fit_bottom_to(&mut self, width: u32, height: u32) {
        let Some(bottom) = self.layers.first_mut() else {
            return;
        };
        let from = (bottom.pixmap.width(), bottom.pixmap.height());
        if from == (width, height) {
            return;
        }
        let next = fit_copy(&bottom.pixmap, width, height);
        let info = ResizeInfo {
            mode: ResizeMode::Canvas,
            from_w: from.0,
            from_h: from.1,
            to_w: next.width(),
            to_h: next.height(),
            anchor: Some(ResizeAnchor::TopLeft),
            filter: None,
        };
        let snapshot = encode_snapshot(&next);
        let id = self.next_node_id;
        self.next_node_id += 1;
        bottom.tree.record_resize(id, info, snapshot);
        bottom.pixmap = next;
    }
}

#[cfg(test)]
mod tests;
