//! The PURE geometry of the preview gestures (pure code motion,
//! owner-approved 2026-11-21 file-guard split): the Shift-proportional
//! drag rule ([`proportional_current`] — owner request 2026-11-16,
//! Paint.NET style) and the TESTING diameter-circle rule for the Ellipse
//! kind ([`diameter_rect`] + the live [`diameter_guides`] overlay shapes —
//! owner request 2026-11-21). Pure functions of their arguments — the
//! gesture code and the tests share this math; nothing here touches `App`.

use eframe::egui::{self, Color32, Pos2, Rect, Shape, Stroke, pos2};

/// The free drag `current` point pulled onto the proportional square that
/// grows from `start` along the DOMINANT axis (Paint.NET's Shift-constrained
/// shapes — owner request 2026-11-16): the larger of the two deltas wins as
/// the side length, and each axis keeps its drag direction, so the square
/// always reaches toward the pointer from the anchor corner. A strictly
/// one-sided drag (one delta zero) still yields a square: the zero axis
/// extends by the full side in the down/right direction (Rust's
/// `f32::signum` maps `+0.0` to `1.0`), matching Paint.NET — a Shift drag
/// never produces a degenerate shape once the pointer has moved. Pure — no
/// egui, no `App` — so the rule stays unit-testable.
pub(super) fn proportional_current(start: Pos2, current: Pos2) -> Pos2 {
    let dx = current.x - start.x;
    let dy = current.y - start.y;
    let side = dx.abs().max(dy.abs());
    pos2(start.x + side * dx.signum(), start.y + side * dy.signum())
}

/// The TESTING diameter-circle rule (owner request 2026-11-21): with the
/// Ellipse kind and `Shift` held (the panel's feature checkbox on), the
/// drag's two points do NOT corner the shape — they both lie ON the circle,
/// and the segment between them is its DIAMETER: the center is the segment's
/// midpoint and the radius is half its length. The returned rect is the
/// circle's bounding square — the exact opposite corners
/// [`crate::tools::rasterize_shape`] inscribes the ellipse into, so the
/// committed boundary passes through both drag points. Pure — no egui
/// state, no `App` — so the rule stays unit-testable.
pub(super) fn diameter_rect(start: Pos2, current: Pos2) -> Rect {
    let center = Pos2::new((start.x + current.x) * 0.5, (start.y + current.y) * 0.5);
    let radius = (current - start).length() * 0.5;
    Rect::from_center_size(center, egui::Vec2::splat(radius * 2.0))
}

/// The TESTING diameter-circle guides (owner request 2026-11-21): while a
/// diameter gesture is live, the overlay draws WHAT IS MEASURED on top of
/// the shape — a thin DASHED chord between the two drag points and a small
/// dot at the segment's midpoint (the circle's center; the white-core +
/// dark-ring marker style of the polyline's vertex handles, visible on any
/// canvas color) — so the rule reads while dragging. Overlay-only: the
/// commit rasterizes just the ellipse, the guides die with the gesture.
/// Screen-space (the overlay's coordinate system); pure — no egui state, no
/// `App` — so the geometry stays unit-testable.
pub(super) fn diameter_guides(a: Pos2, b: Pos2, color: Color32) -> Vec<Shape> {
    let mut shapes = Shape::dashed_line(&[a, b], Stroke::new(1.0, color), 4.0, 3.0);
    let center = Pos2::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5);
    shapes.push(Shape::circle_filled(center, 2.5, Color32::WHITE));
    shapes.push(Shape::circle_stroke(
        center,
        2.5,
        Stroke::new(1.0, Color32::from_black_alpha(180)),
    ));
    shapes
}
