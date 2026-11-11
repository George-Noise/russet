//! Floating window placement (SPEC §3.11): drop-time edge anchoring (the
//! window stays where the user dropped it) and per-axis repositioning when
//! the main window is resized. Everything about §3.11 lives here: the
//! `WindowPlacement` state kept per window in `App`, the pure anchor /
//! reposition math, the glue that applies it after each `Window::show`, and
//! all of its unit tests (in the sibling `placement/tests.rs`). `app.rs` stays
//! the orchestrator. Since the owner request 2026-11-11 this covers all four
//! floating windows — "Tools", "Palette", "History" and "Layers" — and also
//! hosts the one-shot §3.2 first-open ratio of the Tools window.

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

/// 1.75rem — the extra first-open LIFT of the "Layers" window above the
/// usual [`FIRST_OPEN_GAP`]: its bottom edge sits 0.5rem + 1.75rem = 2.25rem
/// above the main window's bottom edge so the zoom % label in the canvas's
/// bottom-right corner stays readable (owner request 2026-11-10).
pub(super) const LAYERS_FIRST_OPEN_LIFT: f32 = 1.75 * REM;

/// Floating-window magnetism (§3.11): when a window is DROPPED with an
/// edge closer than this distance to a main-window edge, that axis anchors
/// to the edge at the dropped gap — the window is never pulled flush. 2rem;
/// egui 0.37 has no rem API, so this is 2 × the 16 pt default body font
/// size (see [`REM`]). Anchors are decided only when a drag ends — never on
/// main-window resizes.
const WINDOW_SNAP_DISTANCE: f32 = 2.0 * REM;

/// First-open vertical gap ratio of the "Tools" window (§3.2, owner request
/// 2026-11-11): 40% of the FREE vertical space sits above the window and
/// 60% below it — a bit higher than the former dead center (owner: "расстояние
/// до верхнего края 40%, до нижнего 60%"). Applied once, on the first frame
/// the window is shown, when its auto-sized height is first known (see
/// [`WindowPlacement::first_open_ratio_y`]); afterwards §3.11 takes over and
/// the user's placement always wins.
pub(super) const TOOLS_FIRST_OPEN_RATIO: f32 = 0.4;

/// Placement bookkeeping for one floating window ("Tools"/"Palette"/
/// "History"/"Layers").
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
    /// One-shot §3.2 first-open placement of the Tools window (owner request
    /// 2026-11-11): on the FIRST frame the window is shown its auto-sized
    /// height becomes known, and the window is moved so the free vertical
    /// space splits [`TOOLS_FIRST_OPEN_RATIO`] above / the rest below. A
    /// `pivot` + `default_pos` alone cannot express this (the pivot anchors
    /// a corner/edge, and the height is unknown before the first layout),
    /// so the ratio is applied exactly once through the same `set_window_pos`
    /// persistence the §3.11 repositioning uses. `None` = no first-open fix
    /// (Palette/History/Layers anchor to window edges instead).
    first_open_ratio_y: Option<f32>,
}

impl WindowPlacement {
    /// Placement state of a window whose first-open position needs its own
    /// auto-sized height (§3.2 Tools window: 40%/60% of the free vertical
    /// space below/above — see [`TOOLS_FIRST_OPEN_RATIO`]).
    pub(super) fn first_open_at_ratio(ratio: f32) -> Self {
        Self {
            first_open_ratio_y: Some(ratio),
            ..Self::default()
        }
    }
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

/// Left-top position that splits the FREE vertical space of `area` around
/// `win_rect` into `ratio` above / `1 - ratio` below (§3.2 first open of the
/// Tools window: 0.4 above, 0.6 below — a bit above center). The horizontal
/// position of the window is kept; a window taller than the area is clamped
/// flush to the top edge. Pure helper of the one-shot first-open placement.
fn first_open_ratio_pos(area: Rect, win_rect: Rect, ratio: f32) -> Pos2 {
    let free = (area.height() - win_rect.height()).max(0.0);
    egui::pos2(win_rect.min.x, area.min.y + ratio.clamp(0.0, 1.0) * free)
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
/// - on the very first frame a window with a first-open ratio is shown
///   (§3.2 Tools window), the ratio is applied exactly — the auto-sized
///   height is known only after this first layout, so the position is
///   persisted for the next frame through the same `set_window_pos` path.
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
    } else if let Some(ratio) = place.first_open_ratio_y.take() {
        // First frame this window is ever shown: its auto-sized height is
        // now known, so the §3.2 first-open ratio can be applied exactly.
        // `place.rect` is still None here (rect is recorded below), which
        // makes this branch unreachable on every later frame — reopening
        // the window via View never re-fires it.
        let left_top = first_open_ratio_pos(content, rect, ratio);
        set_window_pos(ctx, area_id, left_top);
        place.rect = Some(Rect::from_min_size(left_top, rect.size()));
        return;
    } else if let (Some(prev_rect), Some(prev_content)) = (place.rect, prev_content)
        && prev_content != content
    {
        let left_top = reposition_window_pos(prev_rect, prev_content, content, place.anchor);
        set_window_pos(ctx, area_id, left_top);
    }
    place.rect = Some(rect);
}

#[cfg(test)]
mod tests;
