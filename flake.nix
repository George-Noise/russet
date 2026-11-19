# Russet — a small bitmap paint program (egui/eframe, OpenGL via glow).
#
# Build:   nix build              # release (fat LTO) native app
#                                 # -> ./result/bin/russet
# Dev:     nix build .#russet-dev # fast `gates`-profile build (owner request
#                                 # 2026-11-16): compiles minutes faster and
#                                 # still runs the unit tests — for checking
#                                 # that everything builds AND poking the
#                                 # functionality (UI unoptimized, not for feel)
# Run:     nix run                # builds and starts the release NATIVE app —
#                                 # the default keeps its old meaning (owner
#                                 # decision 2026-11-19: easier on the brain,
#                                 # the web server is an explicit web app)
#          nix run .#russet-dev   # ... or the dev build
# Web:     nix run .#web-serve    # trunk serve — the web dev server on
#                                 # http://127.0.0.1:8080
#          nix run .#web-build    # trunk build -> ./dist (dev)
#          nix run .#web-build-release
#                                 # trunk build --release -> ./dist
#          All three run the repo-pinned trunk 0.22.1 (SPEC §8) OUTSIDE the
#          nix sandbox, so trunk self-manages its tools exactly like a
#          manual run: the wasm-bindgen 0.2.130 matching our Cargo.lock and
#          its pinned wasm-opt are downloaded on FIRST use and cached under
#          ~/.cache/trunk. A sandboxed `nix build` of the site is
#          deliberately NOT provided — see the webLane comment.
# Shell:   nix develop            # cargo/clippy/fmt + the pinned trunk, with
#                                 # the GUI libs on LD_LIBRARY_PATH, so
#                                 # `cargo run` works
#
# The wrapper puts BOTH the Wayland and the X11 libraries on
# LD_LIBRARY_PATH, so one binary runs in a Wayland session and under
# plain X11 / XWayland: winit tries Wayland first and falls back to X11
# automatically. To force a specific backend:
#
#   WINIT_UNIX_BACKEND=x11     ./result/bin/russet
#   WINIT_UNIX_BACKEND=wayland ./result/bin/russet
#
# (This is the same env var the About window's platform detection mirrors.)
{
  description = "Russet: a tiny bitmap paint program built with egui/eframe (OpenGL)";

  inputs = {
    # Cargo.toml requires rustc 1.95+ (`rust-version`); nixos-unstable always
    # carries a recent stable rustc. Pin an older nixpkgs branch only if it
    # ships rustc >= 1.95.
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      # Linux desktops: x86_64 and aarch64 (Wayland / X11).
      systems = [ "x86_64-linux" "aarch64-linux" ];

      forSystems = nixpkgs.lib.genAttrs systems;
      pkgsFor = system: nixpkgs.legacyPackages.${system};

      # Libraries the binary dlopen()s at RUNTIME (they are deliberately NOT
      # linked at build time — winit's x11-dl / xkbcommon-dl, glutin's GLX/EGL
      # loaders and wayland-sys all load them by name), so they go into the
      # wrapper's LD_LIBRARY_PATH instead of buildInputs:
      #   wayland              — libwayland-client/cursor/egl (wayland-sys)
      #   libxkbcommon         — keyboard mapping (+ the -x11 variant)
      #   libx11 & co.         — X11 backend (x11-dl), also covers XWayland
      #   libGL (glvnd) + mesa — OpenGL: GLX on X11, EGL/GBM on Wayland
      #   fontconfig           — egui system font provider (fontique)
      # No vulkan-loader: the glow renderer never touches Vulkan.
      # X11 libs use the new-style top-level attrs (libx11, not xorg.libX11):
      # the old xorg.* aliases are deprecated in nixpkgs and emit
      # evaluation warnings on every nix command.
      runtimeLibs = pkgs: with pkgs; [
        libGL
        mesa
        wayland
        libxkbcommon
        fontconfig
        libx11
        libxcursor
        libxrandr
        libxi
        libxrender
        libxext
        libxcb
      ];

      # The web (trunk / WASM) lane — owner request 2026-11-19. Trunk is
      # PINNED to 0.22.1 repo-wide (SPEC §8 / AGENTS.md "Repo facts";
      # web/index.html documents why: trunk 0.22's bundled wasm-opt 131
      # rejects the bulk-memory instructions Rust 1.99 emits unless
      # --all-features is passed, and web/index.html carries that workaround
      # — a different trunk invalidates the pin's reasoning). nixpkgs'
      # trunk is 0.21.14 (checked 2026-11-19), so the pin is honored by
      # BUILDING 0.22.1 from the crates.io source here: the .crate ships its
      # Cargo.lock (verified byte-identical to the v0.22.1 tag), and
      # cargoLock.lockFile vendors every dependency from that lock WITHOUT
      # a per-dep hash — the same hash-free mechanism the native packages
      # use. Both fetchurl hashes were computed from the real downloads
      # (2026-11-19, static.crates.io == crates.io api bytes). Default
      # features (update_check + rustls): the openssl-* entries in the lock
      # belong to the UNUSED native-tls permutation and never compile; the
      # rustls path only needs a C compiler (ring 0.17, no perl).
      webLane = pkgs: rec {
        trunk = pkgs.rustPlatform.buildRustPackage rec {
          pname = "trunk";
          version = "0.22.1";

          src = pkgs.fetchurl {
            url = "https://static.crates.io/crates/trunk/trunk-0.22.1.crate";
            # A .crate IS a tar.gz; the truthful .tar.gz name lets the
            # standard unpackPhase unpack it (it does not know ".crate").
            name = "trunk-0.22.1.tar.gz";
            hash = "sha256-XHhGfzu/sYFGFtpfsFzf+bPAGv1MCFQ7KrUeKb96Ohc=";
          };
          sourceRoot = "trunk-0.22.1";

          # The lock FROM the shipped source — byte-identical to the tag's
          # (verified 2026-11-19). Vendoring reads the lock's checksums, so
          # no extra hash churn per dependency.
          cargoLock.lockFile = pkgs.fetchurl {
            url = "https://raw.githubusercontent.com/trunk-rs/trunk/v0.22.1/Cargo.lock";
            hash = "sha256-OQPYFvHUW506NhqNGPexzG0Lbw/H5pL78cUJqqLUJp0=";
          };

          # Trunk's suite downloads its own wasm toolchain — impossible in
          # the sandbox (nixpkgs' trunk skips the same test for the same
          # reason). The binary itself is what this lane needs.
          doCheck = false;

          # zstd-sys (trunk -> zip[default features] -> zstd) hits nixpkgs'
          # cargo-setup-hook, which exports ZSTD_SYS_USE_PKG_CONFIG=1 for
          # EVERY buildRustPackage: zstd-sys's build.rs then probes and links
          # the SYSTEM libzstd instead of its vendored copy (the pkg-config
          # FEATURE is off in the chain — zip pulls zstd with
          # default-features = false; it is purely the hook's env var).
          # Caught by the owner's first `nix run .#web-serve` (2026-11-19):
          # the sandbox had neither pkg-config nor zstd. The same fontconfig
          # pattern the native packages use; fixupPhase rpaths the binary.
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = with pkgs; [ zstd ];

          meta.mainProgram = "trunk";
        };

        # trunk + a Rust toolchain with the wasm32-unknown-unknown std
        # (nixpkgs rustc builds that std by default — verified in
        # rustc.nix 2026-11-19). Deliberately NO wasm-bindgen/wasm-opt on
        # PATH: trunk self-manages them (downloaded to ~/.cache/trunk on
        # first use), which is what keeps the bindgen version matched to
        # OUR Cargo.lock (0.2.130) and the wasm-opt matched to the 0.22.1
        # pin — the exact versions a manual `trunk build` would use.
        # stdenv's cc rides on PATH because cargo compiles HOST build-scripts
        # (wasm-bindgen-shared, rustversion, ...) BEFORE the wasm code, and
        # the host linker must exist there — a plain NixOS shell has none
        # (owner probe 2026-11-19). lld sits next to it for the FINAL wasm
        # link: stock rustc would self-link via its bundled rust-lld, but
        # the owner's cargo environment asks for a PATH `lld` (probe 4; no
        # repo file does — no .cargo/config.toml exists — so most plausibly
        # a ~/.cargo/config.toml from the manual SPEC §8 era). The wrapper
        # honors the request instead of overriding back to rust-lld.
        binPath = pkgs.lib.makeBinPath [
          trunk pkgs.cargo pkgs.rustc pkgs.stdenv.cc pkgs.lld
        ];

        serve = pkgs.writeShellScriptBin "russet-web-serve" ''
          # trunk serve — the dev server (Trunk.toml: http://127.0.0.1:8080).
          # Runs in the CURRENT directory, like a plain `trunk serve`.
          export PATH="${binPath}:$PATH"
          exec trunk serve
        '';

        build = pkgs.writeShellScriptBin "russet-web-build" ''
          # trunk build — dev site into ./dist (gitignored).
          export PATH="${binPath}:$PATH"
          exec trunk build
        '';

        buildRelease = pkgs.writeShellScriptBin "russet-web-build-release" ''
          # trunk build --release — the dist/ for real use.
          export PATH="${binPath}:$PATH"
          exec trunk build --release
        '';
      };
    in
    {
      packages = forSystems (system:
        let
          pkgs = pkgsFor system;
          # Shared derivation arguments for BOTH packages. They were previously
          # one package plus `russet.override { ... }` — but recent nixpkgs
          # builds buildRustPackage through lib.extendMkDerivation, whose
          # results no longer carry `.override` (owner hit
          # "error: attribute 'override' missing" on 2026-11-17), so the dev
          # build is now its own call over the same arguments.
          commonArgs = {
            pname = "russet";
            version = "0.1.0";

            src = ./.;

            # Dependency versions and checksums come from Cargo.lock.
            cargoLock.lockFile = ./Cargo.lock;

            # pkg-config: yeslogic-fontconfig-sys (egui system fonts) finds
            # fontconfig at build time; fontconfig is the ONE library that is
            # linked directly (DT_NEEDED), hence also in buildInputs —
            # buildRustPackage rpaths buildInputs automatically.
            nativeBuildInputs = with pkgs; [
              pkg-config
              makeWrapper
            ];
            buildInputs = with pkgs; [
              fontconfig
            ];

            postFixup = ''
              wrapProgram "$out/bin/russet" \
                --prefix LD_LIBRARY_PATH : '${pkgs.lib.makeLibraryPath (runtimeLibs pkgs)}'
            '';

            # `cargo test` runs the pure unit tests (canvas / history /
            # placement math) — no display needed in the sandbox.
            meta = {
              description =
                "Tiny bitmap paint program (pencil/brush/eraser, zoom/pan, PNG/JPEG)";
              mainProgram = "russet";
            };
          };
        in
        rec {
          russet = pkgs.rustPlatform.buildRustPackage commonArgs;

          # The DEV build (owner request 2026-11-16): the SAME source, Cargo.lock
          # and unit tests as the release package, built in the repo's fast
          # `gates` profile (Cargo.toml: opt-level 0, no debuginfo, no
          # incremental — the AGENTS.md gate lane) instead of the release LTO
          # lane. `buildType`/`checkType` are buildRustPackage's supported
          # profile knobs (build-rust-package/default.nix: the build hook runs
          # `cargo build --profile gates`, the check hook runs
          # `cargo test --profile gates`, and the install hook takes the binary
          # from target/<triple>/gates); doCheck defaults to true. One command
          # therefore proves everything builds (tests included) AND produces a
          # runnable binary — minutes faster than release on a cold build. The
          # UI is UNOPTIMIZED: functional checks, not feel.
          russet-dev = pkgs.rustPlatform.buildRustPackage (commonArgs // {
            buildType = "gates";
            checkType = "gates";
          });

          default = russet;
        });

      apps = forSystems (system:
        let
          web = webLane (pkgsFor system);
        in
        {
          # The default keeps its OLD meaning (owner decision 2026-11-19,
          # same day the trunk lanes landed): `nix run` = build and start
          # the native release app — human muscle memory stays; the web
          # dev server is EXPLICIT: `nix run .#web-serve`.
          default = {
            type = "app";
            program = "${self.packages.${system}.russet}/bin/russet";
            meta.description = "Run Russet (release) — the native desktop app";
          };
          web-serve = {
            type = "app";
            program = "${web.serve}/bin/russet-web-serve";
            meta.description =
              "trunk serve — the web dev server (http://127.0.0.1:8080)";
          };
          web-build = {
            type = "app";
            program = "${web.build}/bin/russet-web-build";
            meta.description = "trunk build — dev site into ./dist";
          };
          web-build-release = {
            type = "app";
            program = "${web.buildRelease}/bin/russet-web-build-release";
            meta.description = "trunk build --release — the dist/ for real use";
          };
          russet = {
            type = "app";
            program = "${self.packages.${system}.russet}/bin/russet";
            meta.description = "Run Russet (release) — the native desktop app";
          };
          russet-dev = {
            type = "app";
            program = "${self.packages.${system}.russet-dev}/bin/russet";
            meta.description = "Run Russet (dev build, fast gates profile)";
          };
        });

      devShells = forSystems (system:
        let
          pkgs = pkgsFor system;
          web = webLane pkgs;
        in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
              rust-analyzer
              pkg-config
              fontconfig
              # The final wasm link needs this in ANY shell the owner's
              # cargo environment drives (it asks for a PATH `lld`; see the
              # webLane binPath comment) — same fix as the wrappers.
              lld
              # The repo-pinned trunk 0.22.1, so a manual `trunk serve` /
              # `trunk build` inside the shell hits the same binary the
              # apps use (its wasm-bindgen/wasm-opt self-manage on first
              # use, cached in ~/.cache/trunk).
              web.trunk
            ];

            # Same libs as the runtime wrapper, so a plain `cargo run` inside
            # the shell works on both Wayland and X11.
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (runtimeLibs pkgs);
          };
        });
    };
}
