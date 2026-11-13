//! Document lifecycle of `App`: New / Open / Save / Save As (native AND
//! web paths), the unsaved-changes gate (`pending_action`), the resets a
//! NEW document implies (history, visual tree, views, log, camera), and
//! the window title that mirrors the document state.
//!
//! Split-off note (owner request 2026-11-13): this is the file-glue half
//! of the former `app.rs`. The dialogs that ASK before acting live in
//! [`super::dialogs`] (they call back into this module), the actual
//! platform I/O in [`crate::file_io`], and the orchestrator — `App` state
//! and the frame flow — in [`super`]. Platform `cfg` branches are allowed
//! here like in `file_io.rs` (AGENTS.md "Repo facts").

use eframe::egui::{self, Color32, Vec2};

use super::App;
use crate::canvas::Canvas;
use crate::file_io;
#[cfg(target_arch = "wasm32")]
use crate::file_io::WebOutcome;
use crate::layers::LayerStack;

/// Default document size: the New dialog's initial values and the initial
/// canvas (see `App::new` in [`super`]).
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
        // A fresh document: ONE transparent layer whose Start root holds
        // the initial canvas (owner request 2026-11-13: all layers are
        // transparent — the checkerboard shows through until painted).
        self.layers = LayerStack::new(width, height);
        self.layer_delete_confirm = None;
        self.layer_rename = None;
        self.reset_tree_views();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = None;
        }
        self.display_name = "Untitled".to_string();
        self.dirty = false;
        // A new document: drop pending gestures and restart the log.
        self.cancel_preview();
        self.cancel_text_edit();
        self.action_log.clear();
        self.action_log.push(format!("New canvas {width}×{height}"));
        self.set_zoom(1.0, None, self.last_panel_rect);
        self.pan = Vec2::ZERO;
        self.repaint_full();
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
        self.layer_rename = None;
        self.reset_tree_views();
        #[cfg(not(target_arch = "wasm32"))]
        {
            self.current_path = Some(loaded.path);
        }
        self.display_name = loaded.name;
        self.dirty = false;
        // A new document: drop pending gestures and restart the log.
        self.cancel_preview();
        self.cancel_text_edit();
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

    /// "Save": to the current path when there is one, Save As otherwise.
    pub(super) fn save(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            if let Some(path) = self.current_path.clone() {
                self.save_to(&path);
            } else {
                self.save_as(ctx);
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.save_as(ctx);
        }
    }

    /// "Save As": always asks for a destination.
    pub(super) fn save_as(&mut self, ctx: &egui::Context) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let _ = ctx;
            match file_io::save_image_as(self.canvas.pixmap(), &self.display_name) {
                Ok(Some(saved)) => self.apply_saved(saved),
                Ok(None) => {}
                Err(error) => self.error_popup = Some(error),
            }
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.web_task = Some(file_io::save_image_as(
                ctx,
                self.canvas.pixmap().clone(),
                self.display_name.clone(),
            ));
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_to(&mut self, path: &std::path::Path) {
        match file_io::save_image_to(self.canvas.pixmap(), path) {
            Ok(()) => self.dirty = false,
            Err(error) => self.error_popup = Some(error),
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
