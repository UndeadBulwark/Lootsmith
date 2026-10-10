//! The skills tab (moved verbatim from the former single-file app.rs).

use eframe::egui;
use crate::theme::{self, Tokens};
use crate::app::*;

impl LootsmithApp {
    pub(crate) fn ui_skills(&mut self, ui: &mut egui::Ui) {
        let Some(class) = crate::skills::for_player_class(&self.read_info.player_class) else {
            theme::muted(ui, "Unknown player class — the skill tree is unavailable.");
            return;
        };

        // Snapshot grades and index the save's skills by normalized leaf name.
        let grades: Vec<i32> = self.skills.iter().map(|r| r.grade).collect();
        let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, row) in self.skills.iter().enumerate() {
            let leaf = row.name.rsplit('.').next().unwrap_or(&row.name);
            index.insert(crate::skills::norm(leaf), i);
        }
        let grade_of = |name: &str| index.get(&crate::skills::norm(name)).copied();

        let spent: i32 = grades.iter().map(|g| (*g).max(0)).sum();
        let available = self.form.general_skill_points;
        let portrait = self.skill_texture(ui.ctx(), &class.portrait);

        // Decode/cache every icon this class needs, once.
        let mut textures: std::collections::HashMap<String, Option<egui::TextureHandle>> =
            std::collections::HashMap::new();
        for tree in &class.trees {
            for tier in &tree.tiers {
                for node in &tier.skills {
                    if !node.icon.is_empty() && !textures.contains_key(&node.icon) {
                        let handle = self.skill_texture(ui.ctx(), &node.icon);
                        textures.insert(node.icon.clone(), handle);
                    }
                }
            }
        }

        // Header.
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("BORDERLANDS 2 SKILL CALCULATOR")
                        .strong()
                        .size(18.0)
                        .color(Tokens::ACCENT),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(&class.name)
                        .strong()
                        .size(28.0)
                        .color(Tokens::ACCENT),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "LEVEL {} {}",
                        self.form.level,
                        class.class_name.to_uppercase()
                    ))
                    .size(14.0),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("Spent {spent}     Available {available}"))
                        .color(Tokens::TEXT_DIM)
                        .size(12.0),
                );
                theme::faint(ui, "Left-click adds a rank, right-click removes.");
                theme::faint(ui, "A tier unlocks every 5 points invested.");
            });
        });
        ui.add_space(6.0);

        let mut edit: Option<(usize, i32)> = None;

        egui::ScrollArea::vertical()
            .id_salt("skill_trees")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    if let Some(texture) = &portrait {
                        ui.add(
                            egui::Image::new(texture)
                                .max_height(430.0)
                                .max_width(300.0),
                        );
                        ui.add_space(16.0);
                    }
                    for tree in class.trees.iter() {
                        tree_card(ui, tree, &grades, &grade_of, &textures, &mut edit);
                        ui.add_space(20.0);
                    }
                });

                // Save skills not matched to a tree node (e.g. the action skill).
                let matched: std::collections::HashSet<usize> = class
                    .trees
                    .iter()
                    .flat_map(|t| &t.tiers)
                    .flat_map(|t| &t.skills)
                    .filter_map(|n| grade_of(&n.skill))
                    .collect();
                let extras: Vec<usize> = (0..grades.len())
                    .filter(|i| !matched.contains(i))
                    .collect();
                if !extras.is_empty() {
                    ui.add_space(10.0);
                    theme::section(ui, "Other skills");
                    ui.horizontal_wrapped(|ui| {
                        for i in extras {
                            let leaf = self.skills[i]
                                .name
                                .rsplit('.')
                                .next()
                                .unwrap_or("?")
                                .to_string();
                            let display = if leaf.starts_with("Skill_") {
                                "Action Skill".to_string()
                            } else {
                                leaf
                            };
                            let grade = grades[i];
                            let label =
                                egui::RichText::new(format!("{display}   {grade}/1")).size(12.0);
                            let label = if grade > 0 {
                                label.color(Tokens::ACCENT)
                            } else {
                                label
                            };
                            if ui
                                .button(label)
                                .on_hover_text(&self.skills[i].name)
                                .clicked()
                                && grade < 1
                            {
                                edit = Some((i, grade + 1));
                            }
                        }
                    });
                }
            });

        // Apply a click: set the grade and keep the point budget consistent.
        if let Some((i, new_grade)) = edit {
            let old = self.skills[i].grade;
            if new_grade != old {
                self.skills[i].grade = new_grade;
                if new_grade > old && self.form.general_skill_points > 0 {
                    self.form.general_skill_points -= 1;
                } else if new_grade < old {
                    self.form.general_skill_points += 1;
                }
                self.apply_skills();
            }
        }
    }

}
