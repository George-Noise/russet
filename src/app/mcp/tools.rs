//! Tool handlers of the MCP server (feature `mcp` only). Every `tools/call`
//! lands here and runs against the live `App` inside the UI frame — through
//! the SAME code paths the hand gestures use, so pixels, the snapshot undo
//! history and the visual history tree can never disagree.
//!
//! Pure helpers (argument parsing, color hex, base64 and the frame-free
//! [`apply_stroke`]) live at the bottom; their tests are in `tools/tests.rs`.

use eframe::egui::{Color32, Pos2};
use serde_json::{Value, json};
use tiny_skia::Pixmap;

use crate::app::App;
use crate::file_io;
use crate::history::{ActionLog, History};
use crate::history_tree::{HistoryTree, LAYER_NAMES, NodeId, PSEUDO_LAYERS};
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
    let layers: Vec<Value> = (0..PSEUDO_LAYERS)
        .map(|index| json!({ "index": index, "name": LAYER_NAMES[index] }))
        .collect();
    Ok(tool_text(
        json!({
            "canvas": { "width": app.canvas.width, "height": app.canvas.height },
            "active_layer": app.history_tree.active_layer(),
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
    let layer = match args.get("layer") {
        None | Some(Value::Null) => app.history_tree.active_layer(),
        Some(value) => {
            let index = value
                .as_u64()
                .ok_or_else(|| "layer must be an integer 0..2".to_string())?
                as usize;
            if index >= PSEUDO_LAYERS {
                return Err(format!("layer is out of range 0..{PSEUDO_LAYERS}"));
            }
            index
        }
    };
    let nodes = app.history_tree.layer_nodes(layer).unwrap_or(&[]);
    let head = app.history_tree.layer_head(layer);
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
                "kind": "pencil",
                "color": hex_color(node.color),
                "is_head": Some(node.id) == head,
                "children": children,
            })
        })
        .collect();
    Ok(tool_text(
        json!({
            "layer": layer,
            "name": LAYER_NAMES[layer],
            "head": head,
            "nodes": tree,
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
    let node = apply_stroke(
        app.canvas.pixmap_mut(),
        &mut app.history,
        &mut app.history_tree,
        &mut app.action_log,
        &points,
        color,
    );
    app.repaint_full();
    app.dirty = true;
    Ok(tool_text(
        json!({
            "added": true,
            "node": node,
            "points": points.len(),
            "color": hex_color(color),
        })
        .to_string(),
    ))
}

/// The exact code path of a real pencil gesture (`App::handle_stroke`),
/// extracted frame-free: an undo snapshot FIRST, then the pixels, then the
/// visual node at the head — so an MCP stroke is indistinguishable from a
/// hand stroke everywhere downstream (undo, the tree, forks after undo).
pub(crate) fn apply_stroke(
    pixmap: &mut Pixmap,
    history: &mut History,
    tree: &mut HistoryTree,
    log: &mut ActionLog,
    points: &[Pos2],
    color: Color32,
) -> Option<NodeId> {
    let &first = points.first()?;
    history.push_snapshot(pixmap.clone());
    log.push("Pencil stroke (MCP)");
    let mut last = first;
    let _ = draw_segment(pixmap, Tool::Pencil, 1.0, color, first, first);
    for &point in points.iter().skip(1) {
        if point != last {
            let _ = draw_segment(pixmap, Tool::Pencil, 1.0, color, last, point);
            last = point;
        }
    }
    tree.record_stroke(color);
    tree.head()
}

fn undo(app: &mut App) -> Result<Value, String> {
    let before = app.history_tree.head();
    app.undo();
    step_reply(app, before)
}

fn redo(app: &mut App) -> Result<Value, String> {
    let before = app.history_tree.head();
    app.redo();
    step_reply(app, before)
}

fn step_reply(app: &mut App, before: Option<NodeId>) -> Result<Value, String> {
    Ok(tool_text(
        json!({
            "changed": app.history_tree.head() != before,
            "head": app.history_tree.head(),
        })
        .to_string(),
    ))
}

fn set_active_layer(app: &mut App, args: &Value) -> Result<Value, String> {
    let index = args
        .get("index")
        .and_then(Value::as_u64)
        .ok_or_else(|| "index must be an integer 0..2".to_string())? as usize;
    if index >= PSEUDO_LAYERS {
        return Err(format!("index is out of range 0..{PSEUDO_LAYERS}"));
    }
    app.history_tree.set_active_layer(index);
    Ok(tool_text(
        json!({ "active_layer": app.history_tree.active_layer() }).to_string(),
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
            "description": "Canvas size, the three pseudo-layers, the active one, current tool and colors.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "get_history_tree",
            "description": "The visual fork history of one pseudo-layer: nodes (id, parent, kind, color, children) and the head, git-style. Pencil strokes only.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "layer": { "type": "integer", "description": "Layer index 0..2; defaults to the active layer." }
                }
            }
        },
        {
            "name": "add_stroke",
            "description": "Draw a 1px pencil polyline in CANVAS pixel coordinates. Commits a real undo snapshot and a real history-tree node — identical to a hand gesture; strokes made after undo fork the tree exactly like in the GUI.",
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
            "description": "Same as Ctrl+Z: steps the pixels back and moves the tree head to the parent.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "redo",
            "description": "Same as Ctrl+Y: steps forward again.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "set_active_layer",
            "description": "Switch the global active pseudo-layer (same as the tree-window switcher).",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "index": { "type": "integer", "description": "0..2" }
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
