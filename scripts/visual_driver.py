#!/usr/bin/env python3
"""Scenario-driven verification for russet over MCP (dev tool, feature `mcp`).

Replaces the ad-hoc per-task scripts: a scenario is a small function in the
SCENARIOS registry, every assertion prints ONE text line, and the exit code is
the verdict. The agent never needs to LOOK at pixels:

  - tree shape  -> get_tree_layout JSON (rows/branches/spans),
  - committed pixels -> get_canvas_snapshot PNG, sampled in CANVAS space with
    PIL (no fragile screen coordinates),
  - screenshots -> a side channel for the OWNER, written only with --shots
    (the owner reviews builds personally; default runs write nothing).

Usage: visual_driver.py <path-to-russet-binary> [--shots]
Run under Xvfb (or any display): the binary opens a window; MCP comes through
piped stdin (mcp_smoke.Russet). New scenario = a ~10-line function + one line
in SCENARIOS.
"""
import base64
import io
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(__file__))
from mcp_smoke import Russet  # noqa: E402

from PIL import Image  # noqa: E402

OUT = os.environ.get("RUSSET_DRIVER_OUT", "/home/z/my-project/download")
DISPLAY = os.environ.get("DISPLAY", ":99")


class Driver(Russet):
    """Russet plus the two calls the smoke client does not need."""

    def layout(self):
        # override: ONE get_tree_layout call (the smoke client calls it twice)
        data = self.tool("get_tree_layout")
        return data["rows"], data["spans"]

    def snapshot(self):
        reply = self.call("tools/call", {"name": "get_canvas_snapshot", "arguments": {}})
        content = reply["result"]["content"]
        image = next(c for c in content if c.get("type") == "image")
        return Image.open(io.BytesIO(base64.b64decode(image["data"]))).convert("RGBA")


class Report:
    """Collects one scenario's assertions; never raises mid-scenario."""

    def __init__(self):
        self.passed = 0
        self.failures = []

    def check(self, cond, message):
        if cond:
            self.passed += 1
            print("ok -", message)
        else:
            self.failures.append(message)
            print("FAIL -", message)

    def finish(self, name):
        if self.failures:
            print(f"SCENARIO {name}: {len(self.failures)} FAIL")
            return False
        print(f"SCENARIO {name}: green ({self.passed} checks)")
        return True


def rgb(hexstr):
    hexstr = hexstr.lstrip("#")
    return tuple(int(hexstr[i:i + 2], 16) for i in (0, 2, 4))


def mid_of(x0, y0, x1, y1):
    return (x0 + x1) // 2, (y0 + y1) // 2


def any_near(img, x, y, color, tol=30, radius=3):
    """True if a pixel within `radius` of (x, y) matches `color` (opaque-ish).

    The midpoint of a stroke is solid, but the exact landing pixel can sit on
    the AA edge; a small search radius keeps the assert robust, not loose.
    """
    w, h = img.size
    for dy in range(-radius, radius + 1):
        for dx in range(-radius, radius + 1):
            px, py = x + dx, y + dy
            if 0 <= px < w and 0 <= py < h:
                r, g, b, a = img.getpixel((px, py))
                if a > 200 and abs(r - color[0]) <= tol \
                        and abs(g - color[1]) <= tol and abs(b - color[2]) <= tol:
                    return True
    return False


def same_pixel(img, x, y, expected):
    return img.getpixel((x, y)) == expected


def shot(name):
    path = os.path.join(OUT, f"vd_{name}.png")
    subprocess.run(
        ["ffmpeg", "-loglevel", "error", "-y", "-f", "x11grab",
         "-video_size", "1280x800", "-i", DISPLAY, "-frames:v", "1", path],
        check=True,
    )
    print("captured", path)


# --- scenarios -------------------------------------------------------------


def scenario_plain_undo(d, rep):
    """Layer 0: pixels + lanes of a plain undo/redo cycle."""
    d.tool("set_active_layer", {"index": 0})
    base = d.snapshot()
    for i in range(3):
        d.stroke(50 + 40 * i, 60, 90 + 40 * i, 90)
    mid = mid_of(130, 60, 170, 90)  # the LAST stroke's middle (undo removes it)
    img = d.snapshot()
    rows, spans = d.layout()
    rep.check(len(spans) == 1 and all(r["branch"] == 0 for r in rows),
              "3 strokes: one straight lane-0 rail")
    rep.check(any_near(img, *mid, rgb("#222222")),
              "stroke 3 is committed at its midpoint (canvas pixel)")
    d.tool("undo")
    rows, _spans = d.layout()
    rep.check(all(r["branch"] == 0 for r in rows),
              "plain undo keeps the tail on lane 0")
    rep.check(sum(1 for r in rows if r["is_head"]) == 1, "exactly one head row")
    img = d.snapshot()
    rep.check(same_pixel(img, *mid, base.getpixel(mid)),
              "undo restores the exact baseline pixel")
    d.tool("redo")
    img = d.snapshot()
    rep.check(any_near(img, *mid, rgb("#222222")), "redo repaints the stroke")
    rows, _spans = d.layout()
    rep.check(all(r["branch"] == 0 for r in rows), "redo returns along the same rail")


def scenario_fork_after_undo(d, rep):
    """A fresh layer: drawing after undo is what forks; pixels follow the undo."""
    fresh = len(d.tool("get_layers")["layers"])
    d.tool("add_layer")
    d.tool("set_active_layer", {"index": fresh})
    d.stroke(300, 200, 360, 260, "#cc3333")
    d.stroke(380, 200, 440, 260, "#222222")
    d.tool("undo")
    d.stroke(300, 280, 360, 340, "#3388cc")
    rows, _spans = d.layout()
    # rows include the Start root: root, s1, s2 (peeled), s3 (fork)
    branches = [r["branch"] for r in sorted(rows, key=lambda r: r["id"])]
    rep.check(branches == [0, 0, 1, 0],
              f"the abandoned tail peels to lane 1 only after a post-undo stroke (got {branches})")
    img = d.snapshot()
    rep.check(not any_near(img, 410, 230, rgb("#222222")),
              "undo removed the second stroke's pixels from the canvas")
    rep.check(any_near(img, 330, 230, rgb("#cc3333")),
              "the first stroke survives the undo (it is the new head line)")
    rep.check(any_near(img, 330, 310, rgb("#3388cc")),
              "the fork stroke is on the canvas")


def scenario_layers_independent(d, rep):
    """A third layer: tree layers hold independent node lists (ids are global)."""
    while len(d.tool("get_layers")["layers"]) < 3:
        d.tool("add_layer")
    d.tool("set_active_layer", {"index": 2})
    d.stroke(40, 380, 90, 420, "#33aa55")
    rows, _spans = d.layout()
    rep.check(len(rows) == 2 and all(r["branch"] == 0 for r in rows),
              f"the new layer starts from its own root (got {[(r['id'], r['branch']) for r in rows]})")
    d.tool("set_active_layer", {"index": 0})
    rows, _spans = d.layout()
    rep.check(len(rows) == 4, f"layer 0 still holds its root + 3 strokes (got {len(rows)})")
    d.tool("set_active_layer", {"index": 1})
    rows, _spans = d.layout()
    rep.check(len(rows) == 4, f"layer 1 still holds root + tail + fork (got {len(rows)})")


SCENARIOS = [
    ("plain_undo_keeps_lane0", scenario_plain_undo),
    ("fork_after_undo", scenario_fork_after_undo),
    ("layers_independent", scenario_layers_independent),
]


def main():
    args = [a for a in sys.argv[1:] if a != "--shots"]
    shots = "--shots" in sys.argv[2:]
    if not args:
        print(__doc__)
        return 2
    d = Driver(args[0])
    time.sleep(1.0)
    all_green = True
    for name, fn in SCENARIOS:
        print(f"== {name} ==")
        rep = Report()
        try:
            fn(d, rep)
        except Exception as exc:  # a crashed scenario is a failed scenario
            rep.check(False, f"scenario crashed: {type(exc).__name__}: {exc}")
        all_green = rep.finish(name) and all_green
        if shots:
            shot(name)
    print("ALL VISUAL-DRIVER SCENARIOS GREEN" if all_green
          else "VISUAL DRIVER: FAILURES PRESENT")
    return 0 if all_green else 1


if __name__ == "__main__":
    sys.exit(main())
