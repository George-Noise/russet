//! The preview OVERLAY (pure code motion, owner-approved 2026-11-21
//! file-guard split): [`draw_preview_overlay`], the live on-canvas drawing
//! of every in-flight gesture (the straight/curve line, the shape, the
//! diameter guides) — painter-only, no canvas writes. Method on [`App`].

use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, pos2};
use std::f32::consts::TAU;

use super::{App, DragKind, PreviewState, StrokeKind, diameter_guides, diameter_rect};
use crate::tools::{ShapeKind, ShapeOutline};

impl App {
    /// The translucent on-canvas preview drawn on top of the canvas texture
    /// while a gesture is pending (never touches the `Pixmap`).
    pub(in crate::app) fn draw_preview_overlay(&self, painter: &egui::Painter, canvas_rect: Rect) {
        let ppp = self.ctx.pixels_per_point();
        // The live preview follows the selection too (owner request
        // 2026-11-21): the commit will never paint outside the selected
        // region, so the overlay must not show paint there either. The
        // CACHED tight bounds clip pixel-exact for rect selections; for a
        // lasso they clip to the bounding rect (a superset — the commit
        // still clips exactly to the mask).
        let painter = match self.selection.bounds() {
            Some(b) => {
                let min = self.screen_pos(canvas_rect, Pos2::new(b.x as f32, b.y as f32));
                let max = self.screen_pos(
                    canvas_rect,
                    Pos2::new((b.x + b.w as i64) as f32, (b.y + b.h as i64) as f32),
                );
                painter
                    .clone()
                    .with_clip_rect(painter.clip_rect().intersect(Rect::from_two_pos(min, max)))
            }
            None => painter.clone(),
        };
        let screen_stroke =
            |width: f32| Stroke::new((width * self.zoom / ppp).max(1.0), Color32::TRANSPARENT);
        let to_screen = |p: Pos2| self.screen_pos(canvas_rect, p);

        match &self.preview {
            PreviewState::None => {}
            PreviewState::Drag {
                kind,
                start,
                current,
                outline,
                fill,
                width,
                outline_pos,
                diameter,
                ..
            } => {
                // TESTING (owner request 2026-11-21): the diameter-circle
                // gesture previews the COMMITTED bounding square — the
                // circle through both raw drag points — not the raw
                // corner-to-corner rect.
                let (a, b) = if *diameter {
                    let rect = diameter_rect(*start, *current);
                    (to_screen(rect.min), to_screen(rect.max))
                } else {
                    (to_screen(*start), to_screen(*current))
                };
                let stroke = outline
                    .map(|c| {
                        let mut s = screen_stroke(*width);
                        s.color = c;
                        s
                    })
                    .unwrap_or(Stroke::NONE);
                match kind {
                    DragKind::Line => {
                        // Round caps match the committed raster (LineCap::Round);
                        // egui's own line ends are flat (owner request 2026-11-10).
                        painter.add(Shape::line_segment([a, b], stroke));
                        painter.circle_filled(a, stroke.width * 0.5, stroke.color);
                        painter.circle_filled(b, stroke.width * 0.5, stroke.color);
                    }
                    DragKind::Shape(shape_kind) => {
                        let rect = Rect::from_two_pos(a, b);
                        match shape_kind {
                            ShapeKind::Rectangle => {
                                if let Some(fill) = fill {
                                    painter.rect_filled(rect, 0.0, *fill);
                                }
                                if stroke.width > 0.0 {
                                    // The stroke KIND carries the outline
                                    // placement (owner request 2026-11-20):
                                    // outside = fully beyond the two points,
                                    // inside = fully within, centered = the
                                    // classic straddle — the same bands the
                                    // commit's shifted stroke paths draw.
                                    let kind = match outline_pos {
                                        ShapeOutline::Outside => StrokeKind::Outside,
                                        ShapeOutline::Center => StrokeKind::Middle,
                                        ShapeOutline::Inside => StrokeKind::Inside,
                                    };
                                    painter.rect_stroke(rect, 0.0, stroke, kind);
                                }
                            }
                            ShapeKind::Ellipse => {
                                // egui's EllipseShape strokes OUTSIDE the path,
                                // while the commit (tiny-skia) strokes the oval
                                // CENTERED — a closed PathShape with a plain
                                // (Middle-kind) stroke is centered, matching the
                                // commit (owner request 2026-11-10). For the
                                // outline PLACEMENT modes (owner request
                                // 2026-11-20) the commit shifts the stroked oval
                                // by half a stroke width (`tools.rs
                                // ::outline_stroke_bounds`); egui strokes a path
                                // CENTERED, so the preview shifts the SAMPLED
                                // radii by the same half width and keeps the
                                // stroke centered — the band lands where the
                                // commit's does.
                                if rect.width() < 0.5 || rect.height() < 0.5 {
                                    return; // degenerate click; commit draws nothing
                                }
                                let center = rect.center();
                                let (rx, ry) = (rect.width() * 0.5, rect.height() * 0.5);
                                let (rx, ry) = match outline_pos {
                                    ShapeOutline::Center => (rx, ry),
                                    ShapeOutline::Outside => {
                                        (rx + stroke.width * 0.5, ry + stroke.width * 0.5)
                                    }
                                    ShapeOutline::Inside => (
                                        (rx - stroke.width * 0.5).max(0.25),
                                        (ry - stroke.width * 0.5).max(0.25),
                                    ),
                                };
                                // ~3 pt chords keep the polygon smooth at any zoom.
                                let n = ((rx.max(ry) * TAU / 3.0) as usize).clamp(24, 256);
                                let points: Vec<Pos2> = (0..n)
                                    .map(|i| {
                                        let t = TAU * i as f32 / n as f32;
                                        pos2(center.x + rx * t.cos(), center.y + ry * t.sin())
                                    })
                                    .collect();
                                painter.add(Shape::Path(egui::epaint::PathShape {
                                    points,
                                    closed: true,
                                    fill: fill.unwrap_or(Color32::TRANSPARENT),
                                    stroke: stroke.into(),
                                }));
                            }
                        }
                    }
                }
                // TESTING (owner request 2026-11-21): while the diameter
                // rule shapes the gesture, the overlay also draws WHAT IS
                // MEASURED — the dashed chord between the two raw drag
                // points and the center dot at the segment's midpoint —
                // on top of the shape (only diameter gestures ever carry
                // the flag, so Line never draws guides).
                if *diameter {
                    let guide_color = outline.unwrap_or(fill.unwrap_or(Color32::GRAY));
                    painter.extend(diameter_guides(
                        to_screen(*start),
                        to_screen(*current),
                        guide_color,
                    ));
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
                    // Round caps at both ends, matching the committed
                    // raster (LineCap::Round).
                    painter.circle_filled(pts[0], stroke.width * 0.5, stroke.color);
                    let last = pts[pts.len() - 1];
                    painter.circle_filled(last, stroke.width * 0.5, stroke.color);
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
}
