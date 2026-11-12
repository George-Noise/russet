#!/usr/bin/env python3
"""Minimal MCP stdio smoke client for russet (dev tool, feature `mcp`).

Speaks just enough JSON-RPC 2.0 / MCP to drive a running russet instance
spawned with piped stdin: initialize, then tools/call. Every call blocks
until the reply line arrives (one JSON message per stdout line).

Usage: mcp_smoke.py <path-to-russet-binary> [more args are ignored]
"""
import json
import subprocess
import sys

NEXT_ID = 0


class Russet:
    def __init__(self, binary):
        self.proc = subprocess.Popen(
            [binary], stdin=subprocess.PIPE, stdout=subprocess.PIPE,
            stderr=subprocess.DEVNULL, text=True, bufsize=1,
        )
        reply = self.call("initialize", {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": {"name": "russet-smoke", "version": "0"},
        })
        print("initialize ->", reply["result"]["serverInfo"])
        self.notify("notifications/initialized", {})

    def call(self, method, params):
        global NEXT_ID
        NEXT_ID += 1
        message = {"jsonrpc": "2.0", "id": NEXT_ID, "method": method, "params": params}
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()
        while True:
            line = self.proc.stdout.readline()
            if not line:
                raise RuntimeError("russet closed stdout")
            reply = json.loads(line)
            if reply.get("id") == NEXT_ID:
                if "error" in reply:
                    raise RuntimeError(f"{method}: {reply['error']}")
                return reply
            # ignore anything that is not our reply (there is none today)

    def notify(self, method, params):
        message = {"jsonrpc": "2.0", "method": method, "params": params}
        self.proc.stdin.write(json.dumps(message) + "\n")
        self.proc.stdin.flush()

    def tool(self, name, arguments=None):
        text = self.call("tools/call", {"name": name, "arguments": arguments or {}})
        result = text["result"]
        if result.get("isError"):
            raise RuntimeError(f"{name}: {result['content'][0]['text']}")
        return json.loads(result["content"][0]["text"])

    def layout(self):
        return self.tool("get_tree_layout")["rows"], self.tool("get_tree_layout")["spans"]

    def stroke(self, x0, y0, x1, y1, color="#222222"):
        return self.tool("add_stroke", {"points": [[x0, y0], [x1, y1]], "color": color})


def expect(condition, message):
    if not condition:
        raise AssertionError("FAIL: " + message)
    print("ok -", message)


def main():
    binary = sys.argv[1]
    russet = Russet(binary)

    tools = russet.call("tools/list", {})["result"]["tools"]
    expect(len(tools) == 9, f"tools/list offers 9 tools (got {len(tools)})")

    # --- Scenario A: a plain undo must NOT fork anything (owner fix 1) ---
    for i in range(3):
        russet.stroke(50 + 40 * i, 60, 90 + 40 * i, 90)
    rows, spans = russet.layout()
    expect(len(spans) == 1 and all(r["branch"] == 0 for r in rows),
           "3 strokes: one straight lane-0 rail")
    russet.tool("undo")
    rows, spans = russet.layout()
    expect(len(spans) == 1 and all(r["branch"] == 0 for r in rows),
           "after plain undo the undone tail STAYS on lane 0")
    expect(sum(1 for r in rows if r["is_head"]) == 1, "exactly one head row")
    russet.tool("redo")
    rows, _ = russet.layout()
    expect(all(r["branch"] == 0 for r in rows), "redo returns along the same rail")

    # --- Scenario B: drawing after undo IS what forks (owner fix 1) ---
    russet.tool("undo")
    russet.stroke(300, 200, 360, 260, "#cc3333")
    rows, spans = russet.layout()
    tail = [r for r in rows if r["id"] == 3][0]
    expect(tail["branch"] == 1 and tail["fork"],
           "the abandoned tail peels off to lane 1 only now")

    # --- Scenario C: newer branches hug the trunk, late forks reuse lanes ---
    russet.tool("set_active_layer", {"index": 1})
    # a(1) b(2); undo; c(3) forks the root; d(4) e(5); undo; f(6) forks d
    russet.stroke(40, 40, 60, 60)
    russet.stroke(70, 40, 90, 60)
    russet.tool("undo")
    russet.stroke(40, 100, 60, 120, "#3388cc")
    russet.stroke(100, 40, 120, 60)
    russet.stroke(130, 40, 150, 60)
    russet.tool("undo")
    russet.stroke(160, 40, 180, 60, "#33aa55")
    rows, spans = russet.layout()
    branches = [r["branch"] for r in rows]
    expect(branches == [0, 1, 0, 0, 1, 0],
           f"late fork reuses lane 1 below the early branch (got {branches})")
    parent_rows = {s["start_row"]: s["parent_row"] for s in spans if s["parent_row"] is not None}
    expect(sorted(parent_rows.values()) == [0, 3],
           f"both branch spans remember their fork parents (got {parent_rows})")

    # --- Scenario D: recency — the newest fork takes the innermost lane ---
    russet.tool("set_active_layer", {"index": 2})
    russet.stroke(40, 40, 60, 60)
    russet.stroke(70, 40, 90, 60)
    russet.stroke(100, 40, 120, 60)
    russet.tool("undo")
    russet.tool("undo")
    russet.stroke(40, 100, 60, 120, "#3388cc")
    russet.tool("undo")
    russet.stroke(70, 100, 90, 120, "#33aa55")
    russet.tool("undo")
    rows, _ = russet.layout()
    branches = [r["branch"] for r in rows]
    expect(branches == [0, 2, 2, 1, 0],
           f"newest branch on lane 1, older ones pushed right (got {branches})")

    print("ALL MCP SCENARIOS GREEN")

    # Leave the app running for screenshots; the caller kills it.
    return russet


if __name__ == "__main__":
    main()
