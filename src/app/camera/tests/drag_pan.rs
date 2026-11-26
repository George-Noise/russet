//! The mouse DRAG-pan regression (2026-11-25, kept here when the touch
//! tests moved to `touch_gesture.rs`): the 2026-11-13 app.rs split carried
//! the drag-pan DETECTION, the Grabbing cursor and the stroke suppression
//! into camera.rs but dropped the `self.pan += response.drag_delta()`
//! application — the middle-drag and Space+LMB pans moved NOTHING since.
//! The line was restored 2026-11-25; this pins it through a real
//! `Context::run_pass` drag.

use super::super::App;
use eframe::egui::{self, PointerButton, Pos2, Rect, Sense, Vec2};

#[test]
fn a_middle_button_drag_pans_the_view() {
    let mut app = App::new_headless();
    let ctx = egui::Context::default();
    let rect = Rect::from_min_size(Pos2::new(50.0, 50.0), Vec2::new(300.0, 300.0));

    // One pass helper rendering the interactable rect each frame — a
    // warm-up pass first so the NEXT pass's press event hit-tests THIS
    // widget (egui assigns the drag to whatever was under the pointer on
    // the previous pass).
    fn drag_pass(ctx: &egui::Context, events: Vec<egui::Event>, app: &mut App, rect: Rect) {
        let input = egui::RawInput {
            events,
            ..Default::default()
        };
        let mut output = ctx.run_pass(input, |ctx| {
            // `movable(false)` keeps the area's own root widget from
            // stealing the drag (a movable Area senses drags to move
            // itself, and its root covers the whole pass).
            egui::Area::new(egui::Id::unique("drag-pan-test"))
                .movable(false)
                .fixed_pos(Pos2::ZERO)
                .show(ctx, |ui| {
                    // The allocated rect IS the area's content, so the
                    // widget is laid out, registered for the next pass's
                    // hit-test and draggable exactly like a real canvas
                    // widget.
                    let response = ui.allocate_rect(rect, Sense::click_and_drag());
                    app.handle_pan(ui, &response);
                });
        });
        output.textures_delta.clear();
    }

    // TWO warm-up passes: an auto-sized area's first pass is a sizing
    // pass whose widgets are not hit-testable, so the press must arrive on
    // the SECOND pass at the earliest — the widget then sits in the
    // hit-test list the press reads.
    drag_pass(&ctx, vec![], &mut app, rect);
    drag_pass(&ctx, vec![], &mut app, rect);
    // The press at (100, 100) — inside the rect.
    drag_pass(
        &ctx,
        vec![egui::Event::PointerButton {
            pos: Pos2::new(100.0, 100.0),
            button: PointerButton::Middle,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
        &mut app,
        rect,
    );
    assert_eq!(app.pan, Vec2::ZERO, "the press itself pans nothing");

    // The drag to (130, 116) — delta (30, 16) — pans the view with the
    // pointer (the Space+LMB drag rides the same line).
    drag_pass(
        &ctx,
        vec![egui::Event::PointerMoved(Pos2::new(130.0, 116.0))],
        &mut app,
        rect,
    );
    assert_eq!(
        app.pan,
        Vec2::new(30.0, 16.0),
        "the middle drag moves the view with the pointer"
    );
}
