//! The graph painter of the "History Tree" windows: branch rails, fork
//! connectors, dots, labels and color swatches — one non-interactive row
//! per action, exactly one text line tall.
//!
//! Pure painting: no state, no input handling, no scroll logic. The window
//! chrome, the toolbar, the scroll bookkeeping and the per-window view
//! state live in the parent module ([`super`]); the data model and the
//! branch/lane assignment live in [`crate::history_tree`]. Child module of
//! `app::tree_window`, so it can read `App`'s private fields (it only ever
//! needs the shared `history_tree`).

use eframe::egui::{self, Align, Align2, Color32, FontId, Rect, Sense, Shape, Stroke};

use super::{Direction, Pin};
use crate::app::App;
use crate::history_tree::{self, Row};

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
/// Padding after the deepest lane (the gutter's minimum width).
const GUTTER_TAIL_PAD: f32 = 6.0;
/// The ONE color swatch of a row, one text line tall (owner request
/// 2026-11-13: a stroke carries a single color — no pair).
const SWATCH_WIDTH: f32 = 10.0;

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

/// Everything the graph painter needs besides the rows themselves.
pub(super) struct GraphMetrics<'a> {
    pub(super) spans: &'a [history_tree::BranchSpan],
    pub(super) row_height: f32,
    pub(super) body_font: FontId,
    pub(super) text_color: Color32,
}

/// The graph itself: one non-interactive row per action, exactly one text
/// line tall — branch rails, fork connectors, the commit circle, the
/// "pencil" label and the single color swatch.
///
/// Geometry (owner request 2026-11-13): all rails and dots live inside a
/// gutter of `lanes * LANE_STEP` pixels; every connector is a straight
/// horizontal run, a quarter ellipse, then the branch's straight rail,
/// and ALL runs leaving one node share the node's center height (the runs
/// may overlap — not a problem). Branches never merge, so rails are ONE
/// segment each — no per-row stubs, no seams, nothing crossing the
/// parent's row. Labels hug their own node with an opaque backdrop.
pub(super) fn paint_graph(
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
    // Per-row labels (owner request 2026-11-13): kind + node id, measured
    // up front so the row width can fit the widest row.
    let labels: Vec<(String, f32)> = rows
        .iter()
        .map(|row| {
            let text = app.history_tree.node(row.id).map_or_else(
                || "pencil".to_string(),
                |node| format!("{} {}", kind_label(node.kind), node.id),
            );
            let width = ui
                .painter()
                .layout_no_wrap(text.clone(), metrics.body_font.clone(), metrics.text_color)
                .size()
                .x;
            (text, width)
        })
        .collect();

    // Gutter width from the deepest lane; a label sits right after its
    // OWN dot, so the widest row — not the gutter — sets the content
    // width.
    let lane_count = metrics
        .spans
        .iter()
        .map(|span| span.branch + 1)
        .max()
        .unwrap_or(1);
    let gutter_end = GRAPH_PAD_X + lane_count as f32 * LANE_STEP + GUTTER_TAIL_PAD;
    let label_offset = |branch: usize| -> f32 {
        GRAPH_PAD_X + branch as f32 * LANE_STEP + NODE_RADIUS + LABEL_GAP
    };
    let content_width = rows
        .iter()
        .zip(&labels)
        .map(|(row, (_, width))| label_offset(row.branch) + width + LABEL_GAP + SWATCH_WIDTH + 8.0)
        .fold(gutter_end, f32::max);
    let row_width = content_width.max(ui.available_width());

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
            // Root branch (the main line or a detached root): one straight
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
        // EVERY branch leaving one node exits at the SAME height — the
        // parent dot's center line (owner request 2026-11-13); sibling
        // runs may overlap, that is fine.
        let parent_display = display_of[parent_row];
        let parent_branch = rows[parent_display].branch;
        let parent_x = lane_x(parent_branch);
        let parent_y = rects[parent_display].center().y;
        let child_y = center_y(span.start_row);
        let sign = (child_y - parent_y).signum();
        // Clamped so the arc always bends toward the child row.
        let arc = ARC_RADIUS.min((child_y - parent_y).abs() * 0.5).max(1.0);
        let y_run = parent_y;

        // Straight horizontal run from the parent's rail to the arc.
        painter.line_segment(
            [egui::pos2(parent_x, y_run), egui::pos2(x - arc, y_run)],
            Stroke::new(2.0, color),
        );
        // Quarter ellipse: horizontal entry, vertical exit toward the row
        // of the fork's first node (kappa = best cubic fit for a quarter
        // circle; an ellipse of radius LANE_STEP/2 fits the lane width).
        let k = 0.552_3;
        let center = egui::pos2(x - arc, y_run + sign * arc);
        painter.add(Shape::CubicBezier(
            egui::epaint::CubicBezierShape::from_points_stroke(
                [
                    egui::pos2(center.x, y_run),
                    egui::pos2(center.x + k * arc, y_run),
                    egui::pos2(x, center.y + sign * (arc - k * arc)),
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
                egui::pos2(x, y_run + sign * arc),
                egui::pos2(x, center_y(span.end_row)),
            ],
            Stroke::new(2.0, color),
        );
    }

    // Pass 3: dots, labels, color swatches.
    let window_fill = ui.visuals().window_fill;
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
        let (label, label_width) = &labels[i];
        // The label hugs THIS row's node (owner request 2026-11-13); the
        // opaque backdrop keeps it readable over rails it crosses.
        let label_x = rect.left() + label_offset(row.branch);
        let backdrop = Rect::from_min_size(
            egui::pos2(label_x - 3.0, rect.top() + 1.0),
            egui::vec2(label_width + 6.0, row_height - 2.0),
        );
        painter.rect_filled(backdrop, 2.0, window_fill);
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
