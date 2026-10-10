//! The vehicles tab (moved verbatim from the former single-file app.rs).

use eframe::egui;
use crate::app::*;

impl LootsmithApp {
    pub(crate) fn ui_vehicles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Vehicle steering mode");
            ui.add(egui::DragValue::new(&mut self.steering_mode));
        });
        if self.customizations.is_some() {
            ui.label("Skin names resolved from Customizations.json.");
        } else {
            ui.label("Load Customizations.json next to the Asset Library to pick skins by name.");
        }
        ui.separator();

        let customizations = self.customizations.clone();
        for row in self.vehicles.iter_mut() {
            ui.horizontal(|ui| {
                ui.strong(format!("{}:", row.usage));
                for (slot, value) in [("Skin 1", &mut row.skin1), ("Skin 2", &mut row.skin2)] {
                    let current = match (&customizations, value.as_str()) {
                        (Some(_), "None") => "None".to_string(),
                        (Some(c), key) => c.label_for(key),
                        (None, key) => key.to_string(),
                    };
                    egui::ComboBox::from_id_salt(format!("{slot}_{}", row.usage))
                        .selected_text(current)
                        .width(240.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(value == "None", "None").clicked() {
                                *value = "None".to_string();
                            }
                            if let Some(c) = &customizations {
                                for (key, name) in c.skin_options(&row.usage) {
                                    if ui.selectable_label(value == &key, &name).clicked() {
                                        *value = key;
                                    }
                                }
                            }
                        });
                }
            });
        }

        let dirty = self.vehicles_dirty();
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_enabled_ui(dirty, |ui| {
                if ui.button("Apply vehicle changes").clicked() {
                    self.apply_vehicles();
                }
            });
            if dirty {
                ui.label("(unsaved changes)");
            }
        });
    }

}
