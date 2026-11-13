use super::*;

use eframe::egui::Color32;
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

#[test]
fn a_new_stack_holds_one_transparent_layer_with_a_start_root() {
    let stack = LayerStack::new(64, 48);

    assert_eq!(stack.count(), 1);
    let layer = stack.layer(0).expect("layer exists");
    assert_eq!(layer.name(), "Layer 1");
    assert!(layer.visible());
    assert_eq!(layer.pixmap().width(), 64);
    assert!(layer.pixmap().data().iter().all(|&byte| byte == 0));
    assert_eq!(stack.head(), Some(1));
    assert_eq!(stack.node(1).map(|node| node.kind), Some(ActionKind::Start));
    assert_eq!(stack.next_node_id(), 2);
    assert!(!stack.can_undo());
}

#[test]
fn recorded_ids_stay_unique_across_layers() {
    let mut stack = LayerStack::new(32, 32);

    let first = stack
        .record_action(ActionKind::Pencil, Color32::BLACK)
        .expect("encoded");
    let added = stack.add_layer(32, 32);
    let second = stack
        .record_action(ActionKind::Brush, Color32::RED)
        .expect("encoded");

    assert_eq!(first, 2);
    assert_eq!(added, 1, "the new layer becomes active");
    assert_eq!(second, 4, "the new layer's Start root took id 3");
    assert_eq!(stack.layer_nodes(0).map(|nodes| nodes.len()), Some(2));
    assert_eq!(stack.layer_nodes(1).map(|nodes| nodes.len()), Some(2));
    assert_ne!(stack.layer_head(0), stack.layer_head(1));
}

#[test]
fn undo_redo_and_jump_restore_the_snapshotted_pixels() {
    let initial = filled_pixmap(16, 16, Color::from_rgba8(255, 0, 0, 255));
    let mut stack = LayerStack::with_root(initial.clone());
    stack
        .active_mut()
        .pixmap_mut()
        .fill(Color::from_rgba8(0, 0, 255, 255));

    stack
        .record_action(ActionKind::Pencil, Color32::BLACK)
        .expect("encoded");

    assert!(stack.undo());
    let head = stack.head().expect("Start root");
    let restored = decode_snapshot(&stack.node(head).expect("known").snapshot).expect("decodes");
    assert_eq!(
        restored.data(),
        initial.data(),
        "undo shows the Start canvas"
    );

    assert!(stack.redo());
    let head = stack.head().expect("stroke node");
    let restored = decode_snapshot(&stack.node(head).expect("known").snapshot).expect("decodes");
    assert!(restored.data().iter().all(|&byte| byte == 0 || byte == 255));

    assert!(!stack.jump_to(999), "unknown node refused");
}

#[test]
fn add_remove_layers_keeps_the_active_layer_sane() {
    let mut stack = LayerStack::new(16, 16);
    stack.add_layer(16, 16);
    stack.add_layer(16, 16);
    assert_eq!(stack.count(), 3);
    assert_eq!(stack.active_layer(), 2);

    // Removing a layer below the active one shifts the index.
    assert!(stack.remove_layer(0));
    assert_eq!(stack.count(), 2);
    assert_eq!(stack.active_layer(), 1);

    // Removing the active layer clamps to the last one.
    assert!(stack.remove_layer(1));
    assert_eq!(stack.count(), 1);
    assert_eq!(stack.active_layer(), 0);

    // The last layer is refused.
    assert!(!stack.remove_layer(0));
    assert_eq!(stack.count(), 1);
}

#[test]
fn move_layer_reorders_the_composite_and_follows_the_active() {
    let mut stack = LayerStack::new(16, 16);
    stack.add_layer(16, 16);
    let top_name = stack.active_layer_name().to_string();
    stack.set_active_layer(0);
    let bottom_name = stack.active_layer_name().to_string();

    // "Up" moves the bottom layer on top (later in the composite).
    stack.move_layer(0, 1);
    assert_eq!(
        stack.layer(0).map(Layer::name),
        Some(top_name.as_str()),
        "the former top layer moved down"
    );
    assert_eq!(
        stack.layer(1).map(Layer::name),
        Some(bottom_name.as_str()),
        "the former bottom layer moved up"
    );
    assert_eq!(stack.active_layer(), 1, "the active layer follows itself");

    // Moving the top layer up again pushes the active one back to index 0
    // — and the active index follows it there.
    stack.move_layer(0, 1);
    assert_eq!(stack.active_layer_name(), bottom_name);
}

#[test]
fn resize_all_fits_every_buffer_and_keeps_the_trees() {
    let mut stack = LayerStack::new(32, 32);
    stack
        .record_action(ActionKind::Pencil, Color32::BLACK)
        .expect("encoded");
    let nodes_before = stack.layer_nodes(0).map(|nodes| nodes.len());

    stack.resize_all(48, 24);

    assert_eq!(stack.active().pixmap().width(), 48);
    assert_eq!(stack.active().pixmap().height(), 24);
    assert_eq!(
        stack.layer_nodes(0).map(|nodes| nodes.len()),
        nodes_before,
        "nodes stay"
    );
}

#[test]
fn composite_blends_visible_layers_bottom_up() {
    // Bottom: opaque red. Top: 50%-alpha blue (premultiplied (0, 0, 128, 128)).
    let bottom = filled_pixmap(8, 8, Color::from_rgba8(255, 0, 0, 255));
    let top = filled_pixmap(8, 8, Color::from_rgba8(0, 0, 255, 128));
    let mut layers = vec![
        Layer {
            name: "bottom".to_string(),
            visible: true,
            pixmap: bottom,
            tree: LayerTree::new(1, Vec::new()),
        },
        Layer {
            name: "top".to_string(),
            visible: true,
            pixmap: top,
            tree: LayerTree::new(2, Vec::new()),
        },
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
    let layer = Layer {
        name: "single".to_string(),
        visible: true,
        pixmap: filled_pixmap(16, 16, Color::from_rgba8(10, 20, 30, 255)),
        tree: LayerTree::new(1, Vec::new()),
    };
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
