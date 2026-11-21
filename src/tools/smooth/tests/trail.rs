//! Regression tests of the pencil's drifting pen ([`PencilTrail`], owner
//! request 2026-11-20): the 1 px staircase stays one pixel per dominant
//! step and reaches the release pixel on the integer-event pointer streams
//! that ragged the old per-run Bresenham restarts, and the drawn trail is
//! IDENTICAL no matter how the pieces are batched into frames (the
//! machine-dependence the owner reported: different computers coalesce the
//! pointer stream differently).

use super::super::{
    PencilTrail, SmoothPiece, SmoothStroke, Tool, pencil_pixel, rasterize_smooth_pieces,
};
use eframe::egui::{Color32, Pos2, pos2};
use tiny_skia::Pixmap;

fn plotted_pencil_pixels(pixmap: &Pixmap) -> Vec<(usize, usize)> {
    pixmap
        .pixels()
        .iter()
        .enumerate()
        .filter(|(_, c)| c.alpha() == 255 && c.red() == 0)
        .map(|(i, _)| (i % pixmap.width() as usize, i / pixmap.width() as usize))
        .collect()
}

fn is_eight_connected(pixels: &[(usize, usize)]) -> bool {
    let set: std::collections::HashSet<&(usize, usize)> = pixels.iter().collect();
    let Some(&first) = pixels.first() else {
        return true;
    };
    let mut seen = std::collections::HashSet::new();
    let mut queue = vec![first];
    while let Some(p @ (x, y)) = queue.pop() {
        if !seen.insert(p) {
            continue;
        }
        for dx in -1i64..=1 {
            for dy in -1i64..=1 {
                let n = (x as i64 + dx, y as i64 + dy);
                if n.0 >= 0
                    && n.1 >= 0
                    && (n.0 as usize, n.1 as usize) != p
                    && set.contains(&(n.0 as usize, n.1 as usize))
                {
                    queue.push((n.0 as usize, n.1 as usize));
                }
            }
        }
    }
    seen.len() == pixels.len()
}

/// Integer screen pixels a real mouse reports along a 43-degree line: the
/// rounded events wobble around the ideal line, so the smoothed path
/// locally steepens past 45 degrees — the exact input that made the old
/// per-run Bresenham restarts leave double-covered columns and holes.
fn forty_three_degree_events() -> Vec<Pos2> {
    let ux = 43.0f32.to_radians().cos();
    let uy = 43.0f32.to_radians().sin();
    (1..=22)
        .map(|k| {
            let d = 2.0 * k as f32;
            pos2((30.0 + ux * d).round(), (30.0 + uy * d).round())
        })
        .collect()
}

fn smoothed_pieces(events: &[Pos2], start: Pos2) -> (Vec<SmoothPiece>, Pos2) {
    let mut window = SmoothStroke::new(start);
    let mut pieces = Vec::new();
    for event in events {
        if let Some(piece) = window.push(*event) {
            pieces.push(piece);
        }
    }
    if let Some(closer) = window.close() {
        pieces.push(closer);
    }
    (pieces, *events.last().expect("events exist"))
}

fn draw_in_batches(
    pieces: &[SmoothPiece],
    release: Pos2,
    press: Pos2,
    batch: usize,
) -> Vec<(usize, usize)> {
    let mut pixmap = Pixmap::new(96, 96).expect("96x96 pixmap");
    let mut trail = PencilTrail::starting_from(pencil_pixel(press));
    for (i, chunk) in pieces.chunks(batch).enumerate() {
        let is_last = (i + 1) * batch >= pieces.len();
        rasterize_smooth_pieces(
            &mut pixmap,
            Tool::Pencil,
            1.0,
            Color32::BLACK,
            chunk,
            &mut trail,
            if is_last { Some(release) } else { None },
            None,
        )
        .expect("chunk rasterizes");
    }
    plotted_pencil_pixels(&pixmap)
}

#[test]
fn pencil_integer_event_staircase_stays_gap_free_and_reaches_the_release() {
    let events = forty_three_degree_events();
    let (pieces, release) = smoothed_pieces(&events, pos2(30.0, 30.0));
    let pixels = draw_in_batches(&pieces, release, pos2(30.0, 30.0), 3);
    assert!(!pixels.is_empty(), "the stroke draws pixels");
    // Every column between the endpoints carries pixels (no holes — the
    // owner's screenshot showed a broken diagonal), the chain is
    // 8-connected, and the trail ends exactly on the released pixel.
    let mut per_column: std::collections::BTreeMap<i64, i64> = Default::default();
    for (x, y) in &pixels {
        per_column.entry(*x as i64).or_insert(*y as i64);
    }
    let min_x = *per_column.keys().min().expect("columns exist");
    let max_x = *per_column.keys().max().expect("columns exist");
    for x in min_x..=max_x {
        assert!(
            per_column.contains_key(&x),
            "column {x} is empty: a hole in the diagonal"
        );
    }
    let span = (max_x - min_x + 1) as usize;
    // The smoothed path of integer-rounded events wobbles locally past
    // 45 degrees, so an isolated stacked pixel is part of its honest
    // staircase — but it must stay RARE (the old per-run Bresenham restarts
    // stacked one every ~6 px on the same input).
    let stacked = pixels.len() - span;
    assert!(
        stacked * 10 <= span,
        "{stacked} stacked pixels over a {span} px span: the staircase is ragged"
    );
    assert!(is_eight_connected(&pixels), "the staircase is gap-free");
    let release_pixel = pencil_pixel(release);
    assert_eq!(
        *pixels.last().expect("pixels exist"),
        (release_pixel.0 as usize, release_pixel.1 as usize),
        "the pinned trail ends exactly on the release pixel"
    );
}

#[test]
fn pencil_trail_is_identical_regardless_of_frame_batching() {
    let events = forty_three_degree_events();
    let (pieces, release) = smoothed_pieces(&events, pos2(30.0, 30.0));
    let per_event = draw_in_batches(&pieces, release, pos2(30.0, 30.0), 1);
    let per_three = draw_in_batches(&pieces, release, pos2(30.0, 30.0), 3);
    let single_call = draw_in_batches(&pieces, release, pos2(30.0, 30.0), 100);
    assert_eq!(
        per_event, per_three,
        "the same stroke, batched 1 event per frame vs 3"
    );
    assert_eq!(
        per_three, single_call,
        "the same stroke, batched vs one whole-stroke call"
    );
}
