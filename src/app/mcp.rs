//! MCP (Model Context Protocol) stdio server — owner request 2026-11-12,
//! compiled ONLY under the `mcp` cargo feature (`cargo build --features mcp`).
//!
//! Purpose: a development side channel — an AI agent (the one building this
//! app) drives the running GUI over JSON-RPC 2.0 on stdin/stdout: it draws
//! strokes, inspects the visual history tree, undoes and LOOKS at the canvas.
//! A user-with-agent scenario stays a possibility, not a promise.
//!
//! Shape (owner-approved 2026-11-12, chat):
//! - a secondary thread blocking-reads stdin and pushes parsed events into an
//!   mpsc queue; the MAIN thread stays unblocked and drains the queue with
//!   `try_recv` at the top of every frame (`App::mcp_poll`), executing tool
//!   calls against the live `App` state — pixels, the snapshot history and
//!   the visual tree can only be touched there;
//! - replies go back through a second channel to a writer thread, one JSON
//!   message per stdout line;
//! - `serde_json` is an OPTIONAL dependency: without the `mcp` feature none
//!   of this module, neither thread, nor the library exists in the build.
//!
//! Start conditions: `--mcp` on the command line, or stdin already piped (how
//! MCP hosts spawn binaries). A plain terminal launch stays GUI-only so the
//! server never eats interactive keyboard input.

use std::io::{BufRead, IsTerminal, Write};
use std::sync::mpsc;

use eframe::egui;
use serde_json::{Value, json};

use super::App;

mod tools;

#[cfg(test)]
mod tests;

/// Protocol versions this server speaks; a client asking for an unknown
/// version gets [`PROTOCOL_VERSION_LATEST`] (the host decides if that is ok).
const PROTOCOL_VERSIONS: [&str; 3] = ["2024-11-05", "2025-03-26", "2025-06-18"];
const PROTOCOL_VERSION_LATEST: &str = "2025-06-18";

/// One event forwarded from the reader thread to the UI frame.
struct Event {
    id: Option<Value>,
    method: String,
    params: Value,
}

/// Splits one JSON-RPC message into the fields the dispatcher needs.
fn parse_message(message: Value) -> Event {
    Event {
        id: message.get("id").cloned(),
        method: message
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        params: message.get("params").cloned().unwrap_or(Value::Null),
    }
}

/// The handle living in `App`: the event queue, the reply channel and a
/// served counter for the menu-bar badge.
pub(crate) struct McpServer {
    events: mpsc::Receiver<Event>,
    outbox: mpsc::Sender<String>,
    served: u64,
}

impl McpServer {
    /// Starts the stdio pump only when this instance is meant to serve:
    /// explicit `--mcp` (interactive terminals) or a piped stdin (MCP hosts).
    pub(crate) fn maybe_spawn(ctx: egui::Context) -> Option<Self> {
        let flagged = std::env::args().skip(1).any(|arg| arg == "--mcp");
        let piped = !std::io::stdin().is_terminal();
        if flagged || piped {
            Some(Self::spawn(ctx))
        } else {
            None
        }
    }

    fn spawn(ctx: egui::Context) -> Self {
        let (event_tx, event_rx) = mpsc::channel::<Event>();
        let (out_tx, out_rx) = mpsc::channel::<String>();

        // Writer thread: one JSON-RPC message per stdout line, flushed.
        let _ = std::thread::Builder::new()
            .name("mcp-stdout".into())
            .spawn(move || {
                let stdout = std::io::stdout();
                for message in out_rx {
                    let mut lock = stdout.lock();
                    let written = writeln!(lock, "{message}");
                    if written.and_then(|()| lock.flush()).is_err() {
                        break;
                    }
                }
            });

        // Reader thread: blocking stdin lines → events → wake the UI. The
        // main thread never blocks: it only polls the queue with try_recv.
        let out_reader = out_tx.clone();
        let _ = std::thread::Builder::new()
            .name("mcp-stdin".into())
            .spawn(move || {
                let stdin = std::io::stdin();
                for line in stdin.lock().lines() {
                    let Ok(line) = line else { break };
                    if line.trim().is_empty() {
                        continue;
                    }
                    let outcome = match serde_json::from_str::<Value>(&line) {
                        Ok(message) => event_tx.send(parse_message(message)).is_ok(),
                        Err(_) => out_reader
                            .send(error_reply(Value::Null, -32700, "Parse error"))
                            .is_ok(),
                    };
                    if !outcome {
                        break; // the app is gone
                    }
                    ctx.request_repaint();
                }
            });

        Self {
            events: event_rx,
            outbox: out_tx,
            served: 0,
        }
    }

    /// Drains every pending event in the UI frame; tool calls run
    /// synchronously against the live `App` and reply immediately.
    fn poll(&mut self, app: &mut App) {
        while let Ok(event) = self.events.try_recv() {
            self.served += 1;
            if let Some(reply) = dispatch(app, event) {
                let _ = self.outbox.send(reply);
            }
        }
    }
}

impl App {
    /// Frame hook: take the server out (tool calls need `&mut App`), drain
    /// the queue, put it back. Never blocks — `try_recv` only.
    pub(crate) fn mcp_poll(&mut self) {
        let Some(mut server) = self.mcp.take() else {
            return;
        };
        server.poll(self);
        self.mcp = Some(server);
    }

    /// Menu-bar chip: shows whether the server is running and how many
    /// requests it has served so far.
    pub(crate) fn mcp_badge(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        let (text, tip) = match &self.mcp {
            Some(server) => (
                format!("MCP on ({})", server.served),
                "MCP stdio server is running: an AI agent is connected to this instance."
                    .to_string(),
            ),
            None => (
                "MCP idle".to_string(),
                "MCP is compiled in but not serving. Run with --mcp or with a piped stdin."
                    .to_string(),
            ),
        };
        ui.add(
            egui::Label::new(egui::RichText::new(text).small().monospace())
                .sense(egui::Sense::hover()),
        )
        .on_hover_text(tip);
    }
}

/// JSON-RPC dispatch: notifications never reply; unknown methods get
/// -32601; everything else is answered with a result envelope.
fn dispatch(app: &mut App, event: Event) -> Option<String> {
    if event.method.starts_with("notifications/") {
        return None;
    }
    let id = event.id?;
    let outcome = match event.method.as_str() {
        "initialize" => Ok(initialize_result(&event.params)),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({ "tools": tools::tool_declarations() })),
        "tools/call" => Ok(tools::tool_call(app, &event.params)),
        _ => Err((-32601, "Method not found".to_string())),
    };
    Some(envelope(id, outcome))
}

/// Wraps an outcome into the wire format: a result or an error object.
fn envelope(id: Value, outcome: Result<Value, (i32, String)>) -> String {
    match outcome {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }).to_string(),
        Err((code, message)) => error_reply(id, code, &message),
    }
}

fn error_reply(id: Value, code: i32, message: &str) -> String {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } }).to_string()
}

/// MCP lifecycle: echo a supported protocol version, else offer the newest.
fn initialize_result(params: &Value) -> Value {
    let requested = params.get("protocolVersion").and_then(Value::as_str);
    let version = requested
        .filter(|version| PROTOCOL_VERSIONS.contains(version))
        .unwrap_or(PROTOCOL_VERSION_LATEST);
    json!({
        "protocolVersion": version,
        "capabilities": { "tools": {} },
        "serverInfo": {
            "name": "russet",
            "title": "Russet Paint",
            "version": env!("CARGO_PKG_VERSION"),
        },
    })
}
