# Russet — a small bitmap paint program (egui/eframe, OpenGL via glow).
#
# Build:   nix build              # result -> ./result/bin/russet
# Run:     nix run                # builds and starts the app
# Shell:   nix develop            # cargo/clippy/fmt with the GUI libs on
#                                 # LD_LIBRARY_PATH, so `cargo run` works
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
      #   xorg.*               — X11 backend (x11-dl), also covers XWayland
      #   libGL (glvnd) + mesa — OpenGL: GLX on X11, EGL/GBM on Wayland
      #   fontconfig           — egui system font provider (fontique)
      # No vulkan-loader: the glow renderer never touches Vulkan.
      runtimeLibs = pkgs: with pkgs; [
        libGL
        mesa
        wayland
        libxkbcommon
        fontconfig
        xorg.libX11
        xorg.libXcursor
        xorg.libXrandr
        xorg.libXi
        xorg.libXrender
        xorg.libXext
        xorg.libxcb
      ];
    in
    {
      packages = forSystems (system:
        let pkgs = pkgsFor system; in rec {
          russet = pkgs.rustPlatform.buildRustPackage {
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

          default = russet;
        });

      apps = forSystems (system: {
        default = {
          type = "app";
          program = "${self.packages.${system}.russet}/bin/russet";
          meta.description = "Run Russet";
        };
      });

      devShells = forSystems (system:
        let pkgs = pkgsFor system; in {
          default = pkgs.mkShell {
            packages = with pkgs; [
              cargo
              rustc
              clippy
              rustfmt
              rust-analyzer
              pkg-config
              fontconfig
            ];

            # Same libs as the runtime wrapper, so a plain `cargo run` inside
            # the shell works on both Wayland and X11.
            LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (runtimeLibs pkgs);
          };
        });
    };
}
