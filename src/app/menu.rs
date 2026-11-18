//! The top menu bar (File / Edit / Canvas / View / Help — the exact
//! strings and shortcut labels are SPEC §3.1) plus the MCP badge hook on
//! the right end of the bar.
//!
//! Menu items only DISPATCH: the actions themselves live where they belong
//! — document lifecycle in [`super::document`], undo/redo in the
//! orchestrator [`super`], zoom in [`super::camera`] — and the dialogs
//! they open in [`super::dialogs`]. Split off `app.rs` by the owner
//! request 2026-11-13; the View menu checkboxes write `App`'s window flags
//! directly (child-module privilege).

use eframe::egui;

use super::App;

impl App {
    pub(super) fn top_menu(&mut self, ui: &mut egui::Ui) {
        egui::Panel::top("top_menu").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui
                        .add(egui::Button::new("New…").shortcut_text("Ctrl+N"))
                        .clicked()
                    {
                        self.request_new();
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Open…").shortcut_text("Ctrl+O"))
                        .clicked()
                    {
                        self.request_open(ui.ctx());
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Save").shortcut_text("Ctrl+S"))
                        .clicked()
                    {
                        self.save(ui.ctx());
                        ui.close();
                    }
                    if ui
                        .add(egui::Button::new("Save As…").shortcut_text("Ctrl+Shift+S"))
                        .clicked()
                    {
                        self.save_as(ui.ctx());
                        ui.close();
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        ui.separator();
                        if ui.button("Exit").clicked() {
                            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    }
                });

                ui.menu_button("Edit", |ui| {
                    let can_undo = self.layers.can_undo();
                    if ui
                        .add_enabled(can_undo, egui::Button::new("Undo (Ctrl+Z)"))
                        .clicked()
                    {
                        self.undo();
                        ui.close();
                    }
                    let can_redo = self.layers.can_redo();
                    if ui
                        .add_enabled(can_redo, egui::Button::new("Redo (Ctrl+Y)"))
                        .clicked()
                    {
                        self.redo();
                        ui.close();
                    }
                });

                ui.menu_button("Canvas", |ui| {
                    if ui.button("Resize Canvas…").clicked() {
                        self.open_resize_dialog();
                        ui.close();
                    }
                });

                ui.menu_button("View", |ui| {
                    if ui.button("Zoom 100% (Ctrl+0)").clicked() {
                        self.reset_zoom();
                        ui.close();
                    }
                    if ui.button("Fit to Window").clicked() {
                        self.fit_to_window();
                        ui.close();
                    }
                    ui.separator();
                    ui.checkbox(&mut self.tools_window_open, "Tools");
                    ui.checkbox(&mut self.palette_window_open, "Palette");
                    ui.checkbox(&mut self.tree_window_open, "History Tree");
                    ui.checkbox(&mut self.history_window_open, "Tool Log");
                    ui.checkbox(&mut self.layers_window_open, "Layers");
                });

                ui.menu_button("Help", |ui| {
                    // Owner request 2026-11-13: the ITEM is just "About";
                    // the window it opens keeps the "About Russet" title.
                    if ui.button("About").clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                });

                // The theme switcher (owner request 2026-11-18): Dark /
                // Light / System — System follows the OS preference. The
                // choice applies immediately through `apply_theme` (egui
                // re-styles next pass; the icon atlas re-rasterizes).
                ui.menu_button(self.theme_label(), |ui| {
                    for (theme, name) in [
                        (egui::ThemePreference::Dark, "Dark"),
                        (egui::ThemePreference::Light, "Light"),
                        (egui::ThemePreference::System, "System (follow the OS)"),
                    ] {
                        if ui.radio(self.theme == theme, name).clicked() {
                            self.apply_theme(theme);
                            ui.close();
                        }
                    }
                });

                #[cfg(all(feature = "mcp", not(target_arch = "wasm32")))]
                self.mcp_badge(ui);
            });
        });
    }
}
