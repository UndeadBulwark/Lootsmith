//! The raw tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use eframe::egui;

impl LootsmithApp {
    pub(crate) fn ui_raw(&mut self, ui: &mut egui::Ui) {
        let p = *theme::active(ui.ctx());
        ui.add_space(4.0);
        ui.horizontal(|ui| {
            ui.heading("Raw fields");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if crate::ui::widgets::icon_button(
                    ui,
                    crate::ui::pixel::Icon::Close,
                    p.fg,
                    p.pan,
                    24.0,
                    "Clear search",
                )
                .clicked()
                {
                    self.raw_filter.clear();
                }
                ui.add_sized(
                    [260.0, 22.0],
                    egui::TextEdit::singleline(&mut self.raw_filter).hint_text("search fields…"),
                );
            });
        });
        ui.add_space(2.0);
        ui.label(
            "Edit scalar fields and int/string list elements directly. Nested \
             messages and byte blobs are read-only here (use the CLI json dump).",
        );
        ui.add_space(4.0);
        let needle = self.raw_filter.to_lowercase();
        let mut changed = false;
        let mut list_ops: Vec<(u32, RawListOp)> = Vec::new();
        egui::ScrollArea::vertical().id_salt("raw").show(ui, |ui| {
            egui::Grid::new("raw_grid")
                .num_columns(5)
                .striped(true)
                .spacing([12.0, 6.0])
                .show(ui, |ui| {
                    ui.strong("#");
                    ui.strong("Field");
                    ui.strong("Kind");
                    ui.strong("x");
                    ui.strong("Value");
                    ui.end_row();
                    for field in self.raw.iter_mut() {
                        if !needle.is_empty() && !field.name.to_lowercase().contains(&needle) {
                            continue;
                        }
                        crate::ui::widgets::label_num(ui, field.number.to_string(), p.fg, 15.0);
                        ui.label(&field.name);
                        ui.label(&field.kind);
                        crate::ui::widgets::label_num(ui, field.count.to_string(), p.fg, 15.0);
                        match &mut field.edit {
                            RawEdit::Int(v) => {
                                changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                    ui.add(egui::DragValue::new(v)).changed()
                                });
                            }
                            RawEdit::Bool(v) => {
                                changed |= ui.checkbox(v, "").changed();
                            }
                            RawEdit::Str(v) => {
                                changed |= crate::ui::widgets::num_scope(ui, 15.0, |ui| {
                                    ui.add_sized([300.0, 20.0], egui::TextEdit::singleline(v))
                                        .changed()
                                });
                            }
                            RawEdit::None if !field.elements.is_empty() => {
                                let number = field.number;
                                let ops = &mut list_ops;
                                ui.vertical(|ui| {
                                    for i in 0..field.elements.len() {
                                        ui.horizontal(|ui| {
                                            match &mut field.elements[i].edit {
                                                RawEdit::Int(v) => {
                                                    if crate::ui::widgets::num_scope(
                                                        ui,
                                                        15.0,
                                                        |ui| {
                                                            ui.add(egui::DragValue::new(v))
                                                                .changed()
                                                        },
                                                    ) {
                                                        field.elements[i].changed = true;
                                                    }
                                                }
                                                RawEdit::Str(v) => {
                                                    if crate::ui::widgets::num_scope(
                                                        ui,
                                                        15.0,
                                                        |ui| {
                                                            ui.add(
                                                                egui::TextEdit::singleline(v)
                                                                    .desired_width(260.0),
                                                            )
                                                            .changed()
                                                        },
                                                    ) {
                                                        field.elements[i].changed = true;
                                                    }
                                                }
                                                _ => {}
                                            }
                                            if ui
                                                .small_button("×")
                                                .on_hover_text("Remove this element")
                                                .clicked()
                                            {
                                                ops.push((number, RawListOp::Remove(i)));
                                            }
                                        });
                                    }
                                    if ui
                                        .small_button("+")
                                        .on_hover_text("Add an element")
                                        .clicked()
                                    {
                                        ops.push((number, RawListOp::Add));
                                    }
                                });
                            }
                            RawEdit::None => {
                                crate::ui::widgets::label_nums(ui, &field.preview, p.fg, 15.0);
                            }
                        }
                        ui.end_row();
                    }
                });
        });
        for (number, op) in list_ops {
            if let Some(field) = self.raw.iter_mut().find(|f| f.number == number) {
                field.apply = true;
                match op {
                    RawListOp::Add => {
                        let blank = match field.elements.first().map(|e| &e.edit) {
                            Some(RawEdit::Str(_)) => RawEdit::Str(String::new()),
                            _ => RawEdit::Int(0),
                        };
                        field.elements.push(RawElem {
                            edit: blank,
                            changed: true,
                        });
                    }
                    RawListOp::Remove(i) => {
                        if i < field.elements.len() {
                            field.elements.remove(i);
                        }
                    }
                }
            }
        }
        let lists_dirty = self.raw.iter().any(RawField::list_needs_apply);
        if changed || lists_dirty {
            let scalar: Vec<(u32, RawEdit)> = self
                .raw
                .iter()
                .map(|f| (f.number, f.edit.clone()))
                .filter(|(_, e)| !matches!(e, RawEdit::None))
                .collect();
            let int_lists: Vec<(u32, Vec<i32>)> = self
                .raw
                .iter()
                .filter(|f| f.list_needs_apply())
                .filter_map(|f| {
                    let values: Vec<i32> = f
                        .elements
                        .iter()
                        .map(|e| match e.edit {
                            RawEdit::Int(v) => v,
                            _ => 0,
                        })
                        .collect();
                    (!values.is_empty()).then_some((f.number, values))
                })
                .collect();
            let str_lists: Vec<(u32, Vec<String>)> = self
                .raw
                .iter()
                .filter(|f| f.list_needs_apply())
                .filter_map(|f| {
                    let strings: Vec<String> = f
                        .elements
                        .iter()
                        .filter_map(|e| match &e.edit {
                            RawEdit::Str(s) => Some(s.clone()),
                            _ => None,
                        })
                        .collect();
                    (!strings.is_empty()).then_some((f.number, strings))
                })
                .collect();
            if let Some(save_game) = self.save_game.as_mut() {
                for (number, edit) in scalar {
                    match edit {
                        RawEdit::Int(v) => save_game.set_i32(number, v),
                        RawEdit::Bool(v) => save_game.set_bool(number, v),
                        RawEdit::Str(v) => save_game.set_str(number, &v),
                        RawEdit::None => {}
                    }
                }
                for (number, values) in int_lists {
                    save_game.set_int_list(number, &values);
                }
                for (number, values) in str_lists {
                    save_game.set_str_list(number, &values);
                }
            }
            self.dirty = true;
        }
    }
}
