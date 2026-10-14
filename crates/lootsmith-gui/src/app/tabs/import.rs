//! The import tab (copy sections from another save into the open save).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    /// Switch to the Import page, defaulting the source path to the open save.
    pub fn open_import_page(&mut self) {
        self.tab = Tab::Import;
        if self.import_path.trim().is_empty() {
            self.import_path = self.path.clone();
        }
    }

    pub(crate) fn ui_import(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let mut do_import = false;
        card(ui, 620.0, |ui| {
            theme::section(ui, "Import sections");
            theme::muted(ui, "Copy sections from another save into the open save.");
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("Save");
                ui.add_sized(
                    [360.0, 24.0],
                    egui::TextEdit::singleline(&mut self.import_path),
                );
            });
            ui.add_space(8.0);
            egui::Grid::new("import_sel")
                .num_columns(2)
                .spacing([16.0, 4.0])
                .show(ui, |ui| {
                    for (flag, label) in [
                        (&mut self.import_sel.skills, "Skills (SkillData)"),
                        (
                            &mut self.import_sel.missions,
                            "Missions (playthroughs, statuses)",
                        ),
                        (&mut self.import_sel.challenges, "Challenges"),
                        (&mut self.import_sel.travel, "Fast travel stations"),
                        (&mut self.import_sel.vehicles, "Vehicle customizations"),
                        (&mut self.import_sel.dlc, "DLC expansion data"),
                        (&mut self.import_sel.bank_slots, "Bank slots (contents)"),
                    ] {
                        ui.checkbox(flag, "");
                        ui.label(label);
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            if ui
                .button("Import")
                .on_hover_text("Replace the selected sections in the open save")
                .clicked()
            {
                do_import = true;
            }
        });
        if do_import {
            self.do_import();
        }
    }
}
