//! The "Tool Log" window: the read-only action log — it was titled
//! "History" until the owner rename request 2026-11-13 (the egui area id
//! and the internal `history_*` names stay unchanged so placement / area
//! state survive). Entries are plain labels — clicking them does nothing
//! yet by design. Since the owner request 2026-11-11 the window follows
//! the same §3.11 placement as "Tools" and "Palette" (drop-time edge
//! anchoring + resize repositioning, via [`place_floating_window`]); its
//! FIRST-open position stays the bottom-center anchor below. Child module
//! of `app`, so it can touch `App`'s private fields; `app.rs` stays the
//! orchestrator.
//!
//! Split history (owner request 2026-11-17, pure code motion — the
//! 500-line file guard): the former `app/panels.rs` (Tool Log + Layers in
//! one file) is now this file + [`super::layers_window`]. Every method,
//! constant and literal moved verbatim; no behavior change.

use eframe::egui::{self, Context};

use super::App;
use super::placement::place_floating_window;

/// Stable ids, same idea as `windows.rs`: independent of the title text so
/// placement / area state lookups stay stable.
fn history_window_id() -> egui::Id {
    egui::Id::unique("history_window")
}

/// Max height of the Tool Log's entry list. Was 280 pt, HALVED to 140 pt
/// by the owner request 2026-11-13 ("halve the window's max
/// height").
const TOOL_LOG_MAX_HEIGHT: f32 = 140.0;

/// Font size of the Tool Log's entries: 0.5rem = 8 pt (1 rem = 16 pt — egui
/// 0.37 has no rem API, the same convention as `placement::REM`; owner
/// request 2026-11-13). The window's vertical row spacing is zeroed with it
/// (owner request: "pin the gap to 0"), so consecutive log lines sit flush — each
/// row is the 8 pt font's own line height, the closest egui gets to a
/// 0.5rem line height.
const TOOL_LOG_FONT_SIZE: f32 = 8.0;

impl App {
    /// The Tool Log window (owner request 2026-11-13 renamed it from
    /// "History"): the read-only action log, newest first. Entries are
    /// plain labels — clicking them does nothing yet by design.
    /// HIDDEN on a smartphone-sized web viewport (owner request
    /// 2026-11-25, SPEC §3.11's smartphone rule): a phone screen has no
    /// room for a fifth floating window — the log itself keeps recording,
    /// only the window never renders (and the View item is hidden with
    /// it, so no dead toggle exists).
    pub(super) fn history_window(&mut self, ctx: &Context) {
        if !self.history_window_open || self.is_smartphone() {
            return;
        }
        let mut open = true;
        let content = ctx.content_rect();
        // First open (owner request 2026-11-12, amended 2026-11-13): the
        // window sits at the BOTTOM CENTER of the main window and its
        // bottom edge is FLUSH with the bottom edge — the former 0.5rem
        // gap is now 0 ("pinned flush to the bottom", owner request 2026-11-13).
        // §3.11 anchors the dropped gap, so it stays flush through
        // main-window resizes until the user drags it away. Afterwards the
        // same §3.11 magnetism and resize repositioning as Tools/Palette
        // (owner request 2026-11-11).
        let shown = egui::Window::new("Tool Log")
            .id(history_window_id())
            .open(&mut open)
            .pivot(egui::Align2::CENTER_BOTTOM)
            .default_pos(egui::pos2(content.center().x, content.bottom()))
            .default_width(200.0)
            .show(ctx, |ui| {
                ui.set_min_width(180.0);
                // Only this window (owner request 2026-11-13): zero vertical
                // row spacing, so the 8 pt log lines sit flush.
                ui.style_mut().spacing.item_spacing.y = 0.0;
                let entries = self.action_log.items();
                if entries.is_empty() {
                    ui.label(egui::RichText::new("No actions yet.").size(TOOL_LOG_FONT_SIZE));
                } else {
                    egui::ScrollArea::vertical()
                        .max_height(TOOL_LOG_MAX_HEIGHT)
                        .show(ui, |ui| {
                            // Newest at the top, so the latest action is
                            // always visible without scrolling.
                            for entry in entries.iter().rev() {
                                ui.label(
                                    egui::RichText::new(format!("{}. {}", entry.id, entry.label))
                                        .size(TOOL_LOG_FONT_SIZE),
                                );
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
}
