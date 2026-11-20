//! The "Save Image" dialog (SPEC §3.7; owner request 2026-11-19): BOTH
//! File ▸ Save (Ctrl+S) and File ▸ Save As (Ctrl+Shift+S) open it — an
//! in-app dialog replacing the OS save dialog, with everything the save
//! needs collected IN THE DIALOG: the format radio (PNG / JPEG), the file
//! name, the per-format encoding knob (the PNG compression slider, the
//! JPEG quality), the Lossless row (owner request 2026-11-20) and —
//! native only — the destination folder.
//!
//! This module owns the dialog UI, its `App` state prefill and the pure
//! name helpers; the platform save glue lives in [`super::document`]
//! (platform cfg is allowed only there and in `file_io.rs` per AGENTS.md)
//! and the codecs in [`crate::file_io`]. The modal plumbing — centered on
//! every open, content-hugging, the rightmost-affirmative button row — is
//! the shared [`super::dialogs`] helpers.

use eframe::egui::{self, Context};

use super::App;
use super::dialogs::{dialog_button_row, dialog_id, dialog_window, forget_dialog_position};
use crate::file_io::{self, SaveFormat};

/// The file name shown when the current name has no usable text — the
/// fallback of [`base_name_of`] and [`compose_file_name`] alike.
pub(super) const FALLBACK_NAME: &str = "Untitled";

/// The characters stripped from the typed file name: the two path
/// separators (a separator would make `Path::join` write outside the
/// chosen folder) plus the characters Windows forbids in a file name.
const FORBIDDEN_CHARS: [char; 9] = ['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// The file name the Save click writes: the cleaned typed name plus the
/// CHOSEN format's extension — the format radio owns the extension (owner
/// request 2026-11-19: the format is chosen in the dialog). A typed image
/// extension matching the format survives (canonicalized — `.jpeg` becomes
/// `.jpg`), a conflicting one is replaced, a non-image one stays part of
/// the base; separators and the Windows-forbidden characters never reach
/// the file system; an empty name falls back to [`FALLBACK_NAME`].
pub(super) fn compose_file_name(raw: &str, format: SaveFormat) -> String {
    let cleaned: String = raw
        .trim()
        .chars()
        .filter(|c| !FORBIDDEN_CHARS.contains(c))
        .collect();
    format!("{}.{}", base_name_of(&cleaned), format.extension())
}

/// Strips a KNOWN image extension (`.png` / `.jpg` / `.jpeg`,
/// case-insensitive) for the dialog's prefill — the format radio owns the
/// extension, so `display_name` must not pin one. Unknown or no extension:
/// the trimmed name as-is; nothing usable: [`FALLBACK_NAME`].
pub(super) fn base_name_of(name: &str) -> String {
    let name = name.trim();
    // `to_ascii_lowercase` only maps ASCII bytes, so the UTF-8 boundaries
    // of `lower` and `name` are identical and the byte slice is safe.
    let lower = name.to_ascii_lowercase();
    if let Some(base_len) = [".png", ".jpg", ".jpeg"]
        .iter()
        .find_map(|ext| lower.strip_suffix(ext).map(|base| base.len()))
    {
        let base = name[..base_len].trim_end();
        return if base.is_empty() {
            FALLBACK_NAME.to_string()
        } else {
            base.to_string()
        };
    }
    if name.is_empty() {
        FALLBACK_NAME.to_string()
    } else {
        name.to_string()
    }
}

impl App {
    /// Prefills the dialog from the document and shows it. The name comes
    /// from `display_name` without its image extension; the format from the
    /// display name's extension when there is one (else the last choice
    /// stays — fresh sessions start at PNG). The NATIVE refinement from the
    /// current path (folder + extension + name) runs right after this in
    /// [`super::document::save`], where the platform cfg lives.
    pub(super) fn open_save_dialog(&mut self) {
        self.save_file_name = base_name_of(&self.display_name);
        if let Some((_, ext)) = self.display_name.rsplit_once('.')
            && let Some(format) = SaveFormat::from_extension(ext)
        {
            self.save_format = format;
        }
        self.show_save_dialog = true;
    }

    /// The dialog itself: the file name field, the format radio (THE format
    /// choice — owner request 2026-11-19), the per-format encoding knob —
    /// the PNG Compression slider or the JPEG Quality slider (the values
    /// are kept across opens) — the Lossless row (owner request
    /// 2026-11-20: THE guarantee, displayed per format — PNG locks it ON,
    /// JPEG locks it OFF; neither format can do otherwise), the native
    /// folder row and the resulting-name preview. Save (rightmost per
    /// §3.7) hands over to [`super::document::commit_save_dialog`] — the
    /// dialog closes only on success (or, on web, once the download task
    /// is spawned); Cancel and the ✕ close without writing.
    pub(super) fn save_dialog(&mut self, ctx: &Context) {
        if !self.show_save_dialog {
            return;
        }
        let mut open = true;
        let mut save = false;
        let mut cancel = false;
        let id = dialog_id("save");
        dialog_window("Save Image", id, ctx)
            .open(&mut open)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("File name");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.save_file_name).desired_width(160.0),
                    );
                });
                ui.horizontal(|ui| {
                    ui.label("Format");
                    ui.radio_value(
                        &mut self.save_format,
                        SaveFormat::Png,
                        SaveFormat::Png.label(),
                    );
                    ui.radio_value(
                        &mut self.save_format,
                        SaveFormat::Jpeg,
                        SaveFormat::Jpeg.label(),
                    );
                });
                match self.save_format {
                    SaveFormat::Png => {
                        ui.horizontal(|ui| {
                            ui.label("Compression");
                            ui.add(egui::Slider::new(&mut self.save_png_compression, 0..=9));
                        });
                        // PNG cannot be lossy, so the owner-requested
                        // Lossless row is a locked-ON display of the
                        // guarantee, not a toggle (a toggle would be a
                        // placebo — unchecking could not change anything).
                        let mut lossless = true;
                        ui.horizontal(|ui| {
                            ui.add_enabled(false, egui::Checkbox::new(&mut lossless, "Lossless"));
                            ui.weak("(always — PNG is lossless)");
                        });
                    }
                    SaveFormat::Jpeg => {
                        ui.horizontal(|ui| {
                            ui.label("Quality");
                            ui.add(egui::Slider::new(&mut self.save_quality, 1..=100));
                        });
                        // JPEG is always lossy — the pinned encoder cannot
                        // write a lossless JPEG and browsers cannot decode
                        // one — so the row displays the absence of the
                        // guarantee (locked OFF, like the PNG row's ON).
                        let mut lossless = false;
                        ui.horizontal(|ui| {
                            ui.add_enabled(false, egui::Checkbox::new(&mut lossless, "Lossless"));
                            ui.weak("(never — JPEG is always lossy)");
                        });
                    }
                }
                if file_io::FOLDER_PICKING_SUPPORTED {
                    ui.horizontal(|ui| {
                        ui.label("Folder");
                        let shown = self
                            .save_dir
                            .as_deref()
                            .map(|dir| dir.display().to_string())
                            .unwrap_or_else(|| "(not chosen)".to_string());
                        // Truncated so a long path cannot widen the dialog
                        // (a window's measured size never shrinks back).
                        ui.add(egui::Label::new(egui::RichText::new(shown).weak()).truncate());
                        if ui.button("Browse…").clicked()
                            && let Some(picked) =
                                file_io::pick_save_folder(self.save_dir.as_deref())
                        {
                            self.save_dir = Some(picked);
                        }
                    });
                }
                ui.weak(format!(
                    "Saving as: {}",
                    compose_file_name(&self.save_file_name, self.save_format)
                ));
                let (save_hit, cancel_hit) = dialog_button_row(ui, "Save", "Cancel");
                save = save_hit;
                cancel = cancel_hit;
            });
        if save {
            if self.commit_save_dialog() {
                self.show_save_dialog = false;
                forget_dialog_position(ctx, id);
            }
        } else if cancel || !open {
            self.show_save_dialog = false;
            forget_dialog_position(ctx, id);
        }
    }
}

#[cfg(test)]
mod tests;
