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
//! - Snapshot helpers: [`encode_snapshot`] / [`decode_snapshot`]
//!   (lossless PNG roundtrip of the raw premultiplied bytes) and
//!   [`fit_to_canvas`] (crop/pad a node's PNG back onto the current canvas
//!   size — the Resize policy keeps old nodes).
//! - [`composite`]: premultiplied src-over blending of the visible layers
//!   into the display buffer, region-based so strokes stay cheap.
//!
//! Snapshot format note: the PNGs are an INTERNAL RAM container, never
//! shown or saved, so the premultiplied bytes go in and come out untouched
//! — `decode(encode(x)) == x` bit-for-bit. A straight-alpha PNG of
//! premultiplied data could NOT roundtrip losslessly (the 8-bit
//! premultiply/demultiply roundtrip is lossy) — exactly what a snapshot
//! must not be.
//!
//! Neighbours: `canvas.rs` holds the display buffer the composite writes
//! into; `app.rs` orchestrates gestures and texture sync; the DAG model
//! lives in `history_tree.rs`.

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
    /// A new document: ONE transparent layer ("Layer 1") whose Start root
    /// holds the initial canvas. All layers are transparent (owner
    /// request 2026-11-13) — the checkerboard shows through until painted.
    pub fn new(width: u32, height: u32) -> Self {
        let pixmap = pixmap_of(width, height);
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
    /// at the head. Returns the new node id.
    pub fn record_action(&mut self, kind: ActionKind, color: Color32) -> Result<NodeId, String> {
        let snapshot = encode_snapshot(&self.layers[self.active].pixmap);
        let id = self.next_node_id;
        self.next_node_id += 1;
        self.layers[self.active]
            .tree
            .record_action(id, kind, color, snapshot);
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

/// An empty (fully transparent) pixmap with the Canvas clamps; falls back
/// to 1×1 like `Canvas::new` when the allocation is refused.
fn pixmap_of(width: u32, height: u32) -> Pixmap {
    let width = width.clamp(1, 4096);
    let height = height.clamp(1, 4096);
    // Pixmap::new yields a fully transparent buffer.
    Pixmap::new(width, height).unwrap_or_else(|| unreachable!("1x1 pixmap"))
}

/// Lossless fast PNG of a pixmap's raw premultiplied bytes (see the module
/// note on why no premultiply conversion happens here). Uses the `image`
/// crate's FAST compression (already a dependency — no new crates); an
/// encoding failure (practically impossible for in-memory RGBA8) yields an
/// empty snapshot that decodes into a clear error instead of corruption.
pub fn encode_snapshot(pixmap: &Pixmap) -> Vec<u8> {
    let mut png = Vec::new();
    {
        let encoder = image::codecs::png::PngEncoder::new_with_quality(
            &mut png,
            image::codecs::png::CompressionType::Fast,
            image::codecs::png::FilterType::Up,
        );
        let encoded = image::ImageEncoder::write_image(
            encoder,
            pixmap.data(),
            pixmap.width(),
            pixmap.height(),
            image::ExtendedColorType::Rgba8,
        );
        if encoded.is_err() {
            png.clear();
        }
    }
    png
}

/// Decodes a node snapshot back into a pixmap. The bytes come back exactly
/// as they went in ([`encode_snapshot`] is lossless), so they can be
/// treated as premultiplied right away.
pub fn decode_snapshot(bytes: &[u8]) -> Result<Pixmap, String> {
    let image = image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map_err(|err| format!("Corrupt history snapshot: {err}"))?;
    let rgba = image.to_rgba8();
    let Some(mut pixmap) = Pixmap::new(rgba.width(), rgba.height()) else {
        return Err("Invalid snapshot size".to_string());
    };
    pixmap.data_mut().copy_from_slice(rgba.as_raw());
    Ok(pixmap)
}

/// Fits a snapshot onto the current canvas: same size — unchanged; larger
/// — the top-left crop; smaller — padded with transparency at the
/// bottom/right. This is what "the nodes keep their sizes" (Resize policy)
/// shows on restore, and it matches `Canvas::resize_canvas`'s content
/// placement exactly.
pub fn fit_to_canvas(pixmap: Pixmap, width: u32, height: u32) -> Pixmap {
    if pixmap.width() == width && pixmap.height() == height {
        return pixmap;
    }
    fit_copy(&pixmap, width, height)
}

/// The by-reference variant of [`fit_to_canvas`] (no clone when the sizes
/// already match — [`LayerStack::resize_all`] uses it for every layer).
fn fit_copy(src: &Pixmap, width: u32, height: u32) -> Pixmap {
    if src.width() == width && src.height() == height {
        return src.clone();
    }
    let Some(mut out) = Pixmap::new(width, height) else {
        return src.clone();
    };
    let copy_width = src.width().min(width) as usize * 4;
    let copy_height = src.height().min(height) as usize;
    let src_stride = src.width() as usize * 4;
    let dst_stride = width as usize * 4;
    let src_data = src.data();
    let dst = out.data_mut();
    for row in 0..copy_height {
        let src_start = row * src_stride;
        let dst_start = row * dst_stride;
        dst[dst_start..dst_start + copy_width]
            .copy_from_slice(&src_data[src_start..src_start + copy_width]);
    }
    out
}

/// Blends the visible layers (bottom first) over the region `x, y, w, h`
/// of `dst` (the display buffer): clears the region, then premultiplied
/// src-over per layer. All layer pixmaps are canvas-sized like `dst`.
pub fn composite(layers: &[Layer], dst: &mut Pixmap, x: u32, y: u32, w: u32, h: u32) {
    let dst_w = dst.width();
    let dst_h = dst.height();
    let x0 = x.min(dst_w);
    let y0 = y.min(dst_h);
    let x1 = x.saturating_add(w).min(dst_w);
    let y1 = y.saturating_add(h).min(dst_h);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    clear_region(dst, x0, y0, x1 - x0, y1 - y0);
    for layer in layers {
        if !layer.visible() {
            continue;
        }
        blend_region(layer.pixmap(), dst, x0, y0, x1 - x0, y1 - y0);
    }
}

/// Zeros the region's bytes (fully transparent).
fn clear_region(dst: &mut Pixmap, x: u32, y: u32, w: u32, h: u32) {
    let stride = dst.width() as usize * 4;
    let data = dst.data_mut();
    for row in 0..h as usize {
        let start = (y as usize + row) * stride + x as usize * 4;
        let end = start + w as usize * 4;
        data[start..end].fill(0);
    }
}

/// Premultiplied src-over of ONE layer's region into `dst`: for every
/// channel `out = src + dst * (255 - src_alpha) / 255` (rounded). With
/// premultiplied data this is exact over-blending for all four channels at
/// once; the alpha-0 and alpha-255 shortcuts keep flat areas bit-exact.
fn blend_region(src_pixmap: &Pixmap, dst: &mut Pixmap, x: u32, y: u32, w: u32, h: u32) {
    let src_stride = src_pixmap.width() as usize * 4;
    let dst_stride = dst.width() as usize * 4;
    let copy_width = w as usize;
    let copy_height = h as usize;
    let src = src_pixmap.data();
    let dst_data = dst.data_mut();
    for row in 0..copy_height {
        let src_row = y as usize + row;
        // Layer pixmaps are canvas-sized like `dst`, so this never trips;
        // a hypothetical mismatch skips the out-of-bounds rows instead of
        // panicking or blending the wrong rows.
        if src_row >= src_pixmap.height() as usize {
            continue;
        }
        let src_start = src_row * src_stride + x as usize * 4;
        let dst_start = src_row * dst_stride + x as usize * 4;
        for px in 0..copy_width {
            let s = src_start + px * 4;
            let d = dst_start + px * 4;
            let sa = src[s + 3] as u32;
            if sa == 0 {
                continue;
            }
            if sa == 255 {
                dst_data[d..d + 4].copy_from_slice(&src[s..s + 4]);
            } else {
                let inv = 255 - sa;
                for k in 0..4 {
                    let kept = (dst_data[d + k] as u32 * inv + 127) / 255;
                    dst_data[d + k] = src[s + k].saturating_add(kept as u8);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
