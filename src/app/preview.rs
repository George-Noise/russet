//! Interaction and overlay preview for the "preview-then-commit" tools
//! (owner request 2026-11-10): the Line tool (straight drag / curve
//! polyline) and the Shape tool (rectangle / ellipse drag). Nothing touches
//! the canvas `Pixmap` until the gesture is committed, so a cancelled
//! preview never leaves a history snapshot behind. A child module of `app`,
//! so it can touch `App`'s private fields; `app.rs` stays the orchestrator.

use eframe::egui::{self, Color32, Key, PointerButton, Pos2, Rect, Shape, Stroke, StrokeKind};

use crate::tools::{LineMode, ShapeKind, Tool, rasterize_polyline, rasterize_shape};

use super::App;

/// Pending gesture of the preview tools. Coordinates are canvas points.
#[derive(Default)]
pub(super) enum PreviewState {
    #[default]
    None,
    /// A straight line or a shape dragged from `start` to `current`.
    Drag {
        kind: DragKind,
        start: Pos2,
        current: Pos2,
        color: Color32,
        width: f32,
        fill: bool,
    },
    /// A curve-mode polyline: `points` are confirmed vertices, the rubber
    /// band follows the pointer; `mode` is remembered so that switching the
    /// Line mode mid-polyline cancels it.
    Polyline {
        points: Vec<Pos2>,
        rubber: Option<Pos2>,
        color: Color32,
        width: f32,
        mode: LineMode,
    },
}

/// What a [`PreviewState::Drag`] is drawing.
#[derive(Clone, Copy)]
pub(super) enum DragKind {
    Line,
    Shape(ShapeKind),
}

impl App {
    /// Press/drag/release and click handling for the Line and Shape tools.
    pub(super) fn handle_preview_input(
        &mut self,
        ui: &egui::Ui,
        rect: Rect,
        response: &egui::Response,
        panning: bool,
    ) {
        self.cancel_stale_preview();

        let (cursor, primary_pressed, secondary_pressed, primary_double, any_released) =
            ui.input(|i| {
                (
                    i.pointer.latest_pos(),
                    i.pointer.button_pressed(PointerButton::Primary),
                    i.pointer.button_pressed(PointerButton::Secondary),
                    i.pointer.button_double_clicked(PointerButton::Primary),
                    i.pointer.any_released(),
                )
            });
        let (escape, enter, backspace) = ui.input(|i| {
            (
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::Backspace),
            )
        });
        let pos = cursor.map(|c| self.canvas_pos(rect, c));

        // Global finish/cancel keys work while a polyline is pending, even
        // when the pointer left the canvas.
        if let PreviewState::Polyline { .. } = self.preview {
            if escape {
                self.preview = PreviewState::None;
                return;
            }
            if enter || primary_double || secondary_pressed {
                self.finish_polyline();
                return;
            }
        }

        match self.preview {
            PreviewState::None => {
                if panning || !response.hovered() {
                    return;
                }
                let button = if primary_pressed {
                    Some(PointerButton::Primary)
                } else if secondary_pressed {
                    Some(PointerButton::Secondary)
                } else {
                    None
                };
                let Some((button, _)) = button.zip(cursor) else {
                    return;
                };
                let Some(pos) = pos else {
                    return;
                };
                let color = if button == PointerButton::Primary {
                    self.tools.primary
                } else {
                    self.tools.secondary
                };
                let width = self.tools.size;
                match self.tools.tool {
                    Tool::Line => match self.tools.line_mode {
                        LineMode::Straight => {
                            self.preview = PreviewState::Drag {
                                kind: DragKind::Line,
                                start: pos,
                                current: pos,
                                color,
                                width,
                                fill: false,
                            };
                            // Blocks Space-pan while the drag is in progress,
                            // same as a freehand stroke.
                            self.is_stroking = true;
                        }
                        LineMode::Curve => {
                            self.preview = PreviewState::Polyline {
                                points: vec![pos],
                                rubber: Some(pos),
                                color,
                                width,
                                mode: LineMode::Curve,
                            };
                        }
                    },
                    Tool::Shape => {
                        self.preview = PreviewState::Drag {
                            kind: DragKind::Shape(self.tools.shape_kind),
                            start: pos,
                            current: pos,
                            color,
                            width,
                            fill: self.tools.shape_fill,
                        };
                        self.is_stroking = true;
                    }
                    _ => {}
                }
            }
            PreviewState::Drag { .. } => {
                if escape {
                    // Esc cancels an in-progress drag (like Paint.NET's
                    // shape tools); nothing was committed, so no snapshot.
                    self.preview = PreviewState::None;
                    self.is_stroking = false;
                    return;
                }
                if let Some(pos) = pos
                    && let PreviewState::Drag { current, .. } = &mut self.preview
                {
                    *current = pos;
                }
                if any_released {
                    let state = std::mem::take(&mut self.preview);
                    if let PreviewState::Drag {
                        kind,
                        start,
                        current,
                        color,
                        width,
                        fill,
                    } = state
                    {
                        self.commit_drag(kind, start, current, color, width, fill);
                    }
                    self.is_stroking = false;
                }
            }
            PreviewState::Polyline { .. } => {
                if panning {
                    return; // a Space/middle drag pans; clicks are ignored
                }
                if primary_pressed
                    && let Some(pos) = pos
                    && let PreviewState::Polyline { points, rubber, .. } = &mut self.preview
                {
                    points.push(pos);
                    *rubber = Some(pos);
                }
                if let Some(pos) = pos
                    && let PreviewState::Polyline { rubber, .. } = &mut self.preview
                {
                    *rubber = Some(pos);
                }
                if backspace && let PreviewState::Polyline { points, .. } = &mut self.preview {
                    points.pop();
                    if points.is_empty() {
                        self.preview = PreviewState::None;
                    }
                }
            }
        }
    }

    /// Drops pending previews that no longer match the active tool or the
    /// Line mode (e.g. the tool was switched mid-gesture).
    fn cancel_stale_preview(&mut self) {
        match self.preview {
            PreviewState::Drag { kind, .. } => {
                let stale = match kind {
                    DragKind::Line => self.tools.tool != Tool::Line,
                    DragKind::Shape(_) => self.tools.tool != Tool::Shape,
                };
                if stale {
                    self.preview = PreviewState::None;
                    self.is_stroking = false;
                }
            }
            PreviewState::Polyline { mode, .. } => {
                if self.tools.tool != Tool::Line || self.tools.line_mode != mode {
                    self.preview = PreviewState::None;
                }
            }
            PreviewState::None => {}
        }
    }

    /// Commits a finished drag: straight line, rectangle or ellipse.
    fn commit_drag(
        &mut self,
        kind: DragKind,
        start: Pos2,
        current: Pos2,
        color: Color32,
        width: f32,
        fill: bool,
    ) {
        match kind {
            DragKind::Line => {
                // A click without movement draws a dot (round cap).
                self.history.push_snapshot(self.canvas.pixmap().clone());
                let dirty =
                    rasterize_polyline(self.canvas.pixmap_mut(), &[start, current], width, color);
                if let Some(dirty) = dirty {
                    self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
                    self.dirty = true;
                }
                self.action_log.push("Line");
            }
            DragKind::Shape(shape_kind) => {
                let label = match shape_kind {
                    ShapeKind::Rectangle => "Rectangle",
                    ShapeKind::Ellipse => "Ellipse",
                };
                self.history.push_snapshot(self.canvas.pixmap().clone());
                let dirty = rasterize_shape(
                    self.canvas.pixmap_mut(),
                    shape_kind,
                    start,
                    current,
                    width,
                    color,
                    fill,
                );
                if let Some(dirty) = dirty {
                    self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
                    self.dirty = true;
                }
                self.action_log.push(label);
            }
        }
    }

    /// Commits the pending polyline (curve) and logs it.
    fn finish_polyline(&mut self) {
        let state = std::mem::take(&mut self.preview);
        let PreviewState::Polyline {
            mut points,
            color,
            width,
            ..
        } = state
        else {
            return;
        };
        // Double-click commits after adding a duplicate vertex — drop it.
        points.dedup();
        if points.is_empty() {
            return;
        }
        self.history.push_snapshot(self.canvas.pixmap().clone());
        let dirty = rasterize_polyline(self.canvas.pixmap_mut(), &points, width, color);
        if let Some(dirty) = dirty {
            self.sync_texture(dirty.x, dirty.y, dirty.w, dirty.h);
            self.dirty = true;
        }
        self.action_log.push("Curve");
    }

    /// The translucent on-canvas preview drawn on top of the canvas texture
    /// while a gesture is pending (never touches the `Pixmap`).
    pub(super) fn draw_preview_overlay(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let ppp = self.ctx.pixels_per_point();
        let screen_stroke =
            |width: f32| Stroke::new((width * self.zoom / ppp).max(1.0), Color32::TRANSPARENT);
        let to_screen = |p: Pos2| self.screen_pos(canvas_rect, p);

        match &self.preview {
            PreviewState::None => {}
            PreviewState::Drag {
                kind,
                start,
                current,
                color,
                width,
                fill,
            } => {
                let a = to_screen(*start);
                let b = to_screen(*current);
                let mut stroke = screen_stroke(*width);
                stroke.color = *color;
                match kind {
                    DragKind::Line => {
                        painter.add(Shape::line_segment([a, b], stroke));
                    }
                    DragKind::Shape(shape_kind) => {
                        let rect = Rect::from_two_pos(a, b);
                        match shape_kind {
                            ShapeKind::Rectangle => {
                                if *fill {
                                    painter.rect_filled(rect, 0.0, *color);
                                } else {
                                    painter.rect_stroke(rect, 0.0, stroke, StrokeKind::Middle);
                                }
                            }
                            ShapeKind::Ellipse => {
                                painter.add(Shape::Ellipse(egui::epaint::EllipseShape {
                                    center: rect.center(),
                                    radius: rect.size() * 0.5,
                                    fill: if *fill { *color } else { Color32::TRANSPARENT },
                                    stroke: if *fill { Stroke::NONE } else { stroke },
                                    angle: 0.0,
                                }));
                            }
                        }
                    }
                }
            }
            PreviewState::Polyline {
                points,
                rubber,
                color,
                width,
                ..
            } => {
                let mut stroke = screen_stroke(*width);
                stroke.color = *color;
                let mut pts: Vec<Pos2> = points.iter().copied().map(to_screen).collect();
                if let Some(rubber) = rubber {
                    pts.push(to_screen(*rubber));
                }
                if pts.len() >= 2 {
                    painter.add(Shape::line(pts.clone(), stroke));
                }
                // Vertex handles: small white dots, visible on any color.
                for p in points {
                    let s = to_screen(*p);
                    painter.circle_filled(s, 3.0, Color32::WHITE);
                    painter.circle_stroke(s, 3.0, Stroke::new(1.0, Color32::from_black_alpha(180)));
                }
            }
        }
    }

    /// Cancels any pending preview gesture (tool switch, canvas reset…).
    pub(super) fn cancel_preview(&mut self) {
        self.preview = PreviewState::None;
        self.is_stroking = false;
    }
}
