//! The missions tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_missions(&mut self, ui: &mut egui::Ui) {
        let p = *theme::active(ui.ctx());
        let missions_data = self.missions_data.clone();
        let name_for = |number: i32| match &missions_data {
            Some(m) => m
                .label_for_number(number)
                .unwrap_or_else(|| format!("(unknown {number})")),
            None => number.to_string(),
        };

        ui.horizontal(|ui| {
            ui.label("Plot mission");
            crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                ui.add(egui::DragValue::new(&mut self.plot_mission))
            });
            crate::ui::widgets::label_nums(ui, name_for(self.plot_mission), p.fg, 15.0);
        });
        ui.horizontal(|ui| {
            ui.label("Active mission");
            crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                ui.add(egui::DragValue::new(&mut self.active_mission))
            });
            crate::ui::widgets::label_nums(ui, name_for(self.active_mission), p.fg, 15.0);
        });
        ui.separator();

        crate::ui::widgets::label_num(
            ui,
            format!("Missions ({})", self.mission_rows.len()),
            p.fg,
            15.0,
        );
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
                            crate::ui::widgets::label_num(
                                ui,
                                format!("P{}", row.playthrough + 1),
                                p.fg,
                                15.0,
                            );
                            crate::ui::widgets::label_nums(ui, label, p.fg, 15.0);
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

        crate::ui::widgets::label_num(
            ui,
            format!("Challenge resource entries: {}", self.challenges.len()),
            p.fg,
            15.0,
        );
        egui::CollapsingHeader::new("Challenge entries")
            .default_open(false)
            .show(ui, |ui| {
                for (i, challenge) in self.challenges.iter().enumerate() {
                    crate::ui::widgets::label_nums(ui, format!("[{i:>3}] {challenge}"), p.fg, 15.0);
                }
            });

        // ---- challenge stats (field 15 blob) ----
        ui.add_space(8.0);
        theme::section(ui, "Challenge progress");
        ui.horizontal(|ui| {
            ui.label("Prestiges");
            if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                ui.add(egui::DragValue::new(&mut self.prestiges).range(0..=9999))
            })
            .changed()
            {
                self.set_prestige(self.prestiges);
            }
            if crate::ui::widgets::icon_button(
                ui,
                crate::ui::pixel::Icon::Close,
                p.fg,
                p.pan,
                24.0,
                "Clear challenge search",
            )
            .clicked()
            {
                self.challenge_filter.clear();
            }
            ui.add_sized(
                [220.0, 22.0],
                egui::TextEdit::singleline(&mut self.challenge_filter).hint_text("find by id…"),
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
                                if !needle.is_empty() && !entry.id.to_string().contains(&needle) {
                                    continue;
                                }
                                crate::ui::widgets::label_nums(
                                    ui,
                                    format!(
                                        "{}{}",
                                        entry.id,
                                        if entry.previous_value > 0 {
                                            " (reset)"
                                        } else {
                                            ""
                                        }
                                    ),
                                    p.fg,
                                    15.0,
                                );
                                if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                    ui.add(egui::DragValue::new(&mut entry.total_value).speed(0.5))
                                })
                                .changed()
                                {
                                    edits.push((entry.id, entry.total_value));
                                }
                                crate::ui::widgets::label_num(
                                    ui,
                                    entry.previous_value.to_string(),
                                    p.fg,
                                    15.0,
                                );
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
