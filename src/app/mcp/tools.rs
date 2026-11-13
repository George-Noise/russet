//! Tool surface of the MCP server (feature `mcp` only). This file is the
//! MAP of the surface: the dispatch table ([`tool_call`]) and the
//! `tools/list` payload ([`tool_declarations`]) — the two places to read
//! (and edit) when a tool is added. Handler implementations are themed
//! into `tools/queries.rs` (read-only) and `tools/edits.rs` (mutating);
//! the reply glue every handler shares ([`tool_text`], [`tool_error`],
//! [`hex_color`]) lives here. Tests: `tools/tests.rs`.
//!
//! Every `tools/call` runs against the live `App` inside the UI frame —
//! through the SAME code paths the hand gestures use, so pixels, the
//! per-layer PNG snapshot history and the history tree can never disagree.

use eframe::egui::Color32;
use serde_json::{Value, json};

use crate::app::App;

mod edits;
mod queries;
#[cfg(test)]
mod tests;

use edits::{add_layer, add_stroke, redo, set_active_layer, undo};
use queries::{export_png, get_canvas_snapshot, get_history_tree, get_layers, get_tree_layout};

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
        "add_layer" => add_layer(app),
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

fn hex_color(color: Color32) -> String {
    let [r, g, b, _] = color.to_srgba_unmultiplied();
    format!("#{r:02x}{g:02x}{b:02x}")
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
            "name": "add_layer",
            "description": "\"+ Add layer\" from the Layers window: appends a transparent layer on top, makes it active and gives it a fresh history tree. The agent needs this before set_active_layer can address the new index.",
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
