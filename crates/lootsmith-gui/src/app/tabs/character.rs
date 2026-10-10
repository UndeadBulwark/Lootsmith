//! The character tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    pub fn change_class(&mut self, player_class: &str) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let is_dlc = player_class.contains("Lilac") || player_class.contains("Tulip");
        let package = if player_class.contains("Lilac") {
            6 // Psycho Pack
        } else if player_class.contains("Tulip") {
            2 // Mechromancer
        } else {
            0
        };
        save_game.set_str(1, player_class);
        save_game.set_bool(44, is_dlc);
        save_game.set_i32(45, package);
        self.read_info.player_class = player_class.to_string();
        self.dirty = true;
        self.status = format!(
            "class changed to {} — skills and the action skill may need re-spending",
            crate::skills::class_display_name(player_class)
        );
    }

    /// How many one-off level challenges (Digistruct Peak OP levels) the
    /// character has completed (field 40).
    pub fn overpower_level(&self) -> i32 {
        self.save_game
            .as_ref()
            .map(|s| s.message.count(40) as i32)
            .unwrap_or(0)
    }

    /// Clear all completed one-off level challenges (resets OP level to 0).
    pub fn reset_overpower(&mut self) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        save_game.message.remove_all(40);
        self.dirty = true;
        self.status = "overpower levels reset".into();
    }

    pub(crate) fn ui_character(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let customizations = self.customizations.clone();
        let usage = lootsmith_core::usage_for_player_class(&self.read_info.player_class);
        let mut changed = false;
        let mut change_class: Option<String> = None;
        let mut reset_op = false;
        card(ui, 620.0, |ui| {
            theme::section(ui, "Character");
            egui::Grid::new("character")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Name");
                    changed |= ui
                        .add_sized(
                            [240.0, 22.0],
                            egui::TextEdit::singleline(&mut self.form.character_name),
                        )
                        .changed();
                    ui.end_row();
                    ui.label("Class");
                    let classes = crate::skills::class_ids();
                    let current = crate::skills::class_display_name(&self.read_info.player_class);
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("class_choice")
                            .selected_text(&current)
                            .width(220.0)
                            .show_ui(ui, |ui| {
                                for id in &classes {
                                    let label = crate::skills::class_display_name(id);
                                    let selected = *id == self.read_info.player_class;
                                    if ui.selectable_label(selected, label).clicked() {
                                        change_class = Some((*id).to_string());
                                    }
                                }
                            });
                        if self.overpower_level() > 0
                            && ui
                                .button("Reset OP")
                                .on_hover_text("Clear all Digistruct Peak overpower completions")
                                .clicked()
                        {
                            reset_op = true;
                        }
                    });
                    ui.end_row();
                    ui.label("Overpower level");
                    let op = self.overpower_level();
                    ui.label(if op > 0 {
                        op.to_string()
                    } else {
                        "—".to_string()
                    });
                    ui.end_row();
                    ui.label("Level");
                    changed |= ui.add(egui::DragValue::new(&mut self.form.level)).changed();
                    ui.end_row();
                    ui.label("Experience");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.form.exp_points))
                        .changed();
                    ui.end_row();
                    ui.label("General skill points");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.form.general_skill_points))
                        .changed();
                    ui.end_row();
                    ui.label("Specialist skill points");
                    changed |= ui
                        .add(egui::DragValue::new(&mut self.form.specialist_skill_points))
                        .changed();
                    ui.end_row();
                    ui.label("Head");
                    customization_combo(
                        ui,
                        &customizations,
                        usage,
                        "Head",
                        &mut self.form.head,
                        "char_head",
                    );
                    ui.end_row();
                    ui.label("Skin");
                    customization_combo(
                        ui,
                        &customizations,
                        usage,
                        "Skin",
                        &mut self.form.skin,
                        "char_skin",
                    );
                    ui.end_row();
                    ui.label("Save id");
                    ui.label(self.read_info.save_id.to_string());
                    ui.end_row();
                    ui.label("GUID");
                    ui.label(&self.read_info.guid);
                    ui.end_row();
                    ui.label("Last saved");
                    ui.label(&self.read_info.last_saved);
                    ui.end_row();
                });
        });
        if changed {
            self.dirty = true;
        }
        if customizations.is_none() {
            ui.add_space(6.0);
            theme::muted(
                ui,
                "Load the game-data folder to pick heads and skins by name.",
            );
        }
        if let Some(class) = change_class {
            self.change_class(&class);
        }
        if reset_op {
            self.reset_overpower();
        }
    }
}
