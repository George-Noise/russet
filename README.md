# Russet

Build and run instructions for NixOS, Windows, and Web. The repo pins its Rust
toolchain (`rust-toolchain.toml`, 1.99.1) — the first cargo command installs it
automatically via rustup. Rust 1.95 or newer is required for all targets.

## NixOS (Linux)

```bash
nix build    # build -> ./result/bin/russet
nix run      # build and start the app
nix develop  # dev shell with cargo/clippy/rustfmt; plain `cargo run` works here too
```

The built binary runs in a Wayland session and under X11 automatically (winit
tries Wayland first and falls back to X11). To force a specific backend:

```bash
WINIT_UNIX_BACKEND=x11 ./result/bin/russet
WINIT_UNIX_BACKEND=wayland ./result/bin/russet
```

`nix build` / `nix run` also work on any other Linux distribution that has Nix
installed — not only NixOS.

## Windows

Prerequisites:

- [Rust](https://rustup.rs) — the repo pins 1.99.1 (`rust-toolchain.toml`) and
  rustup installs it on the first cargo command; on Windows this means the
  `x86_64-pc-windows-msvc` host toolchain and the Visual Studio Build Tools with
  the "Desktop development with C++" workload (rustup offers to install them).

Run (development build; the first run compiles all dependencies once, after
that it starts in seconds):

```powershell
cargo run
```

Release build (slow — fat LTO; run it manually when everything is ready):

```powershell
cargo build --release
```

The finished binary is `target\release\russet.exe`.

## MCP build (optional, development feature)

An optional compile-time feature adds an MCP (Model Context Protocol) stdio
server so an AI agent can drive the running app (draw strokes, inspect the
history tree, undo, read the canvas) while you watch the window:

```powershell
cargo build --release --features mcp
target\release\russet.exe --mcp
```

The server speaks JSON-RPC 2.0 on stdin/stdout, one message per line — this is
what MCP hosts (Claude Desktop etc.) expect:

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"demo","version":"0"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

Without `--features mcp` none of it exists in the binary: no server code, no
threads, no `serde_json` in the dependency graph — the default build is
byte-for-byte the light one. With the feature compiled in, the server starts
only when run with `--mcp` or when stdin is already piped (how MCP hosts spawn
binaries); a normal terminal launch stays GUI-only.

## Web (WASM)

One-time setup:

```bash
rustup target add wasm32-unknown-unknown
cargo install trunk --locked --version 0.22.1
```

The trunk version is pinned: the page (`web/index.html`) passes wasm-opt flags
tuned for trunk 0.22, so a different trunk version can break the release
build. Development server:

```bash
trunk serve      # open http://localhost:8080
```

Release build:

```bash
trunk build --release
```

The static site is written to `dist/` — serve that folder with any web server,
for example `python3 -m http.server -d dist`, and open the printed URL.

## Contributing

This repository is developed by LLM agents under the owner's direction; the
owner pushes to `master` directly. For how to file issues or contribute via
your own agent — including the one-logical-change-per-PR rule — see
[CONTRIBUTING.md](CONTRIBUTING.md).
