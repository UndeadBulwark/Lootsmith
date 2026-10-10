//! The Ammo tab: per-pool current amount and SDU upgrade level.

use crate::app::*;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_ammo(&mut self, ui: &mut egui::Ui) {
        let p = *theme::active(ui.ctx());
        // ---- SDU table (Black Market upgrades) ----
        theme::section(ui, "Storage Deck Upgrades (Black Market)");
        ui.add_space(4.0);
        let mut max_all = false;
        let mut sdu_edit: Option<(usize, i32)> = None;
        let mut backpack_edit: Option<i32> = None;
        let mut bank_edit: Option<i32> = None;
        egui::Grid::new("sdu_grid")
            .num_columns(2)
            .striped(true)
            .spacing([20.0, 6.0])
            .show(ui, |ui| {
                ui.strong("Upgrade");
                ui.strong("Level");
                ui.end_row();
                for (i, (name, level)) in lootsmith_core::savegame::BLACK_MARKET_KEYS
                    .iter()
                    .zip(self.sdus.iter_mut())
                    .enumerate()
                {
                    crate::ui::widgets::label_nums(ui, format!("{} ({})", i + 1, name), p.fg, 15.0);
                    if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(level).range(0..=if i >= 7 { 9 } else { 127 }))
                    })
                    .changed()
                    {
                        sdu_edit = Some((i + 1, *level));
                    }
                    ui.end_row();
                }
            });
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.label("Backpack slots");
            if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.backpack_slots)
                        .range(12..=255)
                        .speed(1.0),
                )
            })
            .changed()
            {
                backpack_edit = Some(self.backpack_slots);
            }
            ui.label("Bank slots");
            if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                ui.add(
                    egui::DragValue::new(&mut self.bank_size)
                        .range(6..=255)
                        .speed(1.0),
                )
            })
            .changed()
            {
                bank_edit = Some(self.bank_size);
            }
            if ui
                .button("Max all SDUs")
                .on_hover_text("Ammo to 7; backpack 39; bank 24 (vanilla caps)")
                .clicked()
            {
                max_all = true;
            }
        });
        if let Some((pool, level)) = sdu_edit {
            self.set_sdu(pool, level);
        }
        if let Some(slots) = backpack_edit {
            self.set_backpack_slots(slots);
        }
        if let Some(slots) = bank_edit {
            self.set_bank_size(slots);
        }
        if max_all {
            self.max_all_sdus();
        }
        ui.add_space(10.0);

        // ---- ammo pools ----
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
        egui::ScrollArea::vertical().id_salt("ammo").show(ui, |ui| {
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
                        crate::ui::widgets::label_num(ui, format!("{}", row + 1), p.fg, 15.0);
                        crate::ui::widgets::label_nums(ui, pool.label(), p.fg, 15.0);
                        crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                            ui.add(egui::DragValue::new(&mut pool.amount).speed(1.0))
                        });
                        crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                            ui.add(egui::DragValue::new(&mut pool.upgrade_level).range(0..=10))
                        });
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
