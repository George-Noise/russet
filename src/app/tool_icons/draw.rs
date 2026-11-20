//! The icon DRAWINGS behind the Tools window's buttons and the Shape
//! panel's buttons: the per-tool `draw_*` functions, the
//! [`rasterize_tool_icon`] dispatcher and the Shape-panel set
//! ([`rasterize_shape_icon`] + its `draw_kind_preview` / `draw_mode`). Every
//! icon is closed polygons + round-cap strokes through the parent's SHARED
//! PRIMITIVES ([`super::fill`], [`super::stroke`], [`super::arc_pts`],
//! [`super::quad_pts`], [`super::icon_transform`], [`super::OUTLINE_W`] —
//! imported below), laid out in the logical 24×24 space. The pipeline and
//! the color policy live in the parent module ([`super`]): the atlas, the
//! cursor conversion and the texture upload; the monochrome icons take the
//! fixed [`super::TOOL_INK`] passed in as the `text` argument, the colored
//! ones carry their own fixed palettes inline. The drawing facts owned HERE
//! are the hand-held tools' angle ([`TOOL_ANGLE`]), the pipette tip
//! ([`PIPETTE_TIP`] — the cursor hotspot's source) and the Shape panel's
//! preview fill ([`SHAPE_PREVIEW_FILL`]).
//!
//! File layout: the owner-approved 2026-11-19 file-guard split (pure code
//! motion — the parent was 673 lines; the drawings moved over VERBATIM, no
//! body or color changed).

use super::{OUTLINE_W, arc_pts, fill, icon_transform, quad_pts, stroke};
use crate::tools::{ShapeKind, Tool};
use eframe::egui::Color32;
use tiny_skia::{Pixmap, Transform};

// ---------------------------------------------------------------------------
// The icons (owner specs 2026-11-18; the v3 refinements after the owner's
// V36 pass: the theme-aware light rim was REVERTED to the dark rim in both
// themes — the icon buttons carry a light-theme background now — and the
// pipette's glass became a full CYLINDER with a cone only at the tip plus a
// wider collar; the v2 same-day refinements: outlines thicker, the pipette's
// collar widened into the classic fat band, the Line icon a CURVE (the tool
// has curve mode), the Shape icon's diamond became a circle, and the Text
// icon is a serif-curved T with a small lowercase t beside it). Colored:
// Pencil (classic yellow body, wood cone, black graphite, metal ferrule,
// eraser on the end), Brush (varnished-wood handle + metal ferrule + a ROUND
// volumetric bristle belly with a loaded tip), Eraser (classic pink slab),
// Eyedropper (glass cylinder with a liquid stripe + WIDE metal collar +
// round rubber bulb — also the canvas cursor). Monochrome (theme text
// color): Line, Shape, Text.
// ---------------------------------------------------------------------------

/// The angle every hand-held tool is drawn at (tip pointing down-left).
pub(super) const TOOL_ANGLE: f32 = -45.0;

fn draw_pencil(pm: &mut Pixmap, ts: Transform) {
    // Local space: horizontal pencil along y = 12, tip (graphite) on the
    // LEFT. The shaft is the uniform 9.6..14.4 band; the cone tapers from
    // the shaft to the tip, and the outline traces THAT silhouette (an
    // outline through the cone's mid-slope would eat the graphite tip).
    fill(
        pm,
        ts,
        &[(1.8, 12.0), (4.6, 10.6), (4.6, 13.4)],
        Color32::from_rgb(60, 60, 60),
    ); // graphite
    fill(
        pm,
        ts,
        &[(4.6, 10.6), (4.6, 13.4), (9.0, 14.4), (9.0, 9.6)],
        Color32::from_rgb(231, 196, 151), // wood cone
    );
    fill(
        pm,
        ts,
        &[(9.0, 9.6), (18.6, 9.6), (18.6, 12.0), (9.0, 12.0)],
        Color32::from_rgb(253, 194, 92),
    ); // body, light facet
    fill(
        pm,
        ts,
        &[(9.0, 12.0), (18.6, 12.0), (18.6, 14.4), (9.0, 14.4)],
        Color32::from_rgb(236, 159, 43),
    ); // body, dark facet
    fill(
        pm,
        ts,
        &[(18.6, 9.6), (21.2, 9.6), (21.2, 14.4), (18.6, 14.4)],
        Color32::from_rgb(175, 180, 189),
    ); // metal ferrule
    fill(
        pm,
        ts,
        &[(21.2, 9.6), (23.2, 9.6), (23.2, 14.4), (21.2, 14.4)],
        Color32::from_rgb(240, 144, 138),
    ); // eraser end
    stroke(
        pm,
        ts,
        &[
            (1.8, 12.0),
            (4.6, 10.6),
            (9.0, 9.6),
            (23.2, 9.6),
            (23.2, 14.4),
            (9.0, 14.4),
            (4.6, 13.4),
        ],
        true,
        Color32::from_rgb(75, 58, 40),
        OUTLINE_W,
    );
}

/// A round paint brush (owner request 2026-11-18: the old chisel wedge read
/// as a second pencil — the bristles now bulge into a ROUND belly past the
/// ferrule and taper to the tip, the way classic paint apps draw brushes).
/// The belly is split into light/dark facets along the centerline for the
/// volume; the dark loaded tip (paint) stays.
fn draw_brush(pm: &mut Pixmap, ts: Transform) {
    // The bristle silhouette: tip (1.2, 12.0), round belly peaking at
    // x≈7.0 (half-height 2.7), closing slightly into the ferrule at 10.6
    // (half-height 2.0).
    let bristle_top = [
        (1.2, 12.0),
        (2.6, 10.5),
        (4.6, 9.7),
        (7.0, 9.3),
        (9.2, 9.6),
        (10.6, 10.0),
    ];
    let bristle_bottom = [
        (10.6, 14.0),
        (9.2, 14.4),
        (7.0, 14.7),
        (4.6, 14.3),
        (2.6, 13.5),
    ];
    // Volume: light facet above the centerline, dark below (the pencil's
    // facet pattern; the light half keeps the canonical bristle color).
    fill(
        pm,
        ts,
        &[
            (1.2, 12.0),
            (2.6, 10.5),
            (4.6, 9.7),
            (7.0, 9.3),
            (9.2, 9.6),
            (10.6, 10.0),
            (10.6, 12.0),
        ],
        Color32::from_rgb(216, 169, 107), // bristles, light facet
    );
    fill(
        pm,
        ts,
        &[
            (10.6, 12.0),
            (10.6, 14.0),
            (9.2, 14.4),
            (7.0, 14.7),
            (4.6, 14.3),
            (2.6, 13.5),
            (1.2, 12.0),
        ],
        Color32::from_rgb(192, 142, 84), // bristles, dark facet
    );
    // Paint-loaded tip over the facets (the brush has just picked up color).
    fill(
        pm,
        ts,
        &[
            (1.2, 12.0),
            (2.6, 10.5),
            (3.6, 11.0),
            (3.4, 13.0),
            (2.6, 13.5),
        ],
        Color32::from_rgb(110, 74, 40), // loaded tip
    );
    fill(
        pm,
        ts,
        &[(10.6, 9.9), (13.4, 9.9), (13.4, 14.1), (10.6, 14.1)],
        Color32::from_rgb(185, 190, 198), // metal ferrule
    );
    // The ferrule's crimp ridges (two thin darker lines near the bristles).
    for x in [11.6, 12.3] {
        stroke(
            pm,
            ts,
            &[(x, 10.1), (x, 13.9)],
            false,
            Color32::from_rgb(150, 157, 166),
            0.5,
        );
    }
    fill(
        pm,
        ts,
        &[(13.4, 10.4), (23.0, 10.4), (23.0, 12.0), (13.4, 12.0)],
        Color32::from_rgb(180, 113, 60), // handle, light facet
    );
    fill(
        pm,
        ts,
        &[(13.4, 12.0), (23.0, 12.0), (23.0, 13.6), (13.4, 13.6)],
        Color32::from_rgb(148, 85, 39), // handle, dark facet
    );
    stroke(
        pm,
        ts,
        bristle_top
            .iter()
            .chain(
                [
                    (10.6, 9.9),
                    (13.4, 9.9),
                    (13.4, 10.4),
                    (23.0, 10.4),
                    (23.0, 13.6),
                    (13.4, 13.6),
                    (13.4, 14.1),
                ]
                .iter(),
            )
            .chain(bristle_bottom.iter())
            .copied()
            .collect::<Vec<(f32, f32)>>()
            .as_slice(),
        true,
        Color32::from_rgb(78, 58, 36),
        OUTLINE_W,
    );
}

fn draw_eraser(pm: &mut Pixmap, ts: Transform) {
    // The classic pink slab: a parallelogram split along its long midline.
    let a = (4.6, 13.2);
    let b = (14.2, 6.8);
    let c = (19.6, 12.2);
    let d = (10.0, 18.6);
    let m1 = (7.3, 15.9);
    let m2 = (16.9, 9.5);
    fill(pm, ts, &[a, b, m2, m1], Color32::from_rgb(249, 183, 196)); // top face
    fill(pm, ts, &[m1, m2, c, d], Color32::from_rgb(239, 140, 164)); // front face
    stroke(
        pm,
        ts,
        &[a, b, c, d],
        true,
        Color32::from_rgb(92, 52, 66),
        OUTLINE_W,
    );
    stroke(
        pm,
        ts,
        &[m1, m2],
        false,
        Color32::from_rgb(92, 52, 66),
        OUTLINE_W,
    );
}

/// The pipette's TIP in the logical space — the cursor hotspot's source
/// (owner request 2026-11-18: the icon doubles as the canvas cursor, and the
/// tip is where the pointer samples).
pub(super) const PIPETTE_TIP: (f32, f32) = (1.6, 12.0);

/// A classic eyedropper (owner request 2026-11-18: the old all-rectangle
/// stick read as a pen — the glass tapers to the fine tip and the rubber
/// bulb is a ROUND cap, the way paint apps draw pipettes). The blue liquid
/// fills the tip (the sample) and runs as a stripe inside the glass barrel;
/// the tip stays at [`PIPETTE_TIP`] (cursor hotspot). V3 (owner, after the
/// V36 pass — "still a bit like a needle"): the glass is a full CYLINDER
/// (constant 4.4 height) with a cone ONLY at the tip, and the collar between
/// glass and bulb got wider — spike → fat uniform tube → wide ring → bulb.
fn draw_pipette(pm: &mut Pixmap, ts: Transform) {
    // The glass: a cone only at the very tip (1.6 → 5.0), then a straight
    // cylinder (5.0..15.0 at a constant 9.8..14.2). The cylinder is
    // deliberately WIDER than the liquid stripe plus two outline
    // half-widths — otherwise the outline eats the glass and no opaque
    // glass pixel survives rasterization (caught by the glass test).
    fill(
        pm,
        ts,
        &[
            (1.6, 12.0),
            (5.0, 9.8),
            (15.0, 9.8),
            (15.0, 14.2),
            (5.0, 14.2),
        ],
        Color32::from_rgb(213, 223, 231), // glass: short cone + cylinder
    );
    fill(
        pm,
        ts,
        &[(5.6, 11.9), (14.4, 11.5), (14.4, 12.5), (5.6, 12.1)],
        Color32::from_rgb(95, 168, 220), // liquid stripe in the barrel
    );
    fill(
        pm,
        ts,
        &[(1.6, 12.0), (5.0, 11.6), (5.0, 12.4)],
        Color32::from_rgb(95, 168, 220), // liquid filling the tip (the sample)
    );
    fill(
        pm,
        ts,
        &[(15.0, 8.8), (18.0, 8.8), (18.0, 15.2), (15.0, 15.2)],
        Color32::from_rgb(94, 103, 112), // the WIDE metal collar ring
    );
    // The rubber bulb: a full circle pushed ONTO the collar — the bulb is
    // the icon's biggest red mass (d 6.2), the way a real pipette reads.
    // The silhouette arc below shares the same circle: center (19.9, 12.0),
    // r 3.1.
    fill(
        pm,
        ts,
        &arc_pts(19.9, 12.0, 3.1, 0.0, 360.0, 32),
        Color32::from_rgb(217, 87, 68), // rubber bulb
    );
    stroke(
        pm,
        ts,
        [
            (1.6, 12.0),
            (5.0, 9.8),
            (15.0, 9.8),
            (15.0, 8.8),
            (18.0, 8.8),
        ]
        .iter()
        .chain(arc_pts(19.9, 12.0, 3.1, 239.3, 480.7, 28).iter())
        .chain([(18.0, 15.2), (15.0, 15.2), (15.0, 14.2), (5.0, 14.2)].iter())
        .copied()
        .collect::<Vec<(f32, f32)>>()
        .as_slice(),
        true,
        Color32::from_rgb(58, 68, 77),
        OUTLINE_W,
    );
}

/// The Line tool's icon is a CURVE, not a straight segment (owner request
/// 2026-11-18) — the tool itself has curve mode, and the icon should hint
/// at it: a smooth S of two quadratic arcs (the first bends along the
/// bottom, the second turns over the top), unmistakably a curve at both
/// the 256 px design view and the real 24 px button.
fn draw_line(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    let mut curve = quad_pts((4.8, 18.6), (13.5, 18.8), (14.4, 11.4), 14);
    curve.extend(quad_pts((14.4, 11.4), (15.0, 4.2), (19.2, 4.8), 14));
    stroke(pm, ts, &curve, false, mono, 1.9);
}

fn draw_shape(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(
        pm,
        ts,
        &[(3.6, 9.2), (13.8, 9.2), (13.8, 19.4), (3.6, 19.4)],
        true,
        mono,
        1.9,
    );
    // A CIRCLE instead of the old diamond (owner request 2026-11-18).
    stroke(
        pm,
        ts,
        &arc_pts(15.6, 10.6, 5.0, 0.0, 360.0, 40),
        false,
        mono,
        1.9,
    );
}

/// The Text icon: a big serif-curved T with a SMALL lowercase t beside it
/// (owner request 2026-11-18: draw the T with curved lines, and a small
/// lowercase t next to the big one — like type specimens do).
fn draw_text(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    // The big T's top bar with down-curved serif hooks at both ends, then
    // the stem with a curved foot to the right.
    let mut bar = quad_pts((4.0, 10.2), (4.0, 7.2), (5.2, 7.2), 8);
    bar.push((13.0, 7.2));
    bar.extend(quad_pts((13.0, 7.2), (14.2, 7.2), (14.2, 10.2), 8));
    stroke(pm, ts, &bar, false, mono, 2.0);
    let mut stem = vec![(9.1, 7.4)];
    stem.extend(quad_pts((9.1, 16.6), (9.1, 17.8), (10.6, 17.8), 8));
    stroke(pm, ts, &stem, false, mono, 2.0);
    // The small lowercase t: a hooked stem plus its crossbar, drawn thinner.
    let mut small = vec![(17.2, 10.4), (17.2, 16.0)];
    small.extend(quad_pts((17.2, 16.0), (17.2, 17.6), (18.6, 17.6), 8));
    stroke(pm, ts, &small, false, mono, 1.6);
    stroke(pm, ts, &[(15.4, 13.0), (19.0, 13.0)], false, mono, 1.6);
}

/// Rasterizes one tool icon into a `px × px` premultiplied-RGBA pixmap.
/// `px` must be in `1..=256` (the caller clamps it). Returns `None` only if
/// tiny-skia cannot allocate the pixmap (impossible for the clamped range).
pub(super) fn rasterize_tool_icon(tool: Tool, px: usize, text: Color32) -> Option<Pixmap> {
    let mut pm = Pixmap::new(px as u32, px as u32)?;
    match tool {
        Tool::Pencil => draw_pencil(&mut pm, icon_transform(px, TOOL_ANGLE)),
        Tool::Brush => draw_brush(&mut pm, icon_transform(px, TOOL_ANGLE)),
        Tool::Eraser => draw_eraser(&mut pm, icon_transform(px, 0.0)),
        Tool::Eyedropper => draw_pipette(&mut pm, icon_transform(px, TOOL_ANGLE)),
        Tool::Line => draw_line(&mut pm, icon_transform(px, 0.0), text),
        Tool::Shape => draw_shape(&mut pm, icon_transform(px, 0.0), text),
        Tool::Text => draw_text(&mut pm, icon_transform(px, 0.0), text),
    }
    Some(pm)
}

// ---------------------------------------------------------------------------
// The Shape panel's four icons (owner request 2026-11-20): the word radio
// and the Fill/Outline checkboxes became icon buttons in the Tools window's
// style. All four are centered in the logical 24×24 space and share ONE
// geometry per shape — the same square / circle drawn three ways (the gray
// kind preview, the solid mode toggle, the hollow mode toggle) — so the
// panel reads as "which shape" over "how does it paint".
// ---------------------------------------------------------------------------

/// The kind buttons' preview fill: a mid neutral gray — clearly darker than
/// the buttons' light resting fill (so the interior reads as FILLED) and
/// clearly lighter than the ink rim (so the rim still draws the shape's
/// edge). Fixed constant: the icons are theme-independent like the set.
const SHAPE_PREVIEW_FILL: Color32 = Color32::from_gray(160);

/// The shared 15×15 square, centered (4.5..19.5 on both axes).
const SQUARE_PTS: [(f32, f32); 4] = [(4.5, 4.5), (19.5, 4.5), (19.5, 19.5), (4.5, 19.5)];

/// The kind button: the shape as a NEUTRAL PREVIEW — the mid-gray fill plus
/// the ink rim (the same 1.9 stroke the monochrome tool icons draw).
fn draw_kind_preview(pm: &mut Pixmap, ts: Transform, kind: ShapeKind, mono: Color32) {
    match kind {
        ShapeKind::Rectangle => {
            fill(pm, ts, &SQUARE_PTS, SHAPE_PREVIEW_FILL);
            stroke(pm, ts, &SQUARE_PTS, true, mono, 1.9);
        }
        ShapeKind::Ellipse => {
            let circle = arc_pts(12.0, 12.0, 7.5, 0.0, 360.0, 40);
            fill(pm, ts, &circle, SHAPE_PREVIEW_FILL);
            stroke(pm, ts, &circle, true, mono, 1.9);
        }
    }
}

/// A MODE toggle: the current kind drawn SOLID (the Fill state — the whole
/// silhouette in ink) or HOLLOW (the Outline state — the rim only).
fn draw_mode(pm: &mut Pixmap, ts: Transform, kind: ShapeKind, solid: bool, mono: Color32) {
    match kind {
        ShapeKind::Rectangle => {
            if solid {
                fill(pm, ts, &SQUARE_PTS, mono);
            } else {
                stroke(pm, ts, &SQUARE_PTS, true, mono, 1.9);
            }
        }
        ShapeKind::Ellipse => {
            let circle = arc_pts(12.0, 12.0, 7.5, 0.0, 360.0, 40);
            if solid {
                fill(pm, ts, &circle, mono);
            } else {
                stroke(pm, ts, &circle, true, mono, 1.9);
            }
        }
    }
}

/// Rasterizes one Shape-panel icon for the panel's CURRENT kind into a
/// `px × px` premultiplied-RGBA pixmap (the MODE toggles preview it; the
/// KIND buttons ignore the argument — they are fixed previews of their own
/// shape). Same failure contract as [`rasterize_tool_icon`].
pub(super) fn rasterize_shape_icon(
    icon: super::ShapeIcon,
    kind: ShapeKind,
    px: usize,
    text: Color32,
) -> Option<Pixmap> {
    use super::ShapeIcon;
    let mut pm = Pixmap::new(px as u32, px as u32)?;
    let ts = icon_transform(px, 0.0);
    match icon {
        ShapeIcon::Rectangle => draw_kind_preview(&mut pm, ts, ShapeKind::Rectangle, text),
        ShapeIcon::Ellipse => draw_kind_preview(&mut pm, ts, ShapeKind::Ellipse, text),
        ShapeIcon::Fill => draw_mode(&mut pm, ts, kind, true, text),
        ShapeIcon::Outline => draw_mode(&mut pm, ts, kind, false, text),
    }
    Some(pm)
}
