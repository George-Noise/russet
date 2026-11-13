use super::*;
use crate::layers::decode_snapshot;
use serde_json::Value;
use tiny_skia::Color;

#[test]
fn parse_points_reads_xy_pairs() {
    let value = serde_json::json!([[10, 20], [30.5, 40.25]]);

    let points = parse_points(&value).expect("valid pairs");

    assert_eq!(points.len(), 2);
    assert_eq!(points[0], Pos2::new(10.0, 20.0));
    assert_eq!(points[1], Pos2::new(30.5, 40.25));
}

#[test]
fn parse_points_rejects_malformed_input() {
    assert!(parse_points(&Value::Null).is_err());
    assert!(parse_points(&serde_json::json!([])).is_err());
    assert!(parse_points(&serde_json::json!([1, 2])).is_err());
    assert!(parse_points(&serde_json::json!([[1]])).is_err());
    assert!(parse_points(&serde_json::json!([[1, 2, 3]])).is_err());
    assert!(parse_points(&serde_json::json!([["a", 2]])).is_err());
}

#[test]
fn hex_colors_round_trip_through_parse_and_format() {
    let color = parse_hex_color("#Ff8000").expect("6-digit hex");
    let short = parse_hex_color("#f0a").expect("3-digit hex");

    assert_eq!(hex_color(color), "#ff8000");
    assert_eq!(hex_color(short), "#ff00aa");
    assert!(parse_hex_color("ff8000").is_none());
    assert!(parse_hex_color("#12345").is_none());
    assert!(parse_hex_color("#zzzzzz").is_none());
}

#[test]
fn base64_encodes_known_vectors() {
    assert_eq!(base64(b""), "");
    assert_eq!(base64(b"f"), "Zg==");
    assert_eq!(base64(b"fo"), "Zm8=");
    assert_eq!(base64(b"foo"), "Zm9v");
    assert_eq!(base64(b"foobar"), "Zm9vYmFy");
}

#[test]
fn tool_declarations_have_name_description_schema() {
    let tools = tool_declarations();
    let list = tools.as_array().expect("tools array");

    assert_eq!(list.len(), 9);
    let names: Vec<&str> = list
        .iter()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .collect();
    assert!(
        names.contains(&"get_tree_layout"),
        "the layout dump must stay declared"
    );
    for tool in list {
        assert!(
            tool.get("name")
                .and_then(Value::as_str)
                .is_some_and(|name| !name.is_empty())
        );
        assert!(tool.get("description").and_then(Value::as_str).is_some());
        assert!(
            tool.get("inputSchema")
                .map(Value::is_object)
                .unwrap_or(false)
        );
    }
}

/// A frame-free tree mirroring `LayerStack::with_root`: Start id 1, strokes
/// start at 2.
fn fresh(pixmap: &Pixmap) -> LayerTree {
    LayerTree::new(1, encode_snapshot(pixmap))
}

#[test]
fn apply_stroke_records_node_pixels_and_snapshot() {
    let mut pixmap = Pixmap::new(100, 100).expect("pixmap");
    pixmap.fill(Color::from_rgba8(255, 255, 255, 255));
    let mut tree = fresh(&pixmap);
    let mut log = ActionLog::new();

    let node = apply_stroke(
        &mut pixmap,
        &mut tree,
        &mut log,
        2,
        &[Pos2::new(10.0, 10.0), Pos2::new(50.0, 50.0)],
        Color32::BLACK,
    )
    .expect("node id");

    assert_eq!(node, 2);
    assert_eq!(tree.head(), Some(2));
    assert_eq!(log.items().len(), 1);
    assert_eq!(
        pixel_rgb(&pixmap, 30, 30),
        (0, 0, 0),
        "the diagonal passes the midpoint"
    );
    assert_eq!(
        pixel_rgb(&pixmap, 99, 99),
        (255, 255, 255),
        "nothing outside the stroke"
    );
    // The node's snapshot decodes back to the post-stroke buffer.
    let snapshot = tree.node(2).expect("recorded").snapshot.clone();
    let restored = decode_snapshot(&snapshot).expect("decodes");
    assert_eq!(restored.data(), pixmap.data());
}

#[test]
fn apply_stroke_after_undo_forks_like_a_hand_gesture() {
    let mut pixmap = Pixmap::new(100, 100).expect("pixmap");
    let mut tree = fresh(&pixmap);
    let mut log = ActionLog::new();

    apply_stroke(
        &mut pixmap,
        &mut tree,
        &mut log,
        2,
        &[Pos2::new(10.0, 10.0), Pos2::new(20.0, 10.0)],
        Color32::BLACK,
    )
    .expect("first node");
    apply_stroke(
        &mut pixmap,
        &mut tree,
        &mut log,
        3,
        &[Pos2::new(30.0, 10.0), Pos2::new(40.0, 10.0)],
        Color32::BLACK,
    )
    .expect("second node");

    // The real undo (App::undo) restores the parent's PNG AND moves the
    // tree head; the tree half of that step is all this frame-free test
    // needs.
    tree.undo();

    let third = apply_stroke(
        &mut pixmap,
        &mut tree,
        &mut log,
        4,
        &[Pos2::new(25.0, 50.0), Pos2::new(28.0, 50.0)],
        Color32::BLACK,
    )
    .expect("third node");

    let nodes = tree.nodes();
    assert_eq!(third, 4);
    assert_eq!(nodes.len(), 4, "Start plus three strokes");
    assert_eq!(
        nodes[3].parent,
        Some(2),
        "the new stroke hangs off the undone-to node"
    );
    let children_of_first: Vec<NodeId> = nodes
        .iter()
        .filter(|node| node.parent == Some(2))
        .map(|node| node.id)
        .collect();
    assert_eq!(
        children_of_first,
        vec![3, 4],
        "two children = a git-style fork"
    );
    assert_eq!(tree.head(), Some(4));
}

fn pixel_rgb(pixmap: &Pixmap, x: u32, y: u32) -> (u8, u8, u8) {
    let pixel = pixmap.pixel(x, y).expect("in bounds").demultiply();
    (pixel.red(), pixel.green(), pixel.blue())
}
