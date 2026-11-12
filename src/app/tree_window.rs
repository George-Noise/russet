//! The "History Tree" windows (owner request 2026-11-12): a purely visual
//! git-style graph of pencil actions per pseudo-layer, drawn in the spirit
//! of the VS Code Source Control Graph. The data model lives in
//! [`crate::history_tree`]; this module renders it and owns the per-window
//! view state. Child module of `app`, so it can touch `App`'s private
//! fields; `app.rs` stays the orchestrator.
//!
//! Behavior fixed by the owner's answers (2026-11-12):
//! - THREE pseudo-layers with one global active layer; the switcher lives
//!   in the toolbar; switching resets every window's scroll to the end.
//! - Main window: floating, first open at the OLD History spot (right
//!   edge, just below the menu bar), §3.11 placement afterwards. Clones
//!   spawn cascaded below it, are closable, and carry the same §3.11
//!   placement. The "Clone" button copies the scroll state (direction +
//!   anchor), so the user can watch several places of the same tree.
//! - Direction toggle per window: "New at bottom" (top-down, gitk-like;
//!   DEFAULT) and "New at top" (bottom-up, VS Code-like).
//! - Node rows are NOT interactive: circle + "pencil" + two stacked color
//!   squares exactly one text line tall. Branches differ by rail color.
//! - Scroll: while the view sits at the graph's end (where the canvas
//!   state is), it follows new nodes automatically; once the user scrolls
//!   elsewhere the view pins to the node at the viewport's top edge and
//!   stays with it. When a node is deleted (debug "-" button), windows
//!   pinned to it jump to the nearest action FORWARD (backward as last
//!   resort) — or keep following the end if they were at the end.

use std::collections::HashMap;

use eframe::egui::{
    self, Align, Align2, Color32, Context, FontId, Id, Pos2, Rect, Sense, Shape, Stroke, TextStyle,
};

use super::App;
use super::placement::{FIRST_OPEN_GAP, WindowPlacement, place_floating_window};
use crate::history_tree::{self, Change, NodeId, Removal, Row};

/// Stable id of the main window; clones get numbered ids.
fn main_tree_window_id() -> Id {
    Id::unique("history_tree_main_window")
}

/// Window width for the main window and every clone.
const TREE_WINDOW_WIDTH: f32 = 240.0;
/// Scroll-area cap; the window auto-sizes below it.
const TREE_MAX_HEIGHT: f32 = 340.0;
/// Graph geometry (VS Code-graph-like lanes).
const GRAPH_PAD_X: f32 = 12.0;
const LANE_STEP: f32 = 14.0;
const NODE_RADIUS: f32 = 3.5;
const HEAD_RING_EXTRA: f32 = 2.0;
const LABEL_GAP: f32 = 10.0;
/// The two color squares: stacked into a column exactly ONE text line tall.
const SQUARE_WIDTH: f32 = 8.0;
const SQUARE_GAP: f32 = 2.0;
/// Cascade offset of the Nth clone from the main window's first-open spot.
const CLONE_CASCADE_STEP: f32 = 24.0;

/// Rail colors per branch (GitHub-graph palette; readable on both themes).
const BRANCH_COLORS: [Color32; 6] = [
    Color32::from_rgb(0x4c, 0x8d, 0xff),
    Color32::from_rgb(0x3f, 0xb9, 0x50),
    Color32::from_rgb(0xa3, 0x71, 0xf7),
    Color32::from_rgb(0xdb, 0x6d, 0x28),
    Color32::from_rgb(0xdb, 0x61, 0xa2),
    Color32::from_rgb(0x39, 0xc5, 0xcf),
];

fn branch_color(branch: usize) -> Color32 {
    BRANCH_COLORS[branch % BRANCH_COLORS.len()]
}

/// Row label of an action kind (the only info a node carries besides its
/// colors — owner request 2026-11-12: no author, no extra metadata).
fn kind_label(kind: history_tree::ActionKind) -> &'static str {
    match kind {
        history_tree::ActionKind::Pencil => "pencil",
    }
}

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
/// window is shown (they mutate the view list itself).
#[derive(Default)]
struct ViewRequest {
    clone: bool,
    remove: bool,
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

    /// Debug "-": remove the last action of the active layer and retarget
    /// every window that was pinned to it (owner rule 2026-11-12: nearest
    /// action forward, backward as last resort; windows that sat at the
    /// graph's end keep following the end).
    fn remove_tree_last_action(&mut self) {
        let Some(removal) = self.history_tree.remove_last() else {
            return;
        };
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

    /// Global active-layer switch (owner request 2026-11-12): every window
    /// shows the new layer's graph from its end.
    fn switch_tree_layer(&mut self, layer: usize) {
        self.history_tree.set_active_layer(layer);
        let generation = self.history_tree.generation();
        for view in &mut self.tree_views {
            view.anchor = Anchor::End;
            view.pin = Some(Pin::End);
            view.seen_generation = generation;
        }
    }

    /// New / Open dropped the document: follow the (now empty) end again.
    pub(super) fn reset_tree_views(&mut self) {
        let generation = self.history_tree.generation();
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

    // Pseudo-layer switcher: three placeholders, one GLOBAL active layer.
    ui.horizontal(|ui| {
        for (i, name) in history_tree::LAYER_NAMES.iter().enumerate() {
            if ui
                .selectable_label(app.history_tree.active_layer() == i, *name)
                .clicked()
            {
                app.switch_tree_layer(i);
                view.anchor = Anchor::End;
                view.pin = Some(Pin::End);
                view.seen_generation = app.history_tree.generation();
            }
        }
    });

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
            .add_enabled(app.history_tree.head().is_some(), egui::Button::new("-"))
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
    let generation = app.history_tree.generation();
    if view.seen_generation != generation {
        let shifts_rows = match app.history_tree.last_change() {
            Change::Removed | Change::Reset => true,
            Change::Added => direction == Direction::BottomUp,
            Change::Moved => false,
        };
        if shifts_rows && let Anchor::Node(id) = view.anchor {
            view.pin = Some(Pin::Node(id));
        }
        view.seen_generation = generation;
    }

    let layout = app.history_tree.layout();
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
    let out = egui::ScrollArea::vertical()
        .id_salt(view.id.with("scroll"))
        .max_height(TREE_MAX_HEIGHT)
        .auto_shrink([false, true])
        // Top-down: following the end means following the bottom edge.
        .stick_to_bottom(direction == Direction::TopDown)
        .show(ui, |ui| {
            paint_graph(app, ui, &rows, direction, &metrics, pin);
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

/// Everything the graph painter needs besides the rows themselves.
struct GraphMetrics<'a> {
    spans: &'a [history_tree::BranchSpan],
    row_height: f32,
    body_font: FontId,
    text_color: Color32,
}

/// The graph itself: one non-interactive row per action, exactly one text
/// line tall — branch rails, fork curves, the commit circle, the "pencil"
/// label and the two stacked color squares.
fn paint_graph(
    app: &App,
    ui: &mut egui::Ui,
    rows: &[Row],
    direction: Direction,
    metrics: &GraphMetrics<'_>,
    pin: Option<Pin>,
) {
    if rows.is_empty() {
        ui.weak("No actions yet.");
        return;
    }

    let row_height = metrics.row_height;
    let text_color = metrics.text_color;
    let display_index: HashMap<NodeId, usize> = rows
        .iter()
        .enumerate()
        .map(|(i, row)| (row.id, i))
        .collect();
    let label_width = ui
        .painter()
        .layout_no_wrap(
            "pencil".to_string(),
            metrics.body_font.clone(),
            metrics.text_color,
        )
        .size()
        .x;
    let mut curves: Vec<([Pos2; 3], usize)> = Vec::new();
    let row_count = rows.len();

    // One-shot scroll requests must be applied INSIDE this layout pass to
    // land in the same frame; instant (no animation), so they never fight
    // the anchor detection with animation frames.
    let animation = egui::style::ScrollAnimation::none();
    for (i, row) in rows.iter().enumerate() {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(ui.available_width(), row_height), Sense::hover());
        let creation_index = match direction {
            Direction::TopDown => i,
            Direction::BottomUp => row_count - 1 - i,
        };
        let cy = rect.center().y;
        let lane_x = |lane: usize| rect.left() + GRAPH_PAD_X + lane as f32 * LANE_STEP;

        // Rails of every branch alive at this row.
        for span in metrics.spans {
            if span.start_row <= creation_index && creation_index <= span.end_row {
                let x = lane_x(span.branch);
                ui.painter().line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    Stroke::new(2.0, branch_color(span.branch)),
                );
            }
        }

        // Fork curve from the parent circle to this row's circle; painted
        // after the loop because it may span several rows.
        if row.fork
            && let Some(parent) = row.parent
            && let Some(&parent_display) = display_index.get(&parent)
        {
            let parent_branch = rows[parent_display].branch;
            let parent_y = cy + (parent_display as f32 - i as f32) * row_height;
            let from = egui::pos2(lane_x(parent_branch), parent_y);
            let to = egui::pos2(lane_x(row.branch), cy);
            curves.push(([from, egui::pos2(to.x, from.y), to], row.branch));
        }

        // Commit circle; the head (the canvas state) gets a ring.
        let center = egui::pos2(lane_x(row.branch), cy);
        ui.painter()
            .circle_filled(center, NODE_RADIUS, branch_color(row.branch));
        if row.is_head {
            ui.painter().circle_stroke(
                center,
                NODE_RADIUS + HEAD_RING_EXTRA,
                Stroke::new(1.5, text_color),
            );
        }

        let node = app.history_tree.node(row.id);
        // "pencil" + the two color squares: a column exactly one text line
        // tall (primary on top, secondary below).
        let label = node.map_or("pencil", |node| kind_label(node.kind));
        let label_x = center.x + NODE_RADIUS + LABEL_GAP;
        ui.painter().text(
            egui::pos2(label_x, cy),
            Align2::LEFT_CENTER,
            label,
            metrics.body_font.clone(),
            text_color,
        );
        if let Some(node) = node {
            let square_height = (row_height - SQUARE_GAP) / 2.0;
            let x = label_x + label_width + LABEL_GAP;
            for (offset, color) in [
                (0.0, node.primary),
                (square_height + SQUARE_GAP, node.secondary),
            ] {
                let square = Rect::from_min_size(
                    egui::pos2(x, rect.top() + offset),
                    egui::vec2(SQUARE_WIDTH, square_height),
                );
                ui.painter().rect_filled(square, 1.0, color);
                ui.painter().rect_stroke(
                    square,
                    1.0,
                    Stroke::new(1.0, Color32::from_gray(90)),
                    egui::StrokeKind::Inside,
                );
            }
        }

        // Scroll pins (see `tree_content`): the graph's end is the HEAD row —
        // bottom edge in top-down mode, top edge in bottom-up mode.
        if let Some(target) = pin {
            let (hit, align) = match target {
                Pin::End => (
                    app.history_tree.head() == Some(row.id),
                    match direction {
                        Direction::TopDown => Align::Max,
                        Direction::BottomUp => Align::Min,
                    },
                ),
                Pin::Node(id) => (row.id == id, Align::Min),
            };
            if hit {
                response.scroll_to_me_animation(Some(align), animation);
            }
        }
    }

    for (points, branch) in curves {
        ui.painter().add(Shape::QuadraticBezier(
            egui::epaint::QuadraticBezierShape::from_points_stroke(
                points,
                false,
                Color32::TRANSPARENT,
                Stroke::new(2.0, branch_color(branch)),
            ),
        ));
    }
}
