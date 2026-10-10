//! The general tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_general(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        card(ui, 560.0, |ui| {
            theme::section(ui, "General");
            egui::Grid::new("general")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    let mut changed = false;
                    ui.label("Level");
                    ui.horizontal(|ui| {
                        changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                            ui.add(
                                egui::DragValue::new(&mut self.form.level)
                                    .range(1..=80)
                                    .speed(0.2),
                            )
                        })
                        .changed();
                        if ui
                            .button("Max")
                            .on_hover_text("Vanilla cap: level 72")
                            .clicked()
                        {
                            self.form.level = 72;
                            changed = true;
                        }
                        if ui
                            .button("OP max")
                            .on_hover_text("UCP/Overpower cap: level 80")
                            .clicked()
                        {
                            self.form.level = 80;
                            changed = true;
                        }
                    });
                    ui.end_row();
                    ui.label("Experience");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.exp_points))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("General skill points");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.general_skill_points))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("Specialist skill points");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.specialist_skill_points))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("Playthroughs");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.playthroughs))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("Total play time (s)");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.total_play_time))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("Badass mode");
                    changed |= ui.checkbox(&mut self.form.badass, "").changed();
                    ui.end_row();
                    ui.label("Golden keys");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.golden_keys))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("Bank slots");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.bank_slots))
                    })
                    .changed();
                    ui.end_row();
                    if changed {
                        self.dirty = true;
                    }
                });
        });
    }
}
