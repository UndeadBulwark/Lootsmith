//! The currency tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_currency(&mut self, ui: &mut egui::Ui) {
        let p = *crate::theme::active(ui.ctx());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.heading("Currency");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                crate::ui::widgets::checkbox(
                    ui,
                    &mut self.currency_show_unused,
                    "show unused slots",
                );
            });
        });
        ui.add_space(2.0);
        crate::ui::widgets::label_nums(
            ui,
            "Money · Eridium · Seraph crystals · Torgue tokens (slot 4). \
             Slot 3 and 5+ are unused.",
            p.fg,
            15.0,
        );
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
                            if !currency_slot_is_known(i) && !self.currency_show_unused {
                                continue;
                            }
                            ui.label(currency_name(i));
                            changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                    ui.add(egui::DragValue::new(value))
                                })
                            })
                            .changed();
                            ui.end_row();
                        }
                    });
            });
        if changed {
            self.dirty = true;
        }
    }
}
