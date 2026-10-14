//! The character tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme;
use eframe::egui;

/// A segmented pill: `Primary` when selected, `Normal` otherwise. Card-safe in
/// both themes — `toggle_chip`'s unselected text is the toolbar's light `onw`,
/// which all but vanishes on the light card.
fn seg_pill(ui: &mut egui::Ui, label: &str, selected: bool, width: f32) -> bool {
    let (r, _) = ui.allocate_exact_size(egui::vec2(width, 20.0), egui::Sense::click());
    let kind = if selected {
        crate::ui::widgets::ButtonKind::Primary
    } else {
        crate::ui::widgets::ButtonKind::Normal
    };
    crate::ui::widgets::button(ui, r, label, kind).clicked()
}

impl LootsmithApp {
    pub fn change_class(&mut self, player_class: &str) {
        if self.save_game.is_none() {
            return;
        }
        // DLC class package ids, resolved from `Player Classes.json` ->
        // `Downloadable Packages.json`: Krieg (`GD_LilacPackageDef`) = 6,
        // Gaige (`GD_TulipPackageDef`) = 2. (Verified against the dumps.)
        let is_dlc = player_class.contains("Lilac") || player_class.contains("Tulip");
        let package = if player_class.contains("Lilac") {
            6 // Psycho Pack
        } else if player_class.contains("Tulip") {
            2 // Mechromancer
        } else {
            0
        };
        let refund: i32 = self.skills.iter().map(|s| s.grade.max(0)).sum();
        {
            let Some(save_game) = self.save_game.as_mut() else {
                return;
            };
            save_game.set_str(1, player_class);
            save_game.set_bool(44, is_dlc);
            save_game.set_i32(45, package);
        }
        self.form.general_skill_points += refund;

        // The new class has a different skill list: rebuild it from the class's
        // canonical objects (all grade 0) so its tree is editable. Refund every
        // spent point so the header is consistent with the fresh list.
        match crate::skills::for_player_class(player_class) {
            Some(class) => self.rebuild_skill_list(class),
            None => {
                for row in self.skills.iter_mut() {
                    row.grade = 0;
                    row.points = 0;
                }
            }
        }
        self.apply_skills();

        self.read_info.player_class = player_class.to_string();
        self.dirty = true;
        self.status = format!(
            "class changed to {} — skill list rebuilt, {refund} point(s) refunded",
            crate::skills::class_display_name(player_class)
        );
    }

    /// Replace the save's `SkillData` list with `class`'s canonical object list
    /// (all rows grade 0) and refresh the cached rows. Used when the class changes
    /// and to repair a save whose skill list belongs to another class.
    pub fn rebuild_skill_list(&mut self, class: &crate::skills::SkillClass) {
        let order = class.save_order();
        {
            let Some(save_game) = self.save_game.as_mut() else {
                return;
            };
            save_game.set_skill_list(&order);
        }
        self.refresh_skills();
        self.dirty = true;
    }

    /// How many one-off level challenges (Digistruct Peak OP levels) the
    /// character has completed (field 40).
    pub fn overpower_level(&self) -> i32 {
        self.save_game
            .as_ref()
            .map(|s| s.overpower_level())
            .unwrap_or(0)
    }

    /// Set the OP level (0-10).
    pub fn set_overpower(&mut self, level: i32) -> bool {
        let Some(save_game) = self.save_game.as_mut() else {
            return false;
        };
        if !save_game.set_overpower_level(level) {
            return false;
        }
        self.dirty = true;
        self.status = format!("overpower level set to {level}");
        true
    }

    pub(crate) fn ui_character(&mut self, ui: &mut egui::Ui) {
        let p = *theme::active(ui.ctx());
        ui.add_space(10.0);
        let customizations = self.customizations.clone();
        let usage = lootsmith_core::usage_for_player_class(&self.read_info.player_class);
        let mut changed = false;
        let mut change_class: Option<String> = None;
        let mut set_op: Option<i32> = None;
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
                                    // Re-picking the current class must not rebuild
                                    // (and zero) the skill list.
                                    if ui.selectable_label(selected, label).clicked() && !selected {
                                        change_class = Some((*id).to_string());
                                    }
                                }
                            });
                    });
                    ui.end_row();
                    ui.label("Level");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                            ui.add(egui::DragValue::new(&mut self.form.level))
                        })
                    })
                    .changed();
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
                    crate::ui::widgets::label_num(
                        ui,
                        self.read_info.save_id.to_string(),
                        p.fg,
                        15.0,
                    );
                    ui.end_row();
                    ui.label("GUID");
                    crate::ui::widgets::label_nums(ui, &self.read_info.guid, p.fg, 15.0);
                    ui.end_row();
                    ui.label("Last saved");
                    crate::ui::widgets::label_nums(ui, &self.read_info.last_saved, p.fg, 15.0);
                    ui.end_row();
                });
            // Merged in from the former "General" page.
            theme::section(ui, "Progress");
            egui::Grid::new("character_progress")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Playthrough");
                    ui.horizontal(|ui| {
                        for (value, label) in [(0, "None"), (1, "NVHM"), (2, "TVHM"), (3, "UVHM")] {
                            let selected = self.form.playthroughs == value;
                            if seg_pill(ui, label, selected, 52.0) && !selected {
                                self.form.playthroughs = value;
                                changed = true;
                            }
                        }
                    });
                    ui.end_row();
                    ui.label("Total play time (s)");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.total_play_time))
                    })
                    .changed();
                    ui.end_row();
                    ui.label("Badass mode");
                    changed |=
                        crate::ui::widgets::checkbox(ui, &mut self.form.badass, "").changed();
                    ui.end_row();
                    ui.label("Bank slots");
                    changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut self.form.bank_slots))
                    })
                    .changed();
                    ui.end_row();
                });
            ui.add_space(8.0);
            ui.label("Overpower level");
            let op = self.overpower_level();
            ui.horizontal(|ui| {
                for level in 0..=10 {
                    let selected = level == op;
                    if seg_pill(ui, &level.to_string(), selected, 26.0) && !selected {
                        set_op = Some(level);
                    }
                }
            });
            theme::faint(
                ui,
                "OP levels are earned in-game (Digistruct Peak); picking a higher one adds a completion.",
            );
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
        if let Some(level) = set_op {
            self.set_overpower(level);
        }
    }
}
