//! The travel tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_travel(&mut self, ui: &mut egui::Ui) {
        let stations = self.stations.clone();
        let Some(stations) = stations else {
            theme::muted(
                ui,
                "Load Travel Stations.json (next to the Asset Library) to list every station.",
            );
            return;
        };
        let titles = self.dlc_titles.clone();

        // Group every known station: Base Game first, DLC chapters after.
        let mut groups: Vec<(String, Vec<(String, String)>)> =
            vec![("Base Game".to_string(), Vec::new())];
        for station in stations.all() {
            let display = stations.label_for(&station.resource_name);
            match station.dlc_expansion.as_deref() {
                None | Some("") => groups[0].1.push((station.resource_name.clone(), display)),
                Some(expansion) => {
                    let title = titles
                        .as_ref()
                        .and_then(|t| t.title(expansion))
                        .unwrap_or("Unknown");
                    if let Some(group) = groups.iter_mut().find(|(name, _)| name == title) {
                        group.1.push((station.resource_name.clone(), display));
                    } else {
                        groups.push((
                            title.to_string(),
                            vec![(station.resource_name.clone(), display)],
                        ));
                    }
                }
            }
        }
        let p = *theme::active(ui.ctx());
        let visited = format!("visited: {} / {}", self.travel.len(), stations.len());
        crate::ui::widgets::label_nums(ui, &visited, p.dim, 15.0);
        ui.add_space(4.0);

        // Apply deferred checkbox / group edits after rendering.
        let mut toggles: Vec<(String, bool)> = Vec::new();
        let mut group_toggles: Vec<(String, bool)> = Vec::new();
        let mut set_last: Option<String> = None;
        let last = self.travel_last.clone();
        egui::ScrollArea::vertical()
            .id_salt("travel")
            .show(ui, |ui| {
                for (title, rows) in &groups {
                    ui.horizontal(|ui| {
                        ui.strong(title);
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            // with_layout consumes its row; buttons toggle the group.
                            if ui
                                .small_button("Uncheck all")
                                .on_hover_text(format!("clear every {title} station"))
                                .clicked()
                            {
                                group_toggles.push((title.clone(), false));
                            }
                            if ui
                                .small_button("Check all")
                                .on_hover_text(format!("visit every {title} station"))
                                .clicked()
                            {
                                group_toggles.push((title.clone(), true));
                            }
                        });
                    });
                    for (resource, display) in rows {
                        let mut checked = self.travel.contains(resource);
                        let marker = if last.as_deref() == Some(resource.as_str()) {
                            "★ "
                        } else {
                            ""
                        };
                        ui.horizontal(|ui| {
                            if crate::ui::widgets::checkbox(
                                ui,
                                &mut checked,
                                &format!("{marker}{display}"),
                            )
                            .changed()
                            {
                                toggles.push((resource.clone(), checked));
                            }
                            if checked
                                && ui
                                    .button("☆ last")
                                    .on_hover_text("Make this the current station")
                                    .clicked()
                            {
                                set_last = Some(resource.clone());
                            }
                        });
                    }
                }
            });
        for (resource, on) in toggles {
            if on {
                self.add_teleporter(&resource);
            } else if let Some(index) = self.travel.iter().position(|name| *name == resource) {
                self.remove_teleporter(index);
            }
        }
        for (title, on) in group_toggles {
            let resources: Vec<String> = groups
                .iter()
                .find(|(name, _)| *name == title)
                .map(|(_, rows)| rows.iter().map(|(r, _)| r.clone()).collect())
                .unwrap_or_default();
            for resource in resources {
                if on {
                    self.add_teleporter(&resource);
                } else if let Some(index) = self.travel.iter().position(|name| *name == resource) {
                    self.remove_teleporter(index);
                }
            }
        }
        if let Some(name) = set_last {
            self.set_last_teleporter(&name);
        }
    }
}
