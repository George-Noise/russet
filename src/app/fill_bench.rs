//! TEMPORARY fill/wand flood performance benchmark (owner request
//! 2026-11-23, chat): measures in NANOSECONDS the flood READ
//! ([`crate::selection::flood_pixels`] — the core the Fill tool and the
//! magic wand share) and the fill WRITE (the src-over pixel loop of
//! `App::paint_flood_pixels`, mirrored 1:1 below) on a set of
//! deterministic canvases, and writes a log file into `benchmarks/` so a
//! future algorithm change can be re-verified against the old numbers.
//!
//! This is NOT part of the normal tests and the gates NEVER run it:
//!
//! - the whole module compiles only under the cargo feature `fill-bench`
//!   (the default build compiles NONE of it — the same pattern as
//!   `mcp` / `diagnostics`);
//! - at runtime it needs BOTH `RUSSET_BENCH_FILL=1` (the trigger) AND
//!   `RUSSET_BENCH_ALLOWED=1` (the OWNER'S EXPLICIT PERMISSION). The agent
//!   must not set `RUSSET_BENCH_ALLOWED` on its own: without the owner's
//!   explicit chat instruction the benchmark refuses to run — the rule is
//!   recorded in AGENTS.md (the input clause) and PROGRESS.md (the
//!   2026-11-23 entry).
//!
//! The log name is `fill_bench_<mode>_<YYYYMMDDHHMMSS>.log`: the mode
//! (`dev` when debug_assertions is on, `release` otherwise) sits IN the
//! name so the owner's release runs (their machines) are distinguishable
//! from the agent's dev runs (the sandbox), and the date+time part is
//! digits-only down to the second (UTC). The header carries the sandbox
//! specs (CPU model, cores, RAM, kernel, rustc, git HEAD) so logs from
//! different machines stay comparable.
//!
//! Run:
//!   RUSSET_BENCH_FILL=1 RUSSET_BENCH_ALLOWED=1 cargo run --features fill-bench
//! (release for the owner's own machines: add `--release`).

use crate::egui::{Color32, pos2};
use crate::selection::{Region, flood_pixels};
use crate::tools::pick_color;
use std::fmt::Write as _;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use tiny_skia::{Pixmap, PremultipliedColorU8};

/// Called from `main` before the window opens. Returns `true` when the
/// benchmark ran (the process must then exit without opening a GUI);
/// `false` means "not requested" — the normal launch continues.
pub fn maybe_run() -> bool {
    if std::env::var("RUSSET_BENCH_FILL").as_deref() != Ok("1") {
        return false;
    }
    if std::env::var("RUSSET_BENCH_ALLOWED").as_deref() != Ok("1") {
        eprintln!(
            "The fill/wand flood benchmark REFUSED: RUSSET_BENCH_ALLOWED=1 is \
             missing.\nThis benchmark runs only by the owner's explicit \
             permission (the owner's own machine, or an explicit chat \
             instruction to the agent); the rule lives in AGENTS.md (the \
             input clause) and PROGRESS.md (the 2026-11-23 entry)."
        );
        std::process::exit(2);
    }
    run();
    true
}

// ---------------------------------------------------------------------------
// The cases
// ---------------------------------------------------------------------------

struct Case {
    name: &'static str,
    w: u32,
    h: u32,
    seed: (i64, i64),
    tolerance: u8,
    /// The selection rectangle (a, b) in canvas pixels, `None` = no
    /// selection (the fill fires anywhere).
    sel: Option<((i64, i64), (i64, i64))>,
    warmup: usize,
    measured: usize,
    note: &'static str,
}

fn cases() -> Vec<Case> {
    // The iteration counts tier by canvas area: the big canvases get fewer
    // measured runs so the whole suite stays inside a couple of minutes in
    // a dev build (the tiering is printed into the log).
    let tier = |w: u32, h: u32| -> (usize, usize) {
        match w as u64 * h as u64 {
            a if a <= 4096 => (3, 15),
            a if a <= 500_000 => (2, 10),
            _ => (1, 5),
        }
    };
    let (w1, m1) = tier(64, 64);
    let (w2, m2) = tier(800, 600);
    let (w3, m3) = tier(1920, 1080);
    vec![
        Case {
            name: "solid_64x64_fill_all",
            w: 64,
            h: 64,
            seed: (32, 32),
            tolerance: 0,
            sel: None,
            warmup: w1,
            measured: m1,
            note: "a tiny solid canvas: the flood covers everything",
        },
        Case {
            name: "solid_800x600_fill_all",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 0,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "a medium solid canvas: the flood covers everything",
        },
        Case {
            name: "solid_1920x1080_fill_all",
            w: 1920,
            h: 1080,
            seed: (960, 540),
            tolerance: 0,
            sel: None,
            warmup: w3,
            measured: m3,
            note: "a full-HD solid canvas: the volume worst case (every pixel queued)",
        },
        Case {
            name: "noise_800x600_seed_center",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 0,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "a high-frequency 8-color noise: the flood is small, but every \
                   unmatched border pixel is re-sampled up to 4 times (the \
                   per-pixel BFS tax)",
        },
        Case {
            name: "stripes_800x600_seed_center",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 0,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "2px vertical stripes: many narrow runs — the shape a span \
                   algorithm would eat in one pass per stripe",
        },
        Case {
            name: "gradient_800x600_tol6_seed_mid",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 6,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "a smooth gradient with tolerance 6: long diagonal bands, like \
                   anti-aliased artwork",
        },
        Case {
            name: "blob_800x600_fill_inside",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 0,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "a black disc on white (the drawn-blob shape): the fill paints \
                   the disc",
        },
        Case {
            name: "hole_800x600_fill_inside",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 0,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "a transparent hole bounded by a white ring (the cut shape): the \
                   flood runs through transparent pixels — the crash scenario's \
                   shape",
        },
        Case {
            name: "all_masked_800x600_outside_sel",
            w: 800,
            h: 600,
            seed: (780, 560),
            tolerance: 0,
            sel: Some(((300, 280), (330, 310))),
            warmup: w2,
            measured: m2,
            note: "the flood covers the whole canvas while the selection masks it \
                   all out — the V78 crash path's cost (the write skips every \
                   pixel)",
        },
        Case {
            name: "dot_800x600_one_pixel",
            w: 800,
            h: 600,
            seed: (400, 300),
            tolerance: 0,
            sel: None,
            warmup: w2,
            measured: m2,
            note: "a single isolated pixel: the smallest possible fill",
        },
    ]
}

// ---------------------------------------------------------------------------
// Deterministic canvases (a fixed xorshift state — the same pixels on every
// machine and every run, so the logs are comparable)
// ---------------------------------------------------------------------------

fn xorshift(state: &mut u64) -> u64 {
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    *state
}

fn opaque(r: u8, g: u8, b: u8) -> PremultipliedColorU8 {
    PremultipliedColorU8::from_rgba(r, g, b, 255).expect("opaque color premultiplies to itself")
}

fn blank(w: u32, h: u32, fill: PremultipliedColorU8) -> Pixmap {
    let mut pm = Pixmap::new(w, h).expect("a valid canvas size");
    for px in pm.pixels_mut() {
        *px = fill;
    }
    pm
}

fn canvas_solid(w: u32, h: u32) -> Pixmap {
    blank(w, h, opaque(255, 255, 255))
}

fn canvas_noise(w: u32, h: u32) -> Pixmap {
    let palette = [
        opaque(24, 24, 24),
        opaque(96, 40, 40),
        opaque(40, 96, 40),
        opaque(40, 40, 96),
        opaque(160, 160, 32),
        opaque(32, 160, 160),
        opaque(160, 32, 160),
        opaque(230, 230, 230),
    ];
    let mut state = 0x5eed_2026_1123_u64;
    let mut pm = Pixmap::new(w, h).expect("a valid canvas size");
    for px in pm.pixels_mut() {
        *px = palette[(xorshift(&mut state) >> 33) as usize % palette.len()];
    }
    pm
}

fn canvas_stripes(w: u32, h: u32) -> Pixmap {
    let (black, white) = (opaque(20, 20, 20), opaque(255, 255, 255));
    let mut pm = Pixmap::new(w, h).expect("a valid canvas size");
    for y in 0..h as usize {
        for x in 0..w as usize {
            pm.pixels_mut()[y * w as usize + x] = if x % 4 < 2 { black } else { white };
        }
    }
    pm
}

fn canvas_gradient(w: u32, h: u32) -> Pixmap {
    let mut pm = Pixmap::new(w, h).expect("a valid canvas size");
    for y in 0..h as usize {
        for x in 0..w as usize {
            pm.pixels_mut()[y * w as usize + x] = opaque(
                ((x as u32 * 255) / (w - 1).max(1)) as u8,
                ((y as u32 * 255) / (h - 1).max(1)) as u8,
                128,
            );
        }
    }
    pm
}

fn disc(w: u32, h: u32, inside: PremultipliedColorU8, outside: PremultipliedColorU8) -> Pixmap {
    let mut pm = blank(w, h, outside);
    let (cx, cy) = (w as f64 / 2.0, h as f64 / 2.0);
    let r = (w.min(h) as f64) * 0.3;
    for y in 0..h as usize {
        for x in 0..w as usize {
            let dx = x as f64 + 0.5 - cx;
            let dy = y as f64 + 0.5 - cy;
            if dx * dx + dy * dy <= r * r {
                pm.pixels_mut()[y * w as usize + x] = inside;
            }
        }
    }
    pm
}

fn canvas_blob(w: u32, h: u32) -> Pixmap {
    disc(w, h, opaque(20, 20, 20), opaque(255, 255, 255))
}

fn canvas_hole(w: u32, h: u32) -> Pixmap {
    // The cut shape: a fully transparent interior, a solid ring around it.
    disc(w, h, opaque(0, 0, 0), opaque(255, 255, 255))
}

fn canvas_dot(w: u32, h: u32, seed: (i64, i64)) -> Pixmap {
    let mut pm = canvas_solid(w, h);
    pm.pixels_mut()[seed.1 as usize * w as usize + seed.0 as usize] = opaque(20, 20, 20);
    pm
}

fn canvas_for(case: &Case) -> Pixmap {
    match case.name {
        "solid_64x64_fill_all" | "solid_800x600_fill_all" | "solid_1920x1080_fill_all" => {
            canvas_solid(case.w, case.h)
        }
        "noise_800x600_seed_center" => canvas_noise(case.w, case.h),
        "stripes_800x600_seed_center" => canvas_stripes(case.w, case.h),
        "gradient_800x600_tol6_seed_mid" => canvas_gradient(case.w, case.h),
        "blob_800x600_fill_inside" => canvas_blob(case.w, case.h),
        "hole_800x600_fill_inside" => canvas_hole(case.w, case.h),
        "all_masked_800x600_outside_sel" => canvas_solid(case.w, case.h),
        "dot_800x600_one_pixel" => canvas_dot(case.w, case.h, case.seed),
        other => panic!("a case without a canvas builder: {other}"),
    }
}

// ---------------------------------------------------------------------------
// The write side (a 1:1 mirror of App::paint_flood_pixels' pixel loop)
// ---------------------------------------------------------------------------

/// The same src-over write the fill click does (the fill color, the
/// selection skip, the bounds skip, the written bbox). The layer/history
/// bookkeeping around it is app state, not algorithm — the bench measures
/// the pixel work.
fn paint_flood(
    pixels: &[(i64, i64)],
    color: Color32,
    clip: Option<&crate::selection::Mask>,
    pm: &mut Pixmap,
) -> (i64, i64, i64, i64) {
    let (mut min_x, mut min_y) = (i64::MAX, i64::MAX);
    let (mut max_x, mut max_y) = (i64::MIN, i64::MIN);
    let (r, g, b, a) = (color.r(), color.g(), color.b(), color.a());
    let (stride, height) = (pm.width() as usize, pm.height() as i64);
    let dst = pm.pixels_mut();
    let ia = 255 - a as u32;
    let mix = |s: u32, d: u8| (s + (d as u32 * ia + 127) / 255) as u8;
    for (x, y) in pixels {
        if let Some(mask) = clip
            && !mask.get(*x, *y)
        {
            continue;
        }
        if *x < 0 || *y < 0 || *x >= stride as i64 || *y >= height {
            continue;
        }
        let d = dst[*y as usize * stride + *x as usize];
        let out = PremultipliedColorU8::from_rgba(
            mix(r as u32, d.red()),
            mix(g as u32, d.green()),
            mix(b as u32, d.blue()),
            mix(a as u32, d.alpha()),
        )
        .unwrap_or(d);
        dst[*y as usize * stride + *x as usize] = out;
        min_x = min_x.min(*x);
        min_y = min_y.min(*y);
        max_x = max_x.max(*x);
        max_y = max_y.max(*y);
    }
    (
        min_x,
        min_y,
        max_x.saturating_sub(min_x) + 1,
        max_y.saturating_sub(min_y) + 1,
    )
}

// ---------------------------------------------------------------------------
// The harness
// ---------------------------------------------------------------------------

/// (min, median, mean, p95) over the measured runs, in nanoseconds.
fn stats(ns: &mut [u128]) -> (u128, u128, u128, u128) {
    ns.sort_unstable();
    let n = ns.len();
    let median = ns[n / 2];
    let mean = ns.iter().sum::<u128>() / n as u128;
    // p95: the ceil(0.95 * n)-th value of the sorted list (15 samples → the
    // max, 10 → the max, 5 → the max).
    let p95 = ns[(n * 95).div_ceil(100).max(1) - 1];
    (ns[0], median, mean, p95)
}

struct CaseResult {
    flood: (u128, u128, u128, u128),
    write: (u128, u128, u128, u128),
    pixels: usize,
    bbox: (i64, i64, i64, i64),
}

fn measure(case: &Case, color: Color32) -> CaseResult {
    let pm = canvas_for(case);
    // The mask via the REAL production path: a Selection with one ADD rect
    // op (the same `replace` + `build_mask` the tool clicks drive).
    let mask = case.sel.map(|((ax, ay), (bx, by))| {
        let mut sel = crate::selection::Selection::default();
        sel.replace(
            Some(Region::Rect {
                a: pos2(ax as f32, ay as f32),
                b: pos2(bx as f32, by as f32),
            }),
            case.w,
            case.h,
        );
        sel.build_mask(case.w, case.h)
            .expect("the bench selection rectangle is inside the canvas")
    });
    let mut flood_ns = Vec::with_capacity(case.measured);
    let mut write_ns = Vec::with_capacity(case.measured);
    let mut pixels = 0usize;
    let mut bbox = (0i64, 0i64, 0i64, 0i64);
    for i in 0..case.warmup + case.measured {
        // The READ: the flood core with the SAME sampler the click uses.
        let t = Instant::now();
        let flood = flood_pixels(case.w, case.h, case.seed, case.tolerance, |px, py| {
            pick_color(&pm, px as u32, py as u32)
        });
        flood_ns.push(t.elapsed().as_nanos());
        // The WRITE: on a per-run copy (the clone is untimed — the bench
        // measures the pixel work, not the buffer copy).
        let mut pm_w = pm.clone();
        let t = Instant::now();
        let bb = paint_flood(&flood, color, mask.as_ref(), &mut pm_w);
        write_ns.push(t.elapsed().as_nanos());
        if i >= case.warmup {
            pixels = flood.len();
            bbox = bb;
        }
    }
    CaseResult {
        flood: stats(&mut flood_ns),
        write: stats(&mut write_ns),
        pixels,
        bbox,
    }
}

// ---------------------------------------------------------------------------
// The log
// ---------------------------------------------------------------------------

/// The UTC civil date from unix seconds (Howard Hinnant's civil_from_days).
fn utc_from_epoch(secs: u64) -> (i64, u32, u32, u32, u32, u32) {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    (
        y,
        m,
        d,
        (rem / 3600) as u32,
        ((rem % 3600) / 60) as u32,
        (rem % 60) as u32,
    )
}

/// The first `key: value` line's trimmed value from a /proc-style file.
fn proc_field(path: &str, key: &str) -> Option<String> {
    std::fs::read_to_string(path).ok()?.lines().find_map(|l| {
        if l.starts_with(key) {
            l.split(':').nth(1).map(|v| v.trim().to_string())
        } else {
            None
        }
    })
}

/// A single-value file (e.g. /proc/sys/kernel/osrelease), trimmed.
fn read_trimmed(path: &str) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|s| s.trim().to_string())
}

fn command_output(prog: &str, args: &[&str]) -> String {
    let try_run = |path: &str| {
        std::process::Command::new(path)
            .args(args)
            .output()
            .ok()
            .filter(|o| o.status.success())
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
    };
    // The plain PATH lookup first, then the rustup default location — the
    // bench may run from an environment whose PATH lacks ~/.cargo/bin.
    try_run(prog)
        .or_else(|| {
            try_run(&format!(
                "{}/.cargo/bin/{prog}",
                std::env::var("HOME").unwrap_or_default()
            ))
        })
        .unwrap_or_else(|| "unknown".to_string())
}

fn run() {
    let started = Instant::now();
    let mode = if cfg!(debug_assertions) {
        "dev"
    } else {
        "release"
    };
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, mo, d, hh, mm, ss) = utc_from_epoch(secs);
    let stamp = format!("{y:04}{mo:02}{d:02}{hh:02}{mm:02}{ss:02}");
    std::fs::create_dir_all("benchmarks").expect("benchmarks/ is creatable");
    let path = format!("benchmarks/fill_bench_{mode}_{stamp}.log");

    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);
    let mut log = String::new();
    let _ = writeln!(log, "russet fill/wand flood benchmark");
    let _ = writeln!(
        log,
        "generated_utc={y:04}-{mo:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}Z epoch_secs={secs}"
    );
    let _ = writeln!(
        log,
        "mode={mode} (debug_assertions={}; the dev build adds overflow checks and no inlining)",
        cfg!(debug_assertions)
    );
    let _ = writeln!(
        log,
        "os={} family={} kernel={}",
        std::env::consts::OS,
        std::env::consts::FAMILY,
        read_trimmed("/proc/sys/kernel/osrelease")
            .or_else(|| Some(command_output("uname", &["-r"])))
            .unwrap_or_else(|| "unknown".into())
    );
    let _ = writeln!(
        log,
        "cpu_model={}",
        proc_field("/proc/cpuinfo", "model name").unwrap_or_else(|| "unknown".into())
    );
    let _ = writeln!(
        log,
        "cpu_cores_available={cores} mem_total_kb={}",
        proc_field("/proc/meminfo", "MemTotal").unwrap_or_else(|| "unknown".into())
    );
    let _ = writeln!(
        log,
        "rustc={} git_head={}",
        command_output("rustc", &["--version"]),
        command_output("git", &["rev-parse", "HEAD"])
    );
    let _ = writeln!(
        log,
        "color_pipeline=premultiplied RGBA8 (tiny-skia); pick_color demultiplies per probe; \
         the write is the src-over mix loop of App::paint_flood_pixels"
    );
    let _ = writeln!(
        log,
        "timer=Instant.elapsed().as_nanos(); per-case: warmup runs discarded, then \
         min/median/mean/p95 over the measured runs; full_click_mean=flood_mean+write_mean"
    );
    writeln!(log).expect("the in-memory log never fails to write");

    let fill_color = Color32::from_rgb(20, 90, 200);
    for case in cases() {
        let r = measure(&case, fill_color);
        let full_mean = r.flood.2 + r.write.2;
        let _ = writeln!(
            log,
            "{} | {}x{} | seed=({},{}) | tol={} | sel={} | {}/{} | flood {} / {} / {} / {} | write {} / {} / {} / {} | full_click_mean={} | {} px | bbox=({},{},{},{}) | {}",
            case.name,
            case.w,
            case.h,
            case.seed.0,
            case.seed.1,
            case.tolerance,
            case.sel.map_or("none", |_| "rect"),
            case.warmup,
            case.measured,
            r.flood.0,
            r.flood.1,
            r.flood.2,
            r.flood.3,
            r.write.0,
            r.write.1,
            r.write.2,
            r.write.3,
            full_mean,
            r.pixels,
            r.bbox.0,
            r.bbox.1,
            r.bbox.2,
            r.bbox.3,
            case.note,
        );
    }
    let _ = writeln!(log, "wall_time_secs={:.1}", started.elapsed().as_secs_f32());

    std::fs::write(&path, &log).expect("the benchmark log is writable");
    println!("FILL BENCH ({mode}): the log is written to {path}");
    println!("{log}");
}
