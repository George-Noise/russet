//! Floating window placement (SPEC §3.11): drop-time edge anchoring (the
//! window stays where the user dropped it) and per-axis repositioning when
//! the main window is resized. Everything about §3.11 lives here: the
//! `WindowPlacement` state kept per window in `App`, the pure anchor /
//! reposition math, the glue that applies it after each `Window::show`, and
//! all of its unit tests. `app.rs` stays the orchestrator.

use eframe::egui::{self, AreaState, Id, LayerId, Order, Pos2, Rect};

/// One CSS-style `rem`: egui 0.37 has no rem API, so this is the 16 pt
/// default body font size. The floating windows' first-open offsets
/// (§3.2/§3.3) and the §3.11 snap distance are expressed in it.
const REM: f32 = 16.0;

/// 0.5rem — the gap between a floating window and the main-window edges on
/// its FIRST open (§3.2/§3.3): the Tools window is offset 0.5rem from the
/// left edge, the Palette window 0.5rem from both the left and the bottom
/// edge. Applied to the `Window::default_pos` anchors in [`super::windows`];
/// afterwards the §3.11 placement logic treats the offset as a dropped gap.
pub(super) const FIRST_OPEN_GAP: f32 = 0.5 * REM;

/// Floating-window magnetism (§3.11): when a window is DROPPED with an
/// edge closer than this distance to a main-window edge, that axis anchors
/// to the edge at the dropped gap — the window is never pulled flush. 2rem;
/// egui 0.37 has no rem API, so this is 2 × the 16 pt default body font
/// size (see [`REM`]). Anchors are decided only when a drag ends — never on
/// main-window resizes.
const WINDOW_SNAP_DISTANCE: f32 = 2.0 * REM;

/// Placement bookkeeping for one floating window ("Tools"/"Palette").
#[derive(Default)]
pub(super) struct WindowPlacement {
    /// Rect of the last frame the window was shown.
    rect: Option<Rect>,
    /// True while the window is dragged by its title bar; the falling edge
    /// (drag end) is where §3.11 anchors are decided.
    dragging: bool,
    /// Edge anchors fixed when the last drag ended. An anchored axis keeps
    /// its dropped gap to that main-window edge on resizes; a free axis
    /// keeps its relative placement (gap-ratio formula).
    anchor: WindowAnchor,
}

/// Per-axis anchor of a floating window to one main-window edge (§3.11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum AxisAnchor {
    /// Free axis: keeps its relative placement (gap-ratio formula) on
    /// main-window resizes.
    #[default]
    None,
    /// Anchored to the lower-coordinate edge (left on x, top on y): keeps
    /// the gap the user dropped the window at.
    Min,
    /// Anchored to the higher-coordinate edge (right on x, bottom on y).
    Max,
}

/// Anchors of one floating window, fixed when its last title-bar drag ended.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct WindowAnchor {
    x: AxisAnchor,
    y: AxisAnchor,
}

/// One axis of the drop-time magnetism: the axis anchors to an edge if the
/// window edge was dropped closer than `distance` to it. If BOTH edges
/// qualify at once (window nearly as wide/tall as the main window),
/// `conflict` wins — left on the horizontal axis, bottom on the vertical
/// one (§3.11).
fn anchor_axis(min_gap: f32, max_gap: f32, distance: f32, conflict: AxisAnchor) -> AxisAnchor {
    let min_close = min_gap < distance;
    let max_close = max_gap < distance;
    if min_close && max_close {
        conflict
    } else if min_close {
        AxisAnchor::Min
    } else if max_close {
        AxisAnchor::Max
    } else {
        AxisAnchor::None
    }
}

/// Anchors of a dropped window: per axis, the edge it was dropped closer
/// than `distance` to (left on horizontal ties, bottom on vertical ones).
/// The window itself stays exactly where the user dropped it — the anchor
/// only decides how it follows that edge on later main-window resizes.
fn window_anchor(rect: Rect, area: Rect, distance: f32) -> WindowAnchor {
    WindowAnchor {
        x: anchor_axis(
            rect.min.x - area.min.x,
            area.max.x - rect.max.x,
            distance,
            AxisAnchor::Min, // horizontal conflict → left
        ),
        y: anchor_axis(
            rect.min.y - area.min.y,
            area.max.y - rect.max.y,
            distance,
            AxisAnchor::Max, // vertical conflict → bottom
        ),
    }
}

/// New window LEFT/TOP coordinate along one axis after the available range
/// changed from `old` to `new` (§3.11). An anchored axis slides with its
/// edge, keeping the gap the user dropped the window at (clamped into the
/// new range); a free axis keeps its relative placement: the ratio between
/// the gap to the lower edge (`win.0 - old.0`) and the gap to the higher
/// edge (`old.1 - win.1`) is preserved (0 = flush left/top, 1 = flush
/// right/bottom, 0.5 = centered). Pure repositioning — it never moves a
/// window flush and never changes anchors.
fn reposition_axis(old: (f32, f32), new: (f32, f32), win: (f32, f32), anchor: AxisAnchor) -> f32 {
    let free_new = ((new.1 - new.0) - (win.1 - win.0)).max(0.0);
    let lo = new.0;
    let hi = new.0 + free_new;
    match anchor {
        AxisAnchor::Min => (lo + (win.0 - old.0).max(0.0)).min(hi),
        AxisAnchor::Max => (hi - (old.1 - win.1).max(0.0)).max(lo),
        AxisAnchor::None => {
            let min_gap = (win.0 - old.0).max(0.0);
            let max_gap = (old.1 - win.1).max(0.0);
            let free_old = min_gap + max_gap;
            let ratio = if free_old > 0.0 {
                min_gap / free_old
            } else {
                0.0
            };
            (lo + ratio * (hi - lo)).clamp(lo, hi)
        }
    }
}

/// New left-top of a floating window after the available area changed, so
/// that an axis anchored to an edge keeps the dropped gap, a top-left
/// window stays top-left, a centered window stays centered and a
/// bottom-right window stays bottom-right (§3.11).
fn reposition_window_pos(
    prev_rect: Rect,
    prev_area: Rect,
    new_area: Rect,
    anchor: WindowAnchor,
) -> Pos2 {
    egui::pos2(
        reposition_axis(
            (prev_area.min.x, prev_area.max.x),
            (new_area.min.x, new_area.max.x),
            (prev_rect.min.x, prev_rect.max.x),
            anchor.x,
        ),
        reposition_axis(
            (prev_area.min.y, prev_area.max.y),
            (new_area.min.y, new_area.max.y),
            (prev_rect.min.y, prev_rect.max.y),
            anchor.y,
        ),
    )
}

/// Moves a floating window's underlying area to `left_top`.
///
/// `Window::current_pos` cannot be used here: in title-bar mode
/// `Window::show_dyn` discards it every frame (it restores the pre-begin
/// position, which is how title-bar drags are applied). Rewriting the area
/// state right after `Window::show` persists into the next frame instead.
///
/// Also used outside §3.11 for one-shot repositioning of transient tool
/// windows (the Text tool moves its input window to the placed anchor).
pub(super) fn set_window_pos(ctx: &egui::Context, area_id: Id, left_top: Pos2) {
    if let Some(mut state) = AreaState::load(ctx, area_id) {
        state.set_left_top_pos(left_top);
        // Windows live in `Order::Middle` (the `Window`/`Area` default);
        // `Areas::get_mut` is `pub(crate)`, `set_state` is public.
        ctx.memory_mut(|mem| {
            mem.areas_mut()
                .set_state(LayerId::new(Order::Middle, area_id), state)
        });
    }
}

/// Placement logic for one floating window, run right after `Window::show`:
///
/// - while the window is dragged by its title bar it follows the pointer
///   freely — §3.11 magnetism never moves a window during the drag and
///   never pulls it flush to an edge;
/// - when the drag ends, each axis whose edge the window was dropped
///   closer than `WINDOW_SNAP_DISTANCE` to a main-window edge is anchored
///   to that edge AT THE DROPPED GAP — the window stays exactly where the
///   user placed it (left wins over right, bottom over top); dropping
///   outside the 2rem zones clears all anchors;
/// - after a main-window resize — and never while dragging — an anchored
///   axis keeps its dropped gap and a free axis keeps its relative
///   placement via the gap-ratio formula. Resizes never anchor, unanchor
///   or move anything flush.
pub(super) fn place_floating_window(
    ctx: &egui::Context,
    area_id: Id,
    rect: Rect,
    content: Rect,
    prev_content: Option<Rect>,
    place: &mut WindowPlacement,
) {
    // The title-bar drag response; the same id `window.rs` uses internally.
    let dragging = ctx
        .read_response(area_id.with("__title_click"))
        .is_some_and(|response| response.dragged());

    if dragging {
        place.dragging = true;
    } else if place.dragging {
        // Drag just ended: fix the window where the USER dropped it.
        place.anchor = window_anchor(rect, content, WINDOW_SNAP_DISTANCE);
        place.dragging = false;
    } else if let (Some(prev_rect), Some(prev_content)) = (place.rect, prev_content)
        && prev_content != content
    {
        let left_top = reposition_window_pos(prev_rect, prev_content, content, place.anchor);
        set_window_pos(ctx, area_id, left_top);
    }
    place.rect = Some(rect);
}

#[cfg(test)]
mod tests {
    use super::{
        AxisAnchor, WINDOW_SNAP_DISTANCE, WindowAnchor, anchor_axis, reposition_axis,
        reposition_window_pos, window_anchor,
    };
    use eframe::egui::{Rect, pos2, vec2};

    fn rect(x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(pos2(x, y), vec2(w, h))
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn anchor_axis_covers_both_sides_and_dead_zone() {
        assert_eq!(
            anchor_axis(10.0, 40.0, 32.0, AxisAnchor::Min),
            AxisAnchor::Min
        );
        assert_eq!(
            anchor_axis(40.0, 4.0, 32.0, AxisAnchor::Min),
            AxisAnchor::Max
        );
        assert_eq!(
            anchor_axis(40.0, 40.0, 32.0, AxisAnchor::Min),
            AxisAnchor::None
        );
        // Exactly at the distance: "closer than" is strict, so no anchor.
        assert_eq!(
            anchor_axis(32.0, 40.0, 32.0, AxisAnchor::Min),
            AxisAnchor::None
        );
    }

    #[test]
    fn anchor_axis_conflict_takes_the_requested_side() {
        // Both edges within reach: the horizontal axis must pick LEFT,
        // the vertical one BOTTOM (§3.11).
        assert_eq!(
            anchor_axis(10.0, 20.0, 32.0, AxisAnchor::Min),
            AxisAnchor::Min
        );
        assert_eq!(
            anchor_axis(10.0, 20.0, 32.0, AxisAnchor::Max),
            AxisAnchor::Max
        );
    }

    #[test]
    fn dropped_window_anchors_to_each_close_edge() {
        let area = rect(0.0, 0.0, 800.0, 600.0);
        // The magnet only decides the ANCHOR; the window stays exactly
        // where the user dropped it — never pulled flush (§3.11).
        assert_eq!(
            window_anchor(rect(10.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::None
            }
        );
        assert_eq!(
            window_anchor(rect(690.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::Max,
                y: AxisAnchor::None
            }
        );
        assert_eq!(
            window_anchor(rect(300.0, 8.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::None,
                y: AxisAnchor::Min
            }
        );
        assert_eq!(
            window_anchor(rect(300.0, 542.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::None,
                y: AxisAnchor::Max
            }
        );
    }

    #[test]
    fn dropped_window_middle_exact_distance_and_flush() {
        let area = rect(0.0, 0.0, 800.0, 600.0);
        // Mid-screen: no edge within reach.
        assert_eq!(
            window_anchor(rect(300.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor::default()
        );
        // Exactly 2rem from the left edge: not "closer than".
        assert_eq!(
            window_anchor(rect(32.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE).x,
            AxisAnchor::None
        );
        // Dropped flush: gap 0 is a valid dropped gap — anchored left, so
        // the window stays flush through resizes.
        assert_eq!(
            window_anchor(rect(0.0, 300.0, 100.0, 50.0), area, WINDOW_SNAP_DISTANCE).x,
            AxisAnchor::Min
        );
    }

    #[test]
    fn anchor_conflict_prefers_left_and_bottom() {
        // 150 pt window in a 200 pt main window: both horizontal gaps (20/30)
        // and both vertical gaps (20/30) are within reach → left and bottom.
        let area = rect(0.0, 0.0, 200.0, 200.0);
        assert_eq!(
            window_anchor(rect(20.0, 20.0, 150.0, 150.0), area, WINDOW_SNAP_DISTANCE),
            WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::Max
            }
        );
    }

    #[test]
    fn reposition_free_axis_keeps_anchors_and_center() {
        let old = (0.0, 1000.0);
        let new = (0.0, 500.0);
        assert_close(
            reposition_axis(old, new, (0.0, 200.0), AxisAnchor::None),
            0.0,
        ); // left-anchored
        assert_close(
            reposition_axis(old, new, (800.0, 1000.0), AxisAnchor::None),
            300.0,
        ); // right-anchored
        assert_close(
            reposition_axis(old, new, (400.0, 600.0), AxisAnchor::None),
            150.0,
        ); // centered
    }

    #[test]
    fn reposition_free_axis_keeps_gap_ratio_when_growing_and_shrinking() {
        let old = (0.0, 1000.0);
        // gaps 240/560 → ratio 0.3
        assert_close(
            reposition_axis(old, (0.0, 2000.0), (240.0, 440.0), AxisAnchor::None),
            540.0,
        );
        assert_close(
            reposition_axis(old, (0.0, 500.0), (240.0, 440.0), AxisAnchor::None),
            90.0,
        );
    }

    #[test]
    fn reposition_free_axis_clamps_window_into_new_range() {
        // Window wider than the new range: clamped flush to the lower edge.
        assert_close(
            reposition_axis(
                (0.0, 1000.0),
                (0.0, 100.0),
                (100.0, 300.0),
                AxisAnchor::None,
            ),
            0.0,
        );
        // Window fills the old range exactly (no gaps): keep the lower edge.
        assert_close(
            reposition_axis((0.0, 200.0), (0.0, 600.0), (0.0, 200.0), AxisAnchor::None),
            0.0,
        );
    }

    #[test]
    fn anchored_axis_keeps_the_dropped_gap_on_resize() {
        let old = (0.0, 1000.0);
        // Dropped with a 30 pt left gap: the gap survives grow and shrink.
        assert_close(
            reposition_axis(old, (0.0, 2000.0), (30.0, 230.0), AxisAnchor::Min),
            30.0,
        );
        assert_close(
            reposition_axis(old, (0.0, 500.0), (30.0, 230.0), AxisAnchor::Min),
            30.0,
        );
        // Dropped with a 30 pt right gap: slides with the right edge.
        assert_close(
            reposition_axis(old, (0.0, 2000.0), (770.0, 970.0), AxisAnchor::Max),
            1770.0,
        );
        assert_close(
            reposition_axis(old, (0.0, 500.0), (770.0, 970.0), AxisAnchor::Max),
            270.0,
        );
    }

    #[test]
    fn anchored_axis_clamps_into_the_new_range() {
        let old = (0.0, 1000.0);
        // Left gap 300 no longer fits (free space 200): the window is kept
        // inside the range instead of overflowing the right edge.
        assert_close(
            reposition_axis(old, (0.0, 400.0), (300.0, 500.0), AxisAnchor::Min),
            200.0,
        );
        // Right gap 300 no longer fits: clamped against the left edge.
        assert_close(
            reposition_axis(old, (0.0, 400.0), (500.0, 700.0), AxisAnchor::Max),
            0.0,
        );
    }

    #[test]
    fn reposition_bottom_right_free_window_stays_bottom_right() {
        let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
        let new_area = rect(0.0, 0.0, 500.0, 600.0);
        let prev = rect(800.0, 650.0, 200.0, 150.0); // flush right-bottom
        assert_eq!(
            reposition_window_pos(prev, prev_area, new_area, WindowAnchor::default()),
            pos2(300.0, 450.0)
        );
    }

    #[test]
    fn reposition_does_not_magnetize_free_windows() {
        // 100 pt off both edges: after a resize the window is only
        // re-proportioned, never pulled flush to an edge.
        let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
        let new_area = rect(0.0, 0.0, 400.0, 300.0);
        let result = reposition_window_pos(
            rect(100.0, 100.0, 200.0, 200.0),
            prev_area,
            new_area,
            WindowAnchor::default(),
        );
        assert_close(result.x, 25.0); // x ratio 100/800 = 0.125, free_new 200
        assert_close(result.y, 16.667); // y ratio 100/600 = 1/6, free_new 100
    }

    #[test]
    fn anchored_window_keeps_dropped_gaps_through_resize() {
        let prev_area = rect(0.0, 0.0, 1000.0, 800.0);
        let new_area = rect(0.0, 0.0, 500.0, 600.0);
        // Dropped 25 pt from the left and 20 pt from the top: BOTH gaps are
        // kept exactly, on any resize.
        let anchor = window_anchor(
            rect(25.0, 20.0, 200.0, 150.0),
            prev_area,
            WINDOW_SNAP_DISTANCE,
        );
        assert_eq!(
            anchor,
            WindowAnchor {
                x: AxisAnchor::Min,
                y: AxisAnchor::Min
            }
        );
        assert_eq!(
            reposition_window_pos(rect(25.0, 20.0, 200.0, 150.0), prev_area, new_area, anchor),
            pos2(25.0, 20.0)
        );
        // Dropped flush right-bottom: stays flush right-bottom.
        let flush = window_anchor(
            rect(800.0, 650.0, 200.0, 150.0),
            prev_area,
            WINDOW_SNAP_DISTANCE,
        );
        assert_eq!(
            flush,
            WindowAnchor {
                x: AxisAnchor::Max,
                y: AxisAnchor::Max
            }
        );
        assert_eq!(
            reposition_window_pos(rect(800.0, 650.0, 200.0, 150.0), prev_area, new_area, flush),
            pos2(300.0, 450.0)
        );
    }
}
