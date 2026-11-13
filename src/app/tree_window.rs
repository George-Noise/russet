//! The "History Tree" windows (owner request 2026-11-12): the git-style
//! graph of one layer's history, drawn in the spirit of the VS Code Source
//! Control Graph. Since owner request 2026-11-13 the tree is the REAL
//! history (one PNG canvas per node) and every window follows the GLOBAL
//! active layer — switching happens in the Layers window, the graph is a
//! passenger. LEFT-clicking a node's DOT (never its label) moves the head
//! there and shows that node's canvas. The data model lives in
//! [`crate::history_tree`]; this module renders it and owns the per-window
//! view state (windows, clones, direction, scroll anchoring). The pure
//! graph painter — rails, connectors, dots, labels — lives in the child
//! module [`paint`]. Child module of `app`, so it can touch `App`'s
//! private fields; `app.rs` stays the orchestrator.
//!
//! Behavior fixed by the owner's answers (2026-11-12, layered 2026-11-13):
//! - Every window shows the GLOBAL active layer's graph (switched in the
//!   Layers window since 2026-11-13); switching resets every window's
//!   scroll to the end. A passive label names the layer.
//! - Main window: floating, first open at the OLD History spot (right
//!   edge, just below the menu bar), §3.11 placement afterwards. Clones
//!   spawn cascaded below it, are closable, and carry the same §3.11
//!   placement. The "Clone" button copies the scroll state (direction +
//!   anchor), so the user can watch several places of the same tree.
//! - Direction toggle per window: "New at bottom" (top-down, gitk-like;
//!   DEFAULT) and "New at top" (bottom-up, VS Code-like).
//! - Node rows: circle + "pencil" + ONE color square exactly one text
//!   line tall (owner request 2026-11-13: a stroke has a single color — no
//!   primary/secondary pair; the Start root shows no swatch). The DOT is
//!   interactive since 2026-11-13: LEFT-click moves the head there and the
//!   canvas shows that node's PNG snapshot; the label stays passive. Branches differ by rail
//!   color; the CURRENT timeline is the straight LEFT edge (lane 0), side
//!   branches peel off to the right with straight → quarter-ellipse →
//!   straight connectors (owner request 2026-11-13), and every connector
//!   leaving one node exits at the SAME height — the parent dot's center
//!   line — even when the horizontal runs overlap (owner request
//!   2026-11-13). Labels hug EACH row's own node (kind + node id, e.g.
//!   "pencil 7", on an opaque backdrop) instead of sitting in one global
//!   column right of the whole gutter (owner request 2026-11-13), and
//!   trees wider than the window scroll horizontally.
//! - Scroll: while the view sits at the graph's end (where the canvas
//!   state is), it follows new nodes automatically; once the user scrolls
//!   elsewhere the view pins to the node at the viewport's top edge and
//!   stays with it. When a node is deleted (debug "-" button), windows
//!   pinned to it jump to the nearest action FORWARD (backward as last
//!   resort) — or keep following the end if they were at the end.

use eframe::egui::{self, Align2, Context, FontId, Id, Pos2, TextStyle};

use super::App;
use super::placement::{FIRST_OPEN_GAP, WindowPlacement, place_floating_window};
use crate::history_tree::{Change, NodeId, Removal};

mod paint;

use self::paint::{GraphMetrics, paint_graph};

/// Stable id of the main window; clones get numbered ids.
fn main_tree_window_id() -> Id {
    Id::unique("history_tree_main_window")
}

/// Window width for the main window and every clone.
const TREE_WINDOW_WIDTH: f32 = 240.0;
/// Scroll-area cap; the window auto-sizes below it.
const TREE_MAX_HEIGHT: f32 = 340.0;
/// Cascade offset of the Nth clone from the main window's first-open spot.
const CLONE_CASCADE_STEP: f32 = 24.0;

/// Reading direction of the graph (owner request 2026-11-12): TopDown puts
/// the FIRST action at the top and new ones at the bottom (gitk); BottomUp
/// puts new ones at the top (VS Code).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Direction {
    TopDown,
    BottomUp,
}

/// Persistent scroll mode of one window: either following the graph's end
/// (where the canvas state is) or pinned to a concrete node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Anchor {
    End,
    Node(NodeId),
}

/// One-shot "scroll now" request, applied inside the scroll area's layout
/// pass so it lands in the same frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pin {
    End,
    Node(NodeId),
}

/// View state of ONE History Tree window (the main one or a clone).
/// `Copy` on purpose: the render pass works on a local copy and writes it
/// back, which sidesteps borrow conflicts with the shared tree.
#[derive(Clone, Copy)]
pub(super) struct TreeView {
    id: Id,
    /// Clones can be closed; the main window uses `App::tree_window_open`.
    open: bool,
    direction: Direction,
    anchor: Anchor,
    /// Was the view at the graph's end on the last rendered frame.
    at_end: bool,
    prev_offset: Option<f32>,
    pin: Option<Pin>,
    seen_generation: u64,
    /// First-open cascade offset from the main window's spot (clones only).
    cascade: f32,
    placement: WindowPlacement,
}

impl TreeView {
    /// The main window's initial state: follow the end, top-down.
    pub(super) fn main() -> Self {
        Self {
            id: main_tree_window_id(),
            open: true,
            direction: Direction::TopDown,
            anchor: Anchor::End,
            at_end: true,
            prev_offset: None,
            pin: None,
            seen_generation: 0,
            cascade: 0.0,
            placement: WindowPlacement::default(),
        }
    }
}

/// Actions a window's toolbar requested while rendering; applied AFTER the
/// window is shown (they mutate the view list itself, and the jump mutates
/// the canvas).
#[derive(Default)]
struct ViewRequest {
    clone: bool,
    remove: bool,
    jump: Option<NodeId>,
}

impl App {
    /// Renders the main History Tree window and every clone.
    pub(super) fn tree_windows(&mut self, ctx: &Context) {
        if self.tree_window_open {
            let mut view = self.tree_views[0];
            let mut open = true;
            let mut request = ViewRequest::default();
            let shown = egui::Window::new("History Tree")
                .id(main_tree_window_id())
                .open(&mut open)
                // First open (owner request 2026-11-12): exactly the OLD
                // History window's spot — 0.5rem from the right edge, just
                // below the menu bar; §3.11 takes over afterwards.
                .pivot(Align2::RIGHT_TOP)
                .default_pos(egui::pos2(
                    ctx.content_rect().right() - FIRST_OPEN_GAP,
                    self.last_panel_rect.top() + FIRST_OPEN_GAP,
                ))
                .default_width(TREE_WINDOW_WIDTH)
                .show(ctx, |ui| tree_content(self, ui, &mut view, &mut request));
            if let Some(response) = &shown {
                place_floating_window(
                    ctx,
                    main_tree_window_id(),
                    response.response.rect,
                    ctx.content_rect(),
                    self.last_content_rect,
                    &mut view.placement,
                );
            }
            self.tree_views[0] = view;
            if !open {
                self.tree_window_open = false;
            }
            self.apply_tree_request(0, request);
        }

        // Clones (index >= 1): closable; closing removes them for good.
        let mut index = 1;
        while index < self.tree_views.len() {
            let mut view = self.tree_views[index];
            let mut open = view.open;
            let mut request = ViewRequest::default();
            let title = format!("History Tree {}", index + 1);
            let shown = egui::Window::new(title)
                .id(view.id)
                .open(&mut open)
                .pivot(Align2::RIGHT_TOP)
                .default_pos(self.clone_first_pos(&view))
                .default_width(TREE_WINDOW_WIDTH)
                .show(ctx, |ui| tree_content(self, ui, &mut view, &mut request));
            if let Some(response) = &shown {
                place_floating_window(
                    ctx,
                    view.id,
                    response.response.rect,
                    ctx.content_rect(),
                    self.last_content_rect,
                    &mut view.placement,
                );
            }
            if open {
                view.open = true;
                self.tree_views[index] = view;
                self.apply_tree_request(index, request);
                index += 1;
            } else {
                self.tree_views.remove(index);
            }
        }
    }

    /// First-open position of a clone: the main window's first-open spot
    /// stepped down-left per clone number (the stored placement wins later,
    /// like for every floating window).
    fn clone_first_pos(&self, view: &TreeView) -> Pos2 {
        egui::pos2(
            self.ctx.content_rect().right() - FIRST_OPEN_GAP - view.cascade,
            self.last_panel_rect.top() + FIRST_OPEN_GAP + view.cascade,
        )
    }

    fn apply_tree_request(&mut self, index: usize, request: ViewRequest) {
        if request.remove {
            self.remove_tree_last_action();
        }
        if request.clone {
            self.clone_tree_view(index);
        }
        if let Some(id) = request.jump {
            self.jump_to_history_node(id);
        }
    }

    /// The Clone button: a new window with the SAME scroll state (reading
    /// direction and anchor), one cascade step away. A one-shot pin makes
    /// the first frame land on the copied scroll position.
    fn clone_tree_view(&mut self, source_index: usize) {
        let mut view = self.tree_views[source_index];
        self.tree_clones += 1;
        view.id = Id::unique(("history_tree_clone_window", self.tree_clones));
        view.open = true;
        view.cascade = CLONE_CASCADE_STEP * self.tree_clones as f32;
        view.placement = WindowPlacement::default();
        view.prev_offset = None;
        view.pin = Some(match view.anchor {
            Anchor::End => Pin::End,
            Anchor::Node(id) => Pin::Node(id),
        });
        self.tree_views.push(view);
    }

    /// Debug "-": remove the last action of the active layer (the head's
    /// PNG goes with it; children keep theirs) and retarget every window
    /// that was pinned to it (owner rule 2026-11-12: nearest action
    /// forward, backward as last resort; windows that sat at the graph's
    /// end keep following the end).
    fn remove_tree_last_action(&mut self) {
        let Some(removal) = self.layers.remove_last() else {
            return;
        };
        // The canvas follows the new head (the removed node's state is
        // gone from the tree).
        self.restore_active_layer_state();
        self.dirty = true;
        self.retarget_views_after_removal(&removal);
    }

    fn retarget_views_after_removal(&mut self, removal: &Removal) {
        for view in &mut self.tree_views {
            if view.anchor != Anchor::Node(removal.target) {
                continue;
            }
            view.anchor = if view.at_end {
                Anchor::End
            } else {
                removal
                    .forward
                    .or(removal.backward)
                    .map_or(Anchor::End, Anchor::Node)
            };
            view.pin = Some(match view.anchor {
                Anchor::End => Pin::End,
                Anchor::Node(id) => Pin::Node(id),
            });
        }
    }

    /// Global active-layer switch (owner request 2026-11-13): the Layers
    /// window owns it, and every History Tree window shows the new layer's
    /// graph from its end.
    pub(super) fn switch_active_layer(&mut self, layer: usize) {
        if layer >= self.layers.count() {
            return;
        }
        self.layers.set_active_layer(layer);
        let generation = self.layers.generation();
        for view in &mut self.tree_views {
            view.anchor = Anchor::End;
            view.pin = Some(Pin::End);
            view.seen_generation = generation;
        }
    }

    /// New / Open dropped the document: follow the (now fresh) end again.
    pub(super) fn reset_tree_views(&mut self) {
        let generation = self.layers.generation();
        for view in &mut self.tree_views {
            view.anchor = Anchor::End;
            view.pin = Some(Pin::End);
            view.prev_offset = None;
            view.seen_generation = generation;
        }
    }
}

/// Window content: toolbar (layers, direction, clone, debug remove) plus
/// the scrollable graph. `view` is the caller's local copy — mutations are
/// written back by the caller after the window closes its layout pass.
fn tree_content(app: &mut App, ui: &mut egui::Ui, view: &mut TreeView, request: &mut ViewRequest) {
    ui.set_min_width(200.0);

    // The ACTIVE layer follows the Layers window (owner request 2026-11-13:
    // the switcher moved out of here) — a passive label only.
    ui.weak(format!("Layer: {}", app.layers.active_layer_name()));

    // Direction toggle (per window), Clone, debug remove.
    ui.horizontal(|ui| {
        let direction_label = match view.direction {
            Direction::TopDown => "New at bottom",
            Direction::BottomUp => "New at top",
        };
        if ui
            .button(direction_label)
            .on_hover_text("Toggle graph direction")
            .clicked()
        {
            view.direction = match view.direction {
                Direction::TopDown => Direction::BottomUp,
                Direction::BottomUp => Direction::TopDown,
            };
            view.pin = Some(match view.anchor {
                Anchor::End => Pin::End,
                Anchor::Node(id) => Pin::Node(id),
            });
        }
        if ui
            .button("Clone")
            .on_hover_text("Clone this window (keeps the scroll position)")
            .clicked()
        {
            request.clone = true;
        }
        if ui
            .add_enabled(app.layers.can_remove_last(), egui::Button::new("-"))
            .on_hover_text("Remove last action (debug: tests the scroll fallback)")
            .clicked()
        {
            request.remove = true;
        }
    });
    ui.separator();

    let row_height = ui.text_style_height(&TextStyle::Body);
    let body_font = ui
        .style()
        .text_styles
        .get(&TextStyle::Body)
        .cloned()
        .unwrap_or_else(|| FontId::proportional(row_height));
    let text_color = ui.visuals().text_color();
    let direction = view.direction;

    // Content changed since the last frame? Then an anchored view may need
    // re-pinning: only changes that SHIFT earlier rows move it — plain
    // appends in the top-down order do not.
    let generation = app.layers.generation();
    if view.seen_generation != generation {
        let shifts_rows = match app.layers.last_change() {
            Change::Removed | Change::Reset => true,
            Change::Added => direction == Direction::BottomUp,
            Change::Moved => false,
        };
        if shifts_rows && let Anchor::Node(id) = view.anchor {
            view.pin = Some(Pin::Node(id));
        }
        view.seen_generation = generation;
    }

    let layout = app.layers.layout();
    let mut rows = layout.rows.clone();
    if direction == Direction::BottomUp {
        rows.reverse();
    }

    let pin = view.pin.take();
    let metrics = GraphMetrics {
        spans: &layout.spans,
        row_height,
        body_font,
        text_color,
    };
    // Horizontal scrolling (owner request 2026-11-13): deep branch trees
    // grow to the right, the viewport follows on demand. A dot click comes
    // back as the painter's return value and lands in the request.
    let out = egui::ScrollArea::both()
        .id_salt(view.id.with("scroll"))
        .max_height(TREE_MAX_HEIGHT)
        .auto_shrink([false, true])
        // Top-down: following the end means following the bottom edge.
        .stick_to_bottom(direction == Direction::TopDown)
        .show(ui, |ui| {
            let clicked = paint_graph(app, ui, &rows, direction, &metrics, pin, view.id);
            if clicked.is_some() {
                request.jump = clicked;
            }
        });

    // ---- scroll bookkeeping (after the layout pass) ----
    let offset = out.state.clamped_offset().y;
    let max_scroll = (out.content_size.y - out.inner_rect.height()).max(0.0);
    let at_end = match direction {
        Direction::TopDown => offset >= max_scroll - 0.5,
        Direction::BottomUp => offset <= 0.5,
    };
    // A pin moved the offset programmatically this frame — never read that
    // as a user scroll.
    let user_scrolled = pin.is_none()
        && view
            .prev_offset
            .is_some_and(|prev| (prev - offset).abs() > 0.5);
    if at_end {
        // Sitting at the end of the graph (where the canvas state is):
        // keep following it when new actions appear.
        view.anchor = Anchor::End;
    } else if user_scrolled && !rows.is_empty() {
        // Scrolled somewhere into the tree: pin to the node at the top
        // edge of the viewport.
        let index = ((offset / row_height).floor() as usize).min(rows.len() - 1);
        view.anchor = rows
            .get(index)
            .map(|row| Anchor::Node(row.id))
            .unwrap_or(Anchor::End);
    }
    view.prev_offset = Some(offset);
    view.at_end = at_end;
}
