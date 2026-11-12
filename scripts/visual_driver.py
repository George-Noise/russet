#!/usr/bin/env python3
"""Visual check driver: builds the owner's complaint scenarios in the real
GUI (over MCP) and captures the screen with ffmpeg at each stage.

Stages:
  1. many early branches + a late fork below the main line (reuse check)
  2. several undos: the tail must stay a straight lane-0 rail (no branches)
  3. a new fork after those undos: it hugs lane 1, older ones sit right
"""
import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.dirname(__file__))
from mcp_smoke import Russet  # noqa: E402

OUT = "/home/z/my-project/download"


def shot(name):
    path = os.path.join(OUT, name)
    subprocess.run(
        ["ffmpeg", "-loglevel", "error", "-y", "-f", "x11grab",
         "-video_size", "1280x800", "-i", ":99", "-frames:v", "1", path],
        check=True,
    )
    print("captured", path)


def main():
    binary = sys.argv[1]
    russet = Russet(binary)
    time.sleep(1.0)

    # Stage 1: three early forks from the same node + main line continues,
    # then a LATE fork far below (the owner's point 4 + 5 scenario).
    russet.stroke(40, 40, 60, 60)
    russet.stroke(70, 40, 90, 60)
    russet.stroke(100, 40, 120, 60)
    russet.tool("undo"); russet.tool("undo")          # head at the root
    russet.stroke(40, 120, 60, 140, "#3f8fb9")        # branch 1
    russet.tool("undo")
    russet.stroke(70, 120, 90, 140, "#b93f6a")        # branch 2
    russet.tool("undo")
    russet.stroke(100, 120, 120, 140, "#7a3fb9")      # branch 3 (newest)
    for _ in range(6):                                 # long main line below
        russet.stroke(150 + 25 * _, 40, 170 + 25 * _, 60)
    russet.tool("undo")
    russet.stroke(400, 40, 430, 70, "#3fb950")        # the LATE fork
    time.sleep(0.4)
    rows, _spans = russet.layout()
    print("stage1 branches:", [r["branch"] for r in rows])
    shot("tree_stage1_forks.png")

    # Stage 2: PLAIN undo on a clean layer — the tail must stay a straight
    # lane-0 rail (the owner's point 1). Layer 3 is untouched.
    russet.tool("set_active_layer", {"index": 2})
    for i in range(4):
        russet.stroke(40 + 30 * i, 200, 60 + 30 * i, 230)
    for _ in range(3):
        russet.tool("undo")
    time.sleep(0.4)
    rows, spans = russet.layout()
    print("stage2 branches:", [r["branch"] for r in rows],
          "spans:", len(spans))
    assert len(spans) == 1 and all(r["branch"] == 0 for r in rows), \
        "plain undo must stay linear"
    shot("tree_stage2_plain_undo.png")

    # Stage 3: draw again — a REAL fork appears and hugs lane 1 while the
    # undone tail stays on the main line.
    russet.stroke(300, 300, 360, 340, "#b95f3f")
    time.sleep(0.4)
    rows, _ = russet.layout()
    print("stage3 branches:", [r["branch"] for r in rows])
    # The head was undone all the way to the FIRST stroke, so the new
    # stroke forks the root: the tail (rows 2..4) peels onto lane 1.
    assert [r["branch"] for r in rows] == [0, 1, 1, 1, 0], \
        "only the abandoned tail forks, on lane 1"
    shot("tree_stage3_real_fork.png")

    print("VISUAL DRIVER DONE")
    # Keep the app alive briefly so the last frame settles.
    time.sleep(0.5)


if __name__ == "__main__":
    main()
