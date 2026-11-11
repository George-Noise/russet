//! The "History" and "Layers" windows (owner request 2026-11-10). History
//! is a read-only, non-clickable log of document actions; Layers is a
//! placeholder showing the single implicit background layer (real layers
//! come later). Both are plain floating windows that since the owner request
//! 2026-11-11 follow the same §3.11 placement as "Tools" and "Palette"
//! (drop-time edge anchoring + resize repositioning, via [`place_floating_window`]);
//! their FIRST-open positions stay the edge anchors below. Child module
//! of `app`, so it can touch `App`'s private fields; `app.rs` stays the
//! orchestrator.

use eframe::egui::{self, Color32, Context};

use super::App;
use super::placement::{FIRST_OPEN_GAP, LAYERS_FIRST_OPEN_LIFT, place_floating_window};

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
        let content = ctx.content_rect();
        // First open: 0.5rem from the right edge and exactly 0.5rem below
        // the "File Edit Canvas View Help" strip. `last_panel_rect` is the
        // central panel's rect, whose top edge IS the menu bar's bottom
        // (the no_frame central panel has zero margin), so the window's
        // title sits just under the menu instead of covering it (owner
        // request 2026-11-10). Afterwards the same §3.11 magnetism and
        // resize repositioning as Tools/Palette (owner request 2026-11-11).
        let shown = egui::Window::new("History")
            .id(history_window_id())
            .open(&mut open)
            .pivot(egui::Align2::RIGHT_TOP)
            .default_pos(egui::pos2(
                content.right() - FIRST_OPEN_GAP,
                self.last_panel_rect.top() + FIRST_OPEN_GAP,
            ))
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
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                history_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.history_placement,
            );
        }
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
        let content = ctx.content_rect();
        // First open: 0.5rem from the right edge; the bottom edge sits
        // 0.5rem + 1.75rem above the window's bottom edge so it never
        // covers the zoom % label painted in the canvas's bottom-right
        // corner (owner request 2026-11-10). Afterwards the same §3.11
        // magnetism and resize repositioning as Tools/Palette (owner
        // request 2026-11-11).
        let shown = egui::Window::new("Layers")
            .id(layers_window_id())
            .open(&mut open)
            .pivot(egui::Align2::RIGHT_BOTTOM)
            .default_pos(egui::pos2(
                content.right() - FIRST_OPEN_GAP,
                content.bottom() - FIRST_OPEN_GAP - LAYERS_FIRST_OPEN_LIFT,
            ))
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
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                layers_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.layers_placement,
            );
        }
        if !open {
            self.layers_window_open = false;
        }
    }
}
