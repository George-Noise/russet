use crate::selection::{Mask, erase_masked_by, stamp_masked};
use eframe::egui::{Color32, Pos2};
use tiny_skia::{
    BlendMode, LineCap, Paint, PathBuilder, Pixmap, PremultipliedColorU8, Stroke, Transform,
};

mod smooth;

pub use smooth::{PencilTrail, SmoothPiece, SmoothStroke, pencil_pixel, rasterize_smooth_pieces};
mod shapes;

pub(crate) use shapes::{dirty_rect_for, rasterize_polyline, rasterize_shape};
mod wet;

pub use wet::StrokeWet;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Pencil,
    Brush,
    Eraser,
    /// Samples the canvas pixel color into the primary/secondary well.
    Eyedropper,
    /// Straight lines (drag) and polylines/curves (click points).
    Line,
    /// Rectangle and ellipse outlines, fills or fill+outline, dragged as a
    /// preview.
    Shape,
    /// Places text at the click point through a small input window.
    Text,
    /// Region selection (owner request 2026-11-20): rectangle or lasso
    /// regions, addable/subtractable, with copy/cut/paste/move/crop. One
    /// Tools-window button (left-click cycles the sub-mode forward when the
    /// tool is already active, right-click cycles it backward); the
    /// sub-mode buttons and the ops live in the magnetized panel.
    Select,
    /// The flood fill (owner request 2026-11-23: «можно сделать заливку тем
    /// же методом что и выделение волшебной палочкой, там очень похожий
    /// метод»): a click floods the clicked color band on the active layer
    /// with the primary (LMB) or secondary (RMB) color — the same
    /// [`crate::selection::flood_pixels`] core the magic wand uses. The
    /// LAST variant on purpose: the Tools window's grid iterates the
    /// [`crate::app::tool_icons::TOOLS`] array, and the owner asked for the
    /// fill button to join at the END («иконку заливки сделай самой
    /// последней чтобы не нарушить порядок инструментов в окошке Tools»).
    Fill,
}

/// Sub-mode of the [`Tool::Select`] tool (owner request 2026-11-20).
/// MORE selection kinds are planned ("типов выделений потом будет больше
/// чем 2"), so every consumer iterates [`SelectMode::ALL`] and cycles with
/// [`SelectMode::next`] / [`SelectMode::prev`] — a new variant lands in the
/// array and every cycle and panel row picks it up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SelectMode {
    /// A rectangle dragged between two corners (the default).
    Rect,
    /// A freeform lasso: the drag's pointer path, auto-closed on release.
    Lasso,
    /// The magic wand (owner request 2026-11-23: «выделение волшебной
    /// палочкой как в Paint.Net … нужен простой алгоритм»): a CLICK floods
    /// the clicked color band on the active layer and selects the reached
    /// pixels exactly — [`crate::selection::Region::Wand`], built by the
    /// shared [`crate::selection::flood_pixels`] core. No drag: the wand
    /// applies at press, the Shift/Alt modifiers still choose add/subtract.
    Wand,
}

impl SelectMode {
    /// The canonical order — the cycle, the panel rows and the icon set
    /// iterate this array, so the three always agree.
    pub const ALL: [SelectMode; 3] = [SelectMode::Rect, SelectMode::Lasso, SelectMode::Wand];

    /// The NEXT mode in the cycle — the Tools button's left-click (owner
    /// request 2026-11-20: "по левой кнопке ... переключался выбор какой
    /// конкретно это инструмент выделения").
    pub fn next(self) -> Self {
        let i = Self::ALL.iter().position(|m| m == &self).unwrap_or(0);
        Self::ALL[(i + 1) % Self::ALL.len()]
    }

    /// The PREVIOUS mode in the cycle — the Tools button's right-click
    /// ("по правой то же самое но в обратную сторону").
    pub fn prev(self) -> Self {
        let i = Self::ALL.iter().position(|m| m == &self).unwrap_or(0);
        Self::ALL[(i + Self::ALL.len() - 1) % Self::ALL.len()]
    }
}

/// Sub-mode of the [`Tool::Line`] tool (owner request 2026-11-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineMode {
    /// Press–drag–release draws one straight segment.
    Straight,
    /// Click to add vertices; Enter/double-click commits, Esc cancels.
    Curve,
}

/// Sub-mode of the [`Tool::Shape`] tool (owner request 2026-11-10).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
}

/// How the [`Tool::Shape`] paints, a THREE-way radio (owner request
/// 2026-11-20: the former Fill/Outline toggle pair — whose "both on" state
/// was reachable only by a hidden double click — became one explicit third
/// choice, and exactly one of the three is ever active).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapePaint {
    /// Fill the interior with the secondary well; no outline.
    Fill,
    /// Stroke the boundary with a `size`-wide line in the primary well; no
    /// fill.
    Outline,
    /// Fill first, then stroke the boundary over it — the outline covers
    /// its half of the stroke width, the fill the interior.
    FillOutline,
}

impl ShapePaint {
    /// Whether this mode paints the fill (the secondary well).
    pub fn fills(self) -> bool {
        matches!(self, ShapePaint::Fill | ShapePaint::FillOutline)
    }

    /// Whether this mode paints the outline (the primary well).
    pub fn outlines(self) -> bool {
        matches!(self, ShapePaint::Outline | ShapePaint::FillOutline)
    }
}

/// Where the [`Tool::Shape`]'s outline lies relative to the two mouse points
/// of the drag (owner request 2026-11-20): the two points ARE the shape's
/// boundary, and a `size`-wide outline can lie fully OUTSIDE it, straddle it
/// (how every earlier version drew — the default) or lie fully INSIDE it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShapeOutline {
    /// The outline lies outside the two points: its INNER edge is the
    /// shape's boundary (the drawn footprint grows by the stroke width).
    Outside,
    /// The outline straddles the boundary — half in, half out (the former
    /// and default look).
    Center,
    /// The outline lies inside the two points: its OUTER edge is the
    /// shape's boundary (the footprint stays exactly the two points).
    Inside,
}

pub struct ToolState {
    pub tool: Tool,
    pub size: f32,
    pub primary: Color32,
    pub secondary: Color32,
    pub line_mode: LineMode,
    pub shape_kind: ShapeKind,
    /// How shapes paint — exactly one of Fill / Outline / Fill+Outline
    /// (owner request 2026-11-20: the former `shape_fill`/`shape_outline`
    /// toggle pair became a three-way radio; a shape always paints
    /// something by construction).
    pub shape_paint: ShapePaint,
    /// Where the shape's outline sits relative to the drag's two mouse
    /// points (owner request 2026-11-20): outside / straddling (default) /
    /// inside — the panel's second three-way radio.
    pub shape_outline: ShapeOutline,
    /// TESTING FEATURE (owner request 2026-11-21): with the Ellipse kind
    /// selected, a `Shift` drag no longer pulls the free corner onto the
    /// proportional circle from the anchor corner — the press and release
    /// points lie ON the circle and the segment between them is its
    /// DIAMETER (the center is the segment's midpoint). The magnetized
    /// panel's checkbox toggles it (shown for the Ellipse kind only — the
    /// rectangle's Shift rule is untouched, so the toggle must not tempt
    /// anyone there); ON while the feature is under test.
    pub shape_diameter_circle: bool,
    /// Which kind of region the Select tool drags (owner request
    /// 2026-11-20): rectangle (the default) or lasso. The Tools button
    /// cycles it; the magnetized panel's mode row sets it explicitly.
    pub select_mode: SelectMode,
    /// The flood THRESHOLD the magic wand's selection and the Fill tool's
    /// pour share, in PERCENT 0..=100 (owner request 2026-11-23: «в
    /// приклеенную панельку … добавить ползунок похожий на Size но
    /// выполняющий функцию порога срабатывания в %, я бы выбрал не % но
    /// людям куда проще понимать именно %» — the unit is % FOR PEOPLE, the
    /// algorithm converts it per channel through
    /// [`crate::selection::tolerance_from_percent`]). ONE setting for both
    /// tools: the flood is one shared method («там очень похожий метод»),
    /// so one knob drives it wherever it runs.
    pub flood_tolerance_pct: u8,
    /// Font size (canvas pixels) used by the Text tool.
    pub font_size: f32,
}

impl ToolState {
    pub fn new() -> Self {
        Self {
            tool: Tool::Pencil,
            size: 8.0,
            primary: Color32::BLACK,
            secondary: Color32::WHITE,
            line_mode: LineMode::Straight,
            shape_kind: ShapeKind::Rectangle,
            shape_paint: ShapePaint::Outline,
            shape_outline: ShapeOutline::Center,
            shape_diameter_circle: true,
            select_mode: SelectMode::Rect,
            flood_tolerance_pct: DEFAULT_FLOOD_TOLERANCE_PCT,
            font_size: 24.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DirtyRect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// The sanity ceiling of any stroke width entering the rasterizers — the
/// same bound the Size block's manual-entry box enforces in the UI. Owner
/// request 2026-11-20: sizes above 100 must be enterable ("а сам слайдер
/// пусть и дальше будет ограничен 100"), the ceiling is "not a design
/// limit" and the owner set none — 10 000 only guards against absurd
/// strokes. The rasterizers clamp to THIS, not to the slider-era 1..=100:
/// that old clamp silently ate a manually entered size (owner bug report
/// 2026-11-20: a typed 200 still drew at 100 — the UI accepted the value,
/// every draw path ignored it).
pub const MAX_STROKE_SIZE: f32 = 10_000.0;

/// The startup value of the flood-threshold slider ([`ToolState::
/// flood_tolerance_pct`]): 13 % — the percent form of the previous fixed
/// const `FLOOD_TOLERANCE` = 32/255 (≈ 12.5 %, rounded up to a whole
/// percent the users can read; `tolerance_from_percent(13)` = 33, one step
/// from the old 32).
pub const DEFAULT_FLOOD_TOLERANCE_PCT: u8 = 13;

/// The selection-clipped commit's shared pipeline (owner request 2026-11-21:
/// an active selection confines every canvas-changing tool to its region).
/// `paint` draws the geometry with a transform — `identity()` for the
/// direct path, a translation onto a transparent bbox SCRATCH when a mask
/// is active (`scratch` = true then; the eraser MARKS its footprint there
/// instead of destination-out — the scratch's alpha is the erase strength).
/// The scratch then transfers onto `pixmap` through the mask
/// ([`selection::stamp_masked`] / [`selection::erase_masked_by`]) — src-over
/// is associative and a pure integer translation preserves coverage
/// exactly, so the clipped result is pixel-identical to the direct paint
/// inside the selection. Returns the dirty rect — mask-clipped when
/// clipped, `None` when the selection covers none of it.
pub(crate) fn paint_clipped(
    pixmap: &mut Pixmap,
    dirty: DirtyRect,
    clip: Option<&Mask>,
    erase: bool,
    paint: impl FnOnce(&mut Pixmap, Transform, bool),
) -> Option<DirtyRect> {
    match clip {
        None => {
            paint(pixmap, Transform::identity(), false);
            Some(dirty)
        }
        Some(mask) => {
            let mut scratch = Pixmap::new(dirty.w.max(1), dirty.h.max(1))?;
            let origin = (dirty.x as i64, dirty.y as i64);
            let shift = Transform::from_translate(-(dirty.x as f32), -(dirty.y as f32));
            paint(&mut scratch, shift, true);
            if erase {
                erase_masked_by(pixmap, &scratch, dirty, origin, mask)
            } else {
                stamp_masked(pixmap, &scratch, dirty, origin, mask)
            }
        }
    }
}

pub fn draw_segment(
    pixmap: &mut Pixmap,
    tool: Tool,
    size: f32,
    color: Color32,
    from: Pos2,
    to: Pos2,
    clip: Option<&Mask>,
) -> Option<DirtyRect> {
    let width = match tool {
        Tool::Pencil => 1.0,
        Tool::Brush | Tool::Eraser => size.clamp(1.0, MAX_STROKE_SIZE),
        // Unreachable: the preview tools commit through `rasterize_polyline`
        // / `rasterize_shape`, Select never strokes, the Fill commits in one
        // click through its own handler.
        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text | Tool::Select | Tool::Fill => {
            size.clamp(1.0, MAX_STROKE_SIZE)
        }
    };

    let pad = (width * 0.5).ceil() + 2.0;
    let min_x = from.x.min(to.x) - pad;
    let min_y = from.y.min(to.y) - pad;
    let max_x = from.x.max(to.x) + pad;
    let max_y = from.y.max(to.y) + pad;
    let x0 = min_x.floor().max(0.0) as u32;
    let y0 = min_y.floor().max(0.0) as u32;
    let x1 = max_x.ceil().min(pixmap.width() as f32) as u32;
    let y1 = max_y.ceil().min(pixmap.height() as f32) as u32;
    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    if matches!(tool, Tool::Pencil) {
        // Classic integer Bresenham between the rounded endpoints: exactly one
        // pixel per step, like the pencil in Paint / Paint.NET. tiny-skia's
        // non-AA scanline stroke crosses its 50% coverage threshold differently
        // depending on the subpixel position, which covers two pixels in some
        // places of a diagonal and one in others (ragged 1px/2px lines).
        let [r, g, b, a] = color.to_srgba_unmultiplied();
        // Premultiplied source components (the Pixmap stores premultiplied RGBA).
        let src = (
            r as u32 * a as u32 / 255,
            g as u32 * a as u32 / 255,
            b as u32 * a as u32 / 255,
        );
        let pw = pixmap.width() as i64;
        let ph = pixmap.height() as i64;
        let pixels = pixmap.pixels_mut();
        bresenham_line(
            from.x.round() as i64,
            from.y.round() as i64,
            to.x.round() as i64,
            to.y.round() as i64,
            |x, y| {
                if x < 0 || y < 0 || x >= pw || y >= ph {
                    return;
                }
                // The selection confines the pencil: masked-out pixels stay
                // untouched (the bookkeeping `last_pixel`-style dedupe lives
                // in the trail, not here).
                if let Some(mask) = clip
                    && !mask.get(x, y)
                {
                    return;
                }
                let dst = pixels[(y * pw + x) as usize];
                // src-over: out = src + dst * (1 - src_alpha), rounded.
                let ia = 255 - a as u32;
                let mix = |s: u32, d: u8| (s + (d as u32 * ia + 127) / 255) as u8;
                let out = PremultipliedColorU8::from_rgba(
                    mix(src.0, dst.red()),
                    mix(src.1, dst.green()),
                    mix(src.2, dst.blue()),
                    mix(a as u32, dst.alpha()),
                )
                .unwrap_or(dst);
                pixels[(y * pw + x) as usize] = out;
            },
        );
        return Some(DirtyRect {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        });
    }

    let mut builder = PathBuilder::new();
    builder.move_to(from.x, from.y);
    let (end_x, end_y) = if (to.x - from.x).abs() < 0.01 && (to.y - from.y).abs() < 0.01 {
        (from.x + 0.01, from.y)
    } else {
        (to.x, to.y)
    };
    builder.line_to(end_x, end_y);
    let path = builder.finish()?;

    let erase = matches!(tool, Tool::Eraser);
    let [r, g, b, a] = match tool {
        Tool::Eraser => [0, 0, 0, 255],
        Tool::Pencil | Tool::Brush => color.to_srgba_unmultiplied(),
        Tool::Eyedropper | Tool::Line | Tool::Shape | Tool::Text | Tool::Select | Tool::Fill => {
            color.to_srgba_unmultiplied()
        }
    };
    let stroke = Stroke {
        width,
        line_cap: LineCap::Round,
        ..Default::default()
    };
    // Clipped: the eraser marks its footprint on the scratch (normal
    // blend), the transfer destination-outs by the scratch's alpha.
    paint_clipped(
        pixmap,
        DirtyRect {
            x: x0,
            y: y0,
            w: x1 - x0,
            h: y1 - y0,
        },
        clip,
        erase,
        |pm, ts, scratch| {
            let mut paint = Paint::default();
            paint.set_color_rgba8(r, g, b, a);
            paint.anti_alias = true;
            if erase && !scratch {
                paint.blend_mode = BlendMode::DestinationOut;
            }
            pm.stroke_path(&path, &paint, &stroke, ts, None);
        },
    )
}

/// Blends one premultiplied source pixel over the pixmap pixel at `(x, y)`
/// (premultiplied source-over, rounded). Shared by the pencil plotter and
/// the text rasterizer. Out-of-bounds coordinates are ignored.
pub(crate) fn blend_pixel(
    pixels: &mut [PremultipliedColorU8],
    width: i64,
    height: i64,
    x: i64,
    y: i64,
    src: [u32; 4],
) {
    if x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    let dst = pixels[(y * width + x) as usize];
    // src-over: out = src + dst * (1 - src_alpha), rounded.
    let ia = 255 - src[3];
    let mix = |s: u32, d: u8| (s + (d as u32 * ia + 127) / 255) as u8;
    if let Some(out) = PremultipliedColorU8::from_rgba(
        mix(src[0], dst.red()),
        mix(src[1], dst.green()),
        mix(src[2], dst.blue()),
        mix(src[3], dst.alpha()),
    ) {
        pixels[(y * width + x) as usize] = out;
    }
}

/// Samples the canvas pixel at integer coordinates and returns it as a
/// straight-alpha color for the color wells (the eyedropper tool).
pub fn pick_color(pixmap: &Pixmap, x: u32, y: u32) -> Option<Color32> {
    let pixel = pixmap.pixel(x, y)?.demultiply();
    Some(Color32::from_rgba_unmultiplied(
        pixel.red(),
        pixel.green(),
        pixel.blue(),
        pixel.alpha(),
    ))
}

/// Integer Bresenham line, all quadrants, both endpoints inclusive; `plot`
/// receives every pixel of the line, one pixel per step.
fn bresenham_line(x0: i64, y0: i64, x1: i64, y1: i64, mut plot: impl FnMut(i64, i64)) {
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let (mut x, mut y) = (x0, y0);
    loop {
        plot(x, y);
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests;
