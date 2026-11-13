#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // no console in release on Windows

mod app;
mod canvas;
mod file_io;
mod history;
mod history_tree;
mod layers;
mod palette;
mod text;
mod tools;

use app::App;
#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

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
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            // Owner-set fixed minimum window size (not derived from the default).
            .with_min_inner_size([500.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Russet",
        native_options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
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
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
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
