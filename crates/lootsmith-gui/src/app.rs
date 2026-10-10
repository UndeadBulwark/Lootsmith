//! Lootsmith GUI application state and UI.
//!
//! The UI lives in [`LootsmithApp::build`], independent of the native window, so
//! that both the `eframe` app and the `egui_kittest` harness run identical code.

use eframe::egui;
use lootsmith_core::{
    container, items, AssetGroup, AssetLibraryConfigs, AssetLibraryManager, Platform, SaveGame,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Tab {
    General,
    Character,
    Currency,
    Items,
}

#[derive(Clone)]
pub struct Form {
    pub level: i32,
    pub exp_points: i32,
    pub general_skill_points: i32,
    pub specialist_skill_points: i32,
    pub playthroughs: i32,
    pub total_play_time: i32,
    pub badass: bool,
    pub golden_keys: i32,
    pub bank_slots: i32,
    pub currency: Vec<i32>,
}

impl Default for Form {
    fn default() -> Self {
        Self {
            level: 1,
            exp_points: 0,
            general_skill_points: 0,
            specialist_skill_points: 0,
            playthroughs: 0,
            total_play_time: 0,
            badass: false,
            golden_keys: 0,
            bank_slots: 0,
            currency: Vec::new(),
        }
    }
}

#[derive(Default, Clone)]
pub struct ReadInfo {
    pub player_class: String,
    pub save_id: i32,
    pub guid: String,
    pub last_saved: String,
}

pub struct ItemEdit {
    field_number: u32,
    occurrence: usize,
    decoded: items::PackedData,
    dirty: bool,
}

pub struct LootsmithApp {
    pub path: String,
    pub dumps: String,
    pub manager: Option<Arc<AssetLibraryManager>>,
    pub save_game: Option<SaveGame>,
    pub form: Form,
    pub read_info: ReadInfo,
    pub items: Vec<String>,
    pub status: String,
    pub save_as: String,
    loaded_once: bool,
    pub tab: Tab,
    pub selected_item: Option<usize>,
    item_edit: Option<ItemEdit>,
    choices: HashMap<AssetGroup, Vec<String>>,
}

/// Slot names and display labels for the item editor.
pub fn slots_for(is_weapon: bool) -> &'static [(&'static str, &'static str)] {
    if is_weapon {
        &[
            ("type", "Type"),
            ("balance", "Balance"),
            ("manufacturer", "Manufacturer"),
            ("body", "Body"),
            ("grip", "Grip"),
            ("barrel", "Barrel"),
            ("sight", "Sight"),
            ("stock", "Stock"),
            ("elemental", "Elemental"),
            ("accessory1", "Accessory 1"),
            ("accessory2", "Accessory 2"),
            ("material", "Material"),
            ("prefix", "Prefix"),
            ("title", "Title"),
        ]
    } else {
        &[
            ("item", "Type"),
            ("balance", "Balance"),
            ("manufacturer", "Manufacturer"),
            ("alpha", "Alpha"),
            ("beta", "Beta"),
            ("gamma", "Gamma"),
            ("delta", "Delta"),
            ("epsilon", "Epsilon"),
            ("zeta", "Zeta"),
            ("eta", "Eta"),
            ("theta", "Theta"),
            ("material", "Material"),
            ("prefix", "Prefix"),
            ("title", "Title"),
        ]
    }
}

impl LootsmithApp {
    pub fn new(open: Option<String>, dumps: Option<String>) -> Self {
        let mut app = Self {
            path: open.unwrap_or_default(),
            dumps: dumps.unwrap_or_default(),
            manager: None,
            save_game: None,
            form: Form::default(),
            read_info: ReadInfo::default(),
            items: Vec::new(),
            status: "Open a save to begin (Browse… or --open).".to_string(),
            save_as: String::new(),
            loaded_once: false,
            tab: Tab::General,
            selected_item: None,
            item_edit: None,
            choices: HashMap::new(),
        };
        if !app.dumps.trim().is_empty() {
            app.load_dumps();
        }
        app
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    pub fn is_loaded(&self) -> bool {
        self.save_game.is_some()
    }

    pub fn item_serial(&self, index: usize) -> Option<Vec<u8>> {
        self.save_game
            .as_ref()?
            .packed_entries()
            .get(index)
            .map(|e| e.serial.clone())
    }

    fn configs(&self) -> AssetLibraryConfigs {
        self.manager
            .as_ref()
            .map(|m| m.configs.clone())
            .unwrap_or_else(AssetLibraryConfigs::pc_default)
    }

    pub fn load(&mut self) {
        if self.path.trim().is_empty() {
            self.status = "Enter a save path (or Browse…).".to_string();
            return;
        }
        let data = match std::fs::read(self.path.trim()) {
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
            total_play_time: save_game.total_play_time().unwrap_or(0),
            badass: save_game.get_bool(31).unwrap_or(false),
            golden_keys: save_game.num_golden_keys_notified().unwrap_or(0),
            bank_slots: save_game.max_bank_slots().unwrap_or(0),
            currency: save_game.currency_on_hand(),
        };
        self.read_info = ReadInfo {
            player_class: save_game.player_class().unwrap_or_default(),
            save_id: save_game.save_game_id().unwrap_or(0),
            guid: save_game
                .get_bytes(34)
                .map(|b| b.iter().map(|x| format!("{x:02x}")).collect::<String>())
                .unwrap_or_default(),
            last_saved: save_game.last_saved_date().unwrap_or_default(),
        };
        self.save_game = Some(save_game);
        self.selected_item = None;
        self.item_edit = None;
        self.status = format!(
            "loaded {} ({} bytes, crc32 {})",
            self.path,
            data.len(),
            if save.crc32_valid() { "ok" } else { "MISMATCH" }
        );
        self.refresh_items();
    }

    pub fn load_dumps(&mut self) {
        let path = if self.dumps.trim().is_empty() {
            std::env::var("LOOTSMITH_ASSET_LIBRARY").unwrap_or_default()
        } else {
            self.dumps.clone()
        };
        if path.trim().is_empty() {
            self.manager = None;
            return;
        }
        match AssetLibraryManager::from_path(std::path::Path::new(path.trim())) {
            Ok(m) => {
                self.manager = Some(Arc::new(m));
                self.status = format!("loaded asset library from {path}");
            }
            Err(e) => {
                self.manager = None;
                self.status = format!("asset library error: {e}");
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
                            _ => Ok(format!("({} bytes)", entry.serial.len())),
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

    pub fn select_item(&mut self, index: usize) {
        self.selected_item = Some(index);
        self.item_edit = None;
        self.choices.clear();
        let Some(manager) = self.manager.clone() else {
            self.status = "Load an Asset Library to edit item parts by name.".into();
            return;
        };
        let entries = {
            let Some(save_game) = &self.save_game else {
                return;
            };
            save_game.packed_entries()
        };
        let Some(entry) = entries.get(index) else {
            return;
        };
        let decoded = match items::unpack(&entry.serial, &manager.configs) {
            Ok(d) => d,
            Err(e) => {
                self.status = format!("decode error: {e}");
                return;
            }
        };
        let set = decoded.asset_library_set_id;
        let groups: &[AssetGroup] = if decoded.is_weapon {
            &[
                AssetGroup::WeaponTypes,
                AssetGroup::BalanceDefs,
                AssetGroup::Manufacturers,
                AssetGroup::WeaponParts,
            ]
        } else {
            &[
                AssetGroup::ItemTypes,
                AssetGroup::BalanceDefs,
                AssetGroup::Manufacturers,
                AssetGroup::ItemParts,
            ]
        };
        for group in groups {
            self.choices.insert(*group, manager.choices(*group, set));
        }
        self.item_edit = Some(ItemEdit {
            field_number: entry.field_number,
            occurrence: entry.occurrence,
            decoded,
            dirty: false,
        });
    }

    /// Set a named slot on the currently selected item (programmatic editing).
    pub fn set_selected_part(&mut self, slot: &str, name: &str) -> Result<(), String> {
        let Some(manager) = self.manager.clone() else {
            return Err("no asset library loaded".into());
        };
        let Some(edit) = self.item_edit.as_mut() else {
            return Err("no item selected".into());
        };
        edit.decoded
            .set_part(&manager, slot, name)
            .map_err(|e| e.to_string())?;
        edit.dirty = true;
        Ok(())
    }

    /// The currently selected item re-encoded with its pending edits.
    pub fn selected_item_serial(&self) -> Option<Vec<u8>> {
        let manager = self.manager.as_ref()?;
        let edit = self.item_edit.as_ref()?;
        items::pack(&edit.decoded, &manager.configs).ok()
    }

    pub fn apply_item(&mut self) {
        let Some(edit) = self.item_edit.take() else {
            return;
        };
        let Some(manager) = self.manager.clone() else {
            self.item_edit = Some(edit);
            return;
        };
        let serial = match items::pack(&edit.decoded, &manager.configs) {
            Ok(s) => s,
            Err(e) => {
                self.status = format!("pack error: {e}");
                self.item_edit = Some(edit);
                return;
            }
        };
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        if let Err(e) = save_game.replace_serial(edit.field_number, edit.occurrence, &serial) {
            self.status = format!("update error: {e}");
            return;
        }
        self.status = "item updated (remember to Save)".into();
        let index = self.selected_item;
        self.refresh_items();
        if let Some(i) = index {
            self.select_item(i);
        }
    }

    fn apply_form(&mut self) -> Option<Vec<u8>> {
        let mut save_game = self.save_game.clone()?;
        save_game.set_i32(2, self.form.level);
        save_game.set_i32(3, self.form.exp_points);
        save_game.set_i32(4, self.form.general_skill_points);
        save_game.set_i32(5, self.form.specialist_skill_points);
        save_game.set_i32(7, self.form.playthroughs);
        save_game.set_i32(25, self.form.total_play_time);
        save_game.set_bool(31, self.form.badass);
        save_game.set_i32(48, self.form.golden_keys);
        save_game.set_i32(56, self.form.bank_slots);
        if !self.form.currency.is_empty() {
            save_game.set_int_list_packed(6, &self.form.currency);
        }
        match container::serialize(&save_game.to_bytes(), Platform::PC) {
            Ok(bytes) => Some(bytes),
            Err(e) => {
                self.status = format!("serialize error: {e}");
                None
            }
        }
    }

    pub fn save_in_place(&mut self) {
        if self.path.trim().is_empty() {
            self.status = "nothing to save to".into();
            return;
        }
        let Some(bytes) = self.apply_form() else {
            return;
        };
        let backup = format!("{}.bak", self.path);
        if !std::path::Path::new(&backup).exists() {
            if let Err(e) = std::fs::copy(self.path.trim(), &backup) {
                self.status = format!("backup error: {e}");
                return;
            }
        }
        if let Err(e) = std::fs::write(self.path.trim(), &bytes) {
            self.status = format!("write error: {e}");
            return;
        }
        self.load();
        self.status = format!("saved {} bytes (backup {backup})", bytes.len());
    }

    pub fn save_as(&mut self) {
        let target = self.save_as.trim().to_string();
        if target.is_empty() {
            self.status = "enter a Save As path first".into();
            return;
        }
        let Some(bytes) = self.apply_form() else {
            return;
        };
        if let Err(e) = std::fs::write(&target, &bytes) {
            self.status = format!("write error: {e}");
            return;
        }
        self.status = format!("saved {} bytes to {target}", bytes.len());
    }

    /// Convenience for tests/tools: write the current form to `path`.
    pub fn save_to(&mut self, path: &str) {
        self.save_as = path.to_string();
        self.save_as();
    }

    fn browse_save(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Borderlands 2 save")
            .add_filter("Borderlands 2 save", &["sav"])
            .add_filter("All files", &["*"])
            .pick_file()
        {
            self.path = path.display().to_string();
            self.load();
        }
    }

    fn browse_dumps(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Open Asset Library Manager.json")
            .add_filter("JSON", &["json"])
            .pick_file()
        {
            self.dumps = path.display().to_string();
            self.load_dumps();
        }
    }

    /// Build the full UI. Called by both the native app and the test harness.
    pub fn build(&mut self, ui: &mut egui::Ui) {
        if !self.loaded_once {
            self.loaded_once = true;
            if !self.path.trim().is_empty() {
                self.load();
            }
        }

        egui::Panel::top("inputs").show(ui, |ui| {
            egui::Grid::new("paths").num_columns(4).show(ui, |ui| {
                ui.label("Save file");
                ui.text_edit_singleline(&mut self.path);
                if ui.button("Browse…").clicked() {
                    self.browse_save();
                }
                if ui.button("Load").clicked() {
                    self.load();
                }
                ui.end_row();

                ui.label("Asset Library");
                ui.text_edit_singleline(&mut self.dumps);
                if ui.button("Browse…").clicked() {
                    self.browse_dumps();
                }
                if ui.button("Load dumps").clicked() {
                    self.load_dumps();
                }
                ui.end_row();
            });
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.label(format!("Status: {}", self.status));
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.save_game.is_none() {
                ui.heading("Lootsmith");
                ui.label("Open a Borderlands 2 save to begin.");
                ui.label("Tip: load the Asset Library JSON so items show real names.");
                return;
            }

            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::General, "General");
                ui.selectable_value(&mut self.tab, Tab::Character, "Character");
                ui.selectable_value(&mut self.tab, Tab::Currency, "Currency");
                ui.selectable_value(&mut self.tab, Tab::Items, "Items");
            });
            ui.separator();

            match self.tab {
                Tab::General => self.ui_general(ui),
                Tab::Character => self.ui_character(ui),
                Tab::Currency => self.ui_currency(ui),
                Tab::Items => self.ui_items(ui),
            }
        });
    }

    fn ui_general(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("general").num_columns(2).show(ui, |ui| {
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
            ui.label("Total play time (s)");
            ui.add(egui::DragValue::new(&mut self.form.total_play_time));
            ui.end_row();
            ui.label("Badass mode");
            ui.checkbox(&mut self.form.badass, "");
            ui.end_row();
            ui.label("Golden keys");
            ui.add(egui::DragValue::new(&mut self.form.golden_keys));
            ui.end_row();
            ui.label("Bank slots");
            ui.add(egui::DragValue::new(&mut self.form.bank_slots));
            ui.end_row();
        });
        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_character(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("character").num_columns(2).show(ui, |ui| {
            ui.label("Player class");
            ui.label(&self.read_info.player_class);
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
    }

    fn ui_currency(&mut self, ui: &mut egui::Ui) {
        ui.label("Currency slots (index 0 is money):");
        egui::ScrollArea::vertical()
            .id_salt("currency")
            .max_height(360.0)
            .show(ui, |ui| {
                egui::Grid::new("currency_grid").num_columns(2).show(ui, |ui| {
                    for (i, value) in self.form.currency.iter_mut().enumerate() {
                        ui.label(format!("Slot {i}"));
                        ui.add(egui::DragValue::new(value));
                        ui.end_row();
                    }
                });
            });
        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_items(&mut self, ui: &mut egui::Ui) {
        let mut clicked = None;
        egui::ScrollArea::vertical()
            .id_salt("items_list")
            .max_height(180.0)
            .show(ui, |ui| {
                for (i, line) in self.items.iter().enumerate() {
                    let selected = self.selected_item == Some(i);
                    if ui.selectable_label(selected, line).clicked() {
                        clicked = Some(i);
                    }
                }
            });
        if let Some(i) = clicked {
            self.select_item(i);
        }

        ui.separator();
        if self.manager.is_none() {
            ui.label("Load an Asset Library to edit item parts by name.");
            return;
        }

        let manager = self.manager.clone();
        let mut edit_opt = self.item_edit.take();
        let mut apply = false;
        if let (Some(manager), Some(edit)) = (manager, edit_opt.as_mut()) {
            let is_weapon = edit.decoded.is_weapon;
            let set = edit.decoded.asset_library_set_id;
            for (slot, label) in slots_for(is_weapon) {
                ui.horizontal(|ui| {
                    ui.label(*label);
                    let group = items::slot_group(&edit.decoded.slot, slot).unwrap();
                    let current = match edit.decoded.part(slot) {
                        Some((r, g)) => manager
                            .lookup_asset(g, set, r.asset_index, r.sublibrary_index, r.use_set_id)
                            .unwrap_or_else(|_| format!("#{}/{}", r.asset_index, r.sublibrary_index)),
                        None => "-".to_string(),
                    };
                    let choices = self.choices.get(&group);
                    egui::ComboBox::from_id_salt(format!("slot_{slot}"))
                        .selected_text(current)
                        .width(520.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(false, "None").clicked() {
                                let _ = edit.decoded.set_part(&manager, slot, "None");
                                edit.dirty = true;
                            }
                            if let Some(list) = choices {
                                for choice in list {
                                    if ui.selectable_label(false, choice).clicked() {
                                        let _ = edit.decoded.set_part(&manager, slot, choice);
                                        edit.dirty = true;
                                    }
                                }
                            }
                        });
                });
            }
            ui.separator();
            ui.add_enabled_ui(edit.dirty, |ui| {
                if ui.button("Apply item changes").clicked() {
                    apply = true;
                }
            });
        }
        self.item_edit = edit_opt;
        if apply {
            self.apply_item();
        }
        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_save_buttons(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("Save (backup + write in place)").clicked() {
                self.save_in_place();
            }
            ui.label("Save As");
            ui.text_edit_singleline(&mut self.save_as);
            if ui.button("Save As").clicked() {
                self.save_as();
            }
        });
    }
}

impl eframe::App for LootsmithApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.build(ui);
    }
}
