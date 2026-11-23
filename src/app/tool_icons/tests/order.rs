//! The Tools window's button ORDER (owner request 2026-11-21): the grid is
//! filled row-major over [`TOOLS`] (`windows.rs` chunks it by
//! `TOOL_COLUMNS`), so the array IS the on-screen layout — the owner's pass
//! swapped two pairs: the Eyedropper ↔ the Select, the Text ↔ the Line.

use super::TOOLS;
use crate::tools::Tool;

#[test]
fn the_tools_grid_order_matches_the_2026_11_21_owner_pass() {
    assert_eq!(
        TOOLS,
        [
            Tool::Pencil,
            Tool::Brush,
            Tool::Eraser,
            // The owner's 2026-11-21 swap: the Eyedropper moves up beside
            // the paint trio (row 2, right of the Eraser)…
            Tool::Eyedropper,
            // …the Select follows it, and the Text joins it in row 3 (the
            // Text ↔ Line swap)…
            Tool::Select,
            Tool::Text,
            // …leaving the Shape and the Line to close the grid in row 4.
            Tool::Shape,
            Tool::Line,
            // The fill joins at the END (owner request 2026-11-23: «иконку
            // заливки сделай самой последней чтобы не нарушить порядок
            // инструментов в окошке Tools») — row 5's first cell.
            Tool::Fill,
        ],
        "the grid is row-major over TOOLS: this array is the on-screen layout"
    );
}
