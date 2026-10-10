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

        ui.label(format!("Challenge resource entries: {}", self.challenges.len()));
        egui::CollapsingHeader::new("Challenge entries")
            .default_open(false)
            .show(ui, |ui| {
                for (i, challenge) in self.challenges.iter().enumerate() {
                    ui.label(format!("[{i:>3}] {challenge}"));
                }
            });

        // ---- challenge stats (field 15 blob) ----
        ui.add_space(8.0);
        theme::section(ui, "Challenge progress");
        ui.horizontal(|ui| {
            ui.label("Prestiges");
            if ui
                .add(egui::DragValue::new(&mut self.prestiges).range(0..=9999))
                .changed()
            {
                self.set_prestige(self.prestiges);
            }
            if ui
                .button("✕")
                .on_hover_text("Clear challenge search")
                .clicked()
            {
                self.challenge_filter.clear();
            }
            ui.add_sized(
                [220.0, 22.0],
                egui::TextEdit::singleline(&mut self.challenge_filter)
                    .hint_text("find by id…"),
            );
        });
        if self.challenge_stats.is_empty() {
            theme::muted(ui, "(no challenge-progress blob in this save)");
        } else {
            let needle = self.challenge_filter.trim().to_string();
            let mut edits: Vec<(u16, i32)> = Vec::new();
            egui::ScrollArea::vertical()
                .id_salt("challenge_stats")
                .max_height(260.0)
                .show(ui, |ui| {
                    egui::Grid::new("challenge_grid")
                        .num_columns(3)
                        .striped(true)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            ui.strong("ID");
                            ui.strong("Total");
                            ui.strong("Previous");
                            ui.end_row();
                            for entry in self.challenge_stats.iter_mut() {
                                if !needle.is_empty()
                                    && !entry.id.to_string().contains(&needle)
                                {
                                    continue;
                                }
                                ui.label(format!("{}{}", entry.id, if entry.previous_value > 0 { " (reset)" } else { "" }));
                                if ui
                                    .add(egui::DragValue::new(&mut entry.total_value).speed(0.5))
                                    .changed()
                                {
                                    edits.push((entry.id, entry.total_value));
                                }
                                ui.label(entry.previous_value.to_string());
                                ui.end_row();
                            }
                        });
                });
            for (id, total) in edits {
                if let Some(orig) = self
                    .challenge_stat_orig
                    .iter()
                    .find(|(oid, _)| *oid == id)
                    .map(|(_, v)| *v)
                {
                    if orig != total {
                        self.set_challenge_value(id, total);
                    }
                }
            }
        }
    }

}
