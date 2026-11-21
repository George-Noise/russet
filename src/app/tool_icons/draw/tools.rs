//! The per-TOOL icon drawings (pure code motion, owner-approved 2026-11-21
//! file-guard split): pencil, brush, eraser, pipette, shape, text — plus
//! the atlas's per-tool dispatcher [`rasterize_tool_icon`]. The Line and
//! Select buttons draw their marks through the parent's shared drawings
//! ([`super::draw_line`], [`super::draw_select_rect`]) because the Line
//! panel's Curve choice and the Select panel's mode row reuse the SAME
//! geometry. Every icon is closed polygons + round-cap strokes through the
//! pipeline's primitives (imported from [`super`] and its re-exports).

use super::super::{
    OUTLINE_W, arc_pts, fill, icon_transform, nudged_icon_transform, quad_pts, stroke,
};
use super::{EMBLEM_RECT, PENCIL_NUDGE, TOOL_ANGLE, draw_line, draw_select_rect, emblem_circle};
use crate::tools::Tool;
use eframe::egui::Color32;
use tiny_skia::{Pixmap, Transform};

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

fn draw_shape(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(pm, ts, &EMBLEM_RECT, true, mono, 1.9);
    // A CIRCLE instead of the old diamond (owner request 2026-11-18).
    stroke(pm, ts, &emblem_circle(), false, mono, 1.9);
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
pub(in crate::app::tool_icons) fn rasterize_tool_icon(
    tool: Tool,
    px: usize,
    text: Color32,
) -> Option<Pixmap> {
    let mut pm = Pixmap::new(px as u32, px as u32)?;
    match tool {
        Tool::Pencil => draw_pencil(
            &mut pm,
            // The pencil's emblem reads a touch up-right of the icon's
            // center (owner request 2026-11-21: nudge it "чуть-чуть
            // влево-вниз"); the shift applies AFTER the rotate-about-
            // center — screen-fixed direction, scaled with the raster.
            nudged_icon_transform(px, TOOL_ANGLE, PENCIL_NUDGE.0, PENCIL_NUDGE.1),
        ),
        Tool::Brush => draw_brush(&mut pm, icon_transform(px, TOOL_ANGLE)),
        Tool::Eraser => draw_eraser(&mut pm, icon_transform(px, 0.0)),
        // The TOOLS array's placeholder slot: the live Select button draws
        // its MODE-AWARE mark through `rasterize_select_icon`
        // (`select_texture_id`) — this keeps the atlas's per-tool indices
        // aligned and gives the dashed-rect mark as the static fallback.
        Tool::Select => draw_select_rect(&mut pm, icon_transform(px, 0.0), text),
        Tool::Eyedropper => draw_pipette(&mut pm, icon_transform(px, TOOL_ANGLE)),
        Tool::Line => draw_line(&mut pm, icon_transform(px, 0.0), text),
        Tool::Shape => draw_shape(&mut pm, icon_transform(px, 0.0), text),
        Tool::Text => draw_text(&mut pm, icon_transform(px, 0.0), text),
    }
    Some(pm)
}
