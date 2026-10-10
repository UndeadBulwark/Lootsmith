//! The Ammo tab: per-pool current amount and SDU upgrade level.

use eframe::egui;
use crate::app::*;

impl LootsmithApp {
    pub(crate) fn ui_ammo(&mut self, ui: &mut egui::Ui) {
        if self.ammo.is_empty() {
            theme::muted(ui, "No ammo pools in this save.");
            return;
        }
        ui.add_space(4.0);
        theme::section(ui, "Ammo pools");
        ui.add_space(2.0);
        // Pool edits collect (pool, amount, sdu) and apply afterwards.
        struct PoolEdit {
            pool: usize,
            amount: f32,
            sdu: i32,
        }
        let mut edits: Vec<PoolEdit> = Vec::new();
        egui::ScrollArea::vertical()
            .id_salt("ammo")
            .show(ui, |ui| {
                egui::Grid::new("ammo_grid")
                    .num_columns(3)
                    .striped(true)
                    .spacing([20.0, 6.0])
                    .show(ui, |ui| {
                        ui.strong("#");
                        ui.strong("Pool");
                        ui.strong("Amount");
                        ui.strong("SDU level");
                        ui.end_row();
                        for (i, pool) in self.ammo.iter_mut().enumerate() {
                            let row = i;
                            ui.label(format!("{}", row + 1));
                            ui.label(pool.label());
                            ui.add(egui::DragValue::new(&mut pool.amount).speed(1.0));
                            ui.add(egui::DragValue::new(&mut pool.upgrade_level).range(0..=10));
                            ui.end_row();
                            edits.push(PoolEdit {
                                pool: row,
                                amount: pool.amount,
                                sdu: pool.upgrade_level,
                            });
                        }
                    });
            });
        for edit in edits {
            if let Some(&orig) = self.ammo_orig.get(edit.pool) {
                if orig != (edit.amount, edit.sdu) {
                    self.set_ammo(edit.pool + 1, Some(edit.amount), Some(edit.sdu));
                }
            }
        }
    }
}
