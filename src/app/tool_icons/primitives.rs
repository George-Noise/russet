//! The shared draw primitives every tool icon is built from (pure code
//! motion, owner-approved 2026-11-21 file-guard split — the second cut of
//! the 2026-11-19 one): fills, round-cap strokes, the marching-ants dash
//! walker, the logical-space transform and the point-grid helpers
//! ([`arc_pts`] / [`quad_pts`]). The icon DRAWINGS stay in [`super::draw`]
//! (its own children since the same pass), the atlas/cursor pipeline in
//! the parent; a primitive's consumers import it through the parent's
//! re-exports, so no call site changed.

use eframe::egui::Color32;
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Stroke, Transform};

/// The logical drawing space every icon is laid out in (`0..=24` on both
/// axes); the raster transform maps it onto the `px × px` physical bitmap.
const SIZE: f32 = 24.0;

/// Fills the closed polygon `pts` (logical space, mapped by `ts`) with `color`.
pub(super) fn fill(pm: &mut Pixmap, ts: Transform, pts: &[(f32, f32)], color: Color32) {
    let paint = solid_paint(color);
    if let Some(path) = polygon(pts, true) {
        pm.fill_path(&path, &paint, FillRule::Winding, ts, None);
    }
}

/// Strokes the (optionally closed) polyline `pts` with `color` and round
/// caps/joins — the monochrome icons' whole body and the colored outline.
pub(super) fn stroke(
    pm: &mut Pixmap,
    ts: Transform,
    pts: &[(f32, f32)],
    closed: bool,
    color: Color32,
    w: f32,
) {
    let paint = solid_paint(color);
    if let Some(path) = polygon(pts, closed) {
        let s = Stroke {
            width: w,
            line_cap: LineCap::Round,
            line_join: LineJoin::Round,
            ..Stroke::default()
        };
        pm.stroke_path(&path, &paint, &s, ts, None);
    }
}

/// Walks a polyline (closed = the last point connects back to the first)
/// and emits DASH segments of `dash` separated by `gap` — the marching-ants
/// marks the Select family draws (owner request 2026-11-20: the dashed
/// outline says "selection", the shape says the kind). A dash crossing a
/// corner keeps going as one run, so every emitted run strokes cleanly
/// through `stroke(.., closed=false, ..)`.
pub(super) fn dash_polyline(
    points: &[(f32, f32)],
    closed: bool,
    dash: f32,
    gap: f32,
) -> Vec<Vec<(f32, f32)>> {
    let mut path: Vec<(f32, f32)> = points.to_vec();
    if closed
        && let (Some(&first), Some(&last)) = (path.first(), path.last())
        && first != last
    {
        path.push(first);
    }
    let mut dashes: Vec<Vec<(f32, f32)>> = Vec::new();
    let mut current: Vec<(f32, f32)> = Vec::new();
    let mut want_dash = true;
    let mut remaining = dash;
    for window in path.windows(2) {
        let (a, b) = (window[0], window[1]);
        let len = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        if len <= 0.0 {
            continue;
        }
        let mut t = 0.0;
        while t < len - 1e-6 {
            let step = remaining.min(len - t);
            let u0 = t / len;
            let u1 = (t + step) / len;
            if want_dash {
                if current.is_empty() {
                    current.push((a.0 + (b.0 - a.0) * u0, a.1 + (b.1 - a.1) * u0));
                }
                current.push((a.0 + (b.0 - a.0) * u1, a.1 + (b.1 - a.1) * u1));
            }
            t += step;
            remaining -= step;
            if remaining <= 1e-6 {
                if want_dash && !current.is_empty() {
                    dashes.push(std::mem::take(&mut current));
                }
                want_dash = !want_dash;
                remaining = if want_dash { dash } else { gap };
            }
        }
    }
    if want_dash && !current.is_empty() {
        dashes.push(current);
    }
    dashes
}

/// A diagonal icon (pencil/brush/pipette) is drawn horizontally in the
/// logical space and mapped by scale ∘ rotate-about-center; the same
/// transform maps the pipette tip to the cursor hotspot's pixel.
pub(super) fn icon_transform(px: usize, angle_deg: f32) -> Transform {
    let k = px as f32 / SIZE;
    let (s, c) = angle_deg.to_radians().sin_cos();
    let (cx, cy) = (SIZE / 2.0, SIZE / 2.0);
    // p' = k·(R·(p − c) + c)  =>  linear part k·R, translation k·(c − R·c).
    Transform::from_row(
        k * c,
        k * s,
        -k * s,
        k * c,
        k * (cx - (c * cx - s * cy)),
        k * (cy - (s * cx + c * cy)),
    )
}

/// [`icon_transform`] with a small optical nudge bolted on AFTER the
/// rotate-about-center: the design-space shift (`dx_pt`/`dy_pt` in the
/// 24 pt emblem space, scaled by the same `k`) lands in the transform's
/// TRANSLATION column, so the linear part is untouched and the direction
/// stays screen-fixed under any tool angle. The pencil's
/// `PENCIL_NUDGE` (draw.rs) is the only user.
pub(super) fn nudged_icon_transform(
    px: usize,
    angle_deg: f32,
    dx_pt: f32,
    dy_pt: f32,
) -> Transform {
    let mut ts = icon_transform(px, angle_deg);
    let k = px as f32 / SIZE;
    ts.tx += dx_pt * k;
    ts.ty += dy_pt * k;
    ts
}

fn solid_paint(color: Color32) -> Paint<'static> {
    Paint {
        // egui's Color32 is premultiplied; the fixed palettes and the text
        // color are OPAQUE, so the components pass through unchanged.
        shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(
            color.r(),
            color.g(),
            color.b(),
            color.a(),
        )),
        anti_alias: true,
        ..Paint::default()
    }
}

fn polygon(pts: &[(f32, f32)], closed: bool) -> Option<Path> {
    let mut pb = PathBuilder::new();
    let &(x0, y0) = pts.first()?;
    pb.move_to(x0, y0);
    for &(x, y) in &pts[1..] {
        pb.line_to(x, y);
    }
    if closed {
        pb.close();
    }
    pb.finish()
}

/// Points along a circular arc in the logical space (y-down: 0° = +x, 90° =
/// +y downward). The round parts of the redrawn icons (2026-11-18 owner
/// request: a volumetric round bristle belly, a recognizable rubber bulb)
/// are short line segments through this helper — anti-aliasing hides the
/// tessellation at button sizes.
pub(super) fn arc_pts(
    cx: f32,
    cy: f32,
    r: f32,
    a0_deg: f32,
    a1_deg: f32,
    steps: usize,
) -> Vec<(f32, f32)> {
    (0..=steps)
        .map(|i| {
            let a = (a0_deg + (a1_deg - a0_deg) * i as f32 / steps as f32).to_radians();
            (cx + r * a.cos(), cy + r * a.sin())
        })
        .collect()
}

/// Points along a quadratic Bézier (`p0` → `p1`, bent toward `ctrl`) —
/// the curve icons sample through this (the Line icon is a curve — the
/// tool itself has curve mode; the Text icon's serif hooks are curves).
pub(super) fn quad_pts(
    p0: (f32, f32),
    ctrl: (f32, f32),
    p1: (f32, f32),
    steps: usize,
) -> Vec<(f32, f32)> {
    (0..=steps)
        .map(|i| {
            let t = i as f32 / steps as f32;
            let u = 1.0 - t;
            (
                u * u * p0.0 + 2.0 * u * t * ctrl.0 + t * t * p1.0,
                u * u * p0.1 + 2.0 * u * t * ctrl.1 + t * t * p1.1,
            )
        })
        .collect()
}

/// The colored icons' outline width around the tool's silhouette (owner
/// request 2026-11-18: "a bit thicker" — 0.9 vanished at 1.5rem). V3 (owner,
/// after the V36 pass): the rim is the SAME dark color in BOTH themes — the
/// tool buttons now keep the light-theme background in the dark theme
/// (`windows.rs::tool_button`), so one dark rim reads well everywhere.
pub(super) const OUTLINE_W: f32 = 1.25;
