//! The backpack tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_container(&mut self, ui: &mut egui::Ui, bank: bool) {
        let p = *theme::active(ui.ctx());
        let mut item_action: Option<ItemAction> = None;
        let mut clicked: Option<(Container, usize)> = None;
        let mut apply = false;
        let mut move_to: Option<Container> = None;
        let mut remove = false;

        // ---- action bar ----
        egui::Panel::top("bag_actions")
            .frame(
                egui::Frame::default()
                    .fill(p.bg)
                    .inner_margin(egui::Margin::symmetric(10, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("New weapon").clicked() {
                        item_action = Some(ItemAction::NewWeapon);
                    }
                    if ui.button("New item").clicked() {
                        item_action = Some(ItemAction::NewItem);
                    }
                    ui.add_enabled_ui(self.selected.is_some(), |ui| {
                        if ui.button("Duplicate").clicked() {
                            item_action = Some(ItemAction::Duplicate);
                        }
                        if ui.button("Copy code").clicked() {
                            item_action = Some(ItemAction::CopyCode);
                        }
                    });
                    toolbar_sep(ui);
                    if ui.button("Sync levels").clicked() {
                        item_action = Some(ItemAction::SyncAll);
                    }
                    if ui.button("Sync equipped").clicked() {
                        item_action = Some(ItemAction::SyncEquipped);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_sized(
                            [220.0, 24.0],
                            egui::TextEdit::singleline(&mut self.filter).hint_text("filter items…"),
                        );
                    });
                });
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("Item code").color(p.dim).size(11.5));
                    crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                        ui.add_sized(
                            [520.0, 24.0],
                            egui::TextEdit::singleline(&mut self.item_code)
                                .hint_text("paste BL2(...) codes here"),
                        )
                    });
                    if ui.button("Insert code(s)").clicked() {
                        item_action = Some(ItemAction::InsertCodes);
                    }
                });
            });

        if let Some(action) = item_action {
            match action {
                ItemAction::NewWeapon => self.new_item(bank, true),
                ItemAction::NewItem => self.new_item(bank, false),
                ItemAction::Duplicate => self.duplicate_selected(),
                ItemAction::CopyCode => {
                    self.copy_selected_code(ui.ctx());
                }
                ItemAction::SyncAll => self.sync_levels(false),
                ItemAction::SyncEquipped => self.sync_levels(true),
                ItemAction::InsertCodes => self.insert_codes(bank),
            }
        }

        let rows = if bank {
            self.bank.clone()
        } else {
            self.backpack.clone()
        };
        let needle = self.filter.trim().to_lowercase();
        let visible: Vec<Row> = rows
            .into_iter()
            .filter(|row| {
                needle.is_empty()
                    || row.kind.to_lowercase().contains(&needle)
                    || row.id.to_string().contains(&needle)
                    || row.balance.to_lowercase().contains(&needle)
                    || row.name.to_lowercase().contains(&needle)
            })
            .collect();

        let selected = self.selected;
        let manager = self.manager.clone();
        let mut edit_opt = self.item_edit.take();

        // ---- inspector (only when a row is selected) ----
        if selected.is_some() {
            egui::Panel::right("inspector")
                .resizable(true)
                .default_size(470.0)
                .min_size(330.0)
                .frame(
                    egui::Frame::default()
                        .fill(p.pan)
                        .inner_margin(egui::Margin::symmetric(12, 10)),
                )
                .show(ui, |ui| {
                    let (Some(manager), Some(edit)) = (manager.as_ref(), edit_opt.as_mut()) else {
                        theme::muted(ui, "Load the game-data folder to edit item parts by name.");
                        return;
                    };
                    let display = visible
                        .iter()
                        .position(|r| {
                            r.container == edit.container && r.occurrence == edit.occurrence
                        })
                        .map(|i| i + 1);
                    ui.horizontal(|ui| {
                        let title = match display {
                            Some(n) => format!("{} #{n}", edit.container.label()),
                            None => edit.container.label().to_string(),
                        };
                        crate::ui::widgets::label_nums(ui, &title, p.fg, 14.0);
                        ui.label(theme::mono(format!("id={}", edit.decoded.unique_id)));
                    });
                    let name = item_name(
                        &edit.decoded,
                        &Some(manager.clone()),
                        self.name_parts.as_deref(),
                    );
                    if !name.is_empty() {
                        ui.label(egui::RichText::new(name).strong().size(15.0).color(p.gold));
                    }
                    ui.add_space(6.0);

                    if edit.container == Container::BackpackItem
                        || edit.container == Container::BackpackWeapon
                    {
                        ui.horizontal(|ui| {
                            if edit.container == Container::BackpackItem {
                                ui.label("Quantity");
                                if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                    ui.add(egui::DragValue::new(&mut edit.quantity)).changed()
                                }) {
                                    edit.dirty = true;
                                }
                                if ui.checkbox(&mut edit.equipped, "Equipped").changed() {
                                    edit.dirty = true;
                                }
                            } else {
                                ui.label("Quick slot");
                                if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                    ui.add(egui::DragValue::new(&mut edit.quick_slot)).changed()
                                }) {
                                    edit.dirty = true;
                                }
                            }
                            ui.label("Mark");
                            egui::ComboBox::from_id_salt("mark_edit")
                                .selected_text(player_mark_name(edit.mark))
                                .show_ui(ui, |ui| {
                                    for (value, name) in PLAYER_MARKS.iter().enumerate() {
                                        if ui
                                            .selectable_label(edit.mark == value as i32, *name)
                                            .clicked()
                                        {
                                            edit.mark = value as i32;
                                            edit.dirty = true;
                                        }
                                    }
                                });
                        });
                        ui.add_space(6.0);
                        let (gstage, mgrade) = match &mut edit.decoded.slot {
                            lootsmith_core::items::PackedSlot::Weapon(weapon) => {
                                (&mut weapon.game_stage, &mut weapon.manufacturer_grade_index)
                            }
                            lootsmith_core::items::PackedSlot::Item(item) => {
                                (&mut item.game_stage, &mut item.manufacturer_grade_index)
                            }
                        };
                        ui.horizontal(|ui| {
                            ui.label("Game stage");
                            if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                ui.add(egui::DragValue::new(gstage).range(1..=80)).changed()
                            }) {
                                edit.dirty = true;
                            }
                            ui.label("Manufacturer grade");
                            if crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                ui.add(egui::DragValue::new(mgrade).range(1..=10)).changed()
                            }) {
                                edit.dirty = true;
                            }
                        });
                        ui.add_space(6.0);
                    }

                    let is_weapon = edit.decoded.is_weapon;
                    let set = edit.decoded.asset_library_set_id;

                    egui::Panel::bottom("inspector_actions")
                        .frame(
                            egui::Frame::default()
                                .fill(p.pan)
                                .inner_margin(egui::Margin::symmetric(0, 8)),
                        )
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                if edit.dirty && ui.button("Apply item changes").clicked() {
                                    apply = true;
                                }
                                if edit.container == Container::Bank {
                                    let target = if is_weapon {
                                        Container::BackpackWeapon
                                    } else {
                                        Container::BackpackItem
                                    };
                                    if ui.button("Move to backpack").clicked() {
                                        move_to = Some(target);
                                    }
                                } else if ui.button("Move to bank").clicked() {
                                    move_to = Some(Container::Bank);
                                }
                                if ui.button("Remove").clicked() {
                                    remove = true;
                                }
                            });
                        });

                    theme::section(
                        ui,
                        if is_weapon {
                            "Weapon parts"
                        } else {
                            "Item parts"
                        },
                    );
                    egui::ScrollArea::vertical()
                        .id_salt("inspector_parts")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Grid::new("parts_grid")
                                .num_columns(2)
                                .spacing([10.0, 6.0])
                                .show(ui, |ui| {
                                    for (slot, label) in slots_for(is_weapon) {
                                        crate::ui::widgets::label_nums(ui, *label, p.dim, 12.0);
                                        let group =
                                            items::slot_group(&edit.decoded.slot, slot).unwrap();
                                        let current = match edit.decoded.part(slot) {
                                            Some((r, g)) => manager
                                                .lookup_asset(
                                                    g,
                                                    set,
                                                    r.asset_index,
                                                    r.sublibrary_index,
                                                    r.use_set_id,
                                                )
                                                .unwrap_or_else(|_| {
                                                    format!(
                                                        "#{}/{}",
                                                        r.asset_index, r.sublibrary_index
                                                    )
                                                }),
                                            None => "-".to_string(),
                                        };
                                        let balance_name =
                                            resolve_balance(&Some(manager.clone()), &edit.decoded);
                                        let choices = self.balanced_choices(
                                            &edit.decoded.asset_library_set_id,
                                            &balance_name,
                                            group,
                                            slot,
                                        );
                                        egui::ComboBox::from_id_salt(format!("slot_{slot}"))
                                            .selected_text(crate::ui::widgets::text_with_nums(
                                                &current, p.fg, 15.0,
                                            ))
                                            .width(270.0)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(false, "None").clicked() {
                                                    let _ = edit
                                                        .decoded
                                                        .set_part(manager, slot, "None");
                                                    edit.dirty = true;
                                                }
                                                for choice in &choices {
                                                    if ui.selectable_label(false, choice).clicked()
                                                    {
                                                        let _ = edit
                                                            .decoded
                                                            .set_part(manager, slot, choice);
                                                        edit.dirty = true;
                                                    }
                                                }
                                            });
                                        ui.end_row();
                                    }
                                });
                        });
                });
        }
        self.item_edit = edit_opt;

        // ---- inventory table ----
        egui::CentralPanel::default()
            .frame(egui::Frame::default().fill(p.bg))
            .show(ui, |ui| {
                if visible.is_empty() {
                    ui.add_space(12.0);
                    theme::muted(
                        ui,
                        if bank {
                            "(bank is empty)"
                        } else {
                            "(backpack is empty)"
                        },
                    );
                    return;
                }
                TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .column(Column::exact(48.0))
                    .column(Column::exact(96.0))
                    .column(Column::exact(124.0))
                    .column(Column::exact(52.0))
                    .column(Column::exact(44.0))
                    .column(Column::exact(56.0))
                    .column(Column::exact(90.0))
                    .column(Column::remainder().at_least(180.0))
                    .header(26.0, |mut header| {
                        for name in [
                            "#",
                            "Type",
                            "ID",
                            "Qty",
                            "Eq",
                            "Quick",
                            "Mark",
                            "Balance / Name",
                        ] {
                            header.col(|ui| {
                                ui.label(
                                    egui::RichText::new(name).strong().size(11.5).color(p.dim),
                                );
                            });
                        }
                    })
                    .body(|mut body| {
                        for (i, row) in visible.iter().enumerate() {
                            let is_selected = selected == Some((row.container, row.occurrence));
                            let cell = |base: egui::Color32| if is_selected { p.on } else { base };
                            body.row(24.0, |mut tr| {
                                tr.set_selected(is_selected);
                                tr.col(|ui| {
                                    ui.label(
                                        crate::ui::widgets::num((i + 1).to_string(), 15.0)
                                            .color(cell(p.dim)),
                                    );
                                });
                                tr.col(|ui| {
                                    ui.label(
                                        egui::RichText::new(&row.kind)
                                            .color(cell(if row.is_weapon { p.gold } else { p.dim }))
                                            .size(12.0),
                                    );
                                });
                                tr.col(|ui| {
                                    if ui
                                        .selectable_label(
                                            is_selected,
                                            crate::ui::widgets::num(format!("id={}", row.id), 15.0)
                                                .color(cell(p.fg)),
                                        )
                                        .clicked()
                                    {
                                        clicked = Some((row.container, row.occurrence));
                                    }
                                });
                                tr.col(|ui| {
                                    if row.is_weapon || row.container == Container::Bank {
                                        ui.label("");
                                    } else {
                                        ui.label(
                                            crate::ui::widgets::num(row.quantity.to_string(), 15.0)
                                                .color(cell(p.fg)),
                                        );
                                    }
                                });
                                tr.col(|ui| {
                                    if row.is_weapon || row.container == Container::Bank {
                                        ui.label("");
                                    } else {
                                        ui.label(
                                            egui::RichText::new(if row.equipped {
                                                "✓"
                                            } else {
                                                ""
                                            })
                                            .color(cell(p.fg)),
                                        );
                                    }
                                });
                                tr.col(|ui| {
                                    if row.is_weapon {
                                        ui.label(
                                            crate::ui::widgets::num(
                                                row.quick_slot.to_string(),
                                                15.0,
                                            )
                                            .color(cell(p.fg)),
                                        );
                                    } else {
                                        ui.label("");
                                    }
                                });
                                tr.col(|ui| {
                                    ui.label(
                                        egui::RichText::new(player_mark_name(row.mark))
                                            .color(cell(p.fg)),
                                    );
                                });
                                tr.col(|ui| {
                                    let (text, color) = if row.empty || row.name.is_empty() {
                                        (row.balance.clone(), cell(p.dim))
                                    } else {
                                        (row.name.clone(), cell(p.fg))
                                    };
                                    let tip = if row.name.is_empty() {
                                        row.balance.clone()
                                    } else {
                                        format!("{} — {}", row.name, row.balance)
                                    };
                                    crate::ui::widgets::label_nums(ui, &text, color, 15.0)
                                        .on_hover_text(tip);
                                });
                                if tr.response().clicked() {
                                    clicked = Some((row.container, row.occurrence));
                                }
                            });
                        }
                    });
            });

        if let Some((container, occurrence)) = clicked {
            self.select_entry(container, occurrence);
        }
        if apply {
            self.apply_item();
        }
        if let Some(to) = move_to {
            self.apply_item();
            self.move_selected_to(to);
        }
        if remove {
            self.remove_selected();
        }
    }
}
