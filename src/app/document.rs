//! Document lifecycle of `App`: New / Open / Save / Save As (native AND
//! web paths), the unsaved-changes gate (`pending_action`), the resets a
//! NEW document implies (history, visual tree, views, log, camera), and
//! the window title that mirrors the document state.
//!
//! Split-off note (owner request 2026-11-13): this is the file-glue half
//! of the former `app.rs`. The dialogs that ASK before acting live in
//! [`super::dialogs`], the "Save Image" dialog (owner request 2026-11-19 —
//! BOTH Save and Save As open it) in [`super::save_dialog`], the actual
//! platform I/O in [`crate::file_io`], and the orchestrator — `App` state
//! and the frame flow — in [`super`]. Platform `cfg` branches are allowed
//! here like in `file_io.rs` (AGENTS.md "Repo facts"): the save dialog's
//! native prefill refinement and the Save-click glue are them.

use eframe::egui::{self, Color32, Vec2};

use super::App;
#[cfg(not(target_arch = "wasm32"))]
use super::save_dialog::base_name_of;
use super::save_dialog::compose_file_name;
use crate::canvas::Canvas;
use crate::file_io;
#[cfg(not(target_arch = "wasm32"))]
use crate::file_io::SaveFormat;
#[cfg(target_arch = "wasm32")]
use crate::file_io::WebOutcome;
use crate::layers::LayerStack;

/// Default document size: the New dialog's initial values and the default
/// startup canvas (see `App::build` in [`super`]; the native launch
/// arguments `--canvas-size` override the startup part — `app/launch.rs`).
pub(super) const NEW_WIDTH: u32 = 800;
pub(super) const NEW_HEIGHT: u32 = 600;
/// Canvas dimension clamps of the New / Resize dialogs ([`super::dialogs`]).
pub(super) const MIN_CANVAS: u32 = 1;
pub(super) const MAX_CANVAS: u32 = 4096;

/// A New/Open was requested while the document had unsaved changes: the
/// discard dialog ([`super::dialogs`]) asks first and completes the
/// request from there.
#[derive(Clone, Copy)]
pub(super) enum PendingAction {
    New,
    Open,
}

impl App {
    /// New-document body shared by "New" and the resize-apply path: one
    /// transparent layer with a fresh history tree, plus every reset that
    /// comes with it.
    pub(super) fn reset_canvas(&mut self, width: u32, height: u32) {
        let width = width.clamp(MIN_CANVAS, MAX_CANVAS);
        let height = height.clamp(MIN_CANVAS, MAX_CANVAS);
        self.canvas = Canvas::new(width, height, Color32::TRANSPARENT);
        // A fresh document: ONE WHITE layer whose Start root holds the
        // initial canvas (owner request 2026-11-13: the initial layer is
        // filled white — the §3.8 white New canvas — while every layer
        // added later is transparent).
        self.layers = LayerStack::new(width, height);
        self.layer_delete_confirm = None;
        self.layer_base_choice = false;
        self.layer_rename = None;
        self.reset_tree_views();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = None;
        }
        self.display_name = "Untitled".to_string();
        self.dirty = false;
        // A new document: drop pending gestures and restart the log. The
        // selection clears (its bounds belonged to the old canvas); the
        // in-app clipboard survives — the owner may paste into the new
        // document (owner request 2026-11-20).
        self.cancel_preview();
        self.cancel_text_edit();
        self.cancel_select_gesture();
        self.selection.clear();
        self.action_log.clear();
        self.action_log.push(format!("New canvas {width}×{height}"));
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
        // The white initial layer must reach the display buffer (the
        // canvas pixmap was created transparent above): blend + full upload.
        self.refresh_composite();
    }

    /// Fresh dimensions for the New dialog, then show it.
    pub(super) fn open_new_dialog(&mut self) {
        self.new_width = NEW_WIDTH as f32;
        self.new_height = NEW_HEIGHT as f32;
        self.show_new_dialog = true;
    }

    /// "New": ask before discarding unsaved changes.
    pub(super) fn request_new(&mut self) {
        if self.dirty {
            self.pending_action = Some(PendingAction::New);
        } else {
            self.open_new_dialog();
        }
    }

    /// "Open": ask before discarding unsaved changes.
    pub(super) fn request_open(&mut self, ctx: &egui::Context) {
        if self.dirty {
            self.pending_action = Some(PendingAction::Open);
        } else {
            self.open_now(ctx);
        }
    }

    fn apply_loaded(&mut self, loaded: file_io::LoadedImage) {
        // Open: ONE layer whose canvas and Start snapshot are the loaded
        // picture; the history restarts (SPEC §3.6).
        self.canvas.set_pixmap(loaded.pixmap.clone());
        self.layers = LayerStack::with_root(loaded.pixmap);
        self.layer_delete_confirm = None;
        self.layer_base_choice = false;
        self.layer_rename = None;
        self.reset_tree_views();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = Some(loaded.path);
        }
        self.display_name = loaded.name;
        self.dirty = false;
        // A new document: drop pending gestures and restart the log. The
        // selection clears; the in-app clipboard survives (owner request
        // 2026-11-20, the same rule the New path applies).
        self.cancel_preview();
        self.cancel_text_edit();
        self.cancel_select_gesture();
        self.selection.clear();
        self.action_log.clear();
        let name = self.display_name.clone();
        self.action_log.push(format!("Opened {name}"));
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
        self.repaint_full();
    }

    fn apply_saved(&mut self, saved: file_io::SavedImage) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = Some(saved.path);
        }
        self.display_name = saved.name;
        self.dirty = false;
    }

    /// "Open" without the unsaved-changes gate (the gate already passed).
    pub(super) fn open_now(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            match file_io::open_image() {
                Ok(Some(loaded)) => self.apply_loaded(loaded),
                Ok(None) => {}
                Err(error) => self.error_popup = Some(error),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.web_task = Some(file_io::open_image(ctx));
        }
    }

    /// "Save" (Ctrl+S): opens the in-app "Save Image" dialog (owner request
    /// 2026-11-19) — the format choice (PNG/JPEG), the file name, the JPEG
    /// quality and the native folder live THERE, so Save no longer writes
    /// silently to the current path; it saves the same dialog prefilled
    /// from the document.
    pub(super) fn save(&mut self) {
        self.open_save_dialog();
        #[cfg(not(target_arch = "wasm32"))]
        self.prefill_save_from_path();
    }

    /// "Save As": the SAME dialog since the owner request 2026-11-19 (the
    /// format is chosen in the dialog, so Save and Save As converge).
    pub(super) fn save_as(&mut self) {
        self.save();
    }

    /// Native refinement of the dialog prefill (platform cfg lives here per
    /// AGENTS.md): the document's own folder becomes the dialog's folder and
    /// the current path's extension picks the format radio. Runs AFTER
    /// `open_save_dialog` — the path's facts beat the display-name defaults.
    #[cfg(not(target_arch = "wasm32"))]
    fn prefill_save_from_path(&mut self) {
        let Some(path) = self.current_path.clone() else {
            return;
        };
        self.save_dir = path.parent().map(std::path::Path::to_path_buf);
        if let Some(format) = path
            .extension()
            .and_then(|ext| SaveFormat::from_extension(&ext.to_string_lossy()))
        {
            self.save_format = format;
        }
        if let Some(name) = path.file_name() {
            self.save_file_name = base_name_of(&name.to_string_lossy());
        }
    }

    /// The dialog's Save click (the glue behind
    /// [`super::save_dialog::save_dialog`]): composes the final name,
    /// gathers the dialog's encoding choices into a [`file_io::SaveOptions`]
    /// (both knobs clamped — the slider bounds are not the only writers),
    /// resolves the native folder — the dialog's pick, else the document's
    /// own folder, else the folder picker opens right from the click (a
    /// cancelled pick keeps the dialog open) — and writes. Returns whether
    /// the dialog may close: success (or, on web, the spawned download
    /// task); a write error surfaces through the error popup and the
    /// dialog STAYS OPEN so the name can be fixed.
    pub(super) fn commit_save_dialog(&mut self) -> bool {
        self.save_quality = self.save_quality.clamp(1, 100);
        self.save_png_compression = self.save_png_compression.min(9);
        let name = compose_file_name(&self.save_file_name, self.save_format);
        let options = file_io::SaveOptions {
            format: self.save_format,
            jpeg_quality: self.save_quality,
            png_compression: self.save_png_compression,
        };
        #[cfg(not(target_arch = "wasm32"))]
        {
            let dir = match self.save_dir.clone() {
                Some(dir) => dir,
                None => {
                    match self
                        .current_path
                        .as_ref()
                        .and_then(|path| path.parent().map(std::path::Path::to_path_buf))
                    {
                        Some(dir) => dir,
                        None => match file_io::pick_save_folder(self.save_dir.as_deref()) {
                            Some(picked) => {
                                self.save_dir = Some(picked.clone());
                                picked
                            }
                            // The user cancelled the picker: keep the dialog.
                            None => return false,
                        },
                    }
                }
            };
            match file_io::save_image_as(self.canvas.pixmap(), &dir, &name, options) {
                Ok(saved) => {
                    self.apply_saved(saved);
                    true
                }
                Err(error) => {
                    self.error_popup = Some(error);
                    false
                }
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.web_task = Some(file_io::save_image_as(
                &self.ctx,
                self.canvas.pixmap().clone(),
                name,
                options,
            ));
            true
        }
    }

    /// Web only: polls the in-flight `AsyncFileDialog` task (native I/O is
    /// synchronous and never sets `web_task`).
    #[cfg(target_arch = "wasm32")]
    pub(super) fn poll_file_task(&mut self) {
        let Some(task) = self.web_task.take() else {
            return;
        };
        match task.poll() {
            Some(Ok(WebOutcome::Opened(Some(loaded)))) => self.apply_loaded(loaded),
            Some(Ok(WebOutcome::Opened(None))) | Some(Ok(WebOutcome::Saved(None))) => {}
            Some(Ok(WebOutcome::Saved(Some(saved)))) => self.apply_saved(saved),
            Some(Err(error)) => self.error_popup = Some(error),
            None => self.web_task = Some(task),
        }
    }

    /// "*name - Russet" while the document has unsaved changes.
    pub(super) fn update_title(&mut self, ctx: &egui::Context) {
        let title = format!(
            "{}{} - Russet",
            if self.dirty { "*" } else { "" },
            self.display_name
        );
        if title != self.window_title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.window_title = title;
        }
    }
}
