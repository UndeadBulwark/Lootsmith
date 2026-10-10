//! Lootsmith GUI (Phase 6, in progress).
//!
//! A thin `egui` front end over `lootsmith-core`: open a save, edit the common
//! scalars, list items, and save (in place, with a `.bak` backup). All format
//! logic lives in the library; this only binds widgets to it.

use eframe::egui;
use lootsmith_core::{
    container, items, AssetGroup, AssetLibraryConfigs, AssetLibraryManager, Platform, SaveGame,
};

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([960.0, 640.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Lootsmith",
        options,
        Box::new(|_cc| Ok(Box::new(LootsmithApp::default()))),
    )
}

#[derive(Default)]
struct Form {
    level: i32,
    exp_points: i32,
    general_skill_points: i32,
    specialist_skill_points: i32,
    playthroughs: i32,
    currency: i32,
    golden_keys: i32,
    bank_slots: i32,
}

#[derive(Default)]
struct LootsmithApp {
    path: String,
    dumps: String,
    manager: Option<AssetLibraryManager>,
    save_game: Option<SaveGame>,
    form: Form,
    items: Vec<String>,
    status: String,
}

impl LootsmithApp {
    fn configs(&self) -> AssetLibraryConfigs {
        self.manager
            .as_ref()
            .map(|m| m.configs.clone())
            .unwrap_or_else(AssetLibraryConfigs::pc_default)
    }

    fn load(&mut self) {
        let data = match std::fs::read(&self.path) {
            Ok(d) => d,
            Err(e) => {
                self.status = format!("read error: {e}");
                return;
            }
        };
        let save = match container::unpack(&data, Platform::PC) {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("parse error: {e}");
                return;
            }
        };
        let save_game = match save.save_game() {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("protobuf error: {e}");
                return;
            }
        };
        self.form = Form {
            level: save_game.exp_level().unwrap_or(1),
            exp_points: save_game.exp_points().unwrap_or(0),
            general_skill_points: save_game.general_skill_points().unwrap_or(0),
            specialist_skill_points: save_game.specialist_skill_points().unwrap_or(0),
            playthroughs: save_game.playthroughs_completed().unwrap_or(0),
            currency: save_game.currency_on_hand().first().copied().unwrap_or(0),
            golden_keys: save_game.num_golden_keys_notified().unwrap_or(0),
            bank_slots: save_game.max_bank_slots().unwrap_or(0),
        };
        self.save_game = Some(save_game);
        self.status = format!(
            "loaded {} ({} bytes, crc32 {})",
            self.path,
            data.len(),
            if save.crc32_valid() { "ok" } else { "MISMATCH" }
        );
        self.refresh_items();
    }

    fn load_dumps(&mut self) {
        let path = if self.dumps.trim().is_empty() {
            std::env::var("LOOTSMITH_ASSET_LIBRARY").unwrap_or_default()
        } else {
            self.dumps.clone()
        };
        if path.trim().is_empty() {
            self.manager = None;
            self.status = "no dumps path (asset names unavailable)".into();
            return;
        }
        match AssetLibraryManager::from_path(std::path::Path::new(&path)) {
            Ok(m) => {
                self.manager = Some(m);
                self.status = format!("loaded asset library from {path}");
            }
            Err(e) => {
                self.manager = None;
                self.status = format!("dumps error: {e}");
            }
        }
        self.refresh_items();
    }

    fn refresh_items(&mut self) {
        let mut lines = Vec::new();
        if let Some(save_game) = &self.save_game {
            let configs = self.configs();
            for (index, entry) in save_game.packed_entries().iter().enumerate() {
                match items::unpack(&entry.serial, &configs) {
                    Ok(decoded) => {
                        let kind = if decoded.is_weapon { "weapon" } else { "item" };
                        let balance = match (&self.manager, &decoded.slot) {
                            (Some(m), items::PackedSlot::Weapon(w)) => m.lookup_asset(
                                AssetGroup::BalanceDefs,
                                decoded.asset_library_set_id,
                                w.balance.asset_index,
                                w.balance.sublibrary_index,
                                w.balance.use_set_id,
                            ),
                            (Some(m), items::PackedSlot::Item(i)) => m.lookup_asset(
                                AssetGroup::BalanceDefs,
                                decoded.asset_library_set_id,
                                i.balance.asset_index,
                                i.balance.sublibrary_index,
                                i.balance.use_set_id,
                            ),
                            _ => Ok(format!(
                                "({} bytes)",
                                entry.serial.len()
                            )),
                        }
                        .unwrap_or_else(|e| format!("<{e}>"));
                        lines.push(format!(
                            "#{} {kind} id={} balance={balance}",
                            index + 1,
                            decoded.unique_id
                        ));
                    }
                    Err(e) => lines.push(format!("#{} decode error: {e}", index + 1)),
                }
            }
        }
        self.items = lines;
    }

    fn save(&mut self) {
        let Some(mut save_game) = self.save_game.clone() else {
            self.status = "nothing loaded".into();
            return;
        };
        save_game.set_i32(2, self.form.level);
        save_game.set_i32(3, self.form.exp_points);
        save_game.set_i32(4, self.form.general_skill_points);
        save_game.set_i32(5, self.form.specialist_skill_points);
        save_game.set_i32(7, self.form.playthroughs);
        let mut currency = save_game.get_int_list(6);
        if currency.is_empty() {
            currency.push(self.form.currency);
        } else {
            currency[0] = self.form.currency;
        }
        save_game.set_int_list_packed(6, &currency);
        save_game.set_i32(48, self.form.golden_keys);
        save_game.set_i32(56, self.form.bank_slots);

        let bytes = match container::serialize(&save_game.to_bytes(), Platform::PC) {
            Ok(b) => b,
            Err(e) => {
                self.status = format!("serialize error: {e}");
                return;
            }
        };
        let backup = format!("{}.bak", self.path);
        if !std::path::Path::new(&backup).exists() {
            if let Err(e) = std::fs::copy(&self.path, &backup) {
                self.status = format!("backup error: {e}");
                return;
            }
        }
        if let Err(e) = std::fs::write(&self.path, &bytes) {
            self.status = format!("write error: {e}");
            return;
        }
        self.load();
        self.status = format!("wrote {} bytes (backup {backup})", bytes.len());
    }
}

impl eframe::App for LootsmithApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("inputs").show(ui, |ui| {
            egui::Grid::new("paths").num_columns(3).show(ui, |ui| {
                ui.label("Save file");
                ui.text_edit_singleline(&mut self.path);
                if ui.button("Load").clicked() {
                    self.load();
                }
                ui.end_row();

                ui.label("Asset Library");
                ui.text_edit_singleline(&mut self.dumps);
                if ui.button("Load dumps").clicked() {
                    self.load_dumps();
                }
                ui.end_row();
            });
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(&self.status);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.save_game.is_none() {
                ui.label("Enter a save path and click Load.");
                return;
            }

            egui::Grid::new("fields").num_columns(2).show(ui, |ui| {
                ui.label("Level");
                ui.add(egui::DragValue::new(&mut self.form.level));
                ui.end_row();
                ui.label("Experience");
                ui.add(egui::DragValue::new(&mut self.form.exp_points));
                ui.end_row();
                ui.label("General skill points");
                ui.add(egui::DragValue::new(&mut self.form.general_skill_points));
                ui.end_row();
                ui.label("Specialist skill points");
                ui.add(egui::DragValue::new(&mut self.form.specialist_skill_points));
                ui.end_row();
                ui.label("Playthroughs");
                ui.add(egui::DragValue::new(&mut self.form.playthroughs));
                ui.end_row();
                ui.label("Currency");
                ui.add(egui::DragValue::new(&mut self.form.currency));
                ui.end_row();
                ui.label("Golden keys");
                ui.add(egui::DragValue::new(&mut self.form.golden_keys));
                ui.end_row();
                ui.label("Bank slots");
                ui.add(egui::DragValue::new(&mut self.form.bank_slots));
                ui.end_row();
            });

            ui.separator();
            if ui.button("Save (backup + write in place)").clicked() {
                self.save();
            }
            ui.separator();

            ui.heading(format!("Items ({})", self.items.len()));
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    for line in &self.items {
                        ui.label(line);
                    }
                });
        });
    }
}
