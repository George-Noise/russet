//! The PURE color rules of the preview-then-commit gestures (a child of
//! `app/preview.rs`; placed here from the start 2026-11-16, when the
//! recorded-color mapping joined the wells rule — the parent stays under
//! the 500-line file guard): which color wells a Shape gesture USES
//! ([`shape_gesture_wells`]) and what a Shape commit RECORDS on its
//! history node ([`shape_record_colors`]). Both are pure functions — no
//! egui frame, no `App` access — so the rules stay unit-testable without
//! a context; the tests live in the sibling `preview/tests.rs` through
//! the parent's re-import.
//!
//! The interactive half (the gesture state machine, the commits, the
//! on-canvas preview painting) stays in the parent module.

use eframe::egui::{Color32, PointerButton};

/// Wells of a Shape-tool gesture start: the outline comes from the primary
/// well and the fill from the secondary one, each gated by its half of the
/// Shape panel's paint radio (owner request 2026-11-10; the two checkboxes
/// became a THREE-way radio 2026-11-20 — Fill / Outline / Fill+Outline —
/// and the caller feeds its `fills()`/`outlines()` halves here). A
/// RIGHT-button gesture USES the two wells INVERTED (owner request
/// 2026-11-15: "the colors are used the other way around and without
/// Swap") — the former secondary paints the outline, the former primary
/// the fill — WITHOUT touching the wells themselves: the Palette's
/// primary/secondary stay exactly as they are, unlike the persistent swap
/// the previous behavior performed. The wells cannot be mutated through
/// this helper (no `&mut` access to them, and the caller does not swap), so
/// "without Swap" holds by construction. Returns `(outline, fill)`.
pub(super) fn shape_gesture_wells(
    button: PointerButton,
    shape_outline: bool,
    shape_fill: bool,
    primary: Color32,
    secondary: Color32,
) -> (Option<Color32>, Option<Color32>) {
    let inverted = button == PointerButton::Secondary;
    let (outline_source, fill_source) = if inverted {
        (secondary, primary)
    } else {
        (primary, secondary)
    };
    (
        shape_outline.then_some(outline_source),
        shape_fill.then_some(fill_source),
    )
}

/// What a Shape commit RECORDS on the history node: the outline is the
/// main color and the fill rides along as the second one — but only when
/// the gesture actually used BOTH wells (owner request 2026-11-16: the
/// History Tree row of a two-color gesture shows both colors; a Shape
/// drawn with only one well stays single-color, as do the freehand tools,
/// Line and Text — since the 2026-11-20 radio exactly one of Fill /
/// Outline / Fill+Outline is active, so the both-wells case is the
/// explicit Fill+Outline choice). Falls back to the primary well when both
/// halves are off — unreachable through the radio, which always has one
/// paint active, but kept total like the previous
/// `outline.or(fill).unwrap_or(primary)` rule.
pub(super) fn shape_record_colors(
    outline: Option<Color32>,
    fill: Option<Color32>,
    primary: Color32,
) -> (Color32, Option<Color32>) {
    match (outline, fill) {
        (Some(outline), Some(fill)) => (outline, Some(fill)),
        (Some(outline), None) => (outline, None),
        (None, Some(fill)) => (fill, None),
        (None, None) => (primary, None),
    }
}
