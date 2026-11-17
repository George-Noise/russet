//! Resize half of the layers tests (owner request 2026-11-17): the anchor
//! geometry, the crop/pad offsets, the resampling and the whole-stack
//! `resize_all` commit (buffers + per-layer `resize` nodes). The codec
//! tests live in the sibling `snapshot.rs`, the blending in `composite.rs`.

use super::*;

use tiny_skia::Color;

fn filled_pixmap(width: u32, height: u32, color: Color) -> Pixmap {
    let mut pixmap = Pixmap::new(width, height).expect("test pixmap");
    pixmap.fill(color);
    pixmap
}

fn canvas_info(width: u32, height: u32, anchor: ResizeAnchor) -> ResizeInfo {
    ResizeInfo {
        mode: ResizeMode::Canvas,
        from_w: 0,
        from_h: 0,
        to_w: width,
        to_h: height,
        anchor: Some(anchor),
        filter: None,
    }
}

#[test]
fn anchor_offsets_pin_the_chosen_side_and_center_floors() {
    // Grow 10→14 wide, 10→15 tall: the delta is +4 / +5.
    assert_eq!(anchor_offset(ResizeAnchor::TopLeft, 10, 10, 14, 15), (0, 0));
    assert_eq!(
        anchor_offset(ResizeAnchor::TopRight, 10, 10, 14, 15),
        (4, 0)
    );
    assert_eq!(
        anchor_offset(ResizeAnchor::BottomLeft, 10, 10, 14, 15),
        (0, 5)
    );
    assert_eq!(
        anchor_offset(ResizeAnchor::BottomRight, 10, 10, 14, 15),
        (4, 5)
    );
    // Center: floor of +4/+5 — the extra odd pixel goes right/bottom.
    assert_eq!(anchor_offset(ResizeAnchor::Center, 10, 10, 14, 15), (2, 2));

    // Shrink 10→7 wide, 10→8 tall: the delta is −3 / −2.
    assert_eq!(anchor_offset(ResizeAnchor::TopLeft, 10, 10, 7, 8), (0, 0));
    assert_eq!(anchor_offset(ResizeAnchor::TopRight, 10, 10, 7, 8), (-3, 0));
    assert_eq!(
        anchor_offset(ResizeAnchor::BottomRight, 10, 10, 7, 8),
        (-3, -2)
    );
    // Center crop of an odd amount takes the extra pixel from the LEFT.
    assert_eq!(anchor_offset(ResizeAnchor::Center, 10, 10, 7, 8), (-2, -1));
}

#[test]
fn fit_copy_at_shifts_the_content_by_the_offset_and_pads_transparent() {
    // A 2×2 opaque red square into a 4×4 buffer at offset (1, 1).
    let src = filled_pixmap(2, 2, Color::from_rgba8(255, 0, 0, 255));

    let out = fit_copy_at(&src, 4, 4, 1, 1);
    assert_eq!(out.width(), 4);
    assert_eq!(out.height(), 4);
    let inside = out.pixel(1, 1).expect("in bounds").demultiply();
    assert_eq!((inside.red(), inside.alpha()), (255, 255));
    let padding = out.pixel(0, 0).expect("in bounds");
    assert_eq!(padding.alpha(), 0, "the padding stays transparent");

    // A NEGATIVE offset crops from the left/top: offset (−1, −1) shows the
    // source pixel (1, 1) at the destination origin.
    let cropped = fit_copy_at(&src, 1, 1, -1, -1);
    let corner = cropped.pixel(0, 0).expect("in bounds").demultiply();
    assert_eq!((corner.red(), corner.alpha()), (255, 255));

    // The zero-offset case is the plain top-left crop/pad.
    let top_left = fit_copy_at(&src, 3, 3, 0, 0);
    let first = top_left.pixel(0, 0).expect("in bounds").demultiply();
    assert_eq!((first.red(), first.alpha()), (255, 255));
    let pad = top_left.pixel(2, 2).expect("in bounds");
    assert_eq!(pad.alpha(), 0);
}

#[test]
fn rescale_nearest_neighbor_duplicates_pixels_exactly() {
    // A 2×2 checkerboard: red / transparent on the first row, swapped on
    // the second. Nearest-neighbor upscale ×2 duplicates each pixel.
    let mut src = Pixmap::new(2, 2).expect("test pixmap");
    src.pixels_mut()[0] = tiny_skia::ColorU8::from_rgba(255, 0, 0, 255).premultiply();
    src.pixels_mut()[1] = tiny_skia::ColorU8::from_rgba(0, 0, 0, 0).premultiply();
    src.pixels_mut()[2] = tiny_skia::ColorU8::from_rgba(0, 0, 0, 0).premultiply();
    src.pixels_mut()[3] = tiny_skia::ColorU8::from_rgba(0, 255, 0, 255).premultiply();

    let out = rescale(&src, ResizeFilter::Nearest, 4, 4);
    assert_eq!(out.width(), 4);
    assert_eq!(out.height(), 4);
    // Row 0 duplicates the source row 0: red red transparent transparent.
    let red = out.pixel(0, 0).expect("in bounds").demultiply();
    assert_eq!((red.red(), red.alpha()), (255, 255));
    let red_again = out.pixel(1, 0).expect("in bounds").demultiply();
    assert_eq!((red_again.red(), red_again.alpha()), (255, 255));
    let clear = out.pixel(2, 0).expect("in bounds");
    assert_eq!(clear.alpha(), 0);
    // Row 3 duplicates the source row 1: transparent until the green pixel.
    let bottom_clear = out.pixel(1, 3).expect("in bounds");
    assert_eq!(bottom_clear.alpha(), 0);
    let green = out.pixel(3, 3).expect("in bounds").demultiply();
    assert_eq!((green.green(), green.alpha()), (255, 255));
}

#[test]
fn rescale_keeps_a_same_size_request_bit_exact() {
    let src = filled_pixmap(7, 5, Color::from_rgba8(10, 200, 30, 128));

    let out = rescale(&src, ResizeFilter::Lanczos3, 7, 5);

    assert_eq!(out.data(), src.data(), "×1.0 rescale is a clone");
}

#[test]
fn scaled_layer_size_rounds_proportionally_and_never_hits_zero() {
    // The canvas doubles: a half-canvas layer doubles too.
    assert_eq!(
        scaled_layer_size(400, 300, 800, 600, 1600, 1200),
        (800, 600)
    );
    // A non-canvas-sized layer follows the same factor.
    assert_eq!(scaled_layer_size(100, 50, 800, 600, 1600, 1200), (200, 100));
    // Asymmetric factor, rounding: 33 × 3/2 = 49.5 → 50.
    assert_eq!(scaled_layer_size(33, 7, 2, 2, 3, 3), (50, 11));
    // A degenerate target never yields a zero dimension.
    assert_eq!(scaled_layer_size(1, 1, 1, 1, 1, 1), (1, 1));
}

#[test]
fn canvas_mode_resizes_every_layer_by_the_same_anchor_and_records_nodes() {
    let mut stack = LayerStack::new(20, 10);
    let added = stack.add_layer(20, 10);
    stack.set_active_layer(0);

    stack.resize_all(
        ResizeMode::Canvas,
        30,
        15,
        ResizeAnchor::TopLeft,
        ResizeFilter::Bicubic,
    );

    // Both buffers crop/pad to the new size.
    assert_eq!(stack.layer(0).map(|l| l.pixmap().width()), Some(30));
    assert_eq!(stack.layer(0).map(|l| l.pixmap().height()), Some(15));
    assert_eq!(stack.layer(added).map(|l| l.pixmap().width()), Some(30));
    // Each tree gained ONE resize node on top of the Start root.
    assert_eq!(stack.layer_nodes(0).map(|nodes| nodes.len()), Some(2));
    assert_eq!(stack.layer_nodes(added).map(|nodes| nodes.len()), Some(2));
    let node = stack.layer_nodes(0).expect("nodes")[1].clone();
    assert_eq!(node.kind, ActionKind::Resize);
    let info = node.resize.expect("resize node carries the choices");
    assert_eq!(info.mode, ResizeMode::Canvas);
    assert_eq!(info.anchor, Some(ResizeAnchor::TopLeft));
    assert_eq!(info.filter, None);
    assert_eq!((info.from_w, info.from_h), (20, 10));
    assert_eq!((info.to_w, info.to_h), (30, 15));
    assert_eq!(info.summary(), "canvas 20×10→30×15 (TL)");
}

#[test]
fn image_mode_rescales_layers_proportionally_and_names_the_filter() {
    let mut stack = LayerStack::new(20, 10);
    let added = stack.add_layer(20, 10);

    stack.resize_all(
        ResizeMode::Image,
        40,
        20,
        ResizeAnchor::Center,
        ResizeFilter::Nearest,
    );

    // The bottom layer IS the canvas — it lands exactly on the target.
    assert_eq!(stack.layer(0).map(|l| l.pixmap().width()), Some(40));
    assert_eq!(stack.layer(added).map(|l| l.pixmap().width()), Some(40));
    let node = stack.layer_nodes(added).expect("nodes")[1].clone();
    let info = node.resize.expect("resize node carries the choices");
    assert_eq!(info.mode, ResizeMode::Image);
    assert_eq!(info.anchor, None);
    assert_eq!(info.filter, Some(ResizeFilter::Nearest));
    assert_eq!(info.summary(), "image 20×10→40×20 (nearest neighbor)");
}

#[test]
fn resize_nodes_participate_in_undo_like_any_action() {
    let mut stack = LayerStack::new(20, 10);

    stack.resize_all(
        ResizeMode::Canvas,
        40,
        20,
        ResizeAnchor::TopLeft,
        ResizeFilter::Bilinear,
    );
    assert_eq!(stack.layer(0).map(|l| l.pixmap().width()), Some(40));
    assert!(stack.can_undo());

    assert!(stack.undo());
    // The head is back at the Start root — the NATIVE 20×10 canvas, which
    // the app restores as the layer's buffer (the bottom layer owns the
    // canvas size, so the display buffer follows in app.rs).
    assert_eq!(
        stack.layer(0).map(|l| l.pixmap().width()),
        Some(40),
        "the model keeps buffers; app.rs restores from the snapshot"
    );
    let head = stack.head().expect("Start root");
    let restored = decode_snapshot(&stack.node(head).expect("known").snapshot).expect("decodes");
    assert_eq!(restored.width(), 20);
    assert_eq!(restored.height(), 10);

    assert!(stack.redo());
    let head = stack.head().expect("resize node");
    let restored = decode_snapshot(&stack.node(head).expect("known").snapshot).expect("decodes");
    assert_eq!(restored.width(), 40);
}

#[test]
fn fit_bottom_to_pads_the_new_base_and_records_its_node() {
    let mut stack = LayerStack::new(30, 30);
    // Simulate the divergent-successor state: a smaller bottom layer.
    stack
        .active_mut()
        .set_pixmap(filled_pixmap(10, 8, Color::from_rgba8(255, 0, 0, 255)));

    stack.fit_bottom_to(30, 30);

    // The bottom layer IS the active one here — `active()` sees the pad.
    let bottom = stack.active();
    assert_eq!(
        (bottom.pixmap().width(), bottom.pixmap().height()),
        (30, 30)
    );
    let inside = bottom.pixmap().pixel(5, 5).expect("in bounds").demultiply();
    assert_eq!((inside.red(), inside.alpha()), (255, 255));
    let pad = bottom.pixmap().pixel(25, 25).expect("in bounds");
    assert_eq!(pad.alpha(), 0, "top-left anchored: padding right/bottom");
    let node = stack.layer_nodes(0).expect("nodes")[1].clone();
    assert_eq!(
        node.resize.expect("recorded").summary(),
        "canvas 10×8→30×30 (TL)"
    );
}

#[test]
fn canvas_info_summary_uses_anchor_codes() {
    let mut info = canvas_info(100, 80, ResizeAnchor::BottomRight);
    info.from_w = 50;
    info.from_h = 40;
    assert_eq!(info.summary(), "canvas 50×40→100×80 (BR)");
}
