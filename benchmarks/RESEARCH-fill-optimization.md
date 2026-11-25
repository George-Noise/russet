# RESEARCH: fill and magic-wand optimization — options and measurements

Date: 2026-11-23. The owner's request in chat: fast in `--release`, slow in dev in
some cases; look at the open-source experience; **first describe the options and
take the measurements — do NOT land the optimization yet**. The file lives in the
repository so that after the algorithm change the new benchmark logs can be
compared against the old ones.

## Measurement methodology

Benchmark: `src/app/fill_bench.rs` (the cargo feature `fill-bench`, NOT compiled
by default). Running it requires the owner's explicit permission:

```
RUSSET_BENCH_FILL=1 RUSSET_BENCH_ALLOWED=1 cargo run --features fill-bench        # dev
RUSSET_BENCH_FILL=1 RUSSET_BENCH_ALLOWED=1 cargo run --release --features fill-bench  # release
```

Without `RUSSET_BENCH_ALLOWED=1` the process refuses to start (exit 2). The rule
for the agent: never run the bench without the owner's explicit instruction in
chat (AGENTS.md, the input section; PROGRESS.md, the 2026-11-23 entry).

What is measured: `flood_pixels` (the read — the core of BOTH the fill and the
wand, the same `pick_color` sampler the click uses) and the fill's write (the
src-over loop from `App::paint_flood_pixels`, copied 1:1). `full_click_mean =
flood_mean + write_mean`. All numbers are nanoseconds: min/median/mean/p95 over
the measured runs after the warm-ups. The canvases are deterministic (a fixed
xorshift), so logs from different machines and runs are comparable. Logs go into
`benchmarks/`, named `fill_bench_<dev|release>_<YYYYMMDDHHMMSS>.log` (the
date-time is digits only, second precision, UTC; the mode is in the name). The
log header carries the machine's specs (CPU, cores, RAM, OS kernel, rustc, git
HEAD).

The measurement machine for this file's table: 2 vCPU Intel Xeon, 4 GB RAM,
Linux 5.10.134 — the absolute numbers belong to this sandbox; the dev/release
ratios and the case-to-case ratios transfer.

## Measurements (mean full_click, one fill click)

| Case | pixels filled | dev | release | gap |
|---|---|---|---|---|
| solid_64x64_fill_all | 4 096 | 2.65 ms | 248 µs | ~10.7× |
| solid_800x600_fill_all | 480 000 | 376 ms | 50.1 ms | ~7.5× |
| solid_1920x1080_fill_all | 2 073 600 | 1.72 s | 238 ms | ~7.2× |
| noise_800x600 (4 px) | 4 | 28 µs | 18 µs | ~1.6× |
| stripes_800x600 (a 2×600 stripe) | 1 200 | 0.92 ms | 94 µs | ~9.8× |
| gradient_800x600 tol=6 | 1 200 | 0.79 ms | 82 µs | ~9.7× |
| blob_800x600 (a disk) | 101 780 | 73 ms | 8.4 ms | ~8.7× |
| hole_800x600 (the hole, the crash case) | 101 780 | 79 ms | 8.6 ms | ~9.2× |
| all_masked_800x600 (the flood covers everything, the write masks it all out) | 480 000 | 354 ms | 44.9 ms | ~7.9× |
| dot_800x600 (1 px) | 1 | 29 µs | 17 µs | ~1.6× |

The per-pixel flood-read cost: ~727 ns/px in dev, ~100 ns/px in release
(348 ms / 480 000). Source logs: `benchmarks/fill_bench_dev_20261123195804.log`,
`fill_bench_dev_20261123195853.log`, `fill_bench_release_20261123200459.log`.

What the numbers say: (1) the owner's pain is confirmed quantitatively — 376 ms
per 800×600 click and 1.72 s on FullHD in dev; (2) even release is unhurried for
an editor (50 ms on 800×600, 238 ms on FullHD) — an algorithmic improvement
helps BOTH modes; (3) small fills (~1 px–1 200 px) cost 20–950 µs even in dev —
the bottleneck is exactly the large areas; (4) the write is only ~7–8% of the
click (119 ms of 1.72 s on FullHD in dev) — the bulk of the time is the flood
read.

## Diagnosis of the current code (`src/selection.rs::flood_pixels` + `src/tools.rs::pick_color`)

1. **A per-pixel BFS**: a `VecDeque<(i64,i64)>` of individual pixels; every
   pixel pays a queue push/pop plus 4 neighbor probes. A span approach pushes
   ONE stack element per horizontal run instead of one per pixel.
2. **An expensive sampler**: on EVERY probe `pick_color` calls `pixel(x, y)`
   (a bounds check + Option) and `demultiply()` (a division per channel) — and
   the comparator then unpacks the 4 channels and does 4 i16 subtractions + abs.
   A raw comparison of the packed u32 against a precomputed tolerance fork
   removes all of that.
3. **Non-matching pixels are not marked**: a boundary pixel (its color does not
   match) never gets `visited`, so it is re-sampled up to 4 times (once from
   each queued neighbor). On noisy/thin-stripe art that is up to 4× the needed
   sampler calls.
4. **Allocations on every click**: `vec![false; w*h]` (FullHD = 2 MB) + the
   output `Vec<(i64,i64)>` (FullHD ≈ 32 MB when filling the whole canvas) —
   neither is reused.
5. **`out.sort()` + `dedup()` over the whole list**: the full pixel list is
   sorted O(n log n) although the flood visits every pixel exactly once — the
   sort exists only because the BFS traversal order is chaotic. A span
   algorithm emits the pixels ordered by scanline runs on its own.
6. **i64 pairs in the list**: 16 bytes per pixel; spans are 2-4 numbers per run.

## Optimization options (in descending expected effect)

### 1. A span/scanline flood instead of the per-pixel BFS

The classic: one stack element = a horizontal run `(x1, x2, y, dy)`; filling a
row is one tight loop over the row (cache-friendly), the
`spanAbove`/`spanBelow` flags push exactly one element per new run above/below.
The result matches the current 4-way semantics.

- Sources: lodev.org (floodFillScanlineStack; its 2004 benchmark: 239 ms →
  34 ms, ≈7×); Wikipedia's "Flood fill" (the span family, "2–8× faster than the
  pixel-wise recursive one", "cache- and bitplane-friendly"); imageproc 0.27
  (Rust) implements the fill exactly by Wikipedia's span scheme; Krita uses a
  "more optimized scanline fill".
- Effect: 2–8× on the read (per the sources) + `out.sort()`/`dedup()` disappear
  (the output is already ordered by rows) + 100–1000× fewer elements in the
  queue's stack.
- The trade-off (from the krita-artists thread): a scanline fill "easily leaks
  further along a 1 px line" and does not compose with MyPaint's gap-closing
  (which kept the classic flood). Immaterial for russet: we have no
  gap-closing; the 4-way result is preserved.

### 2. A cheap comparator (raw packed color + tolerance forks)

Compute the seed color and the tolerance bounds ONCE (4 lo/hi pairs, per
channel, in premultiplied bytes), then compare the pixel's raw `u32` from the
`pm.data()` slice with one bounds check per row. Removes `demultiply()` (the
per-channel division) and the Color32 unpacking on every probe; in dev it also
removes the non-inlined closure with an Option. Expect 2–3× on the read in both
modes. Mind the semantics: compare in the same bytes the demultiply produces
now (RGBA unmultiplied) — i.e. either keep the key demultiplied, or convert the
forks into premultiplied space and compare raw bytes (cheaper, but needs a
careful equivalence check on transparent pixels).

### 3. Mark non-matching pixels as "checked"

A second visit map (or widening the current one to 2 states) so a boundary
pixel is sampled exactly once. Cheap to do even before the span rewrite; on
noisy canvases it saves up to half the sampler calls. On the span version it is
partially absorbed by the algorithm itself (a row is walked in one loop, the
repeated probes no longer grow 4×).

### 4. A comparison-result cache keyed by unique colors

The idea from Krita (the color difference is computed in Lab with the result
cached for new colors): a `HashMap<u32, bool>` color → matches/does not. On
flat areas with few unique colors the comparator almost never runs. russet's
tolerance is per-channel, so the key can be the packed color without
demultiplication. An extra gain on top of option 2; on gradients the cache is
useless (every color is unique) — combine with 2/3, do not replace them.

### 5. Buffer reuse (scratch in App)

Keep `visited`, the span stack and the output buffer in `App` and clear (not
reallocate) them per click. Removes 2 MB+ of allocations per FullHD click; in
dev the allocation itself is noticeable.

### 6. Write straight from the spans, no intermediate `Vec<(i64,i64)>`

For the fill: write the pixels (or the whole src-over run per row) right while
walking the spans — the megabyte output Vec, the sort and the dedup disappear;
the bbox is computed from the spans. For the wand: the spans map 1:1 into the
`Mask` (per row, mark bits with one run) — `build_mask` also stops walking the
per-pixel list. Expect another −20–35% of the full click on top of options 1–3,
plus a drop in peak memory.

### 7. The dev profile: `[profile.dev] opt-level = 1`

A cargo-level knob, no algorithm change: compile with basic optimization while
keeping `debug_assertions`/overflow-checks. By the measured ratios (~7–10×)
this brings dev an order of magnitude closer to release in the hot loop; the
price is a somewhat longer dev-build compile.

**REJECTED by the owner 2026-11-23 (chat):** we will NOT lighten dev — the
slowness in dev is a DELIBERATE early symptom sensor; it is exactly how the
owner noticed the fill slowdown. The option stays in the list for the record
only; do not propose it again.

### Rejected / deferred

- **Multithreaded fill (rayon/tiles)**: the write is only ~7% of the click; a
  new dependency; Krita chose not to merge a tiled multithreaded fill
  (problems) — the krita-artists thread. Deferred until really needed.
- **SIMD**: comparing 8 packed u32s at once is possible, but the gain is capped
  by memory/the traversal's branching; worth doing only AFTER the span rewrite
  (option 1), otherwise we optimize the wrong place.
- **8-way neighborhood**: leaks through 1 px diagonal lines (lodev) — it
  changes the fill's semantics, not an optimization option.

## Recommended order (do NOT land without the owner's explicit permission)

1. The targeted read/write refactor: options 1 + 2 + 3 (+6 for the
   write/mask) — expected to be an order of magnitude on large areas in BOTH
   modes, with the 4-way semantics and the pixel result preserved (checked by
   the existing fill/wand tests + a benchmark re-run).
   dev is deliberately NOT lightened — the owner's 2026-11-23 decision (option
   7 rejected, see above).
2. By the remainder: 5 (scratch buffers) and 4 (the comparison cache) — if the
   profiling after step 1 shows they still matter.

Post-landing control: run the benchmark in dev and release, put the new logs
next to the old ones in `benchmarks/`, compare case-by-case (especially
solid_1920x1080_fill_all, all_masked_800x600 and stripes — the span version's
anti-cases).

## Sources (opened 2026-11-23)

- https://en.wikipedia.org/wiki/Flood_fill — the span family, 2–8×,
  cache-friendly, the algorithm's 1990 history.
- https://lodev.org/cgtutor/floodfill.html — the floodFillScanlineStack
  implementation, the spanAbove/spanBelow flags, the 239 ms → 34 ms benchmark,
  a flat Uint32 buffer.
- https://docs.rs/imageproc/latest/imageproc/drawing/fn.flood_fill.html — Rust
  imageproc 0.27: the fill by Wikipedia's span scheme, strict color equality.
- https://krita-artists.org/t/close-gap-in-fill-tool/32043/170 — Krita scanline
  fill (Deif_Lou): a horizontal bias, the incompatibility with MyPaint's
  gap-closing, the Lab color-difference cache, the rejected tiled
  multithreading.


## RESULT 2026-11-23: options 1+2+3+6 landed (the same day, a separate round)

Landed: the span core `flood_spans` (one stack entry per horizontal run, runs
grow past the parent span's bounds — nothing is orphaned), EVERY probed pixel
gets marked (non-matching ones too — exactly one sample per pixel), the
comparator is per-channel lo/hi forks over raw bytes (`FloodProbe`), the app
path reads the layer through `tools::sample_bytes_of` — EXACTLY the bytes the
old pick_color chain fed the comparator (fast paths for opaque and alpha=0,
translucent colors pay the former rounding pipeline — the equality was
brute-forced by a dense sweep in tools/tests.rs). The fill writes from the
spans (`App::paint_flood_spans`, the same src-over, the zero-bbox crash guard
kept); the wand keeps the `Region::Wand` contract, its pixel list is assembled
by a column-wise radix pass in O(n) — the old `out.sort()` over millions of
tuples is gone. Option 7 (the dev profile) was not touched — the owner's
decision (see above).

The BEFORE measurements (this file, top) against AFTER (the logs
`fill_bench_dev_20261123211220.log` / `fill_bench_release_20261123211220.log`,
the same sandbox, the same canvases; `core=` in the log header marks the
rewritten algorithm):

| Case | dev before → after | release before → after |
|---|---|---|
| solid_64x64_fill_all | 2.65 ms → 0.81 ms | 248 µs → 27.7 µs |
| solid_800x600_fill_all | 376 ms → 84 ms | 50.1 ms → 3.08 ms |
| solid_1920x1080_fill_all | 1.72 s → 0.34 s | 238 ms → 13.1 ms |
| stripes_800x600 | 0.92 ms → 0.46 ms | 94 µs → 53.7 µs |
| gradient_800x600 tol6 | 0.79 ms → 0.25 ms | 82 µs → 34 µs |
| blob_800x600 | 73 ms → 16.9 ms | 8.4 ms → 0.68 ms |
| hole_800x600 (the crash case) | 79 ms → 16.9 ms | 8.6 ms → 0.68 ms |
| all_masked_800x600 | 354 ms → 55.2 ms | 44.9 ms → 2.17 ms |
| noise_800x600 (4 px) | 28 µs → 23.6 µs | 18 µs → 20.7 µs |
| dot_800x600 (1 px) | 29 µs → 22 µs | 17 µs → 21.8 µs |

Totals: release 12–21× on large areas (the upper part of the "2–8× +
comparator" forecast), dev 3–6×; the small fills did not regress.
Correctness: 359 tests green, among them the new oracle (the span core against
the naive BFS kept verbatim in the tests, over 30 random canvases × 4 seeds ×
4 tolerances with a palette of opaque/translucent/transparent colors), the
transparent hole, alpha entering the tolerance, the span shape, the dense
sample_bytes_of sweep (~560k combinations against the real pick_color chain).
