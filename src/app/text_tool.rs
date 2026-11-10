//! The Text tool (owner request 2026-11-10): a click on the canvas anchors
//! a small input window ("Text"); the text is placed at that point and
//! committed through [`crate::text::rasterize_text`], so the committed
//! pixels match the on-canvas preview. Only the font size is configurable
//! for now. A child module of `app`, so it can touch `App`'s private
//! fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Color32, Context, Key, PointerButton, Pos2, Rect};

use super::App;

/// Pending text entry of the Text tool.
pub(super) struct TextEditState {
    /// Where the text box's top-left sits, in canvas coordinates.
    anchor: Pos2,
    buffer: String,
    /// Color frozen when the anchor was placed: LMB → primary,
    /// RMB → secondary, like every other tool.
    color: Color32,
    /// Move the input window next to the anchor (and focus the field) on
    /// the next frame — set when the anchor (re)placed.
    reposition: bool,
}

impl App {
    /// Click handling on the canvas while the Text tool is active: each
    /// click (re)places the anchor; a second click while the window is open
    /// moves it.
    pub(super) fn handle_text_input(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
        if panning || !response.hovered() {
            return;
        }
        let (cursor, primary_pressed, secondary_pressed) = ui.input(|i| {
            (
                i.pointer.latest_pos(),
                i.pointer.button_pressed(PointerButton::Primary),
                i.pointer.button_pressed(PointerButton::Secondary),
            )
        });
        let button = if primary_pressed {
            Some(PointerButton::Primary)
        } else if secondary_pressed {
            Some(PointerButton::Secondary)
        } else {
            None
        };
        let Some((button, cursor)) = button.zip(cursor) else {
            return;
        };
        let anchor = self.canvas_pos(rect, cursor);
        let color = if button == PointerButton::Primary {
            self.tools.primary
        } else {
            self.tools.secondary
        };
        match &mut self.text_edit {
            Some(state) => {
                state.anchor = anchor;
                state.color = color;
                state.reposition = true;
            }
            None => {
                self.text_edit = Some(TextEditState {
                    anchor,
                    buffer: String::new(),
                    color,
                    reposition: true,
                });
            }
        }
    }

    /// The small "Text" input window (font size + one-line text field).
    /// Enter or "Place" commits; Esc, "Cancel" or the window's X discards.
    pub(super) fn text_tool_window(&mut self, ctx: &Context) {
        if self.text_edit.is_none() {
            return;
        }
        let mut commit = false;
        let mut cancel = false;
        let mut open = true;
        let window_id = egui::Id::unique("text_tool_window");
        egui::Window::new("Text")
            .id(window_id)
            .collapsible(false)
            .resizable(false)
            .open(&mut open)
            .show(ctx, |ui| {
                let Some(state) = self.text_edit.as_mut() else {
                    return;
                };
                ui.horizontal(|ui| {
                    ui.label("Size");
                    ui.add(egui::Slider::new(&mut self.tools.font_size, 8.0..=200.0));
                });
                let response = ui.add(
                    egui::TextEdit::singleline(&mut state.buffer)
                        .desired_width(240.0)
                        .hint_text("Type text…"),
                );
                if state.reposition {
                    response.request_focus();
                }
                ui.horizontal(|ui| {
                    if ui.button("Place (Enter)").clicked() {
                        commit = true;
                    }
                    if ui.button("Cancel (Esc)").clicked() {
                        cancel = true;
                    }
                });
                ui.weak("Click the canvas to move the text; Enter places, Esc cancels.");
            });

        let (enter, escape) =
            ctx.input(|i| (i.key_pressed(Key::Enter), i.key_pressed(Key::Escape)));
        if commit || enter {
            self.commit_text();
        } else if cancel || escape || !open {
            self.text_edit = None;
        }

        // Move the window next to the anchor once per placed anchor. The
        // area state is rewritten directly (like §3.11 repositioning) since
        // `default_pos` only applies before the first stored position.
        let reposition = self.text_edit.as_ref().is_some_and(|s| s.reposition);
        if reposition {
            let (anchor, _color) = {
                let state = self.text_edit.as_ref().expect("checked above");
                (state.anchor, state.color)
            };
            let canvas_rect = self.canvas_rect(self.last_panel_rect);
            let anchor_screen = self.screen_pos(canvas_rect, anchor);
            super::placement::set_window_pos(
                ctx,
                window_id,
                anchor_screen + egui::vec2(24.0, 24.0),
            );
            if let Some(state) = self.text_edit.as_mut() {
                state.reposition = false;
            }
        }
    }

    /// Rasterizes the buffered text at the anchor (snapshot first), then
    /// closes the input window.
    fn commit_text(&mut self) {
        let Some(state) = self.text_edit.take() else {
            return;
        };
        if state.buffer.trim().is_empty() {
            return;
        }
        self.history.push_snapshot(self.canvas.pixmap().clone());
        let dirty = crate::text::rasterize_text(
            &self.ctx,
            self.canvas.pixmap_mut(),
            &state.buffer,
            state.anchor,
            self.tools.font_size,
            state.color,
        );
        if let Some(dirty) = dirty {
            self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
            self.dirty = true;
        }
        self.action_log.push("Text");
    }

    /// Discards any pending text entry (canvas reset, tool switch…).
    pub(super) fn cancel_text_edit(&mut self) {
        self.text_edit = None;
    }

    /// The on-canvas preview: an anchor marker plus the text rendered with
    /// the same engine that will rasterize it on commit.
    pub(super) fn draw_text_preview(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let Some(state) = &self.text_edit else {
            return;
        };
        let ppp = self.ctx.pixels_per_point();
        let anchor_screen = self.screen_pos(canvas_rect, state.anchor);
        // Crosshair marker: dark ring under a white ring, visible on any
        // canvas color (same style as the brush cursor preview).
        painter.circle_stroke(
            anchor_screen,
            5.0,
            egui::Stroke::new(1.0, Color32::from_black_alpha(160)),
        );
        painter.circle_stroke(
            anchor_screen,
            4.0,
            egui::Stroke::new(1.0, Color32::from_white_alpha(220)),
        );
        if !state.buffer.is_empty() {
            let size = (self.tools.font_size * self.zoom / ppp).max(1.0);
            painter.text(
                anchor_screen,
                egui::Align2::LEFT_TOP,
                &state.buffer,
                egui::FontId::proportional(size),
                state.color,
            );
        }
    }
}
