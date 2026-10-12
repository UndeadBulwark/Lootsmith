//! The World tab: region game stages, world discoveries, lockouts, DLC
//! expansion state, mementos.

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_world(&mut self, ui: &mut egui::Ui) {
        let p = *theme::active(ui.ctx());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            if ui
                .button("Mark all explored")
                .on_hover_text("Every world discovery becomes uncovered")
                .clicked()
            {
                self.mark_all_discovered();
            }
            if ui
                .button("Clear lockouts")
                .on_hover_text("Zero all lockout timers (raid bosses farmable again)")
                .clicked()
            {
                self.clear_all_lockouts();
            }
            if ui
                .button("Clear mementos")
                .on_hover_text("Drop weapon/item mementos (they re-add removed items)")
                .clicked()
            {
                self.clear_mementos();
            }
        });
        ui.add_space(8.0);

        theme::section(ui, "World discoveries");
        let total = self.world_discoveries.len();
        crate::ui::widgets::label_nums(
            ui,
            format!(
                "uncovered: {} / {total}",
                self.world_discoveries
                    .iter()
                    .filter(|d| d.uncovered)
                    .count()
            ),
            p.dim,
            15.0,
        );
        ui.add_space(2.0);
        let mut discovery_edit: Option<(usize, bool)> = None;
        egui::ScrollArea::vertical()
            .id_salt("world_discoveries")
            .show(ui, |ui| {
                egui::Grid::new("discoveries")
                    .num_columns(2)
                    .striped(true)
                    .spacing([12.0, 3.0])
                    .show(ui, |ui| {
                        for (i, discovery) in self.world_discoveries.iter_mut().enumerate() {
                            crate::ui::widgets::label_nums(ui, format!("#{}", i + 1), p.fg, 15.0);
                            if ui
                                .checkbox(
                                    &mut discovery.uncovered,
                                    crate::ui::widgets::text_with_nums(&discovery.name, p.fg, 15.0),
                                )
                                .changed()
                            {
                                discovery_edit = Some((i + 1, discovery.uncovered));
                            }
                            ui.end_row();
                        }
                    });
            });
        if let Some((index, uncovered)) = discovery_edit {
            self.set_discovery(index, uncovered);
        }

        ui.add_space(8.0);
        theme::section(ui, "Region game stages");
        let mut stage_edit: Option<(usize, i32)> = None;
        egui::Grid::new("regions")
            .num_columns(3)
            .striped(true)
            .spacing([12.0, 3.0])
            .show(ui, |ui| {
                for (i, region) in self.world_regions.iter_mut().enumerate() {
                    crate::ui::widgets::label_nums(ui, format!("#{}", i + 1), p.fg, 15.0);
                    crate::ui::widgets::label_nums(ui, &region.region, p.fg, 15.0);
                    if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add(egui::DragValue::new(&mut region.game_stage).range(1..=80))
                    })
                    .changed()
                    {
                        stage_edit = Some((i + 1, region.game_stage));
                    }
                    ui.end_row();
                }
            });
        if let Some((pool, stage)) = stage_edit {
            self.set_world_stage(pool, stage);
        }

        ui.add_space(8.0);
        theme::section(ui, "Lockouts");
        if self.world_lockouts.is_empty() {
            theme::muted(ui, "(no lockout timers)");
        } else {
            for lockout in &self.world_lockouts {
                crate::ui::widgets::label_nums(ui, lockout.label(), p.dim, 15.0);
            }
        }

        ui.add_space(8.0);
        theme::section(ui, "DLC expansions");
        if self.world_expansions.is_empty() {
            theme::muted(ui, "(no DLC expansion data)");
        } else {
            for (i, expansion) in self.world_expansions.iter().enumerate() {
                crate::ui::widgets::label_nums(
                    ui,
                    format!(
                        "#{:<2} tag={:<3} data {} bytes",
                        i + 1,
                        expansion.tag,
                        expansion.data.len()
                    ),
                    p.fg,
                    15.0,
                );
            }
        }

        ui.add_space(8.0);
        theme::section(ui, "Mementos");
        crate::ui::widgets::label_nums(
            ui,
            format!(
                "weapons: {}, items: {} — \"Clear mementos\" above drops them",
                self.mementos.0, self.mementos.1
            ),
            p.fg,
            15.0,
        );
    }
}
