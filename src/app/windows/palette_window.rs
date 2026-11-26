//! The "Palette" window (SPEC §3.3) — a child module of `windows` (owner
//! request 2026-11-19, "do the split of app/windows.rs"): it can touch
//! `App`'s private fields the same way the parent does. FILE LAYOUT (the
//! owner-approved 2026-11-19 file-guard split, pure code motion — the
//! parent was 571 lines after the alpha+hex pass): the whole palette half
//! of the old `windows.rs` moved here VERBATIM — the window itself
//! ([`palette_window`]), the primary/secondary color well with egui's
//! built-in picker popup ([`color_swell_button`]) and the hex-field helpers
//! ([`hex_edited_color`], [`format_color_hex`]). The parent keeps the Tools
//! window, its magnetized settings panel and the shared placement
//! plumbing; `app.rs`'s call sites are unchanged.
//!
//! The wells: a click opens egui's color picker for THAT well's color — the
//! alpha channel editable (`Alpha::OnlyBlend`, owner request 2026-11-19:
//! egui's alpha slider + the A DragValue appear, and egui stops forcing the
//! stored alpha to 1.0, which used to silently erase an Eyedropper-picked
//! translucent pixel on the first popup open; the drawing pipeline in
//! `crate::tools` composites alpha end-to-end, so a translucent well paints
//! translucently) — plus a hex field under the picker (owner request
//! 2026-11-19: "hex is needed too often") parsing the same junk-tolerant
//! syntax as the launch flags. The window's first-open position is §3.3
//! (`Window::pivot` + `default_pos`); where it sits afterwards (drop-time
//! anchoring, resize repositioning) is §3.11 and lives in
//! [`crate::app::placement`].

use eframe::egui::{self, Color32, Context, Id, Rect, Sense};

use crate::app::App;
use crate::app::launch::parse_hex_components;
use crate::app::placement::{first_open_gap, place_floating_window};
use crate::palette::{COLORS, COLS, GRID_WIDTH, ROWS, TILE};

fn palette_window_id() -> Id {
    Id::unique("palette_window")
}

impl App {
    pub(in crate::app) fn palette_window(&mut self, ctx: &Context) {
        if !self.palette_window_open {
            return;
        }
        let mut open = true;
        // The smartphone compact rule (owner request 2026-11-25, SPEC
        // §3.11): FLUSH-0 first-open gaps on a phone web session — since
        // 2026-11-26 keyed on the FROZEN startup latch (a rotation never
        // re-decides it).
        let smartphone = self.smartphone_session();
        let shown = egui::Window::new("Palette")
            .id(palette_window_id())
            .open(&mut open)
            // First open (§3.3): the window's bottom-left corner sits 0.5rem
            // from the main window's left and bottom edges (`Align2::LEFT_BOTTOM`
            // pivot interprets `default_pos` as that corner's position) —
            // FLUSH 0 on a smartphone web session (owner request
            // 2026-11-25, SPEC §3.11 — latched at startup since 2026-11-26).
            // As with Tools, the stored position wins after any move.
            .pivot(egui::Align2::LEFT_BOTTOM)
            .default_pos(
                ctx.content_rect().left_bottom()
                    + egui::vec2(first_open_gap(smartphone), -first_open_gap(smartphone)),
            )
            .auto_sized() // size follows the content; no resize handles
            .show(ctx, |ui| {
                // Top: the primary/secondary color wells. A click on a well
                // opens egui's color picker for that color (the former
                // "Custom…" button, one per color); Swap sits to the right.
                // No hotkey behind it — §3.9 stays exhaustive and the label
                // names no key either (owner request 2026-11-15).
                ui.horizontal(|ui| {
                    color_swell_button(ui, &mut self.tools.primary, 32.0);
                    color_swell_button(ui, &mut self.tools.secondary, 32.0);
                    if ui.button("Swap").clicked() {
                        std::mem::swap(&mut self.tools.primary, &mut self.tools.secondary);
                    }
                });

                // A plain ui.separator() fills all available width; inside an
                // auto-sized window that width feeds back into the window size
                // until the window spans the whole screen. Pin the divider to
                // the swatch-grid width instead (6.0 pt = default height).
                ui.add_sized([GRID_WIDTH, 6.0], egui::Separator::default());

                // Bottom: the seamless preset grid — tiles touch each other
                // (gap 0) and have no outline. LMB sets the primary color,
                // RMB the secondary. Not an egui::Grid: Grid enforces a
                // minimum column width (`interact_size.x`, 40 pt by default),
                // which leaves gaps between the 20 pt tiles. One exact-size
                // block is allocated instead; neighboring tiles share edges,
                // and clicks are resolved manually from the pointer position.
                let grid_size = egui::vec2(GRID_WIDTH, ROWS as f32 * TILE);
                let (grid_rect, grid) = ui.allocate_exact_size(grid_size, Sense::click());
                for row in 0..ROWS {
                    for col in 0..COLS {
                        let tile = Rect::from_min_size(
                            grid_rect.min + egui::vec2(col as f32 * TILE, row as f32 * TILE),
                            egui::vec2(TILE, TILE),
                        );
                        ui.painter()
                            .rect_filled(tile, 0.0, COLORS[row * COLS + col]);
                    }
                }
                if let Some(pos) = grid.interact_pointer_pos() {
                    let dx = pos.x - grid_rect.min.x;
                    let dy = pos.y - grid_rect.min.y;
                    if dx >= 0.0 && dy >= 0.0 {
                        let col = (dx / TILE) as usize;
                        let row = (dy / TILE) as usize;
                        if col < COLS && row < ROWS {
                            let color = COLORS[row * COLS + col];
                            if grid.clicked() {
                                self.tools.primary = color;
                            }
                            if grid.secondary_clicked() {
                                self.tools.secondary = color;
                            }
                        }
                    }
                }
            });
        if let Some(response) = &shown {
            place_floating_window(
                ctx,
                palette_window_id(),
                response.response.rect,
                ctx.content_rect(),
                self.last_content_rect,
                &mut self.palette_placement,
            );
        }
        if !open {
            self.palette_window_open = false;
        }
    }
}

/// A primary/secondary color well: keeps the classic swatch look, but a
/// click opens egui's built-in color picker for THIS color (each well edits
/// its own color; there are no captions).
///
/// The picker edits the ALPHA too (owner request 2026-11-19):
/// `Alpha::OnlyBlend` instead of the former `Alpha::Opaque` — egui shows its
/// own alpha slider and the A DragValue, and stops forcing the stored alpha
/// to 1.0 (which silently erased an Eyedropper-picked translucent pixel on
/// the first popup open). The drawing pipeline already composites alpha
/// end-to-end (`tools.rs`), so a translucent well paints translucently.
///
/// Under the picker sits a hex field (owner request 2026-11-19: "hex is
/// needed too often") parsing the SAME junk-tolerant syntax as the
/// `--primary`/`--secondary` launch flags
/// (`crate::app::launch::parse_hex_components`): every non-hex-digit
/// character is stripped (an optional `0x` prefix dropped first) and the
/// digit COUNT picks the form — `RGB` 3, `RGBA` 4, `RGBAA` 5, `RRGGBB` 6,
/// `RRGGBBA` 7, `RRGGBBAA` 8. An alpha-carrying form sets the alpha, a
/// short form keeps the alpha the well had when this edit session started
/// (the seed snapshot below). The field's raw text AND that seed alpha live
/// in egui's per-well memory (`IdTypeMap`): a string rebuilt from the color
/// every frame would fight the caret while typing.
fn color_swell_button(ui: &mut egui::Ui, color: &mut Color32, size: f32) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(size, size), Sense::click());
    ui.painter().rect_filled(rect, 2.0, *color);
    ui.painter().rect_stroke(
        rect,
        2.0,
        egui::Stroke::new(1.0, Color32::from_gray(40)),
        egui::StrokeKind::Inside,
    );

    // Same wiring as egui's own color button (color_edit_button_hsva): a
    // click toggles the popup; the HSV state is re-derived from the current
    // color every frame and written back only when the picker changed it.
    let mut hsva = egui::ecolor::Hsva::from(*color);
    let mut button_response = response;
    let popup_id = button_response.id.with("color_popup");
    // The hex field's identities, per well (the two wells never share
    // text): the raw text, its edit-session SEED ALPHA (the short forms'
    // fallback, snapshotted with the text) and the TextEdit itself.
    let hex_text_id = button_response.id.with("color_hex_text");
    let hex_seed_alpha_id = button_response.id.with("color_hex_seed_alpha");
    let hex_edit_id = button_response.id.with("color_hex_edit");
    // Is the popup OPEN as of the PREVIOUS frame? `is_id_open` reads the
    // stored state before `.show` runs this frame — exactly the open
    // transition (egui's own `color_edit_button_hsva` reads it the same
    // way). A freshly opened popup re-seeds the hex text below, because the
    // stored text went stale while the popup was closed (a preset click,
    // Swap, a launch flag can all have changed the color meanwhile).
    let was_open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    egui::Popup::menu(&button_response)
        .id(popup_id)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().slider_width = 275.0;
            let mut changed = egui::color_picker::color_picker_hsva_2d(
                ui,
                &mut hsva,
                egui::color_picker::Alpha::OnlyBlend,
            );
            if changed {
                // The sliders only mutate `hsva` — the picker's own
                // writeback (as egui's button does it).
                *color = Color32::from(hsva);
            }
            // Re-seed the hex text whenever the color changed OUTSIDE the
            // field (fresh popup = stale text; the sliders just moved).
            // While the FIELD edits, its raw text stands — the picker part
            // only changes on pointer drags, and a typing frame never drags.
            if !was_open || changed {
                let [_, _, _, seed_alpha] = color.to_srgba_unmultiplied();
                let seed = format_color_hex(*color);
                ui.ctx().memory_mut(|mem| {
                    mem.data.insert_temp(hex_text_id, seed);
                    // The seed alpha rides with the text seed: the alpha the
                    // well had when this edit session started. A live-typed
                    // longer entry PASSES THROUGH alpha-carrying prefixes
                    // (typing `FF7F00` hits the 5-digit `RGBAA` form at
                    // `FF7F0`, whose own alpha briefly applies) — without
                    // the seed that prefix's alpha would leak into the
                    // final 6-digit color, which must keep the session's.
                    mem.data.insert_temp(hex_seed_alpha_id, seed_alpha);
                });
            }
            ui.horizontal(|ui| {
                ui.label("Hex");
                let mut text = ui
                    .ctx()
                    .memory(|mem| mem.data.get_temp::<String>(hex_text_id))
                    .unwrap_or_default();
                let field = ui.add(
                    egui::TextEdit::singleline(&mut text)
                        .id(hex_edit_id)
                        // Monospace: the digits line up while editing.
                        .font(egui::TextStyle::Monospace),
                );
                let edited = field.changed();
                let lost_focus = field.lost_focus();
                if edited {
                    // The direct `*color` write keeps the parsed hex
                    // BIT-EXACT — routing it through `hsva` first would let
                    // the HSV↔RGB float roundtrip nudge the entered bytes.
                    let [_, _, _, current_alpha] = color.to_srgba_unmultiplied();
                    let fallback = ui
                        .ctx()
                        .memory(|mem| mem.data.get_temp::<u8>(hex_seed_alpha_id))
                        .unwrap_or(current_alpha);
                    if let Some(new) = hex_edited_color(fallback, &text) {
                        *color = new;
                        hsva = egui::ecolor::Hsva::from(new);
                        changed = true;
                    }
                }
                if lost_focus {
                    // The edit is over (Enter/click-elsewhere): the field
                    // shows the canonical form of the FINAL color — a valid
                    // entry gets normalized, invalid text reverts to the
                    // current color.
                    let canonical = format_color_hex(*color);
                    ui.ctx()
                        .memory_mut(|mem| mem.data.insert_temp(hex_text_id, canonical));
                } else {
                    // Keep the raw text (mid-typing state included).
                    ui.ctx()
                        .memory_mut(|mem| mem.data.insert_temp(hex_text_id, text));
                }
            });
            if changed {
                button_response.mark_changed();
            }
        });
    button_response
}

/// The hex field's write-back (owner request 2026-11-19): the RGB comes
/// from the text; the alpha comes from the text only when it was written —
/// the alpha-carrying forms (`RGBA` 4, `RGBAA` 5, `RRGGBBA` 7,
/// `RRGGBBAA` 8 digits) — while a short form (`RGB` 3 / `RRGGBB` 6 digits)
/// keeps `fallback_alpha`: the alpha the well had when this edit session
/// started (the seed snapshot in [`color_swell_button`]), the same way the
/// R/G/B DragValues don't touch it. `None` on any other text (also while
/// the user is mid-typing a partial value) — the field then simply applies
/// nothing.
fn hex_edited_color(fallback_alpha: u8, text: &str) -> Option<Color32> {
    parse_hex_components(text).map(|([r, g, b], parsed)| {
        Color32::from_rgba_unmultiplied(r, g, b, parsed.unwrap_or(fallback_alpha))
    })
}

/// The hex field's display form: `#RRGGBBAA` — the alpha is always spelled
/// out now that the picker edits it (an 8-digit paste sets it, a 6-digit
/// paste keeps it — see [`hex_edited_color`]).
fn format_color_hex(color: Color32) -> String {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
}

#[cfg(test)]
mod tests;
