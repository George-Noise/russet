//! The icon DRAWINGS behind the Tools window's buttons and the panels'
//! buttons (owner request 2026-11-18). Every icon is closed polygons +
//! round-cap strokes through the parent's SHARED PRIMITIVES ([`super`]'s
//! `fill`/`stroke`/`arc_pts`/`quad_pts`/`icon_transform`/`OUTLINE_W`),
//! laid out in the logical 24×24 space. The pipeline and the color policy
//! live in the parent module ([`super`]): the atlas, the cursor conversion
//! and the texture upload; the monochrome icons take the fixed
//! [`super::TOOL_INK`] passed in as the `text` argument, the colored ones
//! carry their own fixed palettes inline. The drawing facts owned HERE are
//! the hand-held tools' angle ([`TOOL_ANGLE`]), the pipette tip
//! ([`PIPETTE_TIP`]), the Shape panel's preview fill
//! ([`SHAPE_PREVIEW_FILL`]) and the SHARED geometry several icons reuse
//! (the tool-emblem rect/circle, the Line emblem curve, the dashed Select
//! marks — one geometry for the tool button AND its panel row).
//!
//! FILE LAYOUT (two owner-approved file-guard splits, pure code motion):
//! the 2026-11-19 one moved the drawings out of the pipeline parent; the
//! 2026-11-21 one cut THIS file further — the per-tool drawings live in
//! the child [`tools`], the panel sets in the child [`panels`], and this
//! file keeps only the shared geometry + the re-exports, so the parent
//! module's and the tests' paths are unchanged.

use super::{arc_pts, quad_pts, stroke};
use eframe::egui::Color32;
use tiny_skia::{Pixmap, Transform};

mod panels;
mod tools;

pub(super) use panels::{rasterize_line_icon, rasterize_select_icon, rasterize_shape_icon};
pub(super) use tools::rasterize_tool_icon;

/// The angle every hand-held tool is drawn at (tip pointing down-left).
pub(super) const TOOL_ANGLE: f32 = -45.0;

/// The pencil's optical nudge (owner request 2026-11-21: move the pencil
/// a touch LEFT-DOWN relative to the icon's center — judged against the
/// `--atlas-test` rings). Design units of the 24 pt emblem space; the
/// dispatcher applies it AFTER `icon_transform` (through
/// `primitives::nudged_icon_transform`), so the direction is screen-fixed
/// — the -45° rotation cannot turn it — and it scales with the raster.
pub(super) const PENCIL_NUDGE: (f32, f32) = (-1.0, 1.0);

/// The pipette's TIP in the logical space — the cursor hotspot's source
/// (owner request 2026-11-18: the icon doubles as the canvas cursor, and the
/// tip is where the pointer samples).
pub(super) const PIPETTE_TIP: (f32, f32) = (1.6, 12.0);

/// The Line tool's icon is a CURVE, not a straight segment (owner request
/// 2026-11-18) — the tool itself has curve mode, and the icon should hint
/// at it: a smooth S of two quadratic arcs (the first bends along the
/// bottom, the second turns over the top), unmistakably a curve at both
/// the 256 px design view and the real 24 px button.
/// The Line TOOL's emblem, shared by the Tools window's Line button and the
/// panel's Curve button (owner request 2026-11-20 — one geometry, the Shape
/// emblem's principle): the two quadratic arcs `draw_line` strokes.
fn line_emblem_curve() -> Vec<(f32, f32)> {
    let mut curve = quad_pts((4.8, 18.6), (13.5, 18.8), (14.4, 11.4), 14);
    curve.extend(quad_pts((14.4, 11.4), (15.0, 4.2), (19.2, 4.8), 14));
    curve
}

fn draw_line(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(pm, ts, &line_emblem_curve(), false, mono, 1.9);
}

/// The Shape TOOL's emblem, shared by the Tools window's Shape button and
/// the panel's MODE radios (owner request 2026-11-20: "воспользуемся
/// иконкой инструмента Shape" — one geometry, so the panel's radios and
/// the tool button draw the SAME mark): the rectangle + the circle
/// `draw_shape` strokes.
const EMBLEM_RECT: [(f32, f32); 4] = [(3.6, 9.2), (13.8, 9.2), (13.8, 19.4), (3.6, 19.4)];

/// The emblem's circle (the CIRCLE the owner replaced the diamond with,
/// 2026-11-18): center (15.6, 10.6), r 5.0.
fn emblem_circle() -> Vec<(f32, f32)> {
    arc_pts(15.6, 10.6, 5.0, 0.0, 360.0, 40)
}

/// The kind buttons' preview fill: a mid neutral gray — clearly darker than
/// the buttons' light resting fill (so the interior reads as FILLED) and
/// clearly lighter than the ink rim (so the rim still draws the shape's
/// edge). Fixed constant: the icons are theme-independent like the set.
const SHAPE_PREVIEW_FILL: Color32 = Color32::from_gray(160);

/// The shared 15×15 square, centered (4.5..19.5 on both axes).
const SQUARE_PTS: [(f32, f32); 4] = [(4.5, 4.5), (19.5, 4.5), (19.5, 19.5), (4.5, 19.5)];

/// The dashed RECTANGLE selection mark (owner request 2026-11-20): the
/// dashed outline says "this is a selection", the square says the kind —
/// the same mark the Tools window's Select button draws in Rect mode and
/// the panel's mode row shows.
fn draw_select_rect(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    let square = [(4.0, 4.0), (20.0, 4.0), (20.0, 20.0), (4.0, 20.0)];
    for run in super::dash_polyline(&square, true, 4.5, 3.5) {
        stroke(pm, ts, &run, false, mono, 1.9);
    }
}

/// The dashed LASSO selection mark: the same dashes around an irregular
/// round BLOT (owner request 2026-11-20: "не кружочком, а в виде круглой
/// кляксы"; the second take read as an upside-down PEAR — one smooth
/// taper). Ten anchors: FIVE lobes (radii 9.3/7.6/8.8/7.2/9.0 at
/// 0°/72°/144°/216°/288°) interleaved with FIVE deep WAISTS (5.0–6.2)
/// pulled toward the center — pinched notches between the lumps, no
/// smooth taper anywhere. The shape was picked from rendered candidates
/// (the lobe/waist splat beat a 4-lobe diamond and the pear). Smoothed C1
/// through the anchors' midpoints (each anchor is its segment's quadratic
/// control), then dashed like the rect mark.
fn draw_select_lasso(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    const ANCHORS: [(f32, f32); 10] = [
        (21.3, 12.0), // lobe 0° r9.3
        (16.0, 14.9), // waist 36° r5.0
        (14.3, 19.2), // lobe 72° r7.6
        (10.1, 17.9), // waist 108° r6.2
        (4.9, 17.2),  // lobe 144° r8.8
        (7.4, 12.0),  // waist 180° r4.6
        (6.2, 7.8),   // lobe 216° r7.2
        (10.3, 6.7),  // waist 252° r5.6
        (14.8, 3.4),  // lobe 288° r9.0
        (16.0, 9.1),  // waist 324° r4.9
    ];
    let n = ANCHORS.len();
    let mut blob = Vec::new();
    for (i, &anchor) in ANCHORS.iter().enumerate() {
        let prev = ANCHORS[(i + n - 1) % n];
        let next = ANCHORS[(i + 1) % n];
        let mid0 = ((prev.0 + anchor.0) / 2.0, (prev.1 + anchor.1) / 2.0);
        let mid1 = ((anchor.0 + next.0) / 2.0, (anchor.1 + next.1) / 2.0);
        blob.extend(quad_pts(mid0, anchor, mid1, 8));
    }
    for run in super::dash_polyline(&blob, true, 3.2, 2.6) {
        stroke(pm, ts, &run, false, mono, 1.9);
    }
}
