//! The "History" and "Layers" windows (owner request 2026-11-10). History
//! is a read-only, non-clickable log of document actions; Layers is a
//! placeholder showing the single implicit background layer (real layers
//! come later). Both are plain floating windows WITHOUT §3.11 placement —
//! that contract stays with the "Tools" and "Palette" windows. Child module
//! of `app`, so it can touch `App`'s private fields; `app.rs` stays the
//! orchestrator.

use eframe::egui::{self, Color32, Context};

use super::App;
use super::placement::FIRST_OPEN_GAP;

/// Stable ids, same idea as `windows.rs`: independent of the title text so
/// placement / area state lookups stay stable.
fn history_window_id() -> egui::Id {
    egui::Id::unique("history_window")
}

fn layers_window_id() -> egui::Id {
    egui::Id::unique("layers_window")
}

impl App {
    /// Read-only action log (newest first). Entries are plain labels —
    /// clicking them does nothing yet by design.
    pub(super) fn history_window(&mut self, ctx: &Context) {
        if !self.history_window_open {
            return;
        }
        let mut open = true;
        // First open: 0.5rem from the top-right corner (mirrors the
        // Tools/Palette first-open gaps; no §3.11 anchoring afterwards).
        egui::Window::new("History")
            .id(history_window_id())
            .open(&mut open)
            .pivot(egui::Align2::RIGHT_TOP)
            .default_pos(
                ctx.content_rect().right_top() + egui::vec2(-FIRST_OPEN_GAP, FIRST_OPEN_GAP),
            )
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.set_min_width(180.0);
                let entries = self.action_log.items();
                if entries.is_empty() {
                    ui.weak("No actions yet.");
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(280.0)
                        .show(ui, |ui| {
                            // Newest at the top, so the latest action is
                            // always visible without scrolling.
                            for entry in entries.iter().rev() {
                                ui.label(format!("{}. {}", entry.id, entry.label));
                            }
                        });
                }
            });
        if !open {
            self.history_window_open = false;
        }
    }

    /// Layers placeholder: exactly one row for the implicit background
    /// layer, with a live thumbnail of the canvas. Nothing is interactive
    /// yet — real layers will replace this window's content.
    pub(super) fn layers_window(&mut self, ctx: &Context) {
        if !self.layers_window_open {
            return;
        }
        let mut open = true;
        egui::Window::new("Layers")
            .id(layers_window_id())
            .open(&mut open)
            .pivot(egui::Align2::RIGHT_BOTTOM)
            .default_pos(
                ctx.content_rect().right_bottom() + egui::vec2(-FIRST_OPEN_GAP, -FIRST_OPEN_GAP),
            )
            .auto_sized()
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // 28×28 live thumbnail of the canvas texture (uv covers
                    // the whole canvas, aspect is intentionally squeezed).
                    let (rect, _) =
                        ui.allocate_exact_size(egui::vec2(28.0, 28.0), egui::Sense::hover());
                    let uv = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
                    ui.painter()
                        .image(self.canvas_tex.id(), rect, uv, Color32::WHITE);
                    ui.painter().rect_stroke(
                        rect,
                        2.0,
                        egui::Stroke::new(1.0, Color32::from_gray(40)),
                        egui::StrokeKind::Inside,
                    );
                    ui.vertical(|ui| {
                        ui.label("Background");
                        ui.weak("Single layer (placeholder)");
                    });
                });
            });
        if !open {
            self.layers_window_open = false;
        }
    }
}
