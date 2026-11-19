use super::*;

#[test]
fn pointer_moves_extract_only_move_events_in_order() {
    let events = vec![
        egui::Event::PointerMoved(Pos2::new(1.0, 2.0)),
        egui::Event::PointerGone,
        egui::Event::PointerMoved(Pos2::new(3.0, 4.0)),
        egui::Event::Text("x".into()),
    ];
    assert_eq!(
        pointer_move_positions(&events),
        vec![Pos2::new(1.0, 2.0), Pos2::new(3.0, 4.0)]
    );
}

#[test]
fn a_frame_without_moves_yields_an_empty_stream() {
    assert!(pointer_move_positions(&[egui::Event::PointerGone]).is_empty());
    assert!(pointer_move_positions(&[]).is_empty());
}
