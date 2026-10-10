//! The currency tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_currency(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.heading("Currency");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.currency_show_unused, "show unused slots");
            });
        });
        ui.add_space(2.0);
        ui.label("Index 0 is money. Other slots are eridium, seraph crystals, Torgue tokens, ...");
        ui.add_space(4.0);
        let mut changed = false;
        egui::ScrollArea::vertical()
            .id_salt("currency")
            .max_height(420.0)
            .show(ui, |ui| {
                egui::Grid::new("currency_grid")
                    .num_columns(2)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        for (i, value) in self.form.currency.iter_mut().enumerate() {
                            if i >= 4 && !self.currency_show_unused {
                                continue;
                            }
                            ui.label(if i >= 4 {
                                "Reserved / unused".to_string()
                            } else {
                                currency_name(i)
                            });
                            changed |= ui.add(egui::DragValue::new(value)).changed();
                            ui.end_row();
                        }
                    });
            });
        if changed {
            self.dirty = true;
        }
    }
}
