//! The backpack tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_container(&mut self, ui: &mut egui::Ui, bank: bool) {
        let p = *theme::active(ui.ctx());
        let ppp = ui.ctx().pixels_per_point();
        let mut item_action: Option<ItemAction> = None;
        let mut clicked: Option<(Container, usize)> = None;
        let mut apply = false;
        let mut move_to: Option<Container> = None;
        let mut remove = false;

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
                    || row.type_name.to_lowercase().contains(&needle)
                    || row.id.to_string().contains(&needle)
                    || row.balance.to_lowercase().contains(&needle)
                    || row.name.to_lowercase().contains(&needle)
            })
            .collect();
        let selected = self.selected;
        let manager = self.manager.clone();
        let name_parts = self.name_parts.clone();
        let mut edit_opt = self.item_edit.take();

        let area = ui.max_rect();
        let gap = 16.0;

        // ---- header row: ribbon + slot count + Filter / + Weapon / Paste ----
        let title = if bank { "Bank" } else { "Backpack" };
        let header = egui::Rect::from_min_size(area.min, egui::vec2(area.width(), 52.0));
        let ribbon_w = (title.chars().count() as f32 * 9.0 + 48.0).min(220.0);
        let ribbon_rect = egui::Rect::from_min_size(
            egui::pos2(header.left(), header.center().y - 14.0),
            egui::vec2(ribbon_w, 28.0),
        );
        crate::ui::widgets::ribbon(ui, ribbon_rect, title);
        let (filled, total) = if bank {
            (
                self.bank.len(),
                (self.bank_size.max(self.bank.len() as i32)).max(1) as usize,
            )
        } else {
            (
                self.backpack.len(),
                (self.backpack_slots.max(self.backpack.len() as i32)).max(1) as usize,
            )
        };
        let g = ui.painter().layout_job(crate::ui::widgets::job_with_nums(
            &format!("{filled} of {total} slots filled"),
            p.dim,
            15.0,
        ));
        ui.painter().galley(
            egui::pos2(
                ribbon_rect.right() + 14.0,
                header.center().y - g.size().y / 2.0,
            ),
            g,
            p.dim,
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(header), |ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(120.0, 28.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r,
                    "Paste BL2(…)",
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                {
                    self.show_paste = true;
                }
                let (r, _) = ui.allocate_exact_size(egui::vec2(110.0, 28.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r,
                    "+ Weapon",
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                {
                    item_action = Some(ItemAction::NewWeapon);
                }
                ui.add_sized(
                    [220.0, 24.0],
                    egui::TextEdit::singleline(&mut self.filter).hint_text("Filter items…"),
                );
            });
        });

        // ---- body: list panel (520 px) + workbench, 16 px gap, notched panels ----
        let body =
            egui::Rect::from_min_max(egui::pos2(area.left(), header.bottom() + gap), area.max);
        let list_w = 520.0_f32.min(body.width() * 0.6);
        let list = egui::Rect::from_min_size(body.min, egui::vec2(list_w, body.height()));
        let bench = egui::Rect::from_min_max(egui::pos2(list.right() + gap, body.top()), body.max);
        crate::ui::pixel::panel(ui.painter(), list, ppp, &p);
        crate::ui::pixel::panel(ui.painter(), bench, ppp, &p);

        // list: item table + action row
        let list_inner = list.shrink(14.0);
        let actions_h = 42.0;
        let list_rows = egui::Rect::from_min_max(
            list_inner.min,
            egui::pos2(list_inner.right(), list_inner.bottom() - actions_h),
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(list_rows), |ui| {
            ui.set_clip_rect(list_rows);
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
            let items: Vec<crate::ui::item_table::ItemRow> = visible
                .iter()
                .map(|r| crate::ui::item_table::ItemRow {
                    name: if r.name.is_empty() {
                        crate::ui::item_table::balance_to_name(&r.balance)
                    } else {
                        r.name.clone()
                    },
                    kind: r.type_name.clone(),
                    id: r.id,
                    level: r.level,
                    mark: r.mark,
                    rarity: r.rarity,
                })
                .collect();
            let sel = selected.and_then(|(c, o)| {
                visible
                    .iter()
                    .position(|r| r.container == c && r.occurrence == o)
            });
            if let Some(i) = crate::ui::item_table::item_table(ui, &p, &items, sel) {
                let row = &visible[i];
                clicked = Some((row.container, row.occurrence));
            }
        });
        // action row under the list
        let actions = egui::Rect::from_min_max(
            egui::pos2(list_inner.left(), list_inner.bottom() - actions_h + 8.0),
            list_inner.max,
        );
        ui.scope_builder(egui::UiBuilder::new().max_rect(actions), |ui| {
            ui.horizontal(|ui| {
                let has = selected.is_some();
                let (r1, _) = ui.allocate_exact_size(egui::vec2(96.0, 26.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r1,
                    "Duplicate",
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                    && has
                {
                    item_action = Some(ItemAction::Duplicate);
                }
                let move_label = if bank { "To backpack" } else { "To bank" };
                let (r2, _) = ui.allocate_exact_size(egui::vec2(104.0, 26.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r2,
                    move_label,
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                    && has
                {
                    move_to = Some(if bank {
                        Container::BackpackItem
                    } else {
                        Container::Bank
                    });
                }
                let (r3, _) = ui.allocate_exact_size(egui::vec2(72.0, 26.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r3,
                    "Sync",
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                {
                    item_action = Some(ItemAction::SyncAll);
                }
                let (r4, _) = ui.allocate_exact_size(egui::vec2(84.0, 26.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r4,
                    "Remove",
                    crate::ui::widgets::ButtonKind::Danger,
                )
                .clicked()
                    && has
                {
                    remove = true;
                }
                if edit_opt.as_ref().map(|e| e.dirty).unwrap_or(false) {
                    let (r5, _) =
                        ui.allocate_exact_size(egui::vec2(150.0, 26.0), egui::Sense::click());
                    if crate::ui::widgets::button(
                        ui,
                        r5,
                        "Apply item changes",
                        crate::ui::widgets::ButtonKind::Primary,
                    )
                    .clicked()
                    {
                        apply = true;
                    }
                }
            });
        });

        // ---- workbench ----
        let bench_inner = bench.shrink(14.0);
        ui.scope_builder(egui::UiBuilder::new().max_rect(bench_inner), |ui| {
            ui.set_clip_rect(bench_inner);
            let (Some(manager), Some(edit)) = (manager.as_ref(), edit_opt.as_mut()) else {
                ui.add_space(8.0);
                theme::muted(
                    ui,
                    if selected.is_none() {
                        "Select an item to edit it."
                    } else {
                        "Load the game-data folder to edit item parts by name."
                    },
                );
                return;
            };
            #[allow(unused_mut)]
            let mut sa = egui::ScrollArea::vertical()
                .id_salt("bench_scroll")
                .auto_shrink([false, false]);
            if let Some(off) = self.inspector_scroll {
                sa = sa.vertical_scroll_offset(off);
            }
            sa.show(ui, |ui| {
                let is_weapon = edit.decoded.is_weapon;
                let set = edit.decoded.asset_library_set_id;
                // ribbon
                let (rr, _) = ui.allocate_exact_size(egui::vec2(190.0, 30.0), egui::Sense::hover());
                crate::ui::widgets::ribbon(ui, rr, "Item workbench");
                ui.add_space(12.0);
                let balance_name = resolve_balance(&Some(manager.clone()), &edit.decoded);
                // The weapon type / item definition path, used to resolve the
                // per-slot legal parts (`GameDefs`).
                let type_path = edit.decoded.part("type").and_then(|(r, g)| {
                    if r.is_none() {
                        return None;
                    }
                    manager
                        .lookup_asset(g, set, r.asset_index, r.sublibrary_index, r.use_set_id)
                        .ok()
                });
                let rarity = lootsmith_core::RarityRules::embedded().classify(&balance_name);
                let name = item_name(
                    &edit.decoded,
                    &Some(manager.clone()),
                    self.name_parts.as_deref(),
                );
                // name row: 22 px diamond + 26 px name
                let (nr, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 36.0),
                    egui::Sense::hover(),
                );
                crate::ui::pixel::diamond(
                    ui.painter(),
                    egui::pos2(nr.left() + 14.0, nr.center().y),
                    22.0,
                    crate::ui::item_table::rarity_color(rarity),
                    ppp,
                    &p,
                );
                let raw_name = if name.is_empty() {
                    crate::ui::item_table::balance_to_name(&balance_name)
                } else {
                    name.clone()
                };
                let display_name = crate::ui::item_table::elide_middle(
                    &raw_name,
                    ((nr.width() - 40.0) / 12.0) as usize,
                );
                let g = ui.painter().layout_job(crate::ui::widgets::job_with_nums(
                    &display_name,
                    p.fg,
                    26.0,
                ));
                ui.painter().galley(
                    egui::pos2(nr.left() + 36.0, nr.center().y - g.size().y / 2.0),
                    g,
                    p.fg,
                );
                // subtitle
                let (sr, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 22.0),
                    egui::Sense::hover(),
                );
                let subtitle = format!(
                    "{} · {} · {}",
                    edit.container.label(),
                    rarity.label(),
                    edit.decoded.game_stage()
                );
                let g = ui
                    .painter()
                    .layout_job(crate::ui::widgets::job_with_nums(&subtitle, p.dim, 15.0));
                ui.painter().galley(sr.left_center(), g, p.dim);
                ui.add_space(10.0);
                // four editable stat blocks: LEVEL, GRADE, SLOT / STATUS, MARK
                let (gstage, mgrade) = match &mut edit.decoded.slot {
                    lootsmith_core::items::PackedSlot::Weapon(w) => {
                        (&mut w.game_stage, &mut w.manufacturer_grade_index)
                    }
                    lootsmith_core::items::PackedSlot::Item(it) => {
                        (&mut it.game_stage, &mut it.manufacturer_grade_index)
                    }
                };
                let is_item = edit.container == Container::BackpackItem;
                let bw = (ui.available_width() - 3.0 * 8.0) / 4.0;
                let (row, _) = ui.allocate_exact_size(
                    egui::vec2(ui.available_width(), 72.0),
                    egui::Sense::hover(),
                );
                let labels = [
                    "LEVEL",
                    "GRADE",
                    if is_item { "STATUS" } else { "SLOT" },
                    "MARK",
                ];
                let cells: Vec<egui::Rect> = (0..4)
                    .map(|i| {
                        egui::Rect::from_min_size(
                            row.min + egui::vec2(i as f32 * (bw + 8.0), 0.0),
                            egui::vec2(bw, row.height()),
                        )
                    })
                    .collect();
                for (r, label) in cells.iter().zip(labels) {
                    ui.painter().rect_filled(*r, 0.0, p.pan);
                    ui.painter().rect_stroke(
                        *r,
                        0.0,
                        egui::Stroke::new(2.0, p.wdb),
                        egui::StrokeKind::Inside,
                    );
                    ui.painter().text(
                        r.left_top() + egui::vec2(10.0, 8.0),
                        egui::Align2::LEFT_TOP,
                        label,
                        egui::FontId::proportional(12.0),
                        p.gold,
                    );
                }
                let inner = |r: egui::Rect| {
                    egui::Rect::from_min_max(
                        egui::pos2(r.left() + 8.0, r.top() + 24.0),
                        egui::pos2(r.right() - 8.0, r.bottom() - 6.0),
                    )
                };
                // A VT323 (digit-rule compliant) number cell that is drag-editable.
                let num_cell = |ui: &mut egui::Ui,
                                r: egui::Rect,
                                id: &str,
                                value: &mut i32,
                                lo: i32,
                                hi: i32|
                 -> bool {
                    let mut resp = ui.interact(r, ui.id().with(id), egui::Sense::drag());
                    if resp.dragged() {
                        let d = resp.drag_delta().x.round() as i32;
                        if d != 0 {
                            *value = (*value + d).clamp(lo, hi);
                            resp.mark_changed();
                        }
                    }
                    let g = ui.painter().layout_job(crate::ui::widgets::job_with_nums(
                        &value.to_string(),
                        p.fg,
                        20.0,
                    ));
                    ui.painter().galley(
                        egui::pos2(r.left(), r.center().y - g.size().y / 2.0),
                        g,
                        p.fg,
                    );
                    resp.changed()
                };
                if num_cell(ui, inner(cells[0]), "level", gstage, 1, 80) {
                    edit.dirty = true;
                }
                if num_cell(ui, inner(cells[1]), "grade", mgrade, 1, 10) {
                    edit.dirty = true;
                }
                if is_item {
                    // A word, not a number: Equipped / In-Inventory (0 = In-Inventory).
                    let word = if edit.equipped {
                        "Equipped"
                    } else {
                        "In-Inventory"
                    };
                    if ui
                        .put(
                            inner(cells[2]),
                            egui::Button::new(egui::RichText::new(word).size(17.0)),
                        )
                        .clicked()
                    {
                        edit.equipped = !edit.equipped;
                        edit.dirty = true;
                    }
                } else if num_cell(ui, inner(cells[2]), "slot", &mut edit.quick_slot, 0, 3) {
                    edit.dirty = true;
                }
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner(cells[3])), |ui| {
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
                ui.add_space(12.0);

                theme::section(ui, "Parts");
                egui::Grid::new("parts_grid")
                    .num_columns(2)
                    .spacing([10.0, 6.0])
                    .show(ui, |ui| {
                        for (slot, label) in self.part_slots(is_weapon) {
                            crate::ui::widgets::label_nums(ui, label, p.dim, 12.0);
                            let group = items::slot_group(&edit.decoded.slot, slot).unwrap();
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
                                        format!("#{}/{}", r.asset_index, r.sublibrary_index)
                                    }),
                                None => "-".to_string(),
                            };
                            let choices = self.slot_choices(
                                is_weapon,
                                type_path.as_deref(),
                                &balance_name,
                                slot,
                                group,
                            );
                            let current_label =
                                part_label(name_parts.as_deref(), group, slot, &current);
                            egui::ComboBox::from_id_salt(format!("slot_{slot}"))
                                .selected_text(crate::ui::widgets::text_with_nums(
                                    &current_label,
                                    p.fg,
                                    15.0,
                                ))
                                .width(270.0)
                                .show_ui(ui, |ui| {
                                    if ui.selectable_label(false, "None").clicked() {
                                        let _ = edit.decoded.set_part(manager, slot, "None");
                                        edit.dirty = true;
                                    }
                                    for choice in &choices {
                                        let label =
                                            part_label(name_parts.as_deref(), group, slot, choice);
                                        if ui.selectable_label(false, label).clicked() {
                                            let _ = edit.decoded.set_part(manager, slot, choice);
                                            edit.dirty = true;
                                        }
                                    }
                                });
                            ui.end_row();
                        }
                    });

                ui.add_space(12.0);
                theme::section(ui, "Item code");
                let code = self
                    .item_serial(edit.container, edit.occurrence)
                    .map(|s| items::encode_code(&s))
                    .unwrap_or_default();
                ui.horizontal(|ui| {
                    ui.add_sized(
                        [ui.available_width() - 90.0, 24.0],
                        egui::Label::new(crate::ui::widgets::text_with_nums(&code, p.fg, 15.0)),
                    );
                    let (cr, _) =
                        ui.allocate_exact_size(egui::vec2(80.0, 26.0), egui::Sense::click());
                    if crate::ui::widgets::button(
                        ui,
                        cr,
                        "Copy",
                        crate::ui::widgets::ButtonKind::Normal,
                    )
                    .clicked()
                    {
                        ui.ctx().copy_text(code.clone());
                    }
                });
            });
        });
        self.item_edit = edit_opt;

        if let Some(action) = item_action {
            match action {
                ItemAction::NewWeapon => self.new_item(bank, true),
                ItemAction::Duplicate => self.duplicate_selected(),
                ItemAction::SyncAll => self.sync_levels(false),
            }
        }
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
