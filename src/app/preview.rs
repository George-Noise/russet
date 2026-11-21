//! Interaction and overlay preview for the "preview-then-commit" tools
//! (owner request 2026-11-10): the Line tool (straight drag / curve
//! polyline) and the Shape tool (rectangle / ellipse drag). Nothing touches
//! the canvas `Pixmap` until the gesture is committed, so a cancelled
//! preview never leaves a history snapshot behind. A child module of `app`,
//! so it can touch `App`'s private fields; `app.rs` stays the orchestrator.
//! The PURE Shape color rules (which wells a gesture uses, what the commit
//! records) live in the child [`colors`]. Holding `Shift` during a Shape
//! drag constrains it to a proportional square/circle (Paint.NET-style —
//! owner request 2026-11-16), live, through the pure [`proportional_current`];
//! the TESTING diameter-circle rule (owner request 2026-11-21) replaces that
//! for the Ellipse kind through the pure [`diameter_rect`].

mod colors;
mod commit;
mod geometry;
mod overlay;

use geometry::{diameter_guides, diameter_rect, proportional_current};

use colors::{shape_gesture_wells, shape_record_colors};

use crate::tools::{LineMode, ShapeKind, ShapeOutline, Tool};
use eframe::egui::{self, Color32, Key, PointerButton, Pos2, Rect, StrokeKind};

use super::App;

/// Pending gesture of the preview tools. Coordinates are canvas points.
#[derive(Default)]
pub(super) enum PreviewState {
    #[default]
    None,
    /// A straight line or a shape dragged from `start` to `current`.
    /// A shape stores the colors it will commit with: `outline` is the
    /// primary-well stroke color, `fill` the secondary-well fill color;
    /// `None` = the paint radio (owner request 2026-11-20) leaves that
    /// well out. A straight line always has `outline` set and `fill = None`.
    /// `outline_pos` carries the outline placement the shape commits with
    /// (owner request 2026-11-20; lines never outline, so they store the
    /// default) — stored like the colors, so a mid-drag panel switch
    /// cannot change what the release commits. `diameter` carries the
    /// TESTING diameter-circle rule (owner request 2026-11-21) the same
    /// way, refreshed live every frame like `current`: when set, the two
    /// raw drag points are the committed circle's diameter endpoints, so
    /// both the overlay and the commit draw the circle's bounding square
    /// ([`diameter_rect`]) instead of the corner-to-corner rect.
    Drag {
        kind: DragKind,
        start: Pos2,
        current: Pos2,
        outline: Option<Color32>,
        fill: Option<Color32>,
        width: f32,
        outline_pos: ShapeOutline,
        diameter: bool,
        /// The selection mask snapshot at drag start (owner request
        /// 2026-11-21): the commit transfers only masked pixels — the
        /// selection confines every painting tool. The selection cannot
        /// change mid-drag (the pointer is busy), so the snapshot stays
        /// exact.
        clip: Option<crate::selection::Mask>,
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
        /// The selection mask snapshot at gesture start (owner request
        /// 2026-11-21): the commit transfers only masked pixels.
        clip: Option<crate::selection::Mask>,
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
        let (escape, enter, backspace, shift) = ui.input(|i| {
            (
                i.key_pressed(Key::Escape),
                i.key_pressed(Key::Enter),
                i.key_pressed(Key::Backspace),
                i.modifiers.shift,
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
                let width = self.tools.size;
                match self.tools.tool {
                    Tool::Line => {
                        let color = if button == PointerButton::Primary {
                            self.tools.primary
                        } else {
                            self.tools.secondary
                        };
                        match self.tools.line_mode {
                            LineMode::Straight => {
                                self.preview = PreviewState::Drag {
                                    kind: DragKind::Line,
                                    start: pos,
                                    current: pos,
                                    outline: Some(color),
                                    fill: None,
                                    width,
                                    outline_pos: ShapeOutline::Center,
                                    diameter: false,
                                    clip: self.selection_clip(),
                                };
                                // Blocks Space-pan while the drag is in
                                // progress, same as a freehand stroke.
                                self.is_stroking = true;
                            }
                            LineMode::Curve => {
                                self.preview = PreviewState::Polyline {
                                    points: vec![pos],
                                    rubber: Some(pos),
                                    color,
                                    width,
                                    mode: LineMode::Curve,
                                    clip: self.selection_clip(),
                                };
                            }
                        }
                    }
                    Tool::Shape => {
                        // Primary paints the outline, secondary the fill
                        // (owner request 2026-11-10); a RIGHT-button gesture
                        // uses the two wells INVERTED without swapping them
                        // (owner request 2026-11-15). Exactly one paint is
                        // ever active (the panel's radio), so a color always
                        // reaches the gesture — the pure
                        // [`shape_gesture_wells`].
                        let paint = self.tools.shape_paint;
                        let (outline, fill) = shape_gesture_wells(
                            button,
                            paint.outlines(),
                            paint.fills(),
                            self.tools.primary,
                            self.tools.secondary,
                        );
                        self.preview = PreviewState::Drag {
                            kind: DragKind::Shape(self.tools.shape_kind),
                            start: pos,
                            current: pos,
                            outline,
                            fill,
                            width,
                            outline_pos: self.tools.shape_outline,
                            diameter: false,
                            clip: self.selection_clip(),
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
                    && let PreviewState::Drag {
                        kind,
                        start,
                        current,
                        diameter,
                        ..
                    } = &mut self.preview
                {
                    // Shift held mid-drag: a Shape grows proportionally
                    // (square/circle), live — the free pointer point is
                    // pulled onto the dominant-axis square each frame. The
                    // constrained point is STORED, so both the overlay
                    // preview and the commit rasterize the same geometry.
                    // TESTING (owner request 2026-11-21): with the Ellipse
                    // kind and the panel's diameter-circle checkbox on, the
                    // Shift rule is the DIAMETER circle instead — `current`
                    // stays the raw pointer position (both drag points are
                    // the circle's diameter endpoints) and the flag marks
                    // the geometry the overlay and the commit must convert.
                    // The flag is refreshed live like the point, so the
                    // release commits exactly what the preview showed.
                    let diameter_now = shift
                        && matches!(kind, DragKind::Shape(ShapeKind::Ellipse))
                        && self.tools.shape_diameter_circle;
                    *current = if shift && matches!(kind, DragKind::Shape(_)) && !diameter_now {
                        proportional_current(*start, pos)
                    } else {
                        pos
                    };
                    *diameter = diameter_now;
                }
                if any_released {
                    let state = std::mem::take(&mut self.preview);
                    if let PreviewState::Drag {
                        kind,
                        start,
                        current,
                        outline,
                        fill,
                        width,
                        outline_pos,
                        diameter,
                        clip,
                    } = state
                    {
                        self.commit_drag(
                            kind,
                            start,
                            current,
                            outline,
                            fill,
                            width,
                            outline_pos,
                            diameter,
                            clip.as_ref(),
                        );
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

    /// Cancels any pending preview gesture (tool switch, canvas reset…).
    pub(super) fn cancel_preview(&mut self) {
        self.preview = PreviewState::None;
        self.is_stroking = false;
    }
}

#[cfg(test)]
mod tests;
