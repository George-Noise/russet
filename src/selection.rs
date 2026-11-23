//! The pure selection domain (owner request 2026-11-20): regions of one of
//! two kinds — a rectangle dragged between two corners or a freeform lasso
//! polygon auto-closed on release — combined into ONE selection with
//! replace / add (Shift) / subtract (Alt) semantics, plus the pixel-level
//! helpers the ops (copy / cut / move / crop) build on. Pure functions of a
//! pixmap — the gesture state, the App wiring and the overlay rendering
//! live in [`crate::app::selection`], the crop orchestration (one node per
//! layer, like the Resize dialog's) in [`crate::layers::LayerStack
//! ::crop_all`]. The module is a sibling of `tools`/`text` for the same
//! reason those are top-level: no `App`, no egui interaction — testable
//! math.
//!
//! ## Model
//!
//! A [`Selection`] is a list of OPERATIONS: `(region, add?)` pairs applied
//! in order over the pixel grid — the adds paint their region's pixels in,
//! the subtracts paint them back out. The PIXEL TRUTH ([`Mask`]) is always
//! derivable from the list, so subtract works at the pixel level (a hole
//! cut from a bigger rectangle stays a hole no matter how the regions
//! overlap), while the MOVE just translates the points and everything
//! re-derives. Nothing here allocates beyond the selection's bounding box.
//!
//! ## Pixel semantics (test-pinned)
//!
//! - A RECT region selects every pixel whose column/row lies in
//!   `[floor(min), ceil(max))` of the two drag corners — whole pixels: the
//!   drag (10.3, 5.7)→(40.6, 30.2) selects columns 10..=40 / rows 5..=30.
//! - A LASSO region selects a pixel iff its CENTER (`x + 0.5`, `y + 0.5`)
//!   lies inside the closed polygon (the classic even-odd scanline fill); a
//!   lasso of fewer than 3 distinct points selects nothing.
//! - The ANTS ([`Selection::ants`]) are the TRUE selection contour: the
//!   crack-boundary loops between selected and unselected pixels
//!   (collinear unit steps merged into long segments), so add/subtract
//!   shapes — holes included — outline exactly. Loops trace clockwise
//!   around selected pixels (the selected side stays on the travel's
//!   right); at a diagonal-touch (checkerboard) corner the walk turns
//!   RIGHT, keeping the two touching loops separate.

use crate::tools::{DirtyRect, blend_pixel};
use eframe::egui::Pos2;
use tiny_skia::{Pixmap, PixmapPaint, PremultipliedColorU8, Transform};

/// An integer pixel rectangle (canvas space): columns `x .. x + w`,
/// rows `y .. y + h`. `w`/`h` are `u32`, positions `i64` so a drag corner
/// outside the canvas never overflows on the way to a clamp.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IRect {
    pub x: i64,
    pub y: i64,
    pub w: u32,
    pub h: u32,
}

impl IRect {
    /// The rect clamped to a `width × height` canvas — empty results stay
    /// `None` (a selection dragged fully off the canvas selects nothing).
    pub fn clamped(self, width: u32, height: u32) -> Option<IRect> {
        let x0 = self.x.max(0);
        let y0 = self.y.max(0);
        let x1 = (self.x + self.w as i64).min(width as i64);
        let y1 = (self.y + self.h as i64).min(height as i64);
        if x1 <= x0 || y1 <= y0 {
            return None;
        }
        Some(IRect {
            x: x0,
            y: y0,
            w: (x1 - x0) as u32,
            h: (y1 - y0) as u32,
        })
    }

    /// The smallest rect covering BOTH rects (the mask grid's extent).
    pub fn union(self, other: IRect) -> IRect {
        let x0 = self.x.min(other.x);
        let y0 = self.y.min(other.y);
        let x1 = (self.x + self.w as i64).max(other.x + other.w as i64);
        let y1 = (self.y + self.h as i64).max(other.y + other.h as i64);
        IRect {
            x: x0,
            y: y0,
            w: (x1 - x0).max(1) as u32,
            h: (y1 - y0).max(1) as u32,
        }
    }
}

/// One selectable region. Points are raw canvas points (f32) — the pixel
/// semantics pin how they quantize (module doc).
#[derive(Clone, Debug, PartialEq)]
pub enum Region {
    /// A rectangle dragged between two corners (order-insensitive).
    Rect { a: Pos2, b: Pos2 },
    /// A freeform lasso path, auto-closed (the last point connects to the
    /// first). Fewer than 3 DISTINCT points select nothing.
    Lasso { points: Vec<Pos2> },
}

impl Region {
    /// The region's integer bounding box (unclamped — the caller clamps to
    /// the canvas).
    pub fn integer_bounds(&self) -> IRect {
        let (min_x, max_x, min_y, max_y) = match self {
            Region::Rect { a, b } => (a.x.min(b.x), a.x.max(b.x), a.y.min(b.y), a.y.max(b.y)),
            Region::Lasso { points } => {
                let (mut min_x, mut min_y) = (f32::INFINITY, f32::INFINITY);
                let (mut max_x, mut max_y) = (f32::NEG_INFINITY, f32::NEG_INFINITY);
                for p in points {
                    min_x = min_x.min(p.x);
                    max_x = max_x.max(p.x);
                    min_y = min_y.min(p.y);
                    max_y = max_y.max(p.y);
                }
                (min_x, max_x, min_y, max_y)
            }
        };
        IRect {
            x: min_x.floor() as i64,
            y: min_y.floor() as i64,
            w: ((max_x.ceil() - min_x.floor()) as u32).max(1),
            h: ((max_y.ceil() - min_y.floor()) as u32).max(1),
        }
    }

    /// The region's integer pixel rect (a rect's exact selection area; a
    /// lasso's bounding box). A degenerate lasso (fewer than 3 distinct
    /// points) yields `None`.
    pub fn pixel_rect(&self) -> Option<IRect> {
        if let Region::Lasso { points } = self
            && distinct_points(points).len() < 3
        {
            return None;
        }
        Some(self.integer_bounds())
    }

    /// Whether the canvas POINT hits the region: the rect test is the
    /// point's pixel inside the integer rect; the lasso runs the even-odd
    /// point-in-polygon test at the exact point (the continuous form of the
    /// same rule the scanline fill uses).
    pub fn contains(&self, p: Pos2) -> bool {
        match self {
            Region::Rect { a, b } => {
                let (min_x, max_x) = (a.x.min(b.x).floor(), a.x.max(b.x).ceil());
                let (min_y, max_y) = (a.y.min(b.y).floor(), a.y.max(b.y).ceil());
                p.x >= min_x && p.x < max_x && p.y >= min_y && p.y < max_y
            }
            Region::Lasso { points } => point_in_polygon(p, points),
        }
    }

    /// Paints the region's selected pixels into the mask (or un-paints
    /// them — a subtract paints `false`).
    fn paint(&self, mask: &mut Mask, add: bool) {
        match self {
            Region::Rect { a, b } => {
                let (x0, x1) = (a.x.min(b.x).floor() as i64, a.x.max(b.x).ceil() as i64);
                let (y0, y1) = (a.y.min(b.y).floor() as i64, a.y.max(b.y).ceil() as i64);
                for py in y0..y1 {
                    for px in x0..x1 {
                        mask.set(px, py, add);
                    }
                }
            }
            Region::Lasso { points } => fill_polygon_scanline(mask, points, add),
        }
    }

    /// The region intersected with the canvas rect (owner bug report
    /// 2026-11-20: a move that lands PARTIALLY off-canvas must shrink the
    /// selection with the pixels — the stamp clips them, so the region
    /// list must follow, or the next move "heals" the full shape and
    /// claims pixels that no longer exist). `None` = the region lies
    /// fully outside the canvas (or degenerates).
    pub fn clipped_to(self, width: u32, height: u32) -> Option<Region> {
        match self {
            Region::Rect { a, b } => {
                let (min_x, max_x) = (a.x.min(b.x).max(0.0), a.x.max(b.x).min(width as f32));
                let (min_y, max_y) = (a.y.min(b.y).max(0.0), a.y.max(b.y).min(height as f32));
                if max_x <= min_x || max_y <= min_y {
                    return None;
                }
                Some(Region::Rect {
                    a: Pos2::new(min_x, min_y),
                    b: Pos2::new(max_x, max_y),
                })
            }
            Region::Lasso { points } => {
                let clipped = clip_polygon_to_rect(&points, width as f32, height as f32);
                // Fewer than 3 DISTINCT points select nothing (the module rule).
                (distinct_points(&clipped).len() >= 3).then_some(Region::Lasso { points: clipped })
            }
        }
    }
}

/// One Sutherland–Hodgman half-plane pass: keep the polygon's points where
/// `coord ≥ bound` or `coord ≤ bound`, closing the cut with the segment ×
/// plane crossings. `horizontal` picks the clipped axis.
fn clip_halfplane(poly: Vec<Pos2>, horizontal: bool, bound: f32, keep_ge: bool) -> Vec<Pos2> {
    let inside = |p: &Pos2| {
        let c = if horizontal { p.x } else { p.y };
        if keep_ge { c >= bound } else { c <= bound }
    };
    let cross = |a: &Pos2, b: &Pos2| -> Pos2 {
        let (ca, cb) = if horizontal { (a.x, b.x) } else { (a.y, b.y) };
        // Both endpoints straddle the plane, so cb != ca: t is finite.
        let t = (bound - ca) / (cb - ca);
        if horizontal {
            Pos2::new(bound, a.y + (b.y - a.y) * t)
        } else {
            Pos2::new(a.x + (b.x - a.x) * t, bound)
        }
    };
    let mut out = Vec::with_capacity(poly.len() + 1);
    let n = poly.len();
    for i in 0..n {
        let cur = poly[i];
        let prev = poly[(i + n - 1) % n];
        match (inside(&prev), inside(&cur)) {
            (true, true) => out.push(cur),
            (true, false) => out.push(cross(&prev, &cur)),
            (false, true) => {
                out.push(cross(&prev, &cur));
                out.push(cur);
            }
            (false, false) => {}
        }
    }
    out
}

/// Sutherland–Hodgman: clip the polygon against the canvas rect, one edge
/// half-plane at a time. For a path that leaves and re-enters the canvas
/// SH leaves zero-width connecting bridges — they contribute nothing to
/// the scanline fill, and the ants derive from the MASK, so the true
/// contour stays true.
fn clip_polygon_to_rect(points: &[Pos2], w: f32, h: f32) -> Vec<Pos2> {
    let mut poly = points.to_vec();
    for (horizontal, bound, keep_ge) in [
        (true, 0.0, true),
        (true, w, false),
        (false, 0.0, true),
        (false, h, false),
    ] {
        if poly.len() < 3 {
            return Vec::new();
        }
        poly = clip_halfplane(poly, horizontal, bound, keep_ge);
    }
    poly
}

/// The even-odd point-in-polygon test (the lasso's continuous hit rule).
fn point_in_polygon(p: Pos2, points: &[Pos2]) -> bool {
    let mut inside = false;
    let mut j = points.len().saturating_sub(1);
    for (i, pi) in points.iter().enumerate() {
        let pj = points[j];
        // The half-open edge rule ((yi > py) != (yj > py)) keeps vertices
        // from double-counting.
        let crosses = (pi.y > p.y) != (pj.y > p.y)
            && p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x;
        if crosses {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// The lasso's scanline fill: for each pixel row, intersect the polygon's
/// edges with the row's center line (`y + 0.5`) and fill the spans between
/// crossing pairs — a pixel is selected iff its CENTER is inside (module
/// doc). A degenerate lasso (fewer than 3 distinct points) paints nothing.
fn fill_polygon_scanline(mask: &mut Mask, points: &[Pos2], add: bool) {
    let distinct = distinct_points(points);
    if distinct.len() < 3 {
        return;
    }
    let n = distinct.len();
    for row in mask.y..mask.y + mask.h as i64 {
        let cy = row as f64 + 0.5;
        let mut crossings: Vec<f64> = Vec::new();
        let mut j = n - 1;
        for (i, pi) in distinct.iter().enumerate() {
            let pj = distinct[j];
            let (y_i, y_j) = (pi.y as f64, pj.y as f64);
            if (y_i <= cy && cy < y_j) || (y_j <= cy && cy < y_i) {
                let t = (cy - y_i) / (y_j - y_i);
                crossings.push(pi.x as f64 + t * (pj.x - pi.x) as f64);
            }
            j = i;
        }
        crossings.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        // Crossing pairs span selected pixel runs: pixel px is selected iff
        // its center px + 0.5 lies in [xa, xb).
        for pair in crossings.chunks(2) {
            if let [xa, xb] = pair {
                let px0 = (*xa - 0.5).ceil() as i64;
                let px1 = (*xb - 0.5).ceil() as i64;
                for px in px0..px1 {
                    mask.set(px, row, add);
                }
            }
        }
    }
}

/// Consecutive duplicate points collapsed (the lasso drag may push the same
/// point twice on slow pointer frames).
fn distinct_points(points: &[Pos2]) -> Vec<Pos2> {
    let mut out: Vec<Pos2> = Vec::with_capacity(points.len());
    for p in points {
        if out.last().is_none_or(|last| *last != *p) {
            out.push(*p);
        }
    }
    out
}

/// A bbox-tight boolean grid: bit `(ix, iy)` is the pixel
/// `(x + ix, y + iy)`. Built from a selection's ops by
/// [`Selection::build_mask`], consumed by the pixel helpers below.
#[derive(Clone, Debug)]
pub struct Mask {
    pub x: i64,
    pub y: i64,
    pub w: u32,
    pub h: u32,
    bits: Vec<bool>,
}

impl Mask {
    fn new(x: i64, y: i64, w: u32, h: u32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            bits: vec![false; w as usize * h as usize],
        }
    }

    /// Whether the absolute pixel is selected (outside the grid = false).
    pub fn get(&self, px: i64, py: i64) -> bool {
        let (ix, iy) = (px - self.x, py - self.y);
        if ix < 0 || iy < 0 || ix >= self.w as i64 || iy >= self.h as i64 {
            return false;
        }
        self.bits[iy as usize * self.w as usize + ix as usize]
    }

    fn set(&mut self, px: i64, py: i64, v: bool) {
        let (ix, iy) = (px - self.x, py - self.y);
        if ix < 0 || iy < 0 || ix >= self.w as i64 || iy >= self.h as i64 {
            return;
        }
        self.bits[iy as usize * self.w as usize + ix as usize] = v;
    }

    /// The tight pixel bounds of the SELECTED bits (a subtract can shrink
    /// or empty the union box the grid was built with).
    pub(crate) fn tight_bounds(&self) -> Option<IRect> {
        let (mut min_x, mut min_y) = (i64::MAX, i64::MAX);
        let (mut max_x, mut max_y) = (i64::MIN, i64::MIN);
        for iy in 0..self.h as i64 {
            for ix in 0..self.w as i64 {
                if self.bits[iy as usize * self.w as usize + ix as usize] {
                    let (px, py) = (self.x + ix, self.y + iy);
                    min_x = min_x.min(px);
                    min_y = min_y.min(py);
                    max_x = max_x.max(px + 1);
                    max_y = max_y.max(py + 1);
                }
            }
        }
        if max_x <= min_x {
            return None;
        }
        Some(IRect {
            x: min_x,
            y: min_y,
            w: (max_x - min_x) as u32,
            h: (max_y - min_y) as u32,
        })
    }
}

/// One applied selection operation: the region and whether it ADDS its
/// pixels (Shift gesture / a fresh drag) or SUBTRACTS them (Alt gesture).
#[derive(Clone, Debug, PartialEq)]
pub struct SelectOp {
    pub region: Region,
    pub add: bool,
}

/// The selection state: the op list PLUS the derived views (the tight
/// bounds and the boundary loops) recomputed on every mutation — regions
/// are few and every derived thing is bbox-tight, so a refresh is cheap
/// next to the ops it gates.
#[derive(Clone, Debug, Default)]
pub struct Selection {
    ops: Vec<SelectOp>,
    /// The crack-boundary loops (canvas-space corner points, collinear runs
    /// merged), refreshed with the bounds.
    ants: Vec<Vec<Pos2>>,
    bounds: Option<IRect>,
}

impl Selection {
    pub fn clear(&mut self) {
        self.ops.clear();
        self.ants.clear();
        self.bounds = None;
    }

    pub fn is_empty(&self) -> bool {
        self.bounds.is_none()
    }

    /// The cached boundary loops (empty = no selection).
    pub fn ants(&self) -> &[Vec<Pos2>] {
        &self.ants
    }

    /// The tight pixel bounds — `None` when empty.
    pub fn bounds(&self) -> Option<IRect> {
        self.bounds
    }

    /// Whether the canvas point is selected: inside at least one ADD
    /// region and no SUBTRACT region (the exact continuous form of the
    /// mask rule — the move gesture's hit test).
    pub fn contains(&self, p: Pos2) -> bool {
        self.ops
            .iter()
            .filter(|op| op.region.contains(p))
            .any(|op| op.add)
            && !self.ops.iter().any(|op| !op.add && op.region.contains(p))
    }

    /// The pixel truth of the whole selection: the ops painted in order
    /// over the union bbox of the ADD regions (subtracts outside it remove
    /// nothing), clamped to the canvas.
    pub fn build_mask(&self, width: u32, height: u32) -> Option<Mask> {
        let mut union: Option<IRect> = None;
        for op in &self.ops {
            if !op.add {
                continue;
            }
            if let Some(rect) = op
                .region
                .pixel_rect()
                .and_then(|r| r.clamped(width, height))
            {
                union = Some(match union {
                    None => rect,
                    Some(u) => u.union(rect),
                });
            }
        }
        let union = union?;
        let mut mask = Mask::new(union.x, union.y, union.w, union.h);
        for op in &self.ops {
            op.region.paint(&mut mask, op.add);
        }
        Some(mask)
    }

    /// A plain gesture: `Some(region)` replaces the whole selection, `None`
    /// deselects (a plain lasso click).
    pub fn replace(&mut self, region: Option<Region>, width: u32, height: u32) {
        self.ops = region
            .map(|r| SelectOp {
                region: r,
                add: true,
            })
            .into_iter()
            .collect();
        self.refresh(width, height);
    }

    /// Shift gesture: union the region in.
    pub fn add(&mut self, region: Region, width: u32, height: u32) {
        self.ops.push(SelectOp { region, add: true });
        self.refresh(width, height);
    }

    /// Alt gesture: cut the region's pixels out (appended as a subtract op;
    /// the mask derives it away on the next refresh).
    pub fn subtract(&mut self, region: Region, width: u32, height: u32) {
        self.ops.push(SelectOp { region, add: false });
        self.refresh(width, height);
    }

    /// Moves every region by an integer pixel offset — the move gesture's
    /// commit. Everything (mask, bounds, loops) re-derives from the moved
    /// points.
    pub fn translate(&mut self, dx: i64, dy: i64, width: u32, height: u32) {
        if dx == 0 && dy == 0 {
            return;
        }
        for op in &mut self.ops {
            match &mut op.region {
                Region::Rect { a, b } => {
                    a.x += dx as f32;
                    a.y += dy as f32;
                    b.x += dx as f32;
                    b.y += dy as f32;
                }
                Region::Lasso { points } => {
                    for p in points {
                        p.x += dx as f32;
                        p.y += dy as f32;
                    }
                }
            }
        }
        self.refresh(width, height);
    }

    /// Clips every op's region to the canvas rect and drops the regions
    /// that fell outside entirely; an empty result deselects (owner bug
    /// report 2026-11-20, the move commit's clip). The derived views
    /// (bounds, ants, mask) were always canvas-clamped — but the OPS were
    /// not, and `contains` (the move hit test) and `translate` work on the
    /// ops: an off-canvas excursion would restore the full region shape on
    /// the next move, claiming pixels the canvas no longer holds.
    pub fn clip_to_canvas(&mut self, width: u32, height: u32) {
        self.ops = self
            .ops
            .drain(..)
            .filter_map(|mut op| {
                op.region = op.region.clipped_to(width, height)?;
                Some(op)
            })
            .collect();
        self.refresh(width, height);
    }

    /// Recomputes the tight bounds and the boundary loops from the ops.
    fn refresh(&mut self, width: u32, height: u32) {
        self.ants.clear();
        self.bounds = None;
        if let Some(mask) = self.build_mask(width, height)
            && let Some(bounds) = mask.tight_bounds()
        {
            self.bounds = Some(bounds);
            self.ants = trace_boundary_loops(&mask);
        }
    }
}

/// Traces the mask's crack-boundary loops (module doc): directed unit edges
/// between selected and unselected pixels, chained with the right-turn rule
/// at diagonal-touch corners, collinear runs merged. Canvas-space corners.
fn trace_boundary_loops(mask: &Mask) -> Vec<Vec<Pos2>> {
    /// The direction cycle R → D → L → U (clockwise); the right turn from
    /// `d` is the next entry.
    #[derive(Clone, Copy, PartialEq)]
    enum Dir {
        R,
        D,
        L,
        U,
    }
    impl Dir {
        fn right(self) -> Self {
            match self {
                Dir::R => Dir::D,
                Dir::D => Dir::L,
                Dir::L => Dir::U,
                Dir::U => Dir::R,
            }
        }
    }
    // Outgoing directed edges per corner: at most two (the diagonal-touch
    // case). The walk pops them, so a Vec of removable entries.
    type EdgeTarget = (Dir, (i64, i64));
    type EdgeMap = std::collections::HashMap<(i64, i64), Vec<EdgeTarget>>;
    let mut edges: EdgeMap = std::collections::HashMap::new();
    for py in mask.y..mask.y + mask.h as i64 {
        for px in mask.x..mask.x + mask.w as i64 {
            if !mask.get(px, py) {
                continue;
            }
            let mut edge = |dir: Dir, from: (i64, i64), to: (i64, i64)| {
                edges.entry(from).or_default().push((dir, to));
            };
            if !mask.get(px, py - 1) {
                edge(Dir::R, (px, py), (px + 1, py)); // top edge, rightward
            }
            if !mask.get(px + 1, py) {
                edge(Dir::D, (px + 1, py), (px + 1, py + 1)); // right edge, down
            }
            if !mask.get(px, py + 1) {
                edge(Dir::L, (px + 1, py + 1), (px, py + 1)); // bottom edge, left
            }
            if !mask.get(px - 1, py) {
                edge(Dir::U, (px, py + 1), (px, py)); // left edge, upward
            }
        }
    }
    let mut loops = Vec::new();
    // Chain edges into loops until every edge is consumed. The same-loop
    // continuation at a diagonal-touch corner is the RIGHT turn (module
    // doc); elsewhere the single candidate wins.
    while let Some(&start) = edges.iter().find(|(_, v)| !v.is_empty()).map(|(k, _)| k) {
        let Some((dir0, next0)) = edges.get_mut(&start).and_then(|v| {
            if v.is_empty() {
                None
            } else {
                Some(v.remove(0))
            }
        }) else {
            break;
        };
        let mut points = vec![Pos2::new(start.0 as f32, start.1 as f32)];
        let mut corner = next0;
        let mut incoming = dir0;
        while corner != start {
            let Some(candidates) = edges.get_mut(&corner) else {
                break;
            };
            if candidates.is_empty() {
                break;
            }
            let index = candidates
                .iter()
                .position(|(d, _)| *d == incoming.right())
                .or_else(|| candidates.iter().position(|(d, _)| *d == incoming))
                .unwrap_or(0);
            let (dir, next) = candidates.remove(index);
            points.push(Pos2::new(corner.0 as f32, corner.1 as f32));
            corner = next;
            incoming = dir;
        }
        if corner == start && points.len() >= 4 {
            loops.push(merge_collinear(points));
        }
    }
    loops
}

/// Drops the intermediate corners of collinear unit runs, so a 100-px
/// rectangle's loop is 4 corners, not 400.
fn merge_collinear(points: Vec<Pos2>) -> Vec<Pos2> {
    if points.len() < 3 {
        return points;
    }
    let mut out: Vec<Pos2> = Vec::with_capacity(points.len());
    let n = points.len();
    for i in 0..n {
        let prev = points[(i + n - 1) % n];
        let cur = points[i];
        let next = points[(i + 1) % n];
        let (dx1, dy1) = (cur.x - prev.x, cur.y - prev.y);
        let (dx2, dy2) = (next.x - cur.x, next.y - cur.y);
        let cross = dx1 * dy2 - dy1 * dx2;
        if cross != 0.0 {
            out.push(cur);
        }
    }
    out
}

/// Copies the masked pixels into a tight pixmap (`mask.w × mask.h`):
/// selected pixels carry the source's premultiplied bytes, the rest stay
/// transparent. THE copy payload (the clipboard, the move's floating
/// buffer).
pub fn extract_masked(src: &Pixmap, mask: &Mask) -> Option<Pixmap> {
    let mut out = Pixmap::new(mask.w, mask.h)?;
    let stride = src.width() as usize * 4;
    let data = src.data();
    let out_data = out.data_mut();
    for iy in 0..mask.h as i64 {
        for ix in 0..mask.w as i64 {
            if !mask.get(mask.x + ix, mask.y + iy) {
                continue;
            }
            let (sx, sy) = (mask.x + ix, mask.y + iy);
            if sx < 0 || sy < 0 || sx >= src.width() as i64 || sy >= src.height() as i64 {
                continue;
            }
            let from = sy as usize * stride + sx as usize * 4;
            let to = (iy as usize * mask.w as usize + ix as usize) * 4;
            out_data[to..to + 4].copy_from_slice(&data[from..from + 4]);
        }
    }
    Some(out)
}

/// Sets every masked pixel of `dst` to transparent premultiplied zero —
/// the cut / the move's lift (destination-out with a fully opaque eraser
/// zeroes exactly these pixels; writing zero directly skips the blend).
pub fn erase_masked(dst: &mut Pixmap, mask: &Mask) {
    let (stride, height) = (dst.width() as usize, dst.height() as i64);
    let pixels = dst.pixels_mut();
    let zero = PremultipliedColorU8::from_rgba(0, 0, 0, 0);
    for iy in 0..mask.h as i64 {
        for ix in 0..mask.w as i64 {
            if !mask.get(mask.x + ix, mask.y + iy) {
                continue;
            }
            let (px, py) = (mask.x + ix, mask.y + iy);
            if px < 0 || py < 0 || px >= stride as i64 || py >= height {
                continue;
            }
            if let Some(zero) = zero {
                pixels[py as usize * stride + px as usize] = zero;
            }
        }
    }
}

/// Writes the backup's bytes back over the SAME rect (the move's per-frame
/// restore — a byte copy, not a blend: the region may currently hold the
/// floating content's stamps that must vanish).
pub fn restore_region(dst: &mut Pixmap, backup: &Pixmap, rect: IRect) {
    let Some(clamped) = rect.clamped(dst.width(), dst.height()) else {
        return;
    };
    if backup.width() < clamped.w || backup.height() < clamped.h {
        return;
    }
    let stride = dst.width() as usize * 4;
    let backup_stride = backup.width() as usize * 4;
    let backup_data = backup.data();
    let dst_data = dst.data_mut();
    for row in 0..clamped.h as i64 {
        let from = row as usize * backup_stride;
        let to = (clamped.y + row) as usize * stride + clamped.x as usize * 4;
        let count = clamped.w as usize * 4;
        dst_data[to..to + count].copy_from_slice(&backup_data[from..from + count]);
    }
}

/// Stamps `clip` (premultiplied src-over) at the integer position — the
/// paste and the move's per-frame draw.
pub fn stamp(dst: &mut Pixmap, clip: &Pixmap, x: i64, y: i64) {
    dst.draw_pixmap(
        x as i32,
        y as i32,
        clip.as_ref(),
        &PixmapPaint::default(),
        Transform::identity(),
        None,
    );
}

/// The selection-CLIPPED paint commit's transfer step (owner request
/// 2026-11-21: an active selection confines every canvas-changing tool to
/// its region). The stroke first rasterizes into a transparent SCRATCH
/// pixmap — so the tiny-skia paths keep their exact geometry and AA, and
/// src-over's associativity makes the transferred result pixel-identical
/// to the direct paint — then this blends the scratch's `region` (canvas
/// space) onto `dst` ONLY where the mask selects: each scratch pixel goes
/// down as premultiplied src-over at `origin + scratch pixel` (`origin` is
/// the scratch's canvas position; a full-canvas scratch passes `(0, 0)`).
/// Returns the touched rect (the region ∩ the mask's tight bounds), `None`
/// when that intersection is empty.
pub fn stamp_masked(
    dst: &mut Pixmap,
    scratch: &Pixmap,
    region: DirtyRect,
    origin: (i64, i64),
    mask: &Mask,
) -> Option<DirtyRect> {
    let region = transfer_target(dst, scratch, region, origin, mask)?;
    transfer(dst, scratch, region, origin, mask, Mode::Paint);
    Some(region)
}

/// The clipped ERASER's counterpart: destination-out by the scratch's
/// alpha — every masked pixel fades by `1 − scratch alpha`, the same math
/// tiny-skia's `BlendMode::DestinationOut` applies unmasked (the eraser
/// marks its footprint on the scratch with an opaque paint first, so the
/// scratch's alpha IS the erase strength, AA edges included).
pub fn erase_masked_by(
    dst: &mut Pixmap,
    scratch: &Pixmap,
    region: DirtyRect,
    origin: (i64, i64),
    mask: &Mask,
) -> Option<DirtyRect> {
    let region = transfer_target(dst, scratch, region, origin, mask)?;
    transfer(dst, scratch, region, origin, mask, Mode::Erase);
    Some(region)
}

/// The paint/erase variants of [`transfer`].
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Paint,
    Erase,
}

/// Intersects the canvas-space `region` with the mask's tight bounds, the
/// scratch's coverage (`origin + scratch size`) and `dst`'s bounds — the
/// rect the transfer loop may touch.
fn transfer_target(
    dst: &Pixmap,
    scratch: &Pixmap,
    region: DirtyRect,
    origin: (i64, i64),
    mask: &Mask,
) -> Option<DirtyRect> {
    let bounds = mask.tight_bounds()?;
    let (rx, rx1) = (region.x as i64, (region.x + region.w) as i64);
    let (ry, ry1) = (region.y as i64, (region.y + region.h) as i64);
    let (sx, sx1) = (origin.0, origin.0 + scratch.width() as i64);
    let (sy, sy1) = (origin.1, origin.1 + scratch.height() as i64);
    let (dx, dx1) = (0, dst.width() as i64);
    let (dy, dy1) = (0, dst.height() as i64);
    let x0 = rx.max(bounds.x).max(sx).max(dx);
    let y0 = ry.max(bounds.y).max(sy).max(dy);
    let x1 = rx1.min(bounds.x + bounds.w as i64).min(sx1).min(dx1);
    let y1 = ry1.min(bounds.y + bounds.h as i64).min(sy1).min(dy1);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(DirtyRect {
        x: x0 as u32,
        y: y0 as u32,
        w: (x1 - x0) as u32,
        h: (y1 - y0) as u32,
    })
}

/// The shared masked-transfer loop: for every pixel of the target rect the
/// mask selects, applies the scratch's pixel — [`Mode::Paint`] blends it
/// premultiplied src-over ([`blend_pixel`]), [`Mode::Erase`] scales the
/// destination by `1 − scratch alpha` (destination-out).
fn transfer(
    dst: &mut Pixmap,
    scratch: &Pixmap,
    region: DirtyRect,
    origin: (i64, i64),
    mask: &Mask,
    mode: Mode,
) {
    let (pw, ph) = (dst.width() as i64, dst.height() as i64);
    let scratch_stride = scratch.width() as i64;
    let scratch_pixels = scratch.pixels();
    let pixels = dst.pixels_mut();
    for py in region.y as i64..(region.y + region.h) as i64 {
        for px in region.x as i64..(region.x + region.w) as i64 {
            if !mask.get(px, py) {
                continue;
            }
            let s = scratch_pixels[((py - origin.1) * scratch_stride + (px - origin.0)) as usize];
            if s.alpha() == 0 {
                continue;
            }
            match mode {
                Mode::Paint => blend_pixel(
                    pixels,
                    pw,
                    ph,
                    px,
                    py,
                    [
                        s.red() as u32,
                        s.green() as u32,
                        s.blue() as u32,
                        s.alpha() as u32,
                    ],
                ),
                Mode::Erase => {
                    let idx = (py * pw + px) as usize;
                    let d = pixels[idx];
                    let ia = 255 - s.alpha() as u32;
                    let mix = |c: u8| ((c as u32 * ia + 127) / 255) as u8;
                    if let Some(out) = PremultipliedColorU8::from_rgba(
                        mix(d.red()),
                        mix(d.green()),
                        mix(d.blue()),
                        mix(d.alpha()),
                    ) {
                        pixels[idx] = out;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
