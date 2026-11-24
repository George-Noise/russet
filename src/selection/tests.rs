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
    assert_eq!(SelectMode::Lasso.next(), SelectMode::Wand);
    assert_eq!(SelectMode::Wand.next(), SelectMode::Rect);
    assert_eq!(SelectMode::Wand.prev(), SelectMode::Lasso);
    assert_eq!(SelectMode::Lasso.prev(), SelectMode::Rect);
    assert_eq!(SelectMode::Rect.prev(), SelectMode::Wand);
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

// The flood-fill core (owner request 2026-11-23): the magic wand and the
// Fill tool share ONE method — the tests pin its contract.

#[test]
fn the_flood_gathers_the_contiguous_band_and_stops_at_the_wall() {
    // A 4×3 grid: two RED rows over one BLUE row — the flood from the top
    // left takes the whole red band and never crosses the color wall.
    let flood = |x: i64, y: i64| {
        if (0..4).contains(&x) && (0..3).contains(&y) {
            Some(if y < 2 { Color32::RED } else { Color32::BLUE })
        } else {
            None
        }
    };
    let pixels = flood_pixels(4, 3, (0, 0), 32, flood);
    assert_eq!(
        pixels,
        vec![
            (0, 0),
            (0, 1),
            (1, 0),
            (1, 1),
            (2, 0),
            (2, 1),
            (3, 0),
            (3, 1)
        ],
        "the band, sorted and deduplicated"
    );
}

#[test]
fn the_flood_measures_against_the_seed_not_the_running_neighbor() {
    // The Paint.NET rule: every matched pixel compares to the SEED's color
    // (within the tolerance per channel), so a slow gradient walks off the
    // seed's band and the flood stops — no unbounded chain-creep.
    // Column 0 = the seed red; column 1 = one shade within tolerance;
    // column 2 = far outside it (even though 1 and 2 are close to each
    // other, a chain through 1 must NOT reach 2).
    let flood = |x: i64, _y: i64| {
        Some(match x {
            0 => Color32::from_rgb(120, 0, 0),
            1 => Color32::from_rgb(152, 0, 0), // exactly the 32 edge
            _ => Color32::from_rgb(194, 0, 0), // outside the seed's band
        })
    };
    let pixels = flood_pixels(3, 1, (0, 0), 32, flood);
    assert_eq!(pixels, vec![(0, 0), (1, 0)], "column 2 stays out");
}

#[test]
fn the_tolerance_slider_gates_the_flood_width() {
    // The panel's percent threshold (owner request 2026-11-23) converted
    // per channel: 0 % matches the EXACT color only, 100 % matches
    // everything, and the same gradient splits accordingly.
    let gradient = |x: i64, _y: i64| Some(Color32::from_rgb((60 + 4 * x) as u8, 0, 0));
    assert_eq!(
        flood_pixels(3, 1, (0, 0), tolerance_from_percent(0), gradient),
        vec![(0, 0)],
        "0 %: the exact color only"
    );
    let wide = flood_pixels(3, 1, (0, 0), tolerance_from_percent(100), gradient);
    assert_eq!(wide, vec![(0, 0), (1, 0), (2, 0)], "100 %: everything");
}

#[test]
fn the_percent_to_tolerance_mapping_is_the_panel_contract() {
    assert_eq!(tolerance_from_percent(0), 0);
    assert_eq!(
        tolerance_from_percent(13),
        33,
        "the startup 13 % ≈ the old const 32"
    );
    assert_eq!(tolerance_from_percent(50), 127);
    assert_eq!(tolerance_from_percent(100), 255);
}

// The span-rewrite suite (owner request 2026-11-23: «делаем 1+2+3+запись
// из спанов… главное не забыть что пользователь может использовать и
// полупрозрачные цвета и полностью прозрачные»): the scanline core must
// claim EXACTLY the per-pixel BFS's region — on any alpha.

/// The pre-rewrite per-pixel BFS, kept verbatim as the ORACLE: whatever
/// the span core claims must be exactly this set (sorted `(x, y)`).
fn naive_reference_flood(
    width: u32,
    height: u32,
    seed: (i64, i64),
    tolerance: u8,
    at: &impl Fn(i64, i64) -> Option<[u8; 4]>,
) -> Vec<(i64, i64)> {
    let Some(seed_bytes) = at(seed.0, seed.1) else {
        return Vec::new();
    };
    if seed.0 < 0 || seed.1 < 0 || seed.0 >= width as i64 || seed.1 >= height as i64 {
        return Vec::new();
    }
    let matches = |c: [u8; 4]| -> bool {
        let diff = |a: u8, b: u8| (i16::from(a) - i16::from(b)).abs();
        diff(c[0], seed_bytes[0]) <= i16::from(tolerance)
            && diff(c[1], seed_bytes[1]) <= i16::from(tolerance)
            && diff(c[2], seed_bytes[2]) <= i16::from(tolerance)
            && diff(c[3], seed_bytes[3]) <= i16::from(tolerance)
    };
    let mut visited = vec![false; width as usize * height as usize];
    let mut queue = std::collections::VecDeque::new();
    let mut out = Vec::new();
    if matches(seed_bytes) {
        visited[seed.1 as usize * width as usize + seed.0 as usize] = true;
        queue.push_back(seed);
    }
    while let Some((x, y)) = queue.pop_front() {
        out.push((x, y));
        for (nx, ny) in [(x + 1, y), (x - 1, y), (x, y + 1), (x, y - 1)] {
            if nx < 0 || ny < 0 || nx >= width as i64 || ny >= height as i64 {
                continue;
            }
            let idx = ny as usize * width as usize + nx as usize;
            if visited[idx] {
                continue;
            }
            let Some(color) = at(nx, ny) else {
                continue;
            };
            if matches(color) {
                visited[idx] = true;
                queue.push_back((nx, ny));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

#[test]
fn the_span_core_claims_exactly_the_per_pixel_region_on_random_canvases() {
    // Deterministic xorshift grids over a palette of opaque, translucent
    // and fully transparent colors; several seeds and tolerance levels per
    // grid; the scanline core must match the naive BFS pixel for pixel —
    // with no pixel claimed twice and none missed (the span lengths sum to
    // the region size).
    let mut state = 0x5eed_5ca9_1123_u64;
    let palette: Vec<[u8; 4]> = vec![
        [255, 255, 255, 255],
        [20, 20, 20, 255],
        [200, 40, 40, 255],
        [40, 200, 40, 128],
        [40, 40, 200, 64],
        [0, 0, 0, 0],
        [128, 128, 128, 1],
        [230, 230, 230, 254],
    ];
    for grid in 0..30u64 {
        let width = 8 + (xorshift(&mut state) % 14) as u32;
        let height = 6 + (xorshift(&mut state) % 10) as u32;
        let mut grid_bytes = Vec::with_capacity((width * height) as usize);
        for _ in 0..width * height {
            grid_bytes.push(palette[(xorshift(&mut state) >> 33) as usize % palette.len()]);
        }
        let at = |x: i64, y: i64| -> Option<[u8; 4]> {
            if x < 0 || y < 0 || x >= width as i64 || y >= height as i64 {
                None
            } else {
                Some(grid_bytes[y as usize * width as usize + x as usize])
            }
        };
        for _ in 0..4 {
            let seed = (
                (xorshift(&mut state) % width as u64) as i64,
                (xorshift(&mut state) % height as u64) as i64,
            );
            for tol in [0u8, 8, 64, 255] {
                let spans = flood_spans(width, height, seed, tol, at);
                let span_total: usize = spans.iter().map(|s| (s.x1 - s.x0 + 1) as usize).sum();
                let got = spans_to_sorted_pixels(width, &spans);
                let want = naive_reference_flood(width, height, seed, tol, &at);
                assert_eq!(
                    got, want,
                    "grid {grid} seed {seed:?} tol {tol}: the span core diverged from the BFS"
                );
                assert_eq!(
                    span_total,
                    want.len(),
                    "grid {grid} seed {seed:?} tol {tol}: spans claim each pixel exactly once"
                );
            }
        }
    }
}

#[test]
fn the_flood_walks_transparent_and_translucent_colors() {
    // A fully transparent interior (the cut-hole shape: alpha 0 everywhere
    // between the opaque walls) floods as one region and stops at the
    // walls; a translucent seed matches within the tolerance on EVERY
    // channel including alpha, so a same-rgb opaque neighbor stays out.
    let at = |x: i64, _y: i64| -> Option<[u8; 4]> {
        (0..5).contains(&x).then_some(match x {
            0 | 4 => [200, 0, 0, 255],
            _ => [0, 0, 0, 0],
        })
    };
    let got = spans_to_sorted_pixels(5, &flood_spans(5, 1, (2, 0), 0, at));
    assert_eq!(
        got,
        vec![(1, 0), (2, 0), (3, 0)],
        "the transparent hole floods, the opaque walls stop it"
    );

    let at = |x: i64, _y: i64| -> Option<[u8; 4]> {
        (0..3).contains(&x).then_some(match x {
            0 => [64, 0, 0, 128], // the seed
            1 => [64, 0, 0, 140], // within 12 on alpha
            _ => [64, 0, 0, 255], // same rgb, opaque — 127 apart on alpha
        })
    };
    let got = spans_to_sorted_pixels(3, &flood_spans(3, 1, (0, 0), 12, at));
    assert_eq!(
        got,
        vec![(0, 0), (1, 0)],
        "the alpha channel gates the match"
    );
}

#[test]
fn the_span_core_emits_row_runs_not_pixel_tuples() {
    // A 4×2 solid floods into two whole-row spans — the shape the fill's
    // write consumes directly.
    let at = |_, _| Some([255u8, 255, 255, 255]);
    let spans = flood_spans(4, 2, (1, 0), 0, at);
    assert_eq!(
        spans,
        vec![
            FloodSpan { y: 0, x0: 0, x1: 3 },
            FloodSpan { y: 1, x0: 0, x1: 3 },
        ],
    );
}

#[test]
fn the_flood_off_grid_seed_and_single_pixel_match() {
    assert!(flood_pixels(4, 3, (-1, 0), 32, |_, _| Some(Color32::RED)).is_empty());
    // A lone matching pixel still floods (itself).
    assert_eq!(
        flood_pixels(1, 1, (0, 0), 32, |_, _| Some(Color32::RED)),
        vec![(0, 0)]
    );
}

#[test]
fn a_wand_region_paints_contains_clips_and_translates() {
    // Sorted (x, y) — the flood_pixels contract the variant documents.
    let region = Region::Wand {
        pixels: vec![(1, 0), (1, 1), (2, 1)],
    };
    assert_eq!(
        region.pixel_rect(),
        Some(IRect {
            x: 1,
            y: 0,
            w: 2,
            h: 2
        }),
        "the tight pixel bounds of the set"
    );
    // contains: the EXACT pixel semantics — the point lands in a selected
    // pixel or not, no center rule.
    assert!(region.contains(pos2(1.4, 0.6)));
    assert!(region.contains(pos2(2.9, 1.9)));
    assert!(!region.contains(pos2(2.4, 0.6)));
    assert!(!region.contains(pos2(0.5, 0.5)));
    // paint: the mask carries exactly the set.
    let mut mask = Mask::new(0, 0, 4, 3);
    region.paint(&mut mask, true);
    assert!(mask.get(1, 0) && mask.get(1, 1) && mask.get(2, 1));
    assert!(!mask.get(0, 0) && !mask.get(2, 0));
    // clip: the pixels outside the canvas drop, the rest re-sort.
    let clipped = region
        .clone()
        .clipped_to(2, 3)
        .expect("pixels survive the clip");
    assert_eq!(
        clipped,
        Region::Wand {
            pixels: vec![(1, 0), (1, 1)]
        }
    );
    // translate: the whole set shifts; the ants re-derive through refresh
    // (the (1,1) shift keeps every pixel on the 4×3 canvas — the same
    // canvas-clamped mask the rect/lasso refresh derives).
    let mut selection = Selection::default();
    selection.replace(Some(region), 4, 3);
    assert_eq!(
        selection.bounds(),
        Some(IRect {
            x: 1,
            y: 0,
            w: 2,
            h: 2
        })
    );
    selection.translate(1, 1, 4, 3);
    assert_eq!(
        selection.bounds(),
        Some(IRect {
            x: 2,
            y: 1,
            w: 2,
            h: 2
        })
    );
    assert!(selection.contains(pos2(2.5, 1.5)));
    // An empty set selects nothing (the degenerate rule).
    assert!(Region::Wand { pixels: Vec::new() }.pixel_rect().is_none());
}
