# SANDBOX — agent-sandbox limits & the verified headless-GUI recipe

Why this file exists: on 2026-11-13 an agent session concluded "the GUI cannot run in
this sandbox" and shipped a feature whose GUI pass was left entirely to the owner. That
conclusion was WRONG — russet runs headless with a small user-space fix, and the same
session's follow-up proved it by driving all `visual_driver.py` scenarios green through
the real binary. This file records exactly what is missing, what works, and every
pitfall that cost time, so no future session re-diagnoses any of it.
`scripts/gui_recipe_test.sh` is the executable proof of the recipe — run it once if in
doubt; do not re-derive the steps below.

## What the sandbox is

- Debian 13 (trixie) container, user uid 1001 — **no root**: `sudo` asks for a password,
  `apt-get install` is impossible. `apt-get download <pkg>` (fetches a .deb into the
  current directory) works and is the sanctioned way to get missing shared libraries.
- Network is available: crates.io, deb.debian.org, GitHub are all reachable.
- Rust: rustup has **no default toolchain** — `cargo`/`rustc` fail outside this repo
  with "rustup could not choose a version of rustc to run". Inside the repo
  `rust-toolchain.toml` pins 1.99.1 and rustup fetches it on the first cargo command.
  The SAME trap hits `rustup target add wasm32-unknown-unknown` on a recycled sandbox
  ("rustup could not choose a version of rustup to run" — the target subcommand needs a
  chosen toolchain too): run it as `RUSTUP_TOOLCHAIN=1.99.1 rustup target add
  wasm32-unknown-unknown` (caught 2026-11-23, the persistence round).
  `scripts/verify.sh` exports `PATH="$HOME/.cargo/bin:$PATH"` itself; for ad-hoc
  commands do the same.
- `target/` persists with the project directory between sessions — builds stay warm;
  do not clean it to "fix" anything. A FRESH clone has no `target/` (a ~15–20 min
  cold dep build): restore the owner-hosted prebuilt tree instead — download
  `russet-target.zip` from the GitHub release `target-store` and unzip it in the
  repo root (see AGENTS.md "Commands — Session start"). `scripts/sandbox_check.sh`
  (owner request 2026-11-17) probes all of this at session start; its verdict
  tells which lanes are warm, and every heavy command's duration lands in
  `TIMING.md` (the sleep-chunk source).

## What the GUI needs that the image lacks — and the fix

The windowing stack (eframe → winit → xkbcommon-dl) dlopens X11 bits at runtime. The
image ships Xvfb, libX11, libxcb and libxkbcommon, but misses two small libraries, so
a bare `cargo run` dies before the first frame:

```
thread 'main' panicked at xkbcommon-dl-0.4.2/src/x11.rs:59:28:
Library libxkbcommon-x11.so could not be loaded.
```

The fix needs no root — download both .debs, unpack into a user directory, symlink the
unversioned name, and point `LD_LIBRARY_PATH` at it (all of this is automated in
`scripts/gui_recipe_test.sh`):

```bash
mkdir -p /tmp/xkbfix && cd /tmp/xkbfix
apt-get download libxkbcommon-x11-0 libxcb-xkb1     # the two missing libs
for d in *.deb; do dpkg -x "$d" ext; done
ln -sf libxkbcommon-x11.so.0 ext/usr/lib/x86_64-linux-gnu/libxkbcommon-x11.so
```

Then run the app under a raw Xvfb (not `xvfb-run` — see pitfall 1):

```bash
setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null &
sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 \
LD_LIBRARY_PATH=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu \
    timeout -k 5 20 ./target/debug/russet >/tmp/russet_run.log 2>&1
pkill -x Xvfb
```

Every piece of that incantation is load-bearing (pitfalls 4–7). `/tmp` is wiped when
the sandbox is recycled — re-run the download block once per fresh sandbox (~16 kB + ~130 kB).

## Verified: the full MCP scenario pass runs headless

2026-11-13, all three `visual_driver.py` scenarios green (14 checks) through the real
binary — strokes, undo/redo pixel restoration, git-style forking, per-layer tree
independence:

```bash
setsid Xvfb :99 -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 </dev/null & sleep 2
DISPLAY=:99 LIBGL_ALWAYS_SOFTWARE=1 \
LD_LIBRARY_PATH=/tmp/xkbfix/ext/usr/lib/x86_64-linux-gnu \
    python3 scripts/visual_driver.py ./target/debug/russet
pkill -x Xvfb
```

`--shots` additionally writes screenshots to `$RUSSET_DRIVER_OUT` (default
`/home/z/my-project/download`) — only when the owner asks for them.

## Pitfalls — each one cost real time exactly once

1. `xvfb-run` exists but dies instantly with `xauth command not found` — the image
   has no xauth. Use raw `Xvfb` directly; never debug xvfb-run.
2. The panic text names `libxkbcommon-x11.so`, but `ldd` on the freshly unpacked
   library revealed a second silent hole: `libxcb-xkb.so.1 => not found`. dlopen
   errors never mention transitive holes — always close the loop with
   `LD_LIBRARY_PATH=... ldd <lib> | grep 'not found'` until the list is empty.
3. `xkbcommon-dl` dlopens the UNVERSIONED name `libxkbcommon-x11.so`; the .deb ships
   only `libxkbcommon-x11.so.0` (the unversioned file belongs to the -dev package).
   Without the manual symlink `LD_LIBRARY_PATH` alone does nothing.
4. A background Xvfb started without `setsid` + full redirection keeps the agent
   shell's pipes open — the whole tool call then hangs until its own timeout kills
   everything (observed: a 120 s hang for a 30 s run). Always:
   `setsid Xvfb ... >/tmp/xvfb.log 2>&1 </dev/null &`, and `pkill -x Xvfb` at the end.
5. A GUI app never exits by itself: wrap every run in `timeout -k 5 <s>`. Plain
   SIGTERM (exit 124) is expected and fine; `-k` guarantees SIGKILL if it lingers.
6. `LIBGL_ALWAYS_SOFTWARE=1` — Xvfb has no GPU; make software GL explicit instead of
   gambling on GLX probing.
7. Wayland is not a fallback here: no compositor binary exists in the image. Don't try.
8. `pgrep -a russet` after a kill is a cheap sanity check — a leftover instance holds
   :99 and makes the next run look "hung" for unrelated reasons.
9. `git apply` in a directory WITHOUT its own `.git` (e.g. a tar-unpacked base copy
   under the sandbox workspace) is a SILENT NO-OP: exit 0, zero files changed —
   observed twice 2026-11-16 and it burned the delivery byte-compare both times
   (`--check` passes too, so nothing looks wrong until the diff). Apply delivery
   patches only in a REAL clone: `git commit-tree <base-tree> -m base` +
   `git update-ref refs/heads/delivery-base <commit>` in the repo, `git clone
   --branch delivery-base`, `git apply` there, then `diff -r`.
10. `cargo test --features mcp` (and with it `verify.sh`) HANGS after the tests
    already pass in a RECYCLED sandbox: the session's stdin is an open pipe/pty,
    every `App::build` compiled with the feature spawns the MCP stdin reader on it,
    and the cargo ↔ test-binary teardown never returns (observed 2026-11-17: the
    log already showed "143 passed … finished in 3.4 s" while the tool call still
    burned its whole timeout). FIXED AT THE SOURCE 2026-11-18: `verify.sh` closes
    stdin itself (`exec </dev/null`) — the gate needs no redirect anymore. A BARE
    `cargo test --features mcp` outside the gate still needs the manual pipe
    (`< /dev/null`): the reader sees an immediate EOF and everything exits in seconds.
11. KEY and MODIFIER events never reach the app under a raw Xvfb (found 2026-11-21,
    the diameter-circle probe: three Shift drags all drew UNCONSTRAINED shapes while
    every click and pointer move landed fine): raw Xvfb has no window manager, so no
    window is ever given the X input focus — winit drops the key events entirely.
    MOUSE events still work because X's initial focus is PointerRoot (they follow the
    pointer). Fix: focus the window explicitly once per launch —
    `WID=$(xdotool search --name Russet | head -1); xdotool windowfocus $WID` — then
    `xdotool keydown shift` + drags behave like real input (the drag-probe template's
    `gui_magnet_check.sh` never hit this because its drags carry no modifier keys).
12. `git checkout -- <file>` silently DESTROYS uncommitted work from earlier rounds
    (found 2026-11-21, the selection-clip round: a mechanical call-site patch had to
    be redone and `git checkout -- src/app/preview/tests.rs` restored the file to
    HEAD — which predates the diameter/guides rounds, whose TEST additions lived
    only in the working tree; seven tests vanished, caught only because the test
    count dropped 282 → 276). The tree accumulates rounds between the owner's
    pushes: HEAD is NOT "the last round". Never `git checkout` a file to undo a
    bad edit — re-apply the edit instead; if a revert already happened, restore
    the file from the latest delivery zip (`unzip -p download/russet.zip
    russet/<path>`), which byte-compared to the tree at the last round. Watch the
    test count in every gate run — a silent drop means lost tests, not green.


## What this unlocked, and what still belongs to the owner

- MCP-driven scenario verification now runs in every agent session — the done-gate for
  GUI-visible features can include a green driver run instead of "gates only".
- `add_layer` was added to the MCP surface (2026-11-13) for parity with the Layers
  window "+ Add layer"; without it no scenario can address a second layer, which is
  exactly why two driver scenarios sat stale and unrun.
- Still owner-only (V13, real pointer): node-dot hit areas, double-click rename feel,
  window magnetism, anything about "does it feel right".
