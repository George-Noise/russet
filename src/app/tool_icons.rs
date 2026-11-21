//! The pure rasterization behind the Tools window's icon buttons (owner
//! request 2026-11-18): every icon is a tiny-skia vector drawing rasterized
//! ONCE into a [`tiny_skia::Pixmap`] at the current `pixels_per_point` and
//! served as an egui texture through `load_texture_cached` — the per-frame
//! cost of a button is one textured quad, not a re-tessellation. Since the
//! v4 pass the icons are FULLY theme-independent (the owner's goal: one
//! drawing reads well in both themes): the colored rims are fixed constants
//! and the monochrome ink is the fixed [`TOOL_INK`] — so the atlas
//! re-rasterizes ONLY when the scale changes. Colored and monochrome
//! icons share the SAME draw pipeline: both are polygons/strokes through the
//! same primitives below; monochrome ones take [`TOOL_INK`] where the
//! colored ones take a fixed palette. The Eyedropper's icon
//! doubles as the canvas cursor (owner request 2026-11-18) via
//! `Context::set_cursor_image` — egui 0.37 pushes the RGBA bitmap to the
//! integration (`CustomCursorImage`, STRAIGHT alpha, deduped by `Arc` pointer
//! identity); backends without custom cursors (web) silently fall back to the
//! `CursorIcon` the frame also requests, which stays `Crosshair` for the
//! Eyedropper.
//!
//! The Shape tool's settings panel reuses the SAME pipeline for its EIGHT
//! button icons ([`ShapeIcon`], owner requests 2026-11-20): the atlas builds
//! them in the same pass — ALL eight are fixed drawings (the two kind
//! previews, the three MODE radios drawing the Shape TOOL's emblem, and the
//! three OUTLINE-PLACEMENT radios drawing the corner composition), so the
//! atlas keys on the scale alone.
//!
//! The pipette (and pencil/brush) are drawn pointing DOWN-LEFT — the tip is
//! where the pointer acts, so the cursor hotspot is the pipette tip's mapped
//! pixel. All geometry lives in a logical 24×24 space (1.5rem icons; 1rem =
//! 16 pt because egui 0.37 has no rem API) and one [`Transform`] maps it to
//! the physical-pixel space of the raster.
//!
//! FILE LAYOUT (the owner-approved 2026-11-19 file-guard split, pure code
//! motion): this file owns the ATLAS/CURSOR PIPELINE — the cached textures
//! ([`ensure`], [`texture_id`], [`cursor_bitmap`]) — and the SHARED DRAW
//! PRIMITIVES every icon is built from (`fill`/`stroke`/`solid_paint`/
//! `polygon`/`arc_pts`/`quad_pts`/`icon_transform`). The ICON DRAWINGS
//! themselves (the seven per-tool fns + the Shape panel's set, their fixed
//! color specs, [`TOOL_ANGLE`] and [`PIPETTE_TIP`], behind the
//! [`rasterize_tool_icon`] / [`rasterize_shape_icon`] dispatchers) live in
//! the child [`draw`] module, imported below — the pipeline's call sites and
//! the tests' `super::` paths are unchanged.

use eframe::egui::{self, Color32, Context, Id, TextureHandle, TextureId, TextureOptions};
use std::sync::Arc;
use tiny_skia::{Pixmap, Point, PremultipliedColorU8};

use crate::tools::Tool;

mod draw;

mod primitives;

use primitives::{
    OUTLINE_W, arc_pts, dash_polyline, fill, icon_transform, nudged_icon_transform, quad_pts,
    stroke,
};

use self::draw::{
    PIPETTE_TIP, TOOL_ANGLE, rasterize_line_icon, rasterize_select_icon, rasterize_shape_icon,
    rasterize_tool_icon,
};

/// The button/icon size in logical points: 1.5rem = 24 pt (1rem = 16 pt;
/// egui 0.37 has no rem API — same convention as `placement::FIRST_OPEN_GAP`).
pub(super) const ICON_PT: f32 = 1.5 * 16.0;

/// The SECOND atlas's icon size (owner request 2026-11-21, size pass the
/// same day: the first 12 pt icons read «слишком маленькие — там ещё
/// довольно много свободного места» once the `--atlas-test` rings made the
/// picture's size visible): the History Tree rows GROW to fit the icon —
/// `row_height = max(text line, TREE_ICON_PT + TREE_ICON_GAP)` in
/// `tree_window.rs` with the rows BACK-TO-BACK, so two consecutive icons
/// end exactly TREE_ICON_GAP (0.0625rem, the owner's follow-up the same
/// day) apart — and the emblem becomes the row's tallest element,
/// dominating it the way the Tools button's emblem does. Still the SAME
/// pass and pipeline as the button icons, keyed by the same scale (a
/// native small raster, NOT a downscaled 24 pt texture: the thin strokes
/// would blur).
pub(super) const TREE_ICON_PT: f32 = 16.0;

/// The tools' canonical order — the icon atlas and the Tools window grid
/// iterate this array, so the two always agree. The owner's 2026-11-21
/// pass swapped two pairs for balance: the Eyedropper up beside the paint
/// trio (it picks colors, so it reads paint-adjacent) and the Text up next
/// to Select, leaving the Line to close the grid after the Shape.
pub(super) const TOOLS: [Tool; 8] = [
    Tool::Pencil,
    Tool::Brush,
    Tool::Eraser,
    Tool::Eyedropper,
    Tool::Select,
    Tool::Text,
    Tool::Shape,
    Tool::Line,
];

/// The hover tooltip of a tool button: the name and the keyboard key
/// (owner request 2026-11-18; the same strings the text buttons used).
pub(super) fn label(tool: Tool) -> &'static str {
    match tool {
        Tool::Pencil => "Pencil (P)",
        Tool::Brush => "Brush (B)",
        Tool::Eraser => "Eraser (E)",
        Tool::Select => "Select (V)",
        Tool::Eyedropper => "Eyedropper (I)",
        Tool::Line => "Line (L)",
        Tool::Shape => "Shape (U)",
        Tool::Text => "Text (T)",
    }
}

/// The monochrome icons' ink (owner v4 pass 2026-11-18: FIXED — no longer
/// the theme's text color, which flipped the Line/Shape/Text icons between
/// near-white and near-black): the LIGHT theme's text color (egui light's
/// noninteractive text, gray 80) a bit darker. The owner asked for "the
/// light-theme color, just a bit darker".
const TOOL_INK: Color32 = Color32::from_gray(64);

/// One icon of the Shape tool's settings panel (owner request 2026-11-20:
/// the word radio and the checkboxes became icon buttons in the Tools
/// window's style). The two KIND buttons are fixed previews (gray-filled
/// square / circle); the three MODE radios draw the Shape TOOL's emblem
/// (the rectangle + circle of the Tools window's Shape button) — solid for
/// Fill, hollow for Outline, gray-filled with the ink rim for Fill+Outline
/// (the modes no longer preview the current kind, they reuse the tool's own
/// icon); the three OUTLINE-PLACEMENT radios (owner request 2026-11-20,
/// the second three-way radio) draw ONE corner of a square — whatever the
/// selected kind is — with the gray fill quadrant, the click dot and the
/// outline bands in the position the mode draws them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ShapeIcon {
    Rectangle,
    Ellipse,
    Fill,
    Outline,
    FillOutline,
    OutlineOutside,
    OutlineCenter,
    OutlineInside,
}

/// The Shape-panel icons' canonical order — the atlas and the panel's rows
/// iterate this array, so the two always agree.
pub(super) const SHAPE_ICONS: [ShapeIcon; 8] = [
    ShapeIcon::Rectangle,
    ShapeIcon::Ellipse,
    ShapeIcon::Fill,
    ShapeIcon::Outline,
    ShapeIcon::FillOutline,
    ShapeIcon::OutlineOutside,
    ShapeIcon::OutlineCenter,
    ShapeIcon::OutlineInside,
];

/// The hover tooltip of a Shape-panel button: EXACTLY the words the old
/// word radio / checkboxes showed (owner request 2026-11-20: "при наведении
/// чтобы было написано то что сейчас написано в их надписях"); the third
/// radio — the explicit Fill+Outline choice the same request added — names
/// both. The outline-placement radios name their mode (owner request
/// 2026-11-20: outside the two points / centered as before / inside).
pub(super) fn shape_icon_label(icon: ShapeIcon) -> &'static str {
    match icon {
        ShapeIcon::Rectangle => "Rectangle",
        ShapeIcon::Ellipse => "Ellipse",
        ShapeIcon::Fill => "Fill",
        ShapeIcon::Outline => "Outline",
        ShapeIcon::FillOutline => "Fill + Outline",
        ShapeIcon::OutlineOutside => "Outline outside",
        ShapeIcon::OutlineCenter => "Outline centered",
        ShapeIcon::OutlineInside => "Outline inside",
    }
}

/// The rasterized icon set behind the Tools window. Stored in `App`; rebuilt
/// by [`ensure`] whenever the scale changes (theme-independent since the v4
/// pass — see the module doc).
pub(super) struct Atlas {
    /// What the textures were rasterized for: the `pixels_per_point` (the
    /// icons are theme-independent AND kind-independent since the owner
    /// request 2026-11-20 — the MODE radios draw the Shape tool's emblem).
    key: f32,
    /// The uploaded textures, one per [`Tool`], same order as [`TOOLS`].
    /// The HANDLES are kept on purpose: a `TextureHandle` drop decrements
    /// the texture's retain count (`TextureManager::free`), and a texture
    /// nobody holds is deleted — the renderer then draws nothing where the
    /// freed `TextureId` was sampled (an atlas that stored bare `TextureId`s
    /// rendered empty buttons; caught by the visual-driver screenshot).
    pub(super) handles: Vec<TextureHandle>,
    /// The uploaded textures of the Shape panel's eight buttons, same order
    /// as [`SHAPE_ICONS`] (owner requests 2026-11-20; the same retain-count
    /// reason as `handles` above — the panel holds nothing, so the ATLAS is
    /// the only owner of these handles).
    pub(super) shape_icons: [TextureHandle; 8],
    /// The uploaded textures of the Line panel's two buttons, same order
    /// as [`LINE_ICONS`] (owner request 2026-11-20; the same retain-count
    /// reason as `shape_icons` above).
    pub(super) line_icons: [TextureHandle; 2],
    /// The uploaded textures of the Select family's six buttons (the two
    /// mode marks + the four ops), same order as [`SELECT_ICONS`] (owner
    /// request 2026-11-20; the same retain-count reason as `line_icons`).
    pub(super) select_icons: [TextureHandle; 6],
    /// The SECOND, smaller atlas (owner request 2026-11-21): the History
    /// Tree row icons at [`TREE_ICON_PT`], one per [`Tool`], same order as
    /// [`TOOLS`]. Rasterized in the same `ensure` pass (the same key), just
    /// at the row size — the tree draws a native [`TREE_ICON_PT`] quad, not a blurred
    /// downscale. The same retain-count reason as every set above.
    pub(super) tree_icons: Vec<TextureHandle>,
    /// The pipette's cursor bitmap (STRAIGHT alpha) — set on the canvas
    /// while the Eyedropper is the active tool (owner request 2026-11-18).
    pub(super) pipette_cursor: Option<egui::CustomCursorImage>,
}

/// (Re-)rasterizes the whole icon atlas when needed. Cheap no-op while the
/// stored key matches the context: the expensive part (tiny-skia drawing +
/// the texture upload) runs once per scale change — exactly the "draw once,
/// reuse" pattern the owner asked for (theme-independent since the v4 pass,
/// and kind-independent since the owner request 2026-11-20 — the MODE
/// radios draw the Shape tool's emblem, so the key is the scale alone).
/// Total: on any raster failure the OLD atlas is kept and the call is
/// retried next frame (the pixel size is clamped to `1..=256`, so this is
/// unreachable in practice).
pub(super) fn ensure(atlas: &mut Option<Atlas>, ctx: &Context, debug_border: bool) {
    let ppp = ctx.pixels_per_point();
    if atlas.as_ref().is_some_and(|a| a.key == ppp) {
        return;
    }
    // The monochrome icons take the FIXED ink; the colored ones have their
    // own fixed palettes (same pipeline, different colors — module doc).
    let text = TOOL_INK;
    let px = (ICON_PT * ppp).round().clamp(1.0, 256.0) as usize;

    let mut handles = Vec::with_capacity(TOOLS.len());
    let mut cursor = None;
    for tool in TOOLS {
        let Some(mut pm) = rasterize_tool_icon(tool, px, text) else {
            return; // keep the previous atlas; retried next frame
        };
        if debug_border {
            // The `--atlas-test` ring rides EVERY picture, the pipette
            // cursor bitmap included (it is converted from this same pm).
            paint_debug_border(&mut pm);
        }
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
    // The Shape panel's eight buttons (owner requests 2026-11-20): fixed
    // drawings, the key carries only the scale.
    let mut shape_icons = Vec::with_capacity(SHAPE_ICONS.len());
    for icon in SHAPE_ICONS {
        let Some(mut pm) = rasterize_shape_icon(icon, px, text) else {
            return; // keep the previous atlas; retried next frame
        };
        if debug_border {
            paint_debug_border(&mut pm);
        }
        let image = egui::ColorImage::from_rgba_premultiplied([px, px], pm.data());
        shape_icons.push(ctx.load_texture_cached(
            format!("russet_shape_icon_{icon:?}"),
            Id::unique(format!("russet_shape_icon_{icon:?}")),
            image,
            TextureOptions::LINEAR,
        ));
    }
    let Some(pipette_cursor) = cursor else {
        return; // unreachable: TOOLS contains the Eyedropper
    };
    // Unreachable fallthrough: the Vec was built over SHAPE_ICONS (8 items).
    let Ok(shape_icons) = <[TextureHandle; 8]>::try_from(shape_icons) else {
        return;
    };
    // The Line panel's two buttons (owner request 2026-11-20): the tool's
    // own emblem per mode — the same fixed-drawing pipeline as the Shape
    // set above.
    let mut line_icons = Vec::with_capacity(LINE_ICONS.len());
    for icon in LINE_ICONS {
        let Some(mut pm) = rasterize_line_icon(icon, px, text) else {
            return; // keep the previous atlas; retried next frame
        };
        if debug_border {
            paint_debug_border(&mut pm);
        }
        let image = egui::ColorImage::from_rgba_premultiplied([px, px], pm.data());
        line_icons.push(ctx.load_texture_cached(
            format!("russet_line_icon_{icon:?}"),
            Id::unique(format!("russet_line_icon_{icon:?}")),
            image,
            TextureOptions::LINEAR,
        ));
    }
    let Ok(line_icons) = <[TextureHandle; 2]>::try_from(line_icons) else {
        return;
    };
    // The Select family's six buttons (owner request 2026-11-20): the two
    // mode marks + the four ops — the same fixed-drawing pipeline.
    let mut select_icons = Vec::with_capacity(SELECT_ICONS.len());
    for icon in SELECT_ICONS {
        let Some(mut pm) = rasterize_select_icon(icon, px, text) else {
            return; // keep the previous atlas; retried next frame
        };
        if debug_border {
            paint_debug_border(&mut pm);
        }
        let image = egui::ColorImage::from_rgba_premultiplied([px, px], pm.data());
        select_icons.push(ctx.load_texture_cached(
            format!("russet_select_icon_{icon:?}"),
            Id::unique(format!("russet_select_icon_{icon:?}")),
            image,
            TextureOptions::LINEAR,
        ));
    }
    let Ok(select_icons) = <[TextureHandle; 6]>::try_from(select_icons) else {
        return;
    };
    // The History Tree's small set (owner request 2026-11-21): the same
    // tool drawings at [`TREE_ICON_PT`] — the row-sized second atlas.
    let tree_px = (TREE_ICON_PT * ppp).round().clamp(1.0, 256.0) as usize;
    let mut tree_icons = Vec::with_capacity(TOOLS.len());
    for tool in TOOLS {
        let Some(mut pm) = rasterize_tool_icon(tool, tree_px, text) else {
            return; // keep the previous atlas; retried next frame
        };
        if debug_border {
            paint_debug_border(&mut pm);
        }
        let image = egui::ColorImage::from_rgba_premultiplied([tree_px, tree_px], pm.data());
        tree_icons.push(ctx.load_texture_cached(
            format!("russet_tree_icon_{tool:?}"),
            Id::unique(format!("russet_tree_icon_{tool:?}")),
            image,
            TextureOptions::LINEAR,
        ));
    }
    *atlas = Some(Atlas {
        key: ppp,
        handles,
        shape_icons,
        line_icons,
        select_icons,
        tree_icons,
        pipette_cursor: Some(pipette_cursor),
    });
}

/// The `--atlas-test` diagnostic (owner request 2026-11-21: «чёрно-белая
/// мурашка … в ровно 1 px вне зависимости от масштабирования»): repaints
/// the pixmap's OUTERMOST pixel ring with a static black-and-white speckle.
/// The ring lives in the RASTER, so it is exactly one physical pixel thick
/// at any scale — neither `pixels_per_point` nor the icon size can stretch
/// it (a 16 pt tree icon and a 24 pt button carry the same 1 px ring). The
/// pattern is a per-pixel hash of the coordinates, so a rebuild at the same
/// size replays it exactly — no animation between frames. It paints OVER
/// the emblem on purpose (a diagnostic overlay: some drawings reach the
/// raster's edge — the Outline-placement bands run past the 24 pt edge —
/// and the point is to see where the picture ends and how much of it the
/// emblem leaves empty).
pub(super) fn paint_debug_border(pm: &mut Pixmap) {
    let (w, h) = (pm.width(), pm.height());
    for y in 0..h {
        for x in 0..w {
            if x != 0 && y != 0 && x != w - 1 && y != h - 1 {
                continue;
            }
            let mut s = x.wrapping_mul(0x9E37_79B9) ^ y.wrapping_mul(0x85EB_CA6B);
            s ^= s << 13;
            s ^= s >> 17;
            s ^= s << 5;
            let c = if s & 1 == 0 { 255 } else { 0 };
            pm.pixels_mut()[(y * w + x) as usize] =
                PremultipliedColorU8::from_rgba(c, c, c, 255).expect("opaque colors are valid");
        }
    }
}

/// The icon's TextureId for a tool, if the atlas is ready.
pub(super) fn texture_id(atlas: &Option<Atlas>, tool: Tool) -> Option<TextureId> {
    let i = TOOLS.iter().position(|t| t == &tool)?;
    atlas
        .as_ref()
        .and_then(|a| a.handles.get(i))
        .map(TextureHandle::id)
}

/// A Shape-panel icon's TextureId, if the atlas is ready (owner request
/// 2026-11-20).
pub(super) fn shape_texture_id(atlas: &Option<Atlas>, icon: ShapeIcon) -> Option<TextureId> {
    let i = SHAPE_ICONS.iter().position(|s| s == &icon)?;
    atlas
        .as_ref()
        .and_then(|a| a.shape_icons.get(i))
        .map(TextureHandle::id)
}

/// A Line-panel icon's TextureId, if the atlas is ready (owner request
/// 2026-11-20).
pub(super) fn line_texture_id(atlas: &Option<Atlas>, icon: LineIcon) -> Option<TextureId> {
    let i = LINE_ICONS.iter().position(|s| s == &icon)?;
    atlas
        .as_ref()
        .and_then(|a| a.line_icons.get(i))
        .map(TextureHandle::id)
}

/// A Select-family icon's TextureId, if the atlas is ready (owner request
/// 2026-11-20).
pub(super) fn select_texture_id(atlas: &Option<Atlas>, icon: SelectIcon) -> Option<TextureId> {
    let i = SELECT_ICONS.iter().position(|s| s == &icon)?;
    atlas
        .as_ref()
        .and_then(|a| a.select_icons.get(i))
        .map(TextureHandle::id)
}

/// A History Tree row icon's TextureId — the small second atlas (owner
/// request 2026-11-21), if it is ready.
pub(super) fn tree_texture_id(atlas: &Option<Atlas>, tool: Tool) -> Option<TextureId> {
    let i = TOOLS.iter().position(|t| t == &tool)?;
    atlas
        .as_ref()
        .and_then(|a| a.tree_icons.get(i))
        .map(TextureHandle::id)
}

/// One icon of the Line tool's settings panel (owner request 2026-11-20:
/// the word "Mode" radio became icon buttons in the Tools window's style —
/// the same pass that reshaped the Shape panel, "такие же по стилю и
/// отступам"). Both buttons draw the LINE TOOL's emblem (the mark the
/// Tools window's Line button strokes — one geometry, the Shape MODE
/// trio's principle): the Curve choice strokes it EXACTLY as the tool
/// button does, the Straight choice draws the same stroke UNBENT — the
/// straight segment between the emblem curve's endpoints.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum LineIcon {
    Straight,
    Curve,
}

/// The Line-panel icons' canonical order — the atlas and the panel's row
/// iterate this array, so the two always agree.
pub(super) const LINE_ICONS: [LineIcon; 2] = [LineIcon::Straight, LineIcon::Curve];

/// The hover tooltip of a Line-panel button: EXACTLY the words the old
/// word radio showed (owner request 2026-11-20 — the Shape tooltip rule).
pub(super) fn line_icon_label(icon: LineIcon) -> &'static str {
    match icon {
        LineIcon::Straight => "Straight",
        LineIcon::Curve => "Curve",
    }
}

/// One icon of the Select tool's family (owner request 2026-11-20). The
/// two MODE marks double as the TOOLS WINDOW's Select button icon — the
/// dashed outline says "this is a selection", the shape says WHICH kind
/// (the dashed square = the rectangle, the dashed blob = the lasso) — and
/// reappear in the magnetized panel's mode row; the four OPS marks are the
/// panel's copy/cut/paste/crop buttons. ALL six are fixed drawings, so the
/// atlas key stays the scale alone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectIcon {
    Rect,
    Lasso,
    Copy,
    Cut,
    Paste,
    Crop,
}

/// The Select icons' canonical order — the atlas and the panel's rows
/// iterate this array, so the two always agree.
pub(super) const SELECT_ICONS: [SelectIcon; 6] = [
    SelectIcon::Rect,
    SelectIcon::Lasso,
    SelectIcon::Copy,
    SelectIcon::Cut,
    SelectIcon::Paste,
    SelectIcon::Crop,
];

/// The hover tooltip of a Select-family button: the mode marks carry the
/// gesture contract (drag replaces, Shift adds, Alt subtracts), the ops
/// name the Edit-menu hotkeys.
pub(super) fn select_icon_label(icon: SelectIcon) -> &'static str {
    match icon {
        SelectIcon::Rect => "Rectangle selection — drag replaces, Shift adds, Alt subtracts",
        SelectIcon::Lasso => "Lasso selection — drag replaces, Shift adds, Alt subtracts",
        SelectIcon::Copy => "Copy (Ctrl+C)",
        SelectIcon::Cut => "Cut (Ctrl+X)",
        SelectIcon::Paste => "Paste (Ctrl+V)",
        SelectIcon::Crop => "Crop to selection",
    }
}
// ---------------------------------------------------------------------------
// The shared draw pipeline (owner request 2026-11-18: colored and monochrome
// icons must not duplicate code) — every icon is closed polygons + strokes
// through these three primitives, then one outline pass for the colored ones.
// ---------------------------------------------------------------------------

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
