//! Tool handlers of the MCP server (feature `mcp` only). Every `tools/call`
//! lands here and runs against the live `App` inside the UI frame — through
//! the SAME code paths the hand gestures use, so pixels, the per-layer PNG
//! snapshot history and the history tree can never disagree.
//!
//! Pure helpers (argument parsing, color hex, base64 and the frame-free
//! [`apply_stroke`]) live at the bottom; their tests are in `tools/tests.rs`.

use eframe::egui::{Color32, Pos2};
use serde_json::{Value, json};
use tiny_skia::Pixmap;

use crate::app::App;
use crate::file_io;
use crate::history::ActionLog;
use crate::history_tree::{ActionKind, LayerTree, NodeId};
use crate::layers::encode_snapshot;
use crate::tools::{Tool, draw_segment};

#[cfg(test)]
mod tests;

/// Serves one `tools/call`. Tool-level failures (bad arguments, bad tool)
/// come back as results with `isError: true` — the MCP way for tool errors;
/// the JSON-RPC error object stays reserved for the protocol layer.
pub(crate) fn tool_call(app: &mut App, params: &Value) -> Value {
    let name = params
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let args = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let outcome = match name {
        "get_layers" => get_layers(app),
        "get_history_tree" => get_history_tree(app, &args),
        "get_tree_layout" => get_tree_layout(app, &args),
        "add_stroke" => add_stroke(app, &args),
        "undo" => undo(app),
        "redo" => redo(app),
        "set_active_layer" => set_active_layer(app, &args),
        "get_canvas_snapshot" => get_canvas_snapshot(app),
        "export_png" => export_png(app, &args),
        _ => Err(format!("Unknown tool: {name}")),
    };
    outcome.unwrap_or_else(tool_error)
}

fn tool_text(json_text: String) -> Value {
    json!({ "content": [{ "type": "text", "text": json_text }] })
}

fn tool_error(message: String) -> Value {
    json!({ "content": [{ "type": "text", "text": message }], "isError": true })
}

fn get_layers(app: &App) -> Result<Value, String> {
    let layers: Vec<Value> = (0..app.layers.count())
        .map(|index| {
            json!({
                "index": index,
                "name": app.layers.layer(index).map(|layer| layer.name().to_string()).unwrap_or_default(),
                "visible": app.layers.layer(index).map(|layer| layer.visible()).unwrap_or(false),
            })
        })
        .collect();
    Ok(tool_text(
        json!({
            "canvas": { "width": app.canvas.width, "height": app.canvas.height },
            "active_layer": app.layers.active_layer(),
            "layers": layers,
            "tool": tool_name(app.tools.tool),
            "brush_size": app.tools.size,
            "primary_color": hex_color(app.tools.primary),
            "dirty": app.dirty,
        })
        .to_string(),
    ))
}

fn tool_name(tool: Tool) -> &'static str {
    match tool {
        Tool::Pencil => "pencil",
        Tool::Brush => "brush",
        Tool::Eraser => "eraser",
        Tool::Eyedropper => "eyedropper",
        Tool::Line => "line",
        Tool::Shape => "shape",
        Tool::Text => "text",
    }
}

fn get_history_tree(app: &App, args: &Value) -> Result<Value, String> {
    let layer = parse_layer_arg(app, args)?;
    let nodes = app.layers.layer_nodes(layer).unwrap_or(&[]);
    let head = app.layers.layer_head(layer);
    let name = app
        .layers
        .layer(layer)
        .map(|l| l.name().to_string())
        .unwrap_or_default();
    let tree: Vec<Value> = nodes
        .iter()
        .map(|node| {
            let children: Vec<Value> = nodes
                .iter()
                .filter(|other| other.parent == Some(node.id))
                .map(|child| json!(child.id))
                .collect();
            json!({
                "id": node.id,
                "parent": node.parent,
                "kind": node.kind.label(),
                "color": hex_color(node.color),
                "is_head": Some(node.id) == head,
                "children": children,
            })
        })
        .collect();
    Ok(tool_text(
        json!({
            "layer": layer,
            "name": name,
            "head": head,
            "nodes": tree,
        })
        .to_string(),
    ))
}

fn parse_layer_arg(app: &App, args: &Value) -> Result<usize, String> {
    let count = app.layers.count();
    match args.get("layer") {
        None | Some(Value::Null) => Ok(app.layers.active_layer()),
        Some(value) => {
            let index = value
                .as_u64()
                .ok_or_else(|| "layer must be an integer".to_string())?
                as usize;
            if index >= count {
                return Err(format!("layer is out of range 0..{count}"));
            }
            Ok(index)
        }
    }
}

/// The LAYOUT the History Tree window renders (owner request 2026-11-13,
/// "add more MCP if it helps"): rows in creation order with their lane,
/// fork and head flags, plus one rail span per branch. Lane 0 is the
/// current timeline (head path + redo chain); a plain undo moves NO row
/// out of lane 0 — only a real fork does.
fn get_tree_layout(app: &App, args: &Value) -> Result<Value, String> {
    let layer = parse_layer_arg(app, args)?;
    let Some(layout) = app.layers.layer_layout(layer) else {
        return Err(format!("layer is out of range 0..{}", app.layers.count()));
    };
    let name = app
        .layers
        .layer(layer)
        .map(|l| l.name().to_string())
        .unwrap_or_default();
    let rows: Vec<Value> = layout
        .rows
        .iter()
        .map(|row| {
            json!({
                "id": row.id,
                "parent": row.parent,
                "branch": row.branch,
                "fork": row.fork,
                "is_head": row.is_head,
            })
        })
        .collect();
    let spans: Vec<Value> = layout
        .spans
        .iter()
        .map(|span| {
            json!({
                "branch": span.branch,
                "start_row": span.start_row,
                "end_row": span.end_row,
                "parent_row": span.parent_row,
            })
        })
        .collect();
    Ok(tool_text(
        json!({
            "layer": layer,
            "name": name,
            "head": app.layers.layer_head(layer),
            "rows": rows,
            "spans": spans,
        })
        .to_string(),
    ))
}

fn add_stroke(app: &mut App, args: &Value) -> Result<Value, String> {
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
pub(crate) fn apply_stroke(
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

fn undo(app: &mut App) -> Result<Value, String> {
    let before = app.layers.head();
    app.undo();
    step_reply(app, before)
}

fn redo(app: &mut App) -> Result<Value, String> {
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

fn set_active_layer(app: &mut App, args: &Value) -> Result<Value, String> {
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

fn get_canvas_snapshot(app: &App) -> Result<Value, String> {
    let png = file_io::encode_png(app.canvas.pixmap())?;
    let meta = format!(
        "{}x{} PNG, {} bytes",
        app.canvas.width,
        app.canvas.height,
        png.len()
    );
    Ok(json!({
        "content": [
            { "type": "image", "data": base64(&png), "mimeType": "image/png" },
            { "type": "text", "text": meta },
        ]
    }))
}

fn export_png(app: &mut App, args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "path must be a string ending in .png".to_string())?;
    file_io::save_image_to(app.canvas.pixmap(), std::path::Path::new(path))?;
    Ok(tool_text(json!({ "saved": path }).to_string()))
}

fn parse_points(value: &Value) -> Result<Vec<Pos2>, String> {
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

fn parse_hex_color(text: &str) -> Option<Color32> {
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

fn hex_color(color: Color32) -> String {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    format!("#{r:02x}{g:02x}{b:02x}")
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = u32::from(chunk[0]);
        let b1 = u32::from(chunk.get(1).copied().unwrap_or(0));
        let b2 = u32::from(chunk.get(2).copied().unwrap_or(0));
        let triple = (b0 << 16) | (b1 << 8) | b2;
        let pick = |shift: u32| BASE64_ALPHABET[(triple >> shift) as usize & 0b11_1111] as char;
        out.push(pick(18));
        out.push(pick(12));
        out.push(if chunk.len() > 1 { pick(6) } else { '=' });
        out.push(if chunk.len() > 2 { pick(0) } else { '=' });
    }
    out
}

/// The `tools/list` payload. Descriptions are written for an agent: they
/// name the coordinate space and promise the gesture equivalence.
pub(crate) fn tool_declarations() -> Value {
    json!([
        {
            "name": "get_layers",
            "description": "Canvas size, the real layers (index, name, visibility; index 0 is the composite bottom), the active one, current tool and colors.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "get_history_tree",
            "description": "The fork history of one layer: nodes (id, parent, kind, color, children) and the head, git-style. Every node carries a full PNG snapshot of that layer's canvas after the action; every drawing gesture is a node.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "layer": { "type": "integer", "description": "Layer index; defaults to the active layer." }
                }
            }
        },
        {
            "name": "get_tree_layout",
            "description": "The graph LAYOUT the History Tree window renders for one layer: rows in creation order (id, parent, branch lane, fork flag, is_head) plus one rail span per branch (branch, start_row, end_row, parent_row; rows are 0-based creation indexes). Lane 0 is the current timeline (head path + redo chain) — a plain undo forks nothing; use it to verify forks and lane placement after undo/redo/add_stroke.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "layer": { "type": "integer", "description": "Layer index; defaults to the active layer." }
                }
            }
        },
        {
            "name": "add_stroke",
            "description": "Draw a 1px pencil polyline in CANVAS pixel coordinates on the ACTIVE layer. Commits a real history node (the post-stroke canvas as its PNG snapshot) — identical to a hand gesture; strokes made after undo fork the tree exactly like in the GUI.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "points": {
                        "type": "array",
                        "minItems": 1,
                        "maxItems": 100000,
                        "items": { "type": "array", "items": { "type": "number" } },
                        "description": "[x, y] pairs in canvas pixels."
                    },
                    "color": { "type": "string", "description": "#rgb or #rrggbb; defaults to the current primary color." }
                },
                "required": ["points"]
            }
        },
        {
            "name": "undo",
            "description": "Same as Ctrl+Z on the active layer: restores the parent node's PNG canvas and moves the tree head to it.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "redo",
            "description": "Same as Ctrl+Y: steps forward again (prefers the branch the last undo left).",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "set_active_layer",
            "description": "Switch the global active layer (same as clicking a row in the Layers window; the History Tree windows follow).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "description": "Layer index" }
                },
                "required": ["index"]
            }
        },
        {
            "name": "get_canvas_snapshot",
            "description": "The current canvas bitmap as a PNG (image content) — lets the agent LOOK at its drawing.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "export_png",
            "description": "Save the current canvas bitmap to a PNG file path on this machine.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }
        }
    ])
}
