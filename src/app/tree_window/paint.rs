//! The graph painter of the "History Tree" windows: branch rails, fork
//! connectors, dots, labels and the color chip — one row per action,
//! exactly one text line tall. Since owner request 2026-11-13 the row's
//! DOT is interactive: a LEFT click on it (the hit area is the dot plus a
//! small margin — never the label) asks for a head jump, returned to the
//! caller as [`NodeId`]. The chip is one fill for a single-color gesture
//! and splits into outline|fill halves when the gesture used both colors
//! (a Shape commit with the Fill+Outline paint on — owner request
//! 2026-11-16; the explicit third radio choice since 2026-11-20).
//!
//! Pure painting + the one hit-test; no state, no scroll logic. The window
//! chrome, the toolbar, the scroll bookkeeping and the per-window view
//! state live in the parent module ([`super`]); the data model and the
//! branch/lane assignment live in [`crate::history_tree`]. Child module of
//! `app::tree_window`, so it can read `App`'s private fields (it only ever
//! needs the shared layer stack).

use eframe::egui::{
    self, Align, Align2, Color32, CursorIcon, FontId, Id, PointerButton, Pos2, Rect, Sense, Shape,
    Stroke, Vec2,
};

use super::{Direction, Pin};
use crate::app::App;
use crate::app::tool_icons;
use crate::history_tree::{ActionKind, NodeId, Row, TreeNode};
use crate::tools::Tool;

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
/// The color chip of a row, one text line tall: ONE fill for a
/// single-color gesture; for a two-color one the SAME chip splits into
/// outline|fill halves (owner request 2026-11-16: a tool that used two
/// colors shows both — the 2026-11-13 single-color rule stays for the
/// single-color tools). The width never changes, so the row layout is
/// identical for both cases.
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

/// The dot's click hit area: the circle plus a small margin on every side
/// — a 7 px dot is too small a target — while staying clear of the label
/// that hugs the node (owner request 2026-11-13: the LABEL is never a
/// click target).
const DOT_HIT_MARGIN: f32 = 5.0;

/// Everything the graph painter needs besides the rows themselves.
pub(super) struct GraphMetrics<'a> {
    pub(super) spans: &'a [crate::history_tree::BranchSpan],
    pub(super) row_height: f32,
    pub(super) body_font: FontId,
    pub(super) text_color: Color32,
}

/// The word the row prints next to its dot. Owner request 2026-11-21: the
/// node id is GONE from every row — the reading order IS the id's context
/// and the dot's hover already carries the node. A resize row appends the
/// dialog's choices (owner request 2026-11-17: "canvas 800×600→1000×800
/// (TL)"), a crop row the selection's rect (owner request 2026-11-20).
fn row_label(node: &TreeNode) -> String {
    match (node.resize, node.crop) {
        (Some(info), _) => format!("{} {}", node.kind.label(), info.summary()),
        // CropInfo::summary already OPENS with the kind word ("crop …") —
        // with the label prepended the row read "crop crop …" once the
        // node id (which used to separate the two) left every row (owner
        // request 2026-11-21) — the summary alone carries both words.
        (None, Some(info)) => info.summary(),
        (None, None) => node.kind.label().to_string(),
    }
}

/// The tool whose ICON the row draws instead of the word (owner request
/// 2026-11-21: the pencil row shows the tool's emblem from the small
/// second atlas at [`tool_icons::TREE_ICON_PT`] — sized up from 12 pt the
/// same day, the `--atlas-test` rings showing how much room the row had
/// left). `None` = print [`row_label`]. New tools join
/// one row at a time — the owner's «пока что сделаем только для pencil».
fn row_icon(node: &TreeNode) -> Option<Tool> {
    match node.kind {
        ActionKind::Pencil => Some(Tool::Pencil),
        _ => None,
    }
}

/// The graph itself: one row per action, exactly one text line tall —
/// branch rails, fork connectors, the commit circle (LEFT-click = jump
/// there), the label (the pencil row draws the TOOL'S icon, owner request
/// 2026-11-21) and the single color swatch. Returns the id
/// of the node whose dot was clicked this frame, if any.
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
    window_id: Id,
) -> Option<NodeId> {
    if rows.is_empty() {
        ui.weak("No actions yet.");
        return None;
    }

    let row_height = metrics.row_height;
    let text_color = metrics.text_color;
    // Per-row labels (owner request 2026-11-13, simplified 2026-11-21):
    // the kind word — the node id is gone from EVERY row; a PENCIL row
    // reserves the small icon's width instead of a word (the icon replaces
    // it). A resize node appends the dialog's choices (owner request
    // 2026-11-17: "canvas 800×600→1000×800 (TL)" / "image … (bicubic)"), a
    // crop node the selection's rect (owner request 2026-11-20: "crop
    // 800×600→300×200 @(100,50)") — jumping a layer back beyond the node
    // shows what it did. Measured up front so the row width can fit the
    // widest row.
    let labels: Vec<(String, f32)> = rows
        .iter()
        .map(|row| {
            let node = app.layers.node(row.id);
            let icon_row = node.is_some_and(|node| row_icon(node).is_some());
            let text = node.map_or_else(|| "action".to_string(), row_label);
            let width = if icon_row {
                tool_icons::TREE_ICON_PT
            } else {
                ui.painter()
                    .layout_no_wrap(text.clone(), metrics.body_font.clone(), metrics.text_color)
                    .size()
                    .x
            };
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
                    app.layers.head() == Some(row.id),
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

    // Pass 3: dots (clickable), labels, color swatches.
    let window_fill = ui.visuals().window_fill;
    let mut clicked = None;
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

        // The dot is the LEFT-click switch (owner request 2026-11-13: the
        // canvas shows the clicked node's PNG). The hit area is the dot
        // plus DOT_HIT_MARGIN — never reaching the label. The id is salted
        // with the window id: clones show the same tree and egui ids must
        // not collide across windows.
        let hit_rect =
            Rect::from_center_size(center, Vec2::splat((NODE_RADIUS + DOT_HIT_MARGIN) * 2.0));
        let dot = ui.interact(
            hit_rect,
            window_id.with(("node_dot", row.id)),
            Sense::click(),
        );
        if dot.clicked_by(PointerButton::Primary) {
            clicked = Some(row.id);
        }
        if dot.hovered() {
            ui.ctx().set_cursor_icon(CursorIcon::PointingHand);
            ui.ctx().request_repaint();
        }
        let node = app.layers.node(row.id);
        // The hover text is node-aware: a resize or crop node explains what
        // that commit did to THIS layer (mode/sizes/anchor/filter, or the
        // crop rect).
        let detail = node.and_then(|node| {
            node.resize
                .map(|i| i.summary())
                .or(node.crop.map(|i| i.summary()))
        });
        let hover = detail.map_or_else(
            || "Left-click: switch the canvas to this node".to_string(),
            |summary| format!("Left-click: switch the canvas to this node\n{summary}"),
        );
        dot.on_hover_text(hover);

        let (label, label_width) = &labels[i];
        // The label hugs THIS row's node (owner request 2026-11-13); the
        // opaque backdrop keeps it readable over rails it crosses.
        let label_x = rect.left() + label_offset(row.branch);
        let backdrop = Rect::from_min_size(
            egui::pos2(label_x - 3.0, rect.top() + 1.0),
            egui::vec2(label_width + 6.0, row_height - 2.0),
        );
        painter.rect_filled(backdrop, 2.0, window_fill);
        // The pencil row draws the TOOL'S ICON from the small second atlas
        // (owner request 2026-11-21); the word stays the fallback for the
        // frame before the atlas exists (unreachable: ensure() runs before
        // the windows paint).
        let icon = node.and_then(row_icon);
        match icon.and_then(|tool| tool_icons::tree_texture_id(&app.tool_icons, tool)) {
            Some(texture) => {
                let icon_rect = Rect::from_min_size(
                    Pos2::new(label_x, cy - tool_icons::TREE_ICON_PT / 2.0),
                    Vec2::splat(tool_icons::TREE_ICON_PT),
                );
                painter.image(
                    texture,
                    icon_rect,
                    Rect::from_min_size(Pos2::ZERO, Vec2::splat(1.0)),
                    Color32::WHITE,
                );
            }
            None => {
                painter.text(
                    egui::pos2(label_x, cy),
                    Align2::LEFT_CENTER,
                    label,
                    metrics.body_font.clone(),
                    text_color,
                );
            }
        }
        if let Some(node) = node {
            // The Start root paints no swatch (it carries no stroke color);
            // a resize node has none either (owner request 2026-11-17 —
            // the choices live in the label instead).
            if node.kind != crate::history_tree::ActionKind::Start && node.resize.is_none() {
                let swatch = Rect::from_min_size(
                    egui::pos2(label_x + label_width + LABEL_GAP, rect.top() + 2.0),
                    egui::vec2(SWATCH_WIDTH, row_height - 4.0),
                );
                // A two-color gesture (a Shape commit with outline AND
                // fill) splits the chip into halves — outline left, fill
                // right (owner request 2026-11-16); a single-color gesture
                // stays one fill. The halves stay inside the border that
                // is drawn over them below, so the corners read the same.
                match node.color2 {
                    Some(fill) => {
                        let mid_x = swatch.center().x;
                        painter.rect_filled(
                            Rect::from_min_max(
                                swatch.left_top(),
                                egui::pos2(mid_x, swatch.bottom()),
                            ),
                            0.0,
                            node.color,
                        );
                        painter.rect_filled(
                            Rect::from_min_max(
                                egui::pos2(mid_x, swatch.top()),
                                swatch.right_bottom(),
                            ),
                            0.0,
                            fill,
                        );
                    }
                    None => {
                        painter.rect_filled(swatch, 2.0, node.color);
                    }
                }
                painter.rect_stroke(
                    swatch,
                    2.0,
                    Stroke::new(1.0, Color32::from_gray(90)),
                    egui::StrokeKind::Inside,
                );
            }
        }
    }

    clicked
}

#[cfg(test)]
mod tests;
