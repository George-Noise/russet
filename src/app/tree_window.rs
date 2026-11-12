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
//! - Node rows are NOT interactive: circle + "pencil" + ONE color square
//!   exactly one text line tall (owner request 2026-11-13: a stroke has a
//!   single color — no primary/secondary pair). Branches differ by rail
//!   color; the CURRENT timeline is the straight LEFT edge (lane 0), side
//!   branches peel off to the right with straight → quarter-ellipse →
//!   straight connectors (owner request 2026-11-13). All dots and rails
//!   stay inside a reserved gutter, labels live in a fixed column after
//!   it, and trees wider than the window scroll horizontally.
//! - Scroll: while the view sits at the graph's end (where the canvas
//!   state is), it follows new nodes automatically; once the user scrolls
//!   elsewhere the view pins to the node at the viewport's top edge and
//!   stays with it. When a node is deleted (debug "-" button), windows
//!   pinned to it jump to the nearest action FORWARD (backward as last
//!   resort) — or keep following the end if they were at the end.

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
/// Quarter-ellipse radius of a fork connector: half the lane step, so two
/// adjacent arcs never overlap (owner request 2026-11-13: the fork path is
/// a straight run, a quarter ellipse, then a straight rail to the node).
const ARC_RADIUS: f32 = 7.0;
/// Vertical stagger between successive forks from one parent: each
/// additionally crossed lane drops the horizontal run one step further
/// from the parent dot (the drop itself hides inside the parent's rail),
/// so the runs never overlap (owner request 2026-11-13).
const FORK_DROP_STEP: f32 = 9.0;
/// Padding after the last lane before the label column.
const GUTTER_TAIL_PAD: f32 = 6.0;
/// The ONE color swatch of a row, one text line tall (owner request
/// 2026-11-13: a stroke carries a single color — no pair).
const SWATCH_WIDTH: f32 = 10.0;
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
    // Horizontal scrolling (owner request 2026-11-13): deep branch trees
    // grow to the right, the viewport follows on demand.
    let out = egui::ScrollArea::both()
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
/// line tall — branch rails, fork connectors, the commit circle, the
/// "pencil" label and the single color swatch.
///
/// Geometry (owner request 2026-11-13): all rails and dots live inside a
/// gutter of `lanes * LANE_STEP` pixels; every connector is a straight
/// horizontal run, a quarter ellipse, then the branch's straight rail.
/// Branches never merge, so rails are ONE segment each — no per-row
/// stubs, no seams, nothing crossing the parent's row.
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
    let label_width = ui
        .painter()
        .layout_no_wrap(
            "pencil".to_string(),
            metrics.body_font.clone(),
            metrics.text_color,
        )
        .size()
        .x;

    // Gutter width from the deepest lane; the label column starts after it
    // with the SAME x on every row (VS Code style), so labels never
    // collide with rails regardless of how the tree branches.
    let lane_count = metrics
        .spans
        .iter()
        .map(|span| span.branch + 1)
        .max()
        .unwrap_or(1);
    let gutter_end = GRAPH_PAD_X + lane_count as f32 * LANE_STEP + GUTTER_TAIL_PAD;
    let row_width = (gutter_end + LABEL_GAP + label_width + LABEL_GAP + SWATCH_WIDTH + 8.0)
        .max(ui.available_width());

    // Pass 1: allocate the rows (content size + scroll pins).
    let mut rects: Vec<Rect> = Vec::with_capacity(rows.len());
    let animation = egui::style::ScrollAnimation::none();
    for row in rows {
        let (rect, response) =
            ui.allocate_exact_size(egui::vec2(row_width, row_height), Sense::hover());
        rects.push(rect);
        // Scroll pins: the graph's end is the HEAD row — bottom edge in
        // top-down mode, top edge in bottom-up mode.
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

    // Creation row -> display row (BottomUp reverses the list).
    let display_of: Vec<usize> = (0..rows.len())
        .map(|creation| match direction {
            Direction::TopDown => creation,
            Direction::BottomUp => rows.len() - 1 - creation,
        })
        .collect();
    let center_y = |creation: usize| rects[display_of[creation]].center().y;
    let lane_x = |lane: usize| rects[0].left() + GRAPH_PAD_X + lane as f32 * LANE_STEP;
    let painter = ui.painter();

    // Pass 2: one rail (and connector) per branch, painted BEFORE the dots.
    for span in metrics.spans {
        let color = branch_color(span.branch);
        let x = lane_x(span.branch);
        let Some(parent_row) = span.parent_row else {
            // Root branch (the head path or a detached root): one straight
            // rail from its first to its last node.
            painter.line_segment(
                [
                    egui::pos2(x, center_y(span.start_row)),
                    egui::pos2(x, center_y(span.end_row)),
                ],
                Stroke::new(2.0, color),
            );
            continue;
        };

        // Fork connector: straight run, quarter ellipse, straight rail.
        let parent_display = display_of[parent_row];
        let parent_branch = rows[parent_display].branch;
        let parent_x = lane_x(parent_branch);
        let parent_y = rects[parent_display].center().y;
        let child_y = center_y(span.start_row);
        // Successive forks stagger their runs one step per crossed lane;
        // the drop hides inside the parent's rail. Clamped so the arc
        // always lands above the child row (below in bottom-up mode).
        let sign = (child_y - parent_y).signum();
        let room = ((child_y - parent_y).abs() - ARC_RADIUS).max(0.0);
        let drop = ((span.branch - parent_branch - 1) as f32 * FORK_DROP_STEP).min(room) * sign;
        let y_run = parent_y + drop;

        // Straight horizontal run from the parent's rail to the arc.
        painter.line_segment(
            [
                egui::pos2(parent_x, y_run),
                egui::pos2(x - ARC_RADIUS, y_run),
            ],
            Stroke::new(2.0, color),
        );
        // Quarter ellipse: horizontal entry, vertical exit toward the row
        // of the fork's first node (kappa = best cubic fit for a quarter
        // circle; an ellipse of radius LANE_STEP/2 fits the lane width).
        let k = 0.552_3;
        let center = egui::pos2(x - ARC_RADIUS, y_run + sign * ARC_RADIUS);
        painter.add(Shape::CubicBezier(
            egui::epaint::CubicBezierShape::from_points_stroke(
                [
                    egui::pos2(center.x, y_run),
                    egui::pos2(center.x + k * ARC_RADIUS, y_run),
                    egui::pos2(x, center.y + sign * (ARC_RADIUS - k * ARC_RADIUS)),
                    egui::pos2(x, center.y),
                ],
                false,
                Color32::TRANSPARENT,
                Stroke::new(2.0, color),
            ),
        ));
        // Straight rail down (up in bottom-up mode) to the branch's tip.
        painter.line_segment(
            [
                egui::pos2(x, y_run + sign * ARC_RADIUS),
                egui::pos2(x, center_y(span.end_row)),
            ],
            Stroke::new(2.0, color),
        );
    }

    // Pass 3: dots, labels, color swatches.
    for (i, row) in rows.iter().enumerate() {
        let rect = rects[i];
        let cy = rect.center().y;
        let center = egui::pos2(lane_x(row.branch), cy);
        painter.circle_filled(center, NODE_RADIUS, branch_color(row.branch));
        if row.is_head {
            painter.circle_stroke(
                center,
                NODE_RADIUS + HEAD_RING_EXTRA,
                Stroke::new(1.5, text_color),
            );
        }

        let node = app.history_tree.node(row.id);
        let label = node.map_or("pencil", |node| kind_label(node.kind));
        // The label column sits after the gutter — same x for every row.
        // Rects are in SCREEN coordinates, so anchor at the row's left.
        let label_x = rects[0].left() + gutter_end + LABEL_GAP;
        painter.text(
            egui::pos2(label_x, cy),
            Align2::LEFT_CENTER,
            label,
            metrics.body_font.clone(),
            text_color,
        );
        if let Some(node) = node {
            let swatch = Rect::from_min_size(
                egui::pos2(label_x + label_width + LABEL_GAP, rect.top() + 2.0),
                egui::vec2(SWATCH_WIDTH, row_height - 4.0),
            );
            painter.rect_filled(swatch, 2.0, node.color);
            painter.rect_stroke(
                swatch,
                2.0,
                Stroke::new(1.0, Color32::from_gray(90)),
                egui::StrokeKind::Inside,
            );
        }
    }
}
