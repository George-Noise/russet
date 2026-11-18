//! The pure rasterization behind the Tools window's icon buttons (owner
//! request 2026-11-18): every icon is a tiny-skia vector drawing rasterized
//! ONCE into a [`tiny_skia::Pixmap`] at the current `pixels_per_point` and
//! served as an egui texture through `load_texture_cached` — the per-frame
//! cost of a button is one textured quad, not a re-tessellation. The atlas
//! re-rasterizes only when the scale or the theme changes (owner: don't
//! economize on theme switches, they are too rare). Colored and monochrome
//! icons share the SAME draw pipeline: both are polygons/strokes through the
//! same primitives below; monochrome ones just take the theme's text color
//! where the colored ones take a fixed palette. The Eyedropper's icon
//! doubles as the canvas cursor (owner request 2026-11-18) via
//! `Context::set_cursor_image` — egui 0.37 pushes the RGBA bitmap to the
//! integration (`CustomCursorImage`, STRAIGHT alpha, deduped by `Arc` pointer
//! identity); backends without custom cursors (web) silently fall back to the
//! `CursorIcon` the frame also requests, which stays `Crosshair` for the
//! Eyedropper.
//!
//! The pipette (and pencil/brush) are drawn pointing DOWN-LEFT — the tip is
//! where the pointer acts, so the cursor hotspot is the pipette tip's mapped
//! pixel. All geometry lives in a logical 24×24 space (1.5rem icons; 1rem =
//! 16 pt because egui 0.37 has no rem API) and one [`Transform`] maps it to
//! the physical-pixel space of the raster.

use eframe::egui::{self, Color32, Context, Id, TextureHandle, TextureId, TextureOptions, Theme};
use std::sync::Arc;
use tiny_skia::{
    FillRule, LineCap, LineJoin, Paint, Path, PathBuilder, Pixmap, Point, Stroke, Transform,
};

use crate::tools::Tool;

/// The button/icon size in logical points: 1.5rem = 24 pt (1rem = 16 pt;
/// egui 0.37 has no rem API — same convention as `placement::FIRST_OPEN_GAP`).
pub(super) const ICON_PT: f32 = 1.5 * 16.0;

/// The logical drawing space every icon is laid out in (`0..=24` on both
/// axes); the raster transform maps it onto the `px × px` physical bitmap.
const SIZE: f32 = 24.0;

/// The tools' canonical order — the icon atlas and the Tools window grid
/// iterate this array, so the two always agree.
pub(super) const TOOLS: [Tool; 7] = [
    Tool::Pencil,
    Tool::Brush,
    Tool::Eraser,
    Tool::Eyedropper,
    Tool::Line,
    Tool::Shape,
    Tool::Text,
];

/// The hover tooltip of a tool button: the name and the keyboard key
/// (owner request 2026-11-18; the same strings the text buttons used).
pub(super) fn label(tool: Tool) -> &'static str {
    match tool {
        Tool::Pencil => "Pencil (P)",
        Tool::Brush => "Brush (B)",
        Tool::Eraser => "Eraser (E)",
        Tool::Eyedropper => "Eyedropper (I)",
        Tool::Line => "Line (L)",
        Tool::Shape => "Shape (U)",
        Tool::Text => "Text (T)",
    }
}

/// The rasterized icon set behind the Tools window. Stored in `App`; rebuilt
/// by [`ensure`] whenever the scale or the theme changes.
pub(super) struct Atlas {
    /// `(pixels_per_point, dark)` the textures were rasterized at.
    key: (f32, bool),
    /// The uploaded textures, one per [`Tool`], same order as [`TOOLS`].
    /// The HANDLES are kept on purpose: a `TextureHandle` drop decrements
    /// the texture's retain count (`TextureManager::free`), and a texture
    /// nobody holds is deleted — the renderer then draws nothing where the
    /// freed `TextureId` was sampled (an atlas that stored bare `TextureId`s
    /// rendered empty buttons; caught by the visual-driver screenshot).
    pub(super) handles: Vec<TextureHandle>,
    /// The pipette's cursor bitmap (STRAIGHT alpha) — set on the canvas
    /// while the Eyedropper is the active tool (owner request 2026-11-18).
    pub(super) pipette_cursor: Option<egui::CustomCursorImage>,
}

/// (Re-)rasterizes the whole icon atlas when needed. Cheap no-op while the
/// stored `(pixels_per_point, dark)` key matches the context: the expensive
/// part (tiny-skia drawing + the texture upload) runs once per scale/theme
/// change, exactly the "draw once, reuse" pattern the owner asked for.
/// Total: on any raster failure the OLD atlas is kept and the call is
/// retried next frame (the pixel size is clamped to `1..=256`, so this is
/// unreachable in practice).
pub(super) fn ensure(atlas: &mut Option<Atlas>, ctx: &Context) {
    let ppp = ctx.pixels_per_point();
    let dark = ctx.theme() == Theme::Dark;
    if atlas.as_ref().is_some_and(|a| a.key == (ppp, dark)) {
        return;
    }
    // The monochrome icons take the ACTIVE theme's text color; the colored
    // ones ignore it (same pipeline, different palette — see the module doc).
    let text = ctx
        .style_of(if dark { Theme::Dark } else { Theme::Light })
        .visuals
        .text_color();
    let px = (ICON_PT * ppp).round().clamp(1.0, 256.0) as usize;

    let mut handles = Vec::with_capacity(TOOLS.len());
    let mut cursor = None;
    for tool in TOOLS {
        let Some(pm) = rasterize_tool_icon(tool, px, text) else {
            return; // keep the previous atlas; retried next frame
        };
        if tool == Tool::Eyedropper {
            cursor = cursor_bitmap(&pm);
        }
        let image = egui::ColorImage::from_rgba_premultiplied([px, px], pm.data());
        handles.push(ctx.load_texture_cached(
            format!("russet_tool_icon_{tool:?}"),
            Id::unique(format!("russet_tool_icon_{tool:?}")),
            image,
            TextureOptions::LINEAR,
        ));
    }
    let Some(pipette_cursor) = cursor else {
        return; // unreachable: TOOLS contains the Eyedropper
    };
    *atlas = Some(Atlas {
        key: (ppp, dark),
        handles,
        pipette_cursor: Some(pipette_cursor),
    });
}

/// The icon's TextureId for a tool, if the atlas is ready.
pub(super) fn texture_id(atlas: &Option<Atlas>, tool: Tool) -> Option<TextureId> {
    let i = TOOLS.iter().position(|t| t == &tool)?;
    atlas
        .as_ref()
        .and_then(|a| a.handles.get(i))
        .map(TextureHandle::id)
}

// ---------------------------------------------------------------------------
// The shared draw pipeline (owner request 2026-11-18: colored and monochrome
// icons must not duplicate code) — every icon is closed polygons + strokes
// through these three primitives, then one outline pass for the colored ones.
// ---------------------------------------------------------------------------

/// Fills the closed polygon `pts` (logical space, mapped by `ts`) with `color`.
fn fill(pm: &mut Pixmap, ts: Transform, pts: &[(f32, f32)], color: Color32) {
    let paint = solid_paint(color);
    if let Some(path) = polygon(pts, true) {
        pm.fill_path(&path, &paint, FillRule::Winding, ts, None);
    }
}

/// Strokes the (optionally closed) polyline `pts` with `color` and round
/// caps/joins — the monochrome icons' whole body and the colored outline.
fn stroke(
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

/// A diagonal icon (pencil/brush/pipette) is drawn horizontally in the
/// logical space and mapped by scale ∘ rotate-about-center; the same
/// transform maps the pipette tip to the cursor hotspot's pixel.
fn icon_transform(px: usize, angle_deg: f32) -> Transform {
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

/// The colored icons' thin dark outline around the tool's silhouette.
const OUTLINE_W: f32 = 0.9;

// ---------------------------------------------------------------------------
// The icons (owner specs 2026-11-18). Colored: Pencil (classic yellow body,
// wood cone, black graphite, eraser on the end), Brush (varnished-wood
// handle + small metal ferrule + bristles), Eraser (classic pink slab),
// Eyedropper (pipette — also the canvas cursor). Monochrome (theme text
// color): Line, Shape, Text.
// ---------------------------------------------------------------------------

/// The angle every hand-held tool is drawn at (tip pointing down-left).
const TOOL_ANGLE: f32 = -45.0;

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

fn draw_brush(pm: &mut Pixmap, ts: Transform) {
    fill(
        pm,
        ts,
        &[(1.4, 11.8), (3.4, 11.5), (3.4, 12.5), (1.4, 12.2)],
        Color32::from_rgb(110, 74, 40),
    ); // loaded tip
    fill(
        pm,
        ts,
        &[(3.4, 11.5), (9.0, 10.0), (9.0, 14.0), (3.4, 12.5)],
        Color32::from_rgb(216, 169, 107), // bristles
    );
    fill(
        pm,
        ts,
        &[(9.0, 10.0), (11.6, 10.0), (11.6, 14.0), (9.0, 14.0)],
        Color32::from_rgb(185, 190, 198),
    ); // metal ferrule
    fill(
        pm,
        ts,
        &[(11.6, 10.3), (22.6, 10.3), (22.6, 12.0), (11.6, 12.0)],
        Color32::from_rgb(180, 113, 60),
    ); // handle, light facet
    fill(
        pm,
        ts,
        &[(11.6, 12.0), (22.6, 12.0), (22.6, 13.7), (11.6, 13.7)],
        Color32::from_rgb(148, 85, 39),
    ); // handle, dark facet
    stroke(
        pm,
        ts,
        &[
            (1.4, 12.0),
            (9.0, 10.0),
            (22.6, 10.3),
            (22.6, 13.7),
            (9.0, 14.0),
        ],
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
        Color32::from_rgb(180, 90, 116),
        OUTLINE_W,
    );
    stroke(
        pm,
        ts,
        &[m1, m2],
        false,
        Color32::from_rgb(180, 90, 116),
        OUTLINE_W,
    );
}

/// The pipette's TIP in the logical space — the cursor hotspot's source
/// (owner request 2026-11-18: the icon doubles as the canvas cursor, and the
/// tip is where the pointer samples).
const PIPETTE_TIP: (f32, f32) = (1.6, 12.0);

fn draw_pipette(pm: &mut Pixmap, ts: Transform) {
    fill(
        pm,
        ts,
        &[(1.6, 11.3), (7.4, 11.3), (7.4, 12.7), (1.6, 12.7)],
        Color32::from_rgb(201, 212, 221),
    ); // glass tip tube
    fill(
        pm,
        ts,
        &[(1.6, 11.7), (7.4, 11.7), (7.4, 12.3), (1.6, 12.3)],
        Color32::from_rgb(95, 168, 220),
    ); // liquid in the tip
    fill(
        pm,
        ts,
        &[(7.4, 10.6), (13.8, 10.6), (13.8, 13.4), (7.4, 13.4)],
        Color32::from_rgb(213, 223, 231),
    ); // glass barrel
    fill(
        pm,
        ts,
        &[(7.4, 11.5), (13.8, 11.5), (13.8, 12.5), (7.4, 12.5)],
        Color32::from_rgb(95, 168, 220),
    ); // liquid in the barrel
    fill(
        pm,
        ts,
        &[(13.8, 10.2), (15.2, 10.2), (15.2, 13.8), (13.8, 13.8)],
        Color32::from_rgb(94, 103, 112),
    ); // metal collar
    fill(
        pm,
        ts,
        &[(15.2, 9.9), (21.6, 9.9), (21.6, 14.1), (15.2, 14.1)],
        Color32::from_rgb(217, 87, 68),
    ); // rubber bulb
    stroke(
        pm,
        ts,
        &[
            (1.6, 12.0),
            (7.4, 11.3),
            (7.4, 10.6),
            (13.8, 10.6),
            (13.8, 10.2),
            (21.6, 9.9),
            (21.6, 14.1),
            (13.8, 13.8),
            (13.8, 13.4),
            (7.4, 13.4),
            (7.4, 12.7),
        ],
        true,
        Color32::from_rgb(58, 68, 77),
        OUTLINE_W,
    );
}

fn draw_line(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(pm, ts, &[(5.4, 18.6), (18.6, 5.4)], false, mono, 1.9);
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
    stroke(
        pm,
        ts,
        &[
            (15.6 + 5.0, 10.6),
            (15.6, 10.6 + 5.0),
            (15.6 - 5.0, 10.6),
            (15.6, 10.6 - 5.0),
        ],
        true,
        mono,
        1.9,
    );
}

fn draw_text(pm: &mut Pixmap, ts: Transform, mono: Color32) {
    stroke(pm, ts, &[(6.2, 7.6), (17.8, 7.6)], false, mono, 2.2); // the T's top bar
    stroke(pm, ts, &[(12.0, 7.6), (12.0, 18.8)], false, mono, 2.2); // the T's stem
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

/// Converts the pipette pixmap into the cursor bitmap: egui's
/// `CustomCursorImage` wants STRAIGHT (non-premultiplied) RGBA — the same
/// encoding `winit::window::CustomCursor::from_rgba` takes — and the hotspot
/// at the pipette tip's mapped pixel.
fn cursor_bitmap(pm: &Pixmap) -> Option<egui::CustomCursorImage> {
    let (w, h) = (pm.width(), pm.height());
    if w == 0 || h == 0 || w > u16::MAX as u32 || h > u16::MAX as u32 {
        return None;
    }
    let mut rgba = Vec::with_capacity((w * h * 4) as usize);
    for p in pm.pixels() {
        let c = p.demultiply();
        rgba.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    let mut tip = Point::from_xy(PIPETTE_TIP.0, PIPETTE_TIP.1);
    icon_transform(w as usize, TOOL_ANGLE).map_point(&mut tip);
    let hotspot = [
        (tip.x.round().clamp(0.0, (w - 1) as f32)) as u16,
        (tip.y.round().clamp(0.0, (h - 1) as f32)) as u16,
    ];
    Some(egui::CustomCursorImage {
        rgba: Arc::from(rgba),
        size: [w as u16, h as u16],
        hotspot,
    })
}

#[cfg(test)]
mod tests;
