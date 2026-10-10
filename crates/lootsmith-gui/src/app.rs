//! Lootsmith GUI application state and UI.
//!
//! The UI lives in [`LootsmithApp::build`], independent of the native window, so
//! that both the `eframe` app and the `egui_kittest` harness run identical code.

use eframe::egui;
use lootsmith_core::{
    container, items, AssetGroup, AssetLibraryConfigs, AssetLibraryManager, Container, PackedEntry,
    Platform, SaveGame, TravelStations,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Tab {
    General,
    Character,
    Currency,
    Skills,
    Travel,
    Backpack,
    Bank,
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

/// One selectable row in the backpack/bank list.
#[derive(Clone, Debug)]
pub struct Row {
    pub container: Container,
    pub occurrence: usize,
    pub text: String,
    pub quantity: i32,
    pub equipped: bool,
    pub mark: i32,
    pub quick_slot: i32,
    pub is_weapon: bool,
    /// The blob is a blank placeholder (unique id 0).
    pub empty: bool,
}

/// One editable skill row (a `SkillData` entry, field 8).
#[derive(Clone, Debug)]
pub struct SkillRow {
    pub name: String,
    pub grade: i32,
    pub points: i32,
    pub slot: i32,
    pub orig_grade: i32,
    pub orig_points: i32,
}

/// A readable `Tree.Leaf` label from a full skill resource path.
fn short_skill(name: &str) -> String {
    match name.rsplit_once('.') {
        Some((prefix, leaf)) => {
            let tree = prefix.rsplit('.').next().unwrap_or(prefix);
            format!("{tree}.{leaf}")
        }
        None => name.to_string(),
    }
}

struct ItemEdit {
    container: Container,
    occurrence: usize,
    decoded: items::PackedData,
    quantity: i32,
    equipped: bool,
    orig_quantity: i32,
    orig_equipped: bool,
    dirty: bool,
}

pub struct LootsmithApp {
    pub path: String,
    pub dumps: String,
    pub manager: Option<Arc<AssetLibraryManager>>,
    pub save_game: Option<SaveGame>,
    pub form: Form,
    pub read_info: ReadInfo,
    pub backpack: Vec<Row>,
    pub bank: Vec<Row>,
    pub skills: Vec<SkillRow>,
    /// Visited teleporters (field 16), by station `resource_name`.
    pub travel: Vec<String>,
    pub travel_last: Option<String>,
    pub stations: Option<Arc<TravelStations>>,
    pub stations_path: String,
    pub new_station: Option<String>,
    pub status: String,
    pub save_as: String,
    loaded_once: bool,
    pub tab: Tab,
    /// Selected entry as (container, occurrence).
    pub selected: Option<(Container, usize)>,
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

fn resolve_balance(
    manager: &Option<Arc<AssetLibraryManager>>,
    decoded: &items::PackedData,
) -> String {
    let set = decoded.asset_library_set_id;
    let r = match &decoded.slot {
        items::PackedSlot::Weapon(w) => w.balance,
        items::PackedSlot::Item(i) => i.balance,
    };
    match manager {
        Some(m) => m
            .lookup_asset(
                AssetGroup::BalanceDefs,
                set,
                r.asset_index,
                r.sublibrary_index,
                r.use_set_id,
            )
            .unwrap_or_else(|_| format!("#{}/{}", r.asset_index, r.sublibrary_index)),
        None => format!("#{}/{}", r.asset_index, r.sublibrary_index),
    }
}

fn row_for(
    entry: &PackedEntry,
    display: usize,
    manager: &Option<Arc<AssetLibraryManager>>,
    configs: &AssetLibraryConfigs,
) -> Row {
    match items::unpack(&entry.serial, configs) {
        Ok(decoded) => {
            let kind = match entry.container {
                Container::BackpackItem => "item",
                Container::BackpackWeapon => "weapon",
                Container::Bank => {
                    if decoded.is_weapon {
                        "bank weapon"
                    } else {
                        "bank item"
                    }
                }
            };
            let meta = match entry.container {
                Container::BackpackItem => {
                    format!(" qty={} equipped={}", entry.quantity, entry.equipped)
                }
                Container::BackpackWeapon => format!(" quick={}", entry.quick_slot),
                Container::Bank => String::new(),
            };
            let mark = if decoded.unique_id == 0 { " (empty)" } else { "" };
            Row {
                container: entry.container,
                occurrence: entry.occurrence,
                text: format!(
                    "{kind} #{display} id={} balance={}{meta}{mark}",
                    decoded.unique_id,
                    resolve_balance(manager, &decoded),
                ),
                quantity: entry.quantity,
                equipped: entry.equipped,
                mark: entry.mark,
                quick_slot: entry.quick_slot,
                is_weapon: decoded.is_weapon,
                empty: decoded.unique_id == 0,
            }
        }
        Err(e) => Row {
            container: entry.container,
            occurrence: entry.occurrence,
            text: format!("#{display} decode error: {e}"),
            quantity: entry.quantity,
            equipped: entry.equipped,
            mark: entry.mark,
            quick_slot: entry.quick_slot,
            is_weapon: matches!(entry.container, Container::BackpackWeapon),
            empty: false,
        },
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
            backpack: Vec::new(),
            bank: Vec::new(),
            skills: Vec::new(),
            travel: Vec::new(),
            travel_last: None,
            stations: None,
            stations_path: String::new(),
            new_station: None,
            status: "Open a save to begin (Browse… or --open).".to_string(),
            save_as: String::new(),
            loaded_once: false,
            tab: Tab::General,
            selected: None,
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

    /// The `InventorySerialNumber` bytes of one entry.
    pub fn item_serial(&self, container: Container, occurrence: usize) -> Option<Vec<u8>> {
        self.save_game
            .as_ref()?
            .entries_in(container)
            .get(occurrence)
            .map(|e| e.serial.clone())
    }

    /// Rows shown by a tab (backpack covers items + weapons).
    pub fn rows(&self, tab: Tab) -> &[Row] {
        match tab {
            Tab::Bank => &self.bank,
            _ => &self.backpack,
        }
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
        self.selected = None;
        self.item_edit = None;
        self.status = format!(
            "loaded {} ({} bytes, crc32 {})",
            self.path,
            data.len(),
            if save.crc32_valid() { "ok" } else { "MISMATCH" }
        );
        self.refresh_rows();
        self.refresh_skills();
        self.refresh_travel();
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
        self.refresh_rows();
        self.load_travel_stations();
    }

    fn refresh_rows(&mut self) {
        let configs = self.configs();
        let manager = self.manager.clone();
        let mut backpack = Vec::new();
        let mut bank = Vec::new();
        if let Some(save_game) = &self.save_game {
            for (i, entry) in save_game.packed_entries().iter().enumerate() {
                backpack.push(row_for(entry, i + 1, &manager, &configs));
            }
            for (i, entry) in save_game.bank_entries().iter().enumerate() {
                bank.push(row_for(entry, i + 1, &manager, &configs));
            }
        }
        self.backpack = backpack;
        self.bank = bank;
    }

    pub fn select_entry(&mut self, container: Container, occurrence: usize) {
        self.selected = Some((container, occurrence));
        self.item_edit = None;
        self.choices.clear();
        let Some(manager) = self.manager.clone() else {
            self.status = "Load an Asset Library to edit item parts by name.".into();
            return;
        };
        let entry = {
            let Some(save_game) = &self.save_game else {
                return;
            };
            save_game.entries_in(container).into_iter().nth(occurrence)
        };
        let Some(entry) = entry else {
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
            container,
            occurrence,
            decoded,
            quantity: entry.quantity,
            equipped: entry.equipped,
            orig_quantity: entry.quantity,
            orig_equipped: entry.equipped,
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

    /// Set the stack quantity of the selected backpack item (programmatic).
    pub fn set_selected_quantity(&mut self, quantity: i32) -> Result<(), String> {
        let Some(edit) = self.item_edit.as_mut() else {
            return Err("no item selected".into());
        };
        edit.quantity = quantity;
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
        if !edit.dirty {
            self.item_edit = Some(edit);
            return;
        }
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
        if let Err(e) = save_game.replace_serial(edit.container.field_number(), edit.occurrence, &serial)
        {
            self.status = format!("update error: {e}");
            return;
        }
        if edit.container == Container::BackpackItem {
            if edit.quantity != edit.orig_quantity {
                let _ = save_game.set_entry_quantity(edit.container, edit.occurrence, edit.quantity);
            }
            if edit.equipped != edit.orig_equipped {
                let _ = save_game.set_entry_equipped(edit.container, edit.occurrence, edit.equipped);
            }
        }
        self.status = "item updated (remember to Save)".into();
        let selected = self.selected;
        self.refresh_rows();
        if let Some((container, occurrence)) = selected {
            self.select_entry(container, occurrence);
        }
    }

    /// Move the selected entry to another container (bank <-> backpack).
    pub fn move_selected_to(&mut self, to: Container) {
        let Some((from, occurrence)) = self.selected else {
            return;
        };
        if from == to {
            return;
        }
        let result = {
            let Some(save_game) = self.save_game.as_mut() else {
                return;
            };
            save_game.move_entry(from, occurrence, to)
        };
        match result {
            Ok(new_occurrence) => {
                self.tab = match to {
                    Container::Bank => Tab::Bank,
                    _ => Tab::Backpack,
                };
                self.refresh_rows();
                self.select_entry(to, new_occurrence);
                self.status = format!("moved to {}", to.label());
            }
            Err(e) => self.status = format!("move error: {e}"),
        }
    }

    /// Remove the selected entry from its container.
    pub fn remove_selected(&mut self) {
        let Some((container, occurrence)) = self.selected else {
            return;
        };
        let result = {
            let Some(save_game) = self.save_game.as_mut() else {
                return;
            };
            save_game.remove_entry(container, occurrence)
        };
        match result {
            Ok(()) => {
                self.selected = None;
                self.item_edit = None;
                self.refresh_rows();
                self.status = format!("removed {} #{}", container.label(), occurrence + 1);
            }
            Err(e) => self.status = format!("remove error: {e}"),
        }
    }

    fn refresh_skills(&mut self) {
        self.skills = self
            .save_game
            .as_ref()
            .map(|save_game| {
                save_game
                    .skills()
                    .into_iter()
                    .map(|s| SkillRow {
                        name: s.skill,
                        grade: s.grade,
                        points: s.grade_points,
                        slot: s.equipped_slot_index,
                        orig_grade: s.grade,
                        orig_points: s.grade_points,
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    /// Whether any skill row differs from what is stored in the save.
    pub fn skills_dirty(&self) -> bool {
        self.skills
            .iter()
            .any(|r| r.grade != r.orig_grade || r.points != r.orig_points)
    }

    /// Write changed skill grade/points back into the save.
    pub fn apply_skills(&mut self) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let mut changed = 0usize;
        for (i, row) in self.skills.iter_mut().enumerate() {
            if row.grade != row.orig_grade && save_game.set_skill_grade(i, row.grade).is_ok() {
                row.orig_grade = row.grade;
                changed += 1;
            }
            if row.points != row.orig_points
                && save_game.set_skill_grade_points(i, row.points).is_ok()
            {
                row.orig_points = row.points;
                changed += 1;
            }
        }
        self.status = if changed == 0 {
            "no skill changes".into()
        } else {
            format!("{changed} skill change(s) applied (remember to Save)")
        };
    }

    /// Point at an explicit `Travel Stations.json` and load it.
    pub fn set_stations_path(&mut self, path: String) {
        self.stations_path = path;
        self.load_travel_stations();
        self.refresh_travel();
    }

    fn load_travel_stations(&mut self) {
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if !self.stations_path.trim().is_empty() {
            candidates.push(std::path::PathBuf::from(self.stations_path.trim()));
        }
        if let Ok(env) = std::env::var("LOOTSMITH_TRAVEL_STATIONS") {
            candidates.push(std::path::PathBuf::from(env));
        }
        if let Some(dir) = std::path::Path::new(self.dumps.trim()).parent() {
            candidates.push(dir.join("Travel Stations.json"));
        }
        for candidate in candidates {
            if candidate.exists() {
                if let Ok(stations) = TravelStations::from_path(&candidate) {
                    self.stations = Some(Arc::new(stations));
                    self.status = format!("loaded travel stations from {}", candidate.display());
                    return;
                }
            }
        }
        self.stations = None;
    }

    fn refresh_travel(&mut self) {
        match &self.save_game {
            Some(save_game) => {
                self.travel = save_game.visited_teleporters();
                self.travel_last = save_game.last_visited_teleporter();
            }
            None => {
                self.travel.clear();
                self.travel_last = None;
            }
        }
    }

    /// A label for a save station string, resolved when stations are loaded.
    pub fn travel_label(&self, name: &str) -> String {
        match &self.stations {
            Some(stations) => stations.label_for(name),
            None => name.to_string(),
        }
    }

    pub fn add_teleporter(&mut self, name: &str) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let added = save_game.add_visited_teleporter(name);
        self.status = if added {
            format!("added teleporter {name}")
        } else {
            format!("{name} already visited")
        };
        self.refresh_travel();
    }

    pub fn remove_teleporter(&mut self, index: usize) {
        let Some(name) = self.travel.get(index).cloned() else {
            return;
        };
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let removed = save_game.remove_visited_teleporter(&name);
        self.status = if removed {
            format!("removed teleporter {name}")
        } else {
            format!("{name} was not visited")
        };
        self.refresh_travel();
    }

    pub fn set_last_teleporter(&mut self, name: &str) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        save_game.set_last_visited_teleporter(name);
        self.status = format!("last visited = {name}");
        self.refresh_travel();
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
                ui.selectable_value(&mut self.tab, Tab::Skills, "Skills");
                ui.selectable_value(&mut self.tab, Tab::Travel, "Fast Travel");
                ui.selectable_value(&mut self.tab, Tab::Backpack, "Backpack");
                ui.selectable_value(&mut self.tab, Tab::Bank, "Bank");
            });
            ui.separator();

            match self.tab {
                Tab::General => self.ui_general(ui),
                Tab::Character => self.ui_character(ui),
                Tab::Currency => self.ui_currency(ui),
                Tab::Skills => self.ui_skills(ui),
                Tab::Travel => self.ui_travel(ui),
                Tab::Backpack => self.ui_container(ui, false),
                Tab::Bank => self.ui_container(ui, true),
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

    fn ui_skills(&mut self, ui: &mut egui::Ui) {
        ui.label("Skill ranks (grade 0-5, plus grade points). Apply, then Save.");
        egui::ScrollArea::vertical()
            .id_salt("skills")
            .max_height(420.0)
            .show(ui, |ui| {
                egui::Grid::new("skills_grid")
                    .num_columns(4)
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Skill");
                        ui.strong("Grade");
                        ui.strong("Points");
                        ui.strong("Slot");
                        ui.end_row();
                        for row in self.skills.iter_mut() {
                            ui.label(short_skill(&row.name));
                            ui.add(egui::DragValue::new(&mut row.grade));
                            ui.add(egui::DragValue::new(&mut row.points));
                            ui.label(row.slot.to_string());
                            ui.end_row();
                        }
                    });
            });
        let dirty = self.skills_dirty();
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_enabled_ui(dirty, |ui| {
                if ui.button("Apply skill changes").clicked() {
                    self.apply_skills();
                }
            });
            if dirty {
                ui.label("(unsaved changes)");
            }
        });
        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_travel(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label(format!("Visited stations: {}", self.travel.len()));
            if self.stations.is_some() {
                ui.label("(names resolved from Travel Stations.json)");
            } else {
                ui.label("(load Travel Stations.json next to the Asset Library for names)");
            }
        });

        let choices = self
            .stations
            .as_ref()
            .map(|s| s.destination_labels())
            .unwrap_or_default();
        let mut add_requested = false;
        ui.horizontal(|ui| {
            ui.label("Add station:");
            if choices.is_empty() {
                ui.label("—");
            } else {
                egui::ComboBox::from_id_salt("add_station")
                    .selected_text(
                        self.new_station
                            .clone()
                            .unwrap_or_else(|| "choose station…".to_string()),
                    )
                    .width(420.0)
                    .show_ui(ui, |ui| {
                        for choice in &choices {
                            let selected = self.new_station.as_deref() == Some(choice.as_str());
                            if ui.selectable_label(selected, choice).clicked() {
                                self.new_station = Some(choice.clone());
                            }
                        }
                    });
            }
            ui.add_enabled_ui(self.new_station.is_some(), |ui| {
                if ui.button("Add station").clicked() {
                    add_requested = true;
                }
            });
        });
        if add_requested {
            if let Some(name) = self.new_station.clone() {
                self.add_teleporter(&name);
            }
        }

        let travel = self.travel.clone();
        let last = self.travel_last.clone();
        let stations = self.stations.clone();
        let mut remove = None;
        let mut set_last = None;
        ui.separator();
        egui::ScrollArea::vertical()
            .id_salt("travel")
            .max_height(360.0)
            .show(ui, |ui| {
                if travel.is_empty() {
                    ui.label("(no stations visited)");
                }
                for (i, name) in travel.iter().enumerate() {
                    ui.horizontal(|ui| {
                        let label = match &stations {
                            Some(s) => s.label_for(name),
                            None => name.clone(),
                        };
                        let is_last = last.as_deref() == Some(name.as_str());
                        ui.label(format!("{}{label}", if is_last { "★ " } else { "" }));
                        if ui.small_button("Set last").clicked() {
                            set_last = Some(name.clone());
                        }
                        if ui.small_button("Remove").clicked() {
                            remove = Some(i);
                        }
                    });
                }
            });
        if let Some(name) = set_last {
            self.set_last_teleporter(&name);
        }
        if let Some(i) = remove {
            self.remove_teleporter(i);
        }

        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_container(&mut self, ui: &mut egui::Ui, bank: bool) {
        let rows = if bank {
            self.bank.clone()
        } else {
            self.backpack.clone()
        };
        let selected = self.selected;
        let mut clicked: Option<(Container, usize)> = None;
        egui::ScrollArea::vertical()
            .id_salt(if bank { "bank_list" } else { "backpack_list" })
            .max_height(200.0)
            .show(ui, |ui| {
                if rows.is_empty() {
                    ui.label(if bank { "(bank is empty)" } else { "(backpack is empty)" });
                }
                for row in &rows {
                    let is_selected = selected == Some((row.container, row.occurrence));
                    if ui.selectable_label(is_selected, &row.text).clicked() {
                        clicked = Some((row.container, row.occurrence));
                    }
                }
            });
        if let Some((container, occurrence)) = clicked {
            self.select_entry(container, occurrence);
        }

        ui.separator();
        if self.manager.is_none() {
            ui.label("Load an Asset Library to edit item parts by name.");
            self.ui_save_buttons(ui);
            return;
        }

        let manager = self.manager.clone();
        let mut edit_opt = self.item_edit.take();
        let mut apply = false;
        let mut move_to: Option<Container> = None;
        let mut remove = false;
        if let (Some(manager), Some(edit)) = (manager, edit_opt.as_mut()) {
            let display = rows
                .iter()
                .position(|r| r.container == edit.container && r.occurrence == edit.occurrence)
                .map(|i| i + 1);
            ui.horizontal(|ui| {
                let title = match display {
                    Some(n) => format!("{} #{n}", edit.container.label()),
                    None => edit.container.label().to_string(),
                };
                ui.strong(title);
                ui.label(format!("id={}", edit.decoded.unique_id));
            });
            if edit.container == Container::BackpackItem {
                ui.horizontal(|ui| {
                    ui.label("Quantity");
                    if ui.add(egui::DragValue::new(&mut edit.quantity)).changed() {
                        edit.dirty = true;
                    }
                    if ui.checkbox(&mut edit.equipped, "Equipped").changed() {
                        edit.dirty = true;
                    }
                });
            }
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
            ui.horizontal(|ui| {
                ui.add_enabled_ui(edit.dirty, |ui| {
                    if ui.button("Apply item changes").clicked() {
                        apply = true;
                    }
                });
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
        }
        self.item_edit = edit_opt;
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
