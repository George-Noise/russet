//! Snapshot half of the layers tests: the PNG snapshot codec roundtrips
//! and the canvas-fit policy. The layer-stack tests live in the parent
//! `tests.rs`, the region-blending tests in the sibling `composite.rs`.

use super::*;

use tiny_skia::Color;

fn pixmap(width: u32, height: u32) -> Pixmap {
    Pixmap::new(width, height).expect("test pixmap")
}

fn filled_pixmap(width: u32, height: u32, color: Color) -> Pixmap {
    let mut pixmap = pixmap(width, height);
    pixmap.fill(color);
    pixmap
}

#[test]
fn snapshot_roundtrip_is_byte_exact() {
    let mut pixmap = pixmap(61, 37);
    // Every channel combination: opaque, semi and zero alpha.
    for y in 0..37u32 {
        for x in 0..61u32 {
            let alpha = ((x * 4 + y) % 256) as u8;
            let pixel =
                tiny_skia::ColorU8::from_rgba(x as u8, y as u8, (x + y) as u8, alpha).premultiply();
            pixmap.pixels_mut()[(y * 61 + x) as usize] = pixel;
        }
    }

    let png = encode_snapshot(&pixmap);

    let restored = decode_snapshot(&png).expect("valid snapshot");
    assert_eq!(restored.width(), 61);
    assert_eq!(restored.height(), 37);
    assert_eq!(
        restored.data(),
        pixmap.data(),
        "lossless premultiplied roundtrip"
    );
}

#[test]
fn snapshot_encoding_stays_far_below_the_raw_size_for_flat_art() {
    let pixmap = filled_pixmap(400, 300, Color::from_rgba8(200, 120, 40, 255));

    let png = encode_snapshot(&pixmap);

    let raw = 400 * 300 * 4;
    assert!(
        png.len() < raw / 8,
        "a flat canvas must compress hard: {} vs {}",
        png.len(),
        raw
    );
}

#[test]
fn decode_snapshot_reports_corrupt_bytes() {
    assert!(decode_snapshot(&[]).is_err());
    assert!(decode_snapshot(b"not a png at all").is_err());
}

#[test]
fn fit_to_canvas_crops_larger_snapshots_and_pads_smaller_ones() {
    let big = filled_pixmap(50, 40, Color::from_rgba8(255, 0, 0, 255));
    let small = filled_pixmap(20, 10, Color::from_rgba8(0, 255, 0, 255));

    let cropped = fit_to_canvas(big, 30, 25);
    assert_eq!(cropped.width(), 30);
    assert_eq!(cropped.height(), 25);
    let inside = cropped.pixel(29, 24).expect("in bounds").demultiply();
    assert_eq!((inside.red(), inside.green(), inside.blue()), (255, 0, 0));

    let padded = fit_to_canvas(small, 40, 30);
    assert_eq!(padded.width(), 40);
    assert_eq!(padded.height(), 30);
    let content = padded.pixel(5, 5).expect("in bounds").demultiply();
    assert_eq!(
        (content.red(), content.green(), content.blue()),
        (0, 255, 0)
    );
    let outside = padded.pixel(30, 20).expect("in bounds");
    assert_eq!(outside.alpha(), 0, "the padding stays transparent");
}
