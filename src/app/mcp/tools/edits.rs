//! Mutating MCP tools (feature `mcp`): strokes, undo/redo and layer
//! management — the exact code paths the hand gestures use, so an MCP
//! mutation is indistinguishable from the same GUI action everywhere
//! downstream (pixels, PNG snapshots, the history tree). Frame-free pure
//! helpers ([`apply_stroke`], [`parse_points`], [`parse_hex_color`]) live
//! here too; the dispatch table and reply glue are in the parent
//! `tools.rs`.

use eframe::egui::{Color32, Pos2};
use serde_json::{Value, json};
use tiny_skia::Pixmap;

use crate::app::App;
use crate::history::ActionLog;
use crate::history_tree::{ActionKind, LayerTree, NodeId};
use crate::layers::encode_snapshot;
use crate::tools::{Tool, draw_segment};

use super::{hex_color, tool_text};

pub(super) fn add_stroke(app: &mut App, args: &Value) -> Result<Value, String> {
    let points = parse_points(args.get("points").unwrap_or(&Value::Null))?;
    let color = match args.get("color").and_then(Value::as_str) {
        Some(text) => {
            parse_hex_color(text).ok_or_else(|| "color must be #rgb or #rrggbb".to_string())?
        }
        None => app.tools.primary,
    };
    let next_id = app.layers.next_node_id();
    let node = {
        let (pixmap, tree) = app.layers.active_parts();
        apply_stroke(pixmap, tree, &mut app.action_log, next_id, &points, color)
    };
    if node.is_some() {
        app.layers.advance_next_node_id();
        app.refresh_composite();
        app.dirty = true;
    }
    Ok(tool_text(
        json!({
            "added": node.is_some(),
            "node": node,
            "points": points.len(),
            "color": hex_color(color),
        })
        .to_string(),
    ))
}

/// The exact code path of a real pencil gesture (`App::handle_stroke`),
/// extracted frame-free: the pixels land on the given layer buffer FIRST,
/// then the post-stroke buffer becomes the node's PNG snapshot at the head
/// — so an MCP stroke is indistinguishable from a hand stroke everywhere
/// downstream (undo, the tree, forks after undo). The caller composites
/// the layer into the display buffer and consumes the node id.
pub(super) fn apply_stroke(
    pixmap: &mut Pixmap,
    tree: &mut LayerTree,
    log: &mut ActionLog,
    next_id: NodeId,
    points: &[Pos2],
    color: Color32,
) -> Option<NodeId> {
    let &first = points.first()?;
    log.push("Pencil stroke (MCP)");
    let mut last = first;
    let _ = draw_segment(pixmap, Tool::Pencil, 1.0, color, first, first);
    for &point in points.iter().skip(1) {
        if point != last {
            let _ = draw_segment(pixmap, Tool::Pencil, 1.0, color, last, point);
            last = point;
        }
    }
    let snapshot = encode_snapshot(pixmap);
    tree.record_action(next_id, ActionKind::Pencil, color, snapshot);
    Some(next_id)
}

pub(super) fn undo(app: &mut App) -> Result<Value, String> {
    let before = app.layers.head();
    app.undo();
    step_reply(app, before)
}

pub(super) fn redo(app: &mut App) -> Result<Value, String> {
    let before = app.layers.head();
    app.redo();
    step_reply(app, before)
}

fn step_reply(app: &mut App, before: Option<NodeId>) -> Result<Value, String> {
    Ok(tool_text(
        json!({
            "changed": app.layers.head() != before,
            "head": app.layers.head(),
        })
        .to_string(),
    ))
}

/// The exact code path of the Layers window "+ Add layer" button: a
/// transparent layer on top, active, with its own fresh history tree.
pub(super) fn add_layer(app: &mut App) -> Result<Value, String> {
    let (width, height) = (app.canvas.width, app.canvas.height);
    let index = app.layers.add_layer(width, height);
    app.switch_active_layer(index);
    app.refresh_composite();
    app.dirty = true;
    let name = app.layers.active_layer_name().to_string();
    Ok(tool_text(
        json!({ "index": index, "name": name, "active_layer": app.layers.active_layer() })
            .to_string(),
    ))
}

pub(super) fn set_active_layer(app: &mut App, args: &Value) -> Result<Value, String> {
    let count = app.layers.count();
    let index = args
        .get("index")
        .and_then(Value::as_u64)
        .ok_or_else(|| "index must be an integer".to_string())? as usize;
    if index >= count {
        return Err(format!("index is out of range 0..{count}"));
    }
    app.switch_active_layer(index);
    Ok(tool_text(
        json!({ "active_layer": app.layers.active_layer() }).to_string(),
    ))
}

pub(super) fn parse_points(value: &Value) -> Result<Vec<Pos2>, String> {
    let array = value
        .as_array()
        .ok_or_else(|| "points must be an array of [x, y] pairs".to_string())?;
    if array.is_empty() {
        return Err("points must not be empty".to_string());
    }
    if array.len() > 100_000 {
        return Err("too many points (the cap is 100_000)".to_string());
    }
    array
        .iter()
        .map(|pair| {
            let pair = pair
                .as_array()
                .ok_or_else(|| "each point must be a [x, y] array".to_string())?;
            if pair.len() != 2 {
                return Err("each point must have exactly two numbers".to_string());
            }
            let x = pair[0]
                .as_f64()
                .ok_or_else(|| "point coordinates must be numbers".to_string())?;
            let y = pair[1]
                .as_f64()
                .ok_or_else(|| "point coordinates must be numbers".to_string())?;
            if !x.is_finite() || !y.is_finite() {
                return Err("point coordinates must be finite".to_string());
            }
            Ok(Pos2::new(x as f32, y as f32))
        })
        .collect()
}

pub(super) fn parse_hex_color(text: &str) -> Option<Color32> {
    let hex = text.strip_prefix('#')?;
    let expanded = match hex.len() {
        3 => hex.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => hex.to_string(),
        _ => return None,
    };
    let value = u32::from_str_radix(&expanded, 16).ok()?;
    Some(Color32::from_rgb(
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    ))
}
