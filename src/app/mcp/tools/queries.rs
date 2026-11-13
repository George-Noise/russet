//! Read-only MCP tools (feature `mcp`): they only LOOK at the app — the
//! layer list, the per-layer history tree, the graph layout the History
//! Tree window renders, the canvas snapshot and PNG export. Shared argument
//! parsing for the tree queries ([`parse_layer_arg`]) lives here too; the
//! dispatch table and reply glue are in the parent `tools.rs`.

use serde_json::{Value, json};

use crate::app::App;
use crate::file_io;
use crate::tools::Tool;

use super::{hex_color, tool_text};

pub(super) fn get_layers(app: &App) -> Result<Value, String> {
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

pub(super) fn get_history_tree(app: &App, args: &Value) -> Result<Value, String> {
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
pub(super) fn get_tree_layout(app: &App, args: &Value) -> Result<Value, String> {
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

pub(super) fn get_canvas_snapshot(app: &App) -> Result<Value, String> {
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

pub(super) fn export_png(app: &mut App, args: &Value) -> Result<Value, String> {
    let path = args
        .get("path")
        .and_then(Value::as_str)
        .ok_or_else(|| "path must be a string ending in .png".to_string())?;
    file_io::save_image_to(app.canvas.pixmap(), std::path::Path::new(path))?;
    Ok(tool_text(json!({ "saved": path }).to_string()))
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

pub(super) fn base64(bytes: &[u8]) -> String {
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
