//! The missions tab (moved verbatim from the former single-file app.rs).

use eframe::egui;
use crate::app::*;

impl LootsmithApp {
    pub(crate) fn ui_missions(&mut self, ui: &mut egui::Ui) {
        let missions_data = self.missions_data.clone();
        let name_for = |number: i32| match &missions_data {
            Some(m) => m
                .label_for_number(number)
                .unwrap_or_else(|| format!("(unknown {number})")),
            None => number.to_string(),
        };

        ui.horizontal(|ui| {
            ui.label("Plot mission");
            ui.add(egui::DragValue::new(&mut self.plot_mission));
            ui.label(name_for(self.plot_mission));
        });
        ui.horizontal(|ui| {
            ui.label("Active mission");
            ui.add(egui::DragValue::new(&mut self.active_mission));
            ui.label(name_for(self.active_mission));
        });
        ui.separator();

        ui.label(format!("Missions ({})", self.mission_rows.len()));
        egui::ScrollArea::vertical()
            .id_salt("missions")
            .max_height(280.0)
            .show(ui, |ui| {
                egui::Grid::new("missions_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        for row in self.mission_rows.iter_mut() {
                            let label = match &missions_data {
                                Some(m) => m.label_for(&row.key),
                                None => row.key.clone(),
                            };
                            ui.label(format!("P{}", row.playthrough + 1));
                            ui.label(label);
                            egui::ComboBox::from_id_salt(format!(
                                "mission_status_{}_{}",
                                row.playthrough, row.index
                            ))
                            .selected_text(mission_status_name(row.status))
                            .width(200.0)
                            .show_ui(ui, |ui| {
                                for (value, name) in MISSION_STATUSES.iter().enumerate() {
                                    let selected = row.status == value as i32;
                                    if ui.selectable_label(selected, *name).clicked() {
                                        row.status = value as i32;
                                    }
                                }
                            });
                            ui.end_row();
                        }
                    });
            });

        let dirty = self.missions_dirty();
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_enabled_ui(dirty, |ui| {
                if ui.button("Apply mission changes").clicked() {
                    self.apply_missions();
                }
            });
            if dirty {
                ui.label("(unsaved changes)");
            }
        });
        ui.separator();

        ui.label(format!("Challenges ({})", self.challenges.len()));
        egui::ScrollArea::vertical()
            .id_salt("challenges")
            .max_height(140.0)
            .show(ui, |ui| {
                for (i, challenge) in self.challenges.iter().enumerate() {
                    ui.label(format!("[{i:>3}] {challenge}"));
                }
            });
    }

}
