use super::*;

use eframe::egui::Color32;
use tiny_skia::Color;

// The snapshot-codec, composite and resize assertions live in their own
// topic files (the ~300-line test guard).
mod composite;
mod resize;
mod snapshot;

fn pixmap(width: u32, height: u32) -> Pixmap {
    Pixmap::new(width, height).expect("test pixmap")
}

fn filled_pixmap(width: u32, height: u32, color: Color) -> Pixmap {
    let mut pixmap = pixmap(width, height);
    pixmap.fill(color);
    pixmap
}

#[test]
fn a_new_stack_starts_white_and_added_layers_stay_transparent() {
    let mut stack = LayerStack::new(64, 48);

    assert_eq!(stack.count(), 1);
    let layer = stack.layer(0).expect("layer exists");
    assert_eq!(layer.name(), "Layer 1");
    assert!(layer.visible());
    assert_eq!(layer.pixmap().width(), 64);
    assert!(
        layer.pixmap().data().iter().all(|&byte| byte == 255),
        "the initial layer is filled white"
    );
    assert_eq!(stack.head(), Some(1));
    assert_eq!(stack.node(1).map(|node| node.kind), Some(ActionKind::Start));
    assert_eq!(stack.next_node_id(), 2);
    assert!(!stack.can_undo());

    let added = stack.add_layer(64, 48);
    assert!(
        stack
            .layer(added)
            .is_some_and(|layer| layer.pixmap().data().iter().all(|&byte| byte == 0)),
        "a layer added later stays fully transparent"
    );
}

#[test]
fn recorded_ids_stay_unique_across_layers() {
    let mut stack = LayerStack::new(32, 32);

    let first = stack
        .record_action(ActionKind::Pencil, Color32::BLACK, None)
        .expect("encoded");
    let added = stack.add_layer(32, 32);
    let second = stack
        .record_action(ActionKind::Brush, Color32::RED, None)
        .expect("encoded");

    assert_eq!(first, 2);
    assert_eq!(added, 1, "the new layer becomes active");
    assert_eq!(second, 4, "the new layer's Start root took id 3");
    assert_eq!(stack.layer_nodes(0).map(|nodes| nodes.len()), Some(2));
    assert_eq!(stack.layer_nodes(1).map(|nodes| nodes.len()), Some(2));
    assert_ne!(stack.layer_head(0), stack.layer_head(1));
}

#[test]
fn a_two_color_gesture_stores_both_colors_on_its_node() {
    let mut stack = LayerStack::new(32, 32);

    let shape = stack
        .record_action(ActionKind::Shape, Color32::RED, Some(Color32::BLUE))
        .expect("encoded");
    let stroke = stack
        .record_action(ActionKind::Pencil, Color32::GREEN, None)
        .expect("encoded");

    let node = stack.node(shape).expect("known node");
    assert_eq!(node.color, Color32::RED, "the outline rides in `color`");
    assert_eq!(
        node.color2,
        Some(Color32::BLUE),
        "the fill rides in `color2`"
    );
    assert_eq!(
        stack.node(stroke).map(|node| node.color2),
        Some(None),
        "a single-color gesture carries no second color"
    );
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
        .record_action(ActionKind::Pencil, Color32::BLACK, None)
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
fn resize_all_adapts_every_layer_and_records_one_node_each() {
    let mut stack = LayerStack::new(32, 32);
    stack
        .record_action(ActionKind::Pencil, Color32::BLACK, None)
        .expect("encoded");
    let added = stack.add_layer(32, 32);
    // A mixed-size model (2026-11-17): a non-bottom layer may differ from
    // the canvas — resize_all adapts it like every other layer.
    stack.set_active_layer(added);
    stack
        .active_mut()
        .set_pixmap(Pixmap::new(16, 16).expect("test pixmap"));

    stack.resize_all(
        ResizeMode::Canvas,
        48,
        24,
        ResizeAnchor::TopLeft,
        ResizeFilter::Bicubic,
    );

    assert_eq!(stack.layer(0).map(|l| l.pixmap().width()), Some(48));
    assert_eq!(stack.layer(0).map(|l| l.pixmap().height()), Some(24));
    assert_eq!(stack.layer(added).map(|l| l.pixmap().width()), Some(48));
    assert_eq!(stack.layer(added).map(|l| l.pixmap().height()), Some(24));
    // The resize itself commits ONE node per layer (owner request
    // 2026-11-17) — both trees grew by one, nothing was lost.
    assert_eq!(stack.layer_nodes(0).map(|nodes| nodes.len()), Some(3));
    assert_eq!(stack.layer_nodes(added).map(|nodes| nodes.len()), Some(2));
}
