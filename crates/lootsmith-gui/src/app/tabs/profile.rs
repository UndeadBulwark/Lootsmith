//! The profile tab (`profile.bin`: golden keys, badass rank & tokens,
//! stat bonuses, customization unlocks).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    /// Switch to the Profile page, suggesting a `profile.bin` next to the save.
    pub fn open_profile_page(&mut self) {
        self.tab = Tab::Profile;
        if self.profile_path.trim().is_empty() {
            self.profile_path = self.suggested_profile().unwrap_or_default();
        }
        self.load_profile();
    }

    pub(crate) fn ui_profile(&mut self, ui: &mut egui::Ui) {
        let p = *theme::active(ui.ctx());
        ui.add_space(10.0);
        // 1 = load from disk, 2 = save to disk
        let mut act: Option<u8> = None;
        card(ui, 620.0, |ui| {
            theme::section(ui, "Profile");
            theme::muted(
                ui,
                "Edits profile.bin (shared account data): golden keys, badass rank & \
                 tokens, stat bonuses, customization unlocks.",
            );
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                ui.label("File");
                ui.add_sized(
                    [360.0, 24.0],
                    egui::TextEdit::singleline(&mut self.profile_path),
                );
            });
            ui.add_space(6.0);
            if !self.profile_form.loaded {
                theme::muted(ui, "Profile not loaded yet — set the path and click Load.");
            }
            egui::Grid::new("profile_grid")
                .num_columns(2)
                .spacing([16.0, 6.0])
                .show(ui, |ui| {
                    ui.label("Golden keys");
                    crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.profile_form.golden_keys)
                                .range(0..=100_000),
                        );
                    });
                    ui.end_row();
                    ui.label("Badass rank");
                    crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.profile_form.badass_rank)
                                .range(0..=2_000_000_000),
                        );
                    });
                    ui.end_row();
                    ui.label("Badass tokens");
                    crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(
                            egui::DragValue::new(&mut self.profile_form.badass_tokens)
                                .range(0..=500),
                        );
                    });
                    ui.end_row();
                    ui.label("Unlock all customizations");
                    ui.checkbox(&mut self.profile_form.unlocked, "");
                    ui.end_row();
                });
            ui.add_space(6.0);
            theme::section(ui, "Badass bonus stats");
            egui::Grid::new("profile_stats")
                .num_columns(2)
                .striped(true)
                .spacing([12.0, 4.0])
                .show(ui, |ui| {
                    for (i, (name, value)) in STAT_NAMES
                        .iter()
                        .zip(self.profile_form.stats.iter_mut())
                        .enumerate()
                    {
                        crate::ui::widgets::label_nums(ui, format!("{i}: {name}"), p.fg, 14.0);
                        crate::ui::widgets::num_scope(ui, 14.0, |ui| {
                            ui.add(
                                egui::DragValue::new(value)
                                    .range(0.0..=9_975_792.3)
                                    .speed(1.0),
                            );
                        });
                        ui.end_row();
                    }
                });
            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui
                    .button("Load")
                    .on_hover_text("Re-read profile.bin from disk")
                    .clicked()
                {
                    act = Some(1);
                }
                if ui
                    .button("Save profile")
                    .on_hover_text("Write profile.bin (with a .bak backup)")
                    .clicked()
                {
                    act = Some(2);
                }
            });
        });
        if let Some(one) = act {
            if one == 1 {
                self.load_profile();
            } else {
                self.save_profile();
            }
        }
    }
}
