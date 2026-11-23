//! The wet stroke layer (owner bug report 2026-11-23): premultiplied
//! source-over is not idempotent, so painting a translucent color once per
//! FRAME darkened every overlap — each frame's smooth-piece stroke carries
//! round caps, the next frame's start cap (and the press dot) sat exactly
//! on the previous frame's end cap, and the alpha stacked there
//! («в стыках отрезков кружочек становится менее прозрачным»).
//!
//! A brush/eraser gesture therefore paints through [`StrokeWet`]: the
//! whole stroke accumulates its coverage into a full-alpha SCRATCH —
//! source-over of an OPAQUE paint is a coverage union (the same mask one
//! long single-path stroke would produce, joints included) — and the
//! active layer is recomposited from the gesture-start base snapshot plus
//! the scratch, applying the stroke's alpha ONCE per pixel. The layer thus
//! always holds the exact one-shot result: a joint pixel is
//! indistinguishable from the stroke body, whatever the frame batching.
//! The release records the layer as before (the wet is transparent to
//! history: the press-time snapshot and the release-time node see the same
//! pixels they always did), and undo needs no new machinery. The pencil
//! keeps its direct path — the trail's duplicate-pixel dedupe is its
//! translucency guard (SPEC §3.5).

use eframe::egui::Color32;
use tiny_skia::{Pixmap, PremultipliedColorU8};

use super::DirtyRect;
use crate::selection::Mask;

/// The gesture-level wet layer of ONE brush/eraser stroke: the active
/// layer's pixels at gesture start (`base`) and the stroke's accumulating
/// full-alpha coverage (`scratch`). All three pixmaps are canvas-sized,
/// so one coordinate indexes the same pixel in each.
///
/// The scratch is COVERAGE ONLY: its color channels are never read. The
/// stroke's color (and its alpha) apply once, at [`StrokeWet::composite_region`]
/// time — which is why an eraser marks its footprint with the brush
/// geometry instead (a destination-out paint would carve the stroke's own
/// overlaps out of the scratch instead of unioning them).
pub struct StrokeWet {
    base: Pixmap,
    scratch: Pixmap,
}

impl StrokeWet {
    /// Starts a wet gesture over `layer`'s CURRENT pixels. Call before the
    /// gesture's first paint (the press dot included) — the base is what
    /// every recomposite blends against.
    pub fn start(layer: &Pixmap) -> Option<Self> {
        let scratch = Pixmap::new(layer.width(), layer.height())?;
        Some(Self {
            base: layer.clone(),
            scratch,
        })
    }

    /// The coverage scratch the gesture's pieces rasterize into — one
    /// continuous full-alpha stroke per frame batch (the smooth pieces
    /// join at their shared midpoints, so a frame adds no seams), the
    /// press dot included. Overlaps union; nothing darkens.
    pub fn scratch_mut(&mut self) -> &mut Pixmap {
        &mut self.scratch
    }

    /// Recomposites `region` (canvas coordinates — the rasterizers' dirty
    /// rects) onto `layer` from the base snapshot and the scratch. Paint
    /// blends the stroke `color` at `alpha · coverage` ONCE over the base
    /// pixel; erase scales the base pixel by `1 − coverage`
    /// (destination-out, the scratch's alpha IS the erase strength). A
    /// `clip` mask confines the composite (the gesture's selection
    /// snapshot) — masked-out pixels are never written, so they stay
    /// byte-identical to the base. Returns the recomposited region
    /// (`None` when empty); recompositing the same region twice is
    /// idempotent — the math always restarts from the base.
    pub fn composite_region(
        &self,
        layer: &mut Pixmap,
        region: DirtyRect,
        color: Color32,
        erase: bool,
        clip: Option<&Mask>,
    ) -> Option<DirtyRect> {
        let (w, h) = (self.scratch.width() as i64, self.scratch.height() as i64);
        if (layer.width() as i64, layer.height() as i64) != (w, h)
            || (self.base.width() as i64, self.base.height() as i64) != (w, h)
        {
            // Impossible mid-gesture (a stroke cannot overlap a canvas
            // resize); skip instead of panicking on user data.
            return None;
        }

        let mut x0 = (region.x as i64).clamp(0, w);
        let mut y0 = (region.y as i64).clamp(0, h);
        let mut x1 = ((region.x + region.w) as i64).clamp(0, w);
        let mut y1 = ((region.y + region.h) as i64).clamp(0, h);
        if let Some(bounds) = clip.and_then(Mask::tight_bounds) {
            x0 = x0.max(bounds.x);
            y0 = y0.max(bounds.y);
            x1 = x1.min(bounds.x + bounds.w as i64);
            y1 = y1.min(bounds.y + bounds.h as i64);
        }
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        let clipped = DirtyRect {
            x: x0 as u32,
            y: y0 as u32,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        };

        let [r, g, b, a] = color.to_srgba_unmultiplied();
        let scratch = self.scratch.pixels();
        let base = self.base.pixels();
        let pixels = layer.pixels_mut();
        for py in y0..y1 {
            for px in x0..x1 {
                // The selection confines the composite (owner request
                // 2026-11-21): masked-out pixels keep the base untouched.
                if let Some(mask) = clip
                    && !mask.get(px, py)
                {
                    continue;
                }
                let idx = (py * w + px) as usize;
                let coverage = scratch[idx].alpha() as u32;
                if coverage == 0 {
                    continue;
                }
                let dst = base[idx];
                if erase {
                    // Destination-out by the coverage, ONCE: every channel
                    // scales by 1 − s (the same mix the masked eraser
                    // transfer applies).
                    let ia = 255 - coverage;
                    let fade = |c: u8| ((c as u32 * ia + 127) / 255) as u8;
                    if let Some(out) = PremultipliedColorU8::from_rgba(
                        fade(dst.red()),
                        fade(dst.green()),
                        fade(dst.blue()),
                        fade(dst.alpha()),
                    ) {
                        pixels[idx] = out;
                    }
                } else {
                    // src-over of the stroke color at α·s, ONCE, over the
                    // base pixel (premultiplied, rounded — the same mix
                    // `blend_pixel` applies).
                    let eff = (a as u32 * coverage + 127) / 255;
                    if eff == 0 {
                        continue;
                    }
                    let ia = 255 - eff;
                    let mix = |s: u32, d: u8| (s + (d as u32 * ia + 127) / 255) as u8;
                    if let Some(out) = PremultipliedColorU8::from_rgba(
                        mix((r as u32 * eff + 127) / 255, dst.red()),
                        mix((g as u32 * eff + 127) / 255, dst.green()),
                        mix((b as u32 * eff + 127) / 255, dst.blue()),
                        mix(eff, dst.alpha()),
                    ) {
                        pixels[idx] = out;
                    }
                }
            }
        }
        Some(clipped)
    }
}

#[cfg(test)]
mod tests;
