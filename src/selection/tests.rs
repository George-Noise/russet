//! Tests of the pure selection domain (owner request 2026-11-20): the
//! test-pinned pixel semantics of the module doc, the add/subtract
//! algebra, the boundary-loop tracing (the ants), and the pixel helpers
//! the copy/cut/paste/move/crop ops build on. The Select-mode cycle lives
//! here too — it is the same "future modes" contract ([`SelectMode::ALL`]
//! iterated everywhere).

use super::*;
use crate::tools::SelectMode;
use eframe::egui::pos2;
use tiny_skia::Pixmap;

fn rect(a: (f32, f32), b: (f32, f32)) -> Region {
    Region::Rect {
        a: pos2(a.0, a.1),
        b: pos2(b.0, b.1),
    }
}

fn lasso(points: &[(f32, f32)]) -> Region {
    Region::Lasso {
        points: points.iter().map(|p| pos2(p.0, p.1)).collect(),
    }
}

#[test]
fn rect_region_selects_whole_pixels_between_the_corners() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.3, 5.7), (40.6, 30.2))), 100, 100);

    let mask = sel.build_mask(100, 100).expect("non-empty selection");
    assert_eq!(
        sel.bounds(),
        Some(IRect {
            x: 10,
            y: 5,
            w: 31,
            h: 26
        })
    );
    assert!(mask.get(10, 5));
    assert!(mask.get(40, 30));
    assert!(!mask.get(9, 5));
    assert!(!mask.get(41, 5));
    assert!(!mask.get(10, 4));
}

#[test]
fn lasso_selects_pixels_whose_center_is_inside_the_polygon() {
    let mut sel = Selection::default();
    sel.replace(
        Some(lasso(&[
            (10.0, 10.0),
            (30.0, 10.0),
            (30.0, 30.0),
            (10.0, 30.0),
        ])),
        100,
        100,
    );

    let mask = sel.build_mask(100, 100).expect("non-empty selection");
    assert!(mask.get(10, 10), "the center (10.5, 10.5) is inside");
    assert!(mask.get(29, 29), "the center (29.5, 29.5) is inside");
    assert!(!mask.get(30, 10), "the center (30.5, 10.5) is outside");
    assert!(!mask.get(9, 9), "the center (9.5, 9.5) is outside");
}

#[test]
fn a_degenerate_lasso_selects_nothing() {
    let mut sel = Selection::default();
    sel.replace(Some(lasso(&[(10.0, 10.0), (12.0, 12.0)])), 100, 100);

    assert!(sel.bounds().is_none(), "fewer than 3 points = deselect");
}

#[test]
fn add_unions_and_subtract_cuts_holes() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.0, 10.0), (30.0, 30.0))), 100, 100);
    sel.add(rect((25.0, 25.0), (40.0, 40.0)), 100, 100);

    assert!(sel.contains(pos2(35.0, 35.0)), "only in the added region");
    assert!(sel.contains(pos2(12.0, 12.0)), "in the base rect");

    sel.subtract(rect((10.0, 10.0), (20.0, 20.0)), 100, 100);

    let mask = sel.build_mask(100, 100).expect("the selection survives");
    assert!(!mask.get(12, 12), "inside the cut hole");
    assert!(mask.get(25, 12), "outside the hole but in the base rect");
    assert!(!sel.contains(pos2(12.0, 12.0)));
}

#[test]
fn subtracting_everything_empties_the_selection() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.0, 10.0), (20.0, 20.0))), 100, 100);
    sel.subtract(rect((10.0, 10.0), (20.0, 20.0)), 100, 100);

    assert!(sel.bounds().is_none());
    assert!(sel.ants().is_empty());
}

#[test]
fn boundary_loops_of_a_rect_merge_collinear_runs() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.0, 10.0), (30.0, 30.0))), 100, 100);

    let ants = sel.ants();
    assert_eq!(ants.len(), 1, "one closed loop");
    assert_eq!(ants[0].len(), 4, "the corners only — no unit steps");
    let corners: Vec<(i64, i64)> = ants[0]
        .iter()
        .map(|p| (p.x.round() as i64, p.y.round() as i64))
        .collect();
    for expected in [(10, 10), (30, 10), (30, 30), (10, 30)] {
        assert!(
            corners.contains(&expected),
            "corner {expected:?} in {corners:?}"
        );
    }
}

#[test]
fn a_subtracted_hole_traces_a_second_loop() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.0, 10.0), (40.0, 40.0))), 100, 100);
    sel.subtract(rect((15.0, 15.0), (25.0, 25.0)), 100, 100);

    let ants = sel.ants();
    assert_eq!(ants.len(), 2, "the outer boundary + the hole's boundary");
    assert!(ants.iter().all(|l| l.len() == 4), "both are rectangles");
}

#[test]
fn a_checkerboard_corner_keeps_two_separate_loops() {
    let mut sel = Selection::default();
    sel.add(rect((10.0, 10.0), (11.0, 11.0)), 100, 100); // the pixel (10,10)
    sel.add(rect((11.0, 11.0), (12.0, 12.0)), 100, 100); // the pixel (11,11)

    let ants = sel.ants();
    assert_eq!(ants.len(), 2, "diagonal-touching pixels: two loops");
    assert!(ants.iter().all(|l| l.len() == 4), "each a unit square");
}

#[test]
fn translate_shifts_the_bounds_and_the_hit_test() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.0, 10.0), (20.0, 20.0))), 100, 100);
    sel.translate(5, 7, 100, 100);

    assert_eq!(
        sel.bounds(),
        Some(IRect {
            x: 15,
            y: 17,
            w: 10,
            h: 10
        })
    );
    assert!(sel.contains(pos2(15.5, 17.5)));
    assert!(!sel.contains(pos2(10.5, 10.5)));
}

#[test]
fn extract_masked_and_erase_masked_round_trip_the_pixels() {
    let mut src = Pixmap::new(40, 40).expect("40x40 pixmap");
    src.fill(tiny_skia::Color::BLACK);
    let mut sel = Selection::default();
    sel.replace(Some(rect((5.0, 5.0), (15.0, 15.0))), 40, 40);
    let mask = sel.build_mask(40, 40).expect("non-empty selection");

    let clip = extract_masked(&src, &mask).expect("extracts");
    assert_eq!((clip.width(), clip.height()), (10, 10));
    // The clip IS the mask bbox: every pixel selected — (0,0) and (9,9)
    // carry the source's black.
    assert!(clip.pixel(0, 0).expect("in bounds").alpha() == 255);
    assert!(clip.pixel(9, 9).expect("in bounds").alpha() == 255);

    erase_masked(&mut src, &mask);
    assert!(src.pixel(10, 10).expect("in bounds").alpha() == 0, "erased");
    assert!(
        src.pixel(3, 3).expect("in bounds").alpha() == 255,
        "outside"
    );
    assert!(
        src.pixel(9, 9).expect("in bounds").alpha() == 0,
        "inside the mask"
    );
}

#[test]
fn restore_region_writes_the_backup_bytes_back() {
    let mut dst = Pixmap::new(10, 10).expect("10x10 pixmap");
    dst.fill(tiny_skia::Color::BLACK);
    let mut backup = Pixmap::new(10, 10).expect("10x10 pixmap");
    backup.fill(tiny_skia::Color::WHITE);
    // Simulate the moved-stamp mess: the whole canvas transparent.
    dst.fill(tiny_skia::Color::from_rgba8(0, 0, 0, 0));

    restore_region(
        &mut dst,
        &backup,
        IRect {
            x: 0,
            y: 0,
            w: 10,
            h: 10,
        },
    );
    assert!(
        dst.pixel(5, 5).expect("in bounds").alpha() == 255,
        "restored"
    );
    assert!(
        dst.pixel(5, 5).expect("in bounds").red() == 255,
        "the backup's white, not the pre-restore black"
    );
}

#[test]
fn stamp_blends_the_clip_at_the_integer_position() {
    let mut dst = Pixmap::new(20, 20).expect("20x20 pixmap");
    let mut clip = Pixmap::new(5, 5).expect("5x5 pixmap");
    clip.fill(tiny_skia::Color::BLACK);

    stamp(&mut dst, &clip, 3, 4);
    assert!(dst.pixel(3, 4).expect("in bounds").alpha() == 255);
    assert!(dst.pixel(2, 4).expect("in bounds").alpha() == 0);
    assert!(
        dst.pixel(8, 4).expect("in bounds").alpha() == 0,
        "3+5=8 exclusive"
    );
    assert!(dst.pixel(7, 8).expect("in bounds").alpha() == 255);
}

#[test]
fn the_select_mode_cycles_forward_and_backward() {
    assert_eq!(SelectMode::Rect.next(), SelectMode::Lasso);
    assert_eq!(SelectMode::Lasso.next(), SelectMode::Rect);
    assert_eq!(SelectMode::Lasso.prev(), SelectMode::Rect);
    assert_eq!(SelectMode::Rect.prev(), SelectMode::Lasso);
}

#[test]
fn clipping_a_rect_region_to_the_canvas_shrinks_it() {
    // Owner bug report 2026-11-20: a move that lands partially off-canvas
    // must shrink the region with the pixels.
    let region = Region::Rect {
        a: pos2(85.0, 85.0),
        b: pos2(105.0, 105.0),
    };
    let clipped = region.clipped_to(100, 100).expect("partially inside");
    assert_eq!(
        clipped.pixel_rect(),
        Some(IRect {
            x: 85,
            y: 85,
            w: 15,
            h: 15
        }),
        "the 5px overflow past both edges is cut"
    );
}

#[test]
fn a_region_fully_off_canvas_clips_to_none() {
    let region = Region::Rect {
        a: pos2(120.0, 120.0),
        b: pos2(140.0, 140.0),
    };
    assert!(region.clone().clipped_to(100, 100).is_none());

    // A lasso fully outside one axis clips to nothing as well.
    let lasso = Region::Lasso {
        points: vec![pos2(120.0, 10.0), pos2(140.0, 10.0), pos2(140.0, 30.0)],
    };
    assert!(lasso.clipped_to(100, 100).is_none());
}

#[test]
fn clipping_a_lasso_cuts_the_polygon_at_the_edge() {
    // A triangle straddling the right edge: the clipped polygon keeps only
    // the on-canvas part, with the cut corners ON the edge.
    let lasso = Region::Lasso {
        points: vec![pos2(90.0, 10.0), pos2(110.0, 30.0), pos2(90.0, 50.0)],
    };
    let clipped = lasso.clipped_to(100, 100).expect("partially inside");
    let Region::Lasso { points } = clipped else {
        panic!("a lasso stays a lasso");
    };
    assert!(
        points
            .iter()
            .all(|p| p.x <= 100.0 && p.x >= 0.0 && p.y >= 0.0 && p.y <= 100.0),
        "every clipped vertex sits on the canvas"
    );
    assert!(
        points.len() > 3,
        "the cut introduced new vertices at the crossings"
    );
    // The clipped polygon still selects pixels (the mask builds non-empty).
    let mut selection = Selection::default();
    selection.add(Region::Lasso { points }, 100, 100);
    let mask = selection
        .build_mask(100, 100)
        .expect("pixels inside the canvas");
    assert!(
        mask.tight_bounds().is_some(),
        "the clipped lasso selects pixels"
    );
}

// -----------------------------------------------------------------------
// The selection-clipped commit's transfer primitives (owner request
// 2026-11-21): the stroke rasterizes into a transparent scratch, these
// transfer it onto the layer through the mask. The tool-level behavior is
// pinned in `tools/tests/clip.rs`.
// -----------------------------------------------------------------------

/// An opaque scratch pixel (premultiplied red).
fn red() -> PremultipliedColorU8 {
    PremultipliedColorU8::from_rgba(180, 0, 0, 255).expect("opaque")
}

#[test]
fn stamp_masked_blends_scratch_pixels_only_where_masked() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((5.0, 5.0), (15.0, 15.0))), 20, 20);
    let mask = sel.build_mask(20, 20).expect("non-empty selection");

    // The scratch paints the 10×10 block at (2..12, 2..12); only the part
    // the mask selects (5..12 on both axes) may reach the layer.
    let mut scratch = Pixmap::new(20, 20).expect("the pixmap builds");
    for y in 2..12 {
        for x in 2..12 {
            scratch.pixels_mut()[y * 20 + x] = red();
        }
    }
    let mut dst = Pixmap::new(20, 20).expect("the pixmap builds");
    let region = stamp_masked(
        &mut dst,
        &scratch,
        DirtyRect {
            x: 0,
            y: 0,
            w: 20,
            h: 20,
        },
        (0, 0),
        &mask,
    )
    .expect("the mask intersects the scratch");
    // The contract: the REGION ∩ the mask's tight bounds (the rect the
    // caller composites/syncs — it may exceed the actually-painted part of
    // the scratch).
    assert_eq!((region.x, region.y, region.w, region.h), (5, 5, 10, 10));

    for y in 0..20 {
        for x in 0..20 {
            let px = dst.pixel(x, y).expect("in bounds");
            let inside = (5..12).contains(&x) && (5..12).contains(&y);
            assert_eq!(
                px.alpha() > 0,
                inside,
                "pixel ({x}, {y}) must be {inside} painted"
            );
        }
    }
}

#[test]
fn erase_masked_by_fades_only_masked_pixels_by_scratch_alpha() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((0.0, 0.0), (10.0, 20.0))), 20, 20);
    let mask = sel.build_mask(20, 20).expect("non-empty selection");

    // The layer is fully opaque black; the scratch holds a full-alpha mark
    // on rows 0..5 and a half-alpha mark on rows 10..15 (both columns
    // 0..10, inside the mask) — the masked pixels fade, everything else
    // keeps alpha 255.
    let black = PremultipliedColorU8::from_rgba(0, 0, 0, 255).expect("opaque");
    let mut dst = Pixmap::new(20, 20).expect("the pixmap builds");
    for p in dst.pixels_mut() {
        *p = black;
    }
    let mut scratch = Pixmap::new(20, 20).expect("the pixmap builds");
    for y in 0..5 {
        for x in 0..10 {
            scratch.pixels_mut()[y * 20 + x] = black;
        }
    }
    let half = PremultipliedColorU8::from_rgba(0, 0, 0, 128).expect("half");
    for y in 10..15 {
        for x in 0..10 {
            scratch.pixels_mut()[y * 20 + x] = half;
        }
    }
    erase_masked_by(
        &mut dst,
        &scratch,
        DirtyRect {
            x: 0,
            y: 0,
            w: 20,
            h: 20,
        },
        (0, 0),
        &mask,
    )
    .expect("the mask intersects");

    for y in 0..5 {
        assert_eq!(dst.pixel(5, y).expect("in bounds").alpha(), 0, "full erase");
        assert_eq!(
            dst.pixel(15, y).expect("in bounds").alpha(),
            255,
            "outside the scratch"
        );
    }
    for y in 10..15 {
        // Destination-out by alpha 128: 255 × (255−128)/255 = 127 exactly
        // (premultiplied black stays black).
        assert_eq!(
            dst.pixel(5, y).expect("in bounds").alpha(),
            127,
            "half erase"
        );
    }
    assert_eq!(
        dst.pixel(5, 18).expect("in bounds").alpha(),
        255,
        "outside the scratch's mark"
    );
    assert_eq!(
        dst.pixel(15, 2).expect("in bounds").alpha(),
        255,
        "outside the mask"
    );
}

#[test]
fn masked_transfer_is_none_when_the_mask_misses_the_region() {
    let mut sel = Selection::default();
    sel.replace(Some(rect((10.0, 10.0), (15.0, 15.0))), 20, 20);
    let mask = sel.build_mask(20, 20).expect("non-empty selection");
    let mut scratch = Pixmap::new(20, 20).expect("the pixmap builds");
    scratch.pixels_mut()[0] = red();
    let mut dst = Pixmap::new(20, 20).expect("the pixmap builds");
    // A region in the top-left corner does not overlap the mask at all.
    assert!(
        stamp_masked(
            &mut dst,
            &scratch,
            DirtyRect {
                x: 0,
                y: 0,
                w: 5,
                h: 5
            },
            (0, 0),
            &mask,
        )
        .is_none()
    );
    assert!(
        erase_masked_by(
            &mut dst,
            &scratch,
            DirtyRect {
                x: 0,
                y: 0,
                w: 5,
                h: 5
            },
            (0, 0),
            &mask,
        )
        .is_none()
    );
}
