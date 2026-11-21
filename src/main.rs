#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // no console in release on Windows

mod app;
mod canvas;
mod file_io;
mod history;
mod history_tree;
mod layers;
mod palette;
mod selection;
mod text;
mod tools;

use app::App;
use app::launch::LaunchConfig;
#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

/// The `--help` text of the native launch arguments (owner request
/// 2026-11-17, parsed in `app/launch.rs`). The string lives HERE because
/// printing is entry-point logic and web-only-by-cfg is allowed only in
/// main.rs (AGENTS.md "Repo facts") — the web build has no argv and never
/// prints it. The tool-name list stays in sync with `app/launch.rs`'s
/// `TOOL_NAMES` — pinned by a test.
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const USAGE: &str = "\
russet — a small layered paint program (native build)

Usage: russet [ARGS]

Launch arguments:
  --primary <hex>      the primary (stroke) color well: hex RRGGBB, also
                       #RGB, RRGGBB or RRGGBBAA, e.g. FF0000 or #00FF00
  --secondary <hex>    the secondary color well, the same formats
  --tool <name>        the startup tool: pencil, brush, eraser, select,
                       eyedropper, line, shape, text
  --canvas-size <WxH>  the startup canvas in pixels, e.g. 1024x768
                       (1..=4096 per side)
  --atlas-test         paint a static 1 px black-and-white speckle ring on
                       the outer edge of every icon-atlas picture (the
                       atlas diagnostic)
  -h, --help           print this help and exit

Both `--flag value` and `--flag=value` work. An invalid value is reported
to stderr and the built-in default stays. The web (WASM) build takes no
arguments.";

/// Human-readable name of the platform/windowing system the app runs on,
/// shown in the About window: Web (WASM build), Windows, macOS, or — on
/// Unix — the winit backend that will actually be used: Wayland or X11.
///
/// The check mirrors winit's own Linux backend selection so the label
/// matches reality without extra runtime API: the `WINIT_UNIX_BACKEND`
/// override wins, then winit tries Wayland FIRST (so a compositor socket
/// advertised via `WAYLAND_DISPLAY`/`WAYLAND_SOCKET` means Wayland), and
/// only falls back to X11 (`DISPLAY`). Uses `cfg!()` macros rather than
/// `#[cfg]` attributes — one code path compiles for every target.
pub(crate) fn platform_label() -> &'static str {
    if cfg!(target_arch = "wasm32") {
        "Web (WASM)"
    } else if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        match std::env::var("WINIT_UNIX_BACKEND").as_deref() {
            Ok("x11") => "X11 (Linux)",
            Ok("wayland") => "Wayland (Linux)",
            _ => {
                let wayland_session = std::env::var_os("WAYLAND_DISPLAY").is_some()
                    || std::env::var_os("WAYLAND_SOCKET").is_some();
                if wayland_session {
                    "Wayland (Linux)"
                } else if std::env::var_os("DISPLAY").is_some() {
                    "X11 (Linux)"
                } else {
                    "Unknown (Linux)"
                }
            }
        }
    }
}

/// The rendering backend eframe runs with (the Cargo.toml pins the glow
/// feature): OpenGL on native builds, WebGL in the browser.
pub(crate) fn renderer_label() -> &'static str {
    if cfg!(target_arch = "wasm32") {
        "WebGL (eframe glow)"
    } else {
        "OpenGL (eframe glow)"
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init();
    // Launch arguments (owner request 2026-11-17, `app/launch.rs`): parsed
    // BEFORE the window opens, so `--help` prints and exits without one;
    // parse problems go to stderr, the defaults stay.
    let launch = LaunchConfig::from_env();
    launch.report_warnings();
    if launch.help {
        print!("{USAGE}");
        return Ok(());
    }
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            // Owner-set fixed minimum window size (not derived from the default).
            .with_min_inner_size([500.0, 400.0]),
        // egui-winit's clipboard-shortcut conversion is DISABLED (owner
        // decision 2026-11-20: "я не согласен с разработчиками этой
        // библиотеки"): with it ON, egui-winit intercepts Ctrl+C/X/V,
        // turns them into dedicated `Event::Copy`/`Cut`/`Paste` events and
        // swallows the plain Key events. OFF, the combos arrive as normal
        // Key events (with egui-winit's logical-or-physical key fallback,
        // so non-Latin layouts still match). The web backend is untouched
        // — it synthesizes the same dedicated events from the browser's
        // own copy/cut/paste DOM events, which is the only correct channel
        // there; `handle_hotkeys` consumes both paths.
        clipboard_shortcuts: false,
        ..Default::default()
    };
    eframe::run_native(
        "Russet",
        native_options,
        Box::new(move |cc| Ok(Box::new(App::new_with(cc, &launch)))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();
    let web_options = eframe::WebOptions::default();
    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");
        let canvas = document
            .get_element_by_id("russet_canvas")
            .expect("Failed to find russet_canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("russet_canvas was not a HtmlCanvasElement");
        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                // The web takes no arguments: `from_env` yields nothing on
                // wasm, so the defaults stay (ONE code path with the native
                // entry, no cfg here). `--help` cannot be set on the web —
                // the native build prints the help text instead.
                Box::new(|cc| {
                    let launch = LaunchConfig::from_env();
                    launch.report_warnings();
                    if launch.help {
                        log::info!(
                            "--help has no web equivalent — the native build prints the launch-arguments help to stdout"
                        );
                    }
                    Ok(Box::new(App::new_with(cc, &launch)))
                }),
            )
            .await;
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(()) => {
                    loading_text.remove();
                }
                Err(err) => {
                    loading_text.set_inner_html(
                        "The app has crashed. See the developer console for details.",
                    );
                    panic!("Failed to start eframe: {err:?}");
                }
            }
        }
    });
}
