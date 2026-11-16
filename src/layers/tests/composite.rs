//! Composite half of the layers tests: the region src-over blending the
//! display buffer is fed by. The layer-stack tests live in the parent
//! `tests.rs`, the snapshot-codec tests in the sibling `snapshot.rs`.

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

fn layer(name: &str, pixmap: Pixmap) -> Layer {
    Layer {
        name: name.to_string(),
        visible: true,
        pixmap,
        tree: LayerTree::new(1, Vec::new()),
    }
}

#[test]
fn composite_blends_visible_layers_bottom_up() {
    // Bottom: opaque red. Top: 50%-alpha blue (premultiplied (0, 0, 128, 128)).
    let mut layers = vec![
        layer(
            "bottom",
            filled_pixmap(8, 8, Color::from_rgba8(255, 0, 0, 255)),
        ),
        layer(
            "top",
            filled_pixmap(8, 8, Color::from_rgba8(0, 0, 255, 128)),
        ),
    ];

    let mut dst = pixmap(8, 8);
    composite(&layers, &mut dst, 0, 0, 8, 8);
    let pixel = dst.pixel(4, 4).expect("in bounds").demultiply();
    // src-over: blue stays (128 premultiplied), red is dimmed to
    // 255 * (255-128) / 255 ≈ 127, alpha reaches 255.
    assert_eq!(
        (pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()),
        (127, 0, 128, 255)
    );

    // Hiding the top layer reveals the bottom unchanged.
    layers[1].visible = false;
    let mut dst = pixmap(8, 8);
    composite(&layers, &mut dst, 0, 0, 8, 8);
    let pixel = dst.pixel(4, 4).expect("in bounds").demultiply();
    assert_eq!((pixel.red(), pixel.green(), pixel.blue()), (255, 0, 0));

    // The composite is local: a region outside the drawn area stays empty.
    let mut dst = pixmap(8, 8);
    composite(&layers, &mut dst, 4, 4, 4, 4);
    assert!(dst.pixel(0, 0).is_some_and(|p| p.alpha() == 0));
}

#[test]
fn composite_fills_the_cleared_region_only() {
    let layer = layer(
        "single",
        filled_pixmap(16, 16, Color::from_rgba8(10, 20, 30, 255)),
    );
    let mut dst = pixmap(16, 16);
    dst.data_mut().fill(0x11);

    composite(std::slice::from_ref(&layer), &mut dst, 4, 4, 8, 8);

    let inside = dst.pixel(8, 8).expect("in bounds").demultiply();
    assert_eq!((inside.red(), inside.green(), inside.blue()), (10, 20, 30));
    let outside = dst.pixel(0, 0).expect("in bounds");
    assert_eq!(
        [
            outside.red(),
            outside.green(),
            outside.blue(),
            outside.alpha()
        ],
        [0x11; 4],
        "the untouched region keeps the buffer's prefill"
    );
}
