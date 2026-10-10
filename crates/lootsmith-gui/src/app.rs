//! Lootsmith GUI application state and UI.
//!
//! The UI lives in [`LootsmithApp::build`], independent of the native window, so
//! that both the `eframe` app and the `egui_kittest` harness run identical code.

use eframe::egui;
use lootsmith_core::savegame::Kind;
use lootsmith_core::{
    container, items, mission_status_name, player_mark_name, AssetGroup, AssetLibraryConfigs,
    AssetLibraryManager, Container, Customizations, Missions, PackedEntry, Platform, SaveGame,
    TravelStations, VehicleCustomization, MISSION_STATUSES, PLAYER_MARKS, VEHICLE_FAMILIES,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

/// Borderlands-ish accent colour.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(247, 156, 43);

/// UI colour theme.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Tab {
    General,
    Character,
    Currency,
    Skills,
    Travel,
    Vehicles,
    Missions,
    Backpack,
    Bank,
    Raw,
}

#[derive(Clone)]
pub struct Form {
    pub character_name: String,
    pub head: String,
    pub skin: String,
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
            character_name: String::new(),
            head: "None".to_string(),
            skin: "None".to_string(),
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

/// One editable vehicle family row (a `ChosenVehicleCustomization`, field 57).
#[derive(Clone, Debug)]
pub struct VehicleRow {
    pub usage: String,
    pub family: String,
    pub skin1: String,
    pub skin2: String,
    pub extra: Vec<String>,
    pub orig_skin1: String,
    pub orig_skin2: String,
}

/// One editable mission row inside a playthrough.
#[derive(Clone, Debug)]
pub struct MissionRow {
    pub playthrough: usize,
    pub index: usize,
    pub key: String,
    pub status: i32,
    pub orig_status: i32,
}

/// A scalar edit target for the raw field view.
#[derive(Clone, Debug)]
pub enum RawEdit {
    None,
    Int(i32),
    Str(String),
    Bool(bool),
}

/// One top-level field, for the raw view.
#[derive(Clone, Debug)]
pub struct RawField {
    pub number: u32,
    pub name: String,
    pub kind: String,
    pub count: usize,
    pub preview: String,
    pub edit: RawEdit,
}

fn customization_combo(
    ui: &mut egui::Ui,
    customizations: &Option<Arc<Customizations>>,
    usage: Option<&str>,
    kind: &str,
    value: &mut String,
    id: &str,
) {
    let current = match customizations {
        Some(c) if value.as_str() != "None" => c.label_for(value.as_str()),
        _ => value.clone(),
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(current)
        .width(340.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(*value == "None", "None").clicked() {
                *value = "None".to_string();
            }
            if let (Some(c), Some(usage)) = (customizations, usage) {
                for (key, name) in c.type_options(usage, kind) {
                    if ui.selectable_label(*value == key, &name).clicked() {
                        *value = key;
                    }
                }
            }
        });
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

fn tab_key(tab: Tab) -> &'static str {
    match tab {
        Tab::General => "general",
        Tab::Character => "character",
        Tab::Currency => "currency",
        Tab::Skills => "skills",
        Tab::Travel => "travel",
        Tab::Vehicles => "vehicles",
        Tab::Missions => "missions",
        Tab::Backpack => "backpack",
        Tab::Bank => "bank",
        Tab::Raw => "raw",
    }
}

fn tab_from_key(key: &str) -> Option<Tab> {
    match key {
        "general" => Some(Tab::General),
        "character" => Some(Tab::Character),
        "currency" => Some(Tab::Currency),
        "skills" => Some(Tab::Skills),
        "travel" => Some(Tab::Travel),
        "vehicles" => Some(Tab::Vehicles),
        "missions" => Some(Tab::Missions),
        "backpack" => Some(Tab::Backpack),
        "bank" => Some(Tab::Bank),
        "raw" => Some(Tab::Raw),
        _ => None,
    }
}

/// A toolbar/shortcut action, dispatched after the panel closure.
#[derive(Clone, Copy)]
enum Action {
    Open,
    Save,
    SaveAs,
    Reload,
    LoadData,
    PickData,
    ToggleTheme,
}

/// A backpack/bank toolbar action.
#[derive(Clone, Copy)]
enum ItemAction {
    NewWeapon,
    NewItem,
    Duplicate,
    CopyCode,
    SyncAll,
    SyncEquipped,
    InsertCodes,
}

struct ItemEdit {
    container: Container,
    occurrence: usize,
    decoded: items::PackedData,
    quantity: i32,
    equipped: bool,
    mark: i32,
    quick_slot: i32,
    orig_quantity: i32,
    orig_equipped: bool,
    orig_mark: i32,
    orig_quick_slot: i32,
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
    /// Vehicle family rows (field 57) and steering mode (field 58).
    pub vehicles: Vec<VehicleRow>,
    pub steering_mode: i32,
    pub customizations: Option<Arc<Customizations>>,
    /// Missions (field 18) and challenge list (field 38).
    pub mission_rows: Vec<MissionRow>,
    pub plot_mission: i32,
    pub active_mission: i32,
    pub challenges: Vec<String>,
    pub missions_data: Option<Arc<Missions>>,
    /// Every top-level field, for the raw view.
    pub raw: Vec<RawField>,
    pub theme: Theme,
    theme_applied: bool,
    /// There are edits in memory that have not been written to disk.
    pub dirty: bool,
    pub status: String,
    pub save_as: String,
    /// The `BL2(base64)` item-code box used for copy/paste.
    pub item_code: String,
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
            vehicles: Vec::new(),
            steering_mode: 0,
            customizations: None,
            mission_rows: Vec::new(),
            plot_mission: 0,
            active_mission: 0,
            challenges: Vec::new(),
            missions_data: None,
            raw: Vec::new(),
            theme: Theme::Dark,
            theme_applied: false,
            dirty: false,
            status: "Open a save to begin (Browse… or --open).".to_string(),
            save_as: String::new(),
            item_code: String::new(),
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
            character_name: save_game.character_name().unwrap_or_default(),
            head: save_game
                .applied_customizations()
                .first()
                .cloned()
                .unwrap_or_else(|| "None".to_string()),
            skin: save_game
                .applied_customizations()
                .get(4)
                .cloned()
                .unwrap_or_else(|| "None".to_string()),
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
        self.refresh_vehicles();
        self.refresh_missions();
        self.refresh_raw();
        self.dirty = false;
    }

    /// The directory holding the dumps files (Asset Library Manager, Travel
    /// Stations, Customizations, Missions).
    fn dumps_dir(&self) -> Option<PathBuf> {
        let raw = self.dumps.trim();
        if raw.is_empty() {
            return None;
        }
        let path = PathBuf::from(raw);
        if path.is_dir() {
            Some(path)
        } else {
            path.parent().map(|p| p.to_path_buf())
        }
    }

    /// Load the asset library and the companion dumps. `self.dumps` may be
    /// either a directory or the `Asset Library Manager.json` file.
    pub fn load_dumps(&mut self) {
        let raw = if self.dumps.trim().is_empty() {
            std::env::var("LOOTSMITH_ASSET_LIBRARY").unwrap_or_default()
        } else {
            self.dumps.clone()
        };
        if raw.trim().is_empty() {
            self.manager = None;
            return;
        }
        let path = PathBuf::from(raw.trim());
        let library = if path.is_dir() {
            path.join("Asset Library Manager.json")
        } else {
            path
        };
        match AssetLibraryManager::from_path(&library) {
            Ok(m) => {
                self.manager = Some(Arc::new(m));
                self.status = format!("loaded asset library from {}", library.display());
            }
            Err(e) => {
                self.manager = None;
                self.status = format!("asset library error: {e}");
            }
        }
        self.refresh_rows();
        self.load_travel_stations();
        self.load_customizations();
        self.load_missions_data();
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
            mark: entry.mark,
            quick_slot: entry.quick_slot,
            orig_quantity: entry.quantity,
            orig_equipped: entry.equipped,
            orig_mark: entry.mark,
            orig_quick_slot: entry.quick_slot,
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
        if edit.container == Container::BackpackWeapon && edit.quick_slot != edit.orig_quick_slot {
            let _ = save_game.set_entry_quick_slot(edit.container, edit.occurrence, edit.quick_slot);
        }
        if matches!(
            edit.container,
            Container::BackpackItem | Container::BackpackWeapon
        ) && edit.mark != edit.orig_mark
        {
            let _ = save_game.set_entry_mark(edit.container, edit.occurrence, edit.mark);
        }
        self.dirty = true;
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
                self.dirty = true;
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
                self.dirty = true;
                self.status = format!("removed {} #{}", container.label(), occurrence + 1);
            }
            Err(e) => self.status = format!("remove error: {e}"),
        }
    }

    /// Create a blank weapon/item with a fresh id, append it, and select it.
    pub fn new_item(&mut self, bank: bool, is_weapon: bool) {
        let configs = self.configs();
        let data = items::PackedData {
            is_weapon,
            unique_id: items::random_unique_id(),
            asset_library_set_id: 0,
            slot: if is_weapon {
                items::PackedSlot::Weapon(items::PackedWeapon::default())
            } else {
                items::PackedSlot::Item(items::PackedItem::default())
            },
        };
        let serial = match items::pack(&data, &configs) {
            Ok(serial) => serial,
            Err(e) => {
                self.status = format!("create error: {e}");
                return;
            }
        };
        let container = if bank {
            Container::Bank
        } else if is_weapon {
            Container::BackpackWeapon
        } else {
            Container::BackpackItem
        };
        let result = match self.save_game.as_mut() {
            Some(save_game) => save_game.append_entry(container, &serial),
            None => return,
        };
        let occurrence = match result {
            Ok(occurrence) => occurrence,
            Err(e) => {
                self.status = format!("create error: {e}");
                return;
            }
        };
        self.dirty = true;
        self.tab = if bank { Tab::Bank } else { Tab::Backpack };
        self.refresh_rows();
        self.select_entry(container, occurrence);
        self.status = format!(
            "added a blank {} (edit its parts, then Save)",
            if is_weapon { "weapon" } else { "item" }
        );
    }

    /// Duplicate the selected entry with a fresh unique id.
    pub fn duplicate_selected(&mut self) {
        let Some((container, occurrence)) = self.selected else {
            return;
        };
        let Some(serial) = self.item_serial(container, occurrence) else {
            return;
        };
        let configs = self.configs();
        let Ok(mut decoded) = items::unpack(&serial, &configs) else {
            self.status = "duplicate: cannot decode item".into();
            return;
        };
        decoded.unique_id = items::random_unique_id();
        let Ok(new_serial) = items::pack(&decoded, &configs) else {
            return;
        };
        let result = match self.save_game.as_mut() {
            Some(save_game) => save_game.append_entry(container, &new_serial),
            None => return,
        };
        if let Ok(new_occurrence) = result {
            self.dirty = true;
            self.refresh_rows();
            self.select_entry(container, new_occurrence);
            self.status = "duplicated the selected item".into();
        }
    }

    /// Encode the selected entry as a `BL2(...)` code and copy it.
    pub fn copy_selected_code(&mut self, ctx: &egui::Context) -> bool {
        let Some((container, occurrence)) = self.selected else {
            return false;
        };
        let Some(serial) = self.item_serial(container, occurrence) else {
            return false;
        };
        let code = items::encode_code(&serial);
        ctx.copy_text(code.clone());
        self.item_code = code;
        self.status = "item code copied to the clipboard".into();
        true
    }

    /// Insert every `BL2(...)` code currently in the item-code box.
    pub fn insert_codes(&mut self, bank: bool) {
        let codes = items::decode_codes(&self.item_code);
        if codes.is_empty() {
            self.status = "no BL2(...) codes found in the box".into();
            return;
        }
        let configs = self.configs();
        let mut added = 0usize;
        let mut last: Option<(Container, usize)> = None;
        for bytes in codes {
            let Ok(mut decoded) = items::unpack(&bytes, &configs) else {
                continue;
            };
            decoded.unique_id = items::random_unique_id();
            let Ok(serial) = items::pack(&decoded, &configs) else {
                continue;
            };
            let container = if bank {
                Container::Bank
            } else if decoded.is_weapon {
                Container::BackpackWeapon
            } else {
                Container::BackpackItem
            };
            let result = match self.save_game.as_mut() {
                Some(save_game) => save_game.append_entry(container, &serial),
                None => return,
            };
            if let Ok(occurrence) = result {
                added += 1;
                last = Some((container, occurrence));
            }
        }
        self.dirty = true;
        self.item_code.clear();
        self.refresh_rows();
        if let Some((container, occurrence)) = last {
            self.tab = if bank { Tab::Bank } else { Tab::Backpack };
            self.select_entry(container, occurrence);
        }
        self.status = format!("inserted {added} item(s) from codes");
    }

    /// Set item level (manufacturer grade + game stage) on backpack items.
    pub fn sync_levels(&mut self, equipped_only: bool) {
        let level = self.form.level.clamp(0, 127);
        let configs = self.configs();
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let mut updated = 0usize;
        for container in [Container::BackpackWeapon, Container::BackpackItem] {
            let entries = save_game.entries_in(container);
            for entry in entries {
                let equipped = if container == Container::BackpackWeapon {
                    entry.quick_slot >= 0
                } else {
                    entry.equipped
                };
                if equipped_only && !equipped {
                    continue;
                }
                let Ok(mut decoded) = items::unpack(&entry.serial, &configs) else {
                    continue;
                };
                match &mut decoded.slot {
                    items::PackedSlot::Weapon(w) => {
                        w.manufacturer_grade_index = level;
                        w.game_stage = level;
                    }
                    items::PackedSlot::Item(i) => {
                        i.manufacturer_grade_index = level;
                        i.game_stage = level;
                    }
                }
                let Ok(serial) = items::pack(&decoded, &configs) else {
                    continue;
                };
                if save_game
                    .replace_serial(container.field_number(), entry.occurrence, &serial)
                    .is_ok()
                {
                    updated += 1;
                }
            }
        }
        self.dirty = true;
        self.refresh_rows();
        self.status = format!("synced {updated} item level(s) to {level}");
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
        if changed > 0 {
            self.dirty = true;
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
        if let Some(dir) = self.dumps_dir() {
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
        if added {
            self.dirty = true;
        }
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
        if removed {
            self.dirty = true;
        }
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
        self.dirty = true;
        self.status = format!("last visited = {name}");
        self.refresh_travel();
    }

    fn load_customizations(&mut self) {
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(env) = std::env::var("LOOTSMITH_CUSTOMIZATIONS") {
            candidates.push(std::path::PathBuf::from(env));
        }
        if let Some(dir) = self.dumps_dir() {
            candidates.push(dir.join("Customizations.json"));
        }
        for candidate in candidates {
            if candidate.exists() {
                if let Ok(customizations) = Customizations::from_path(&candidate) {
                    self.customizations = Some(Arc::new(customizations));
                    return;
                }
            }
        }
        self.customizations = None;
    }

    fn refresh_vehicles(&mut self) {
        let mut rows: Vec<VehicleRow> = VEHICLE_FAMILIES
            .iter()
            .map(|(usage, family)| VehicleRow {
                usage: (*usage).to_string(),
                family: (*family).to_string(),
                skin1: "None".to_string(),
                skin2: "None".to_string(),
                extra: Vec::new(),
                orig_skin1: "None".to_string(),
                orig_skin2: "None".to_string(),
            })
            .collect();
        if let Some(save_game) = &self.save_game {
            for vehicle in save_game.vehicles() {
                if let Some(row) = rows.iter_mut().find(|r| r.family == vehicle.family) {
                    row.skin1 = vehicle
                        .customizations
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "None".to_string());
                    row.skin2 = vehicle
                        .customizations
                        .get(1)
                        .cloned()
                        .unwrap_or_else(|| "None".to_string());
                    row.extra = vehicle.customizations.iter().skip(2).cloned().collect();
                    row.orig_skin1 = row.skin1.clone();
                    row.orig_skin2 = row.skin2.clone();
                }
            }
            self.steering_mode = save_game.vehicle_steering_mode().unwrap_or(0);
        } else {
            self.steering_mode = 0;
        }
        self.vehicles = rows;
    }

    /// Whether any vehicle row differs from what is stored in the save.
    pub fn vehicles_dirty(&self) -> bool {
        self.vehicles
            .iter()
            .any(|r| r.skin1 != r.orig_skin1 || r.skin2 != r.orig_skin2)
    }

    /// Write the vehicle families and steering mode back into the save.
    pub fn apply_vehicles(&mut self) {
        let mut list = Vec::new();
        for row in &self.vehicles {
            if row.skin1 != "None" || row.skin2 != "None" || !row.extra.is_empty() {
                let mut customizations = vec![row.skin1.clone(), row.skin2.clone()];
                customizations.extend(row.extra.iter().cloned());
                list.push(VehicleCustomization {
                    family: row.family.clone(),
                    customizations,
                });
            }
        }
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        save_game.set_vehicles(&list);
        save_game.set_vehicle_steering_mode(self.steering_mode);
        self.dirty = true;
        self.status = format!("vehicle settings applied ({} family set(s))", list.len());
        self.refresh_vehicles();
    }

    fn load_missions_data(&mut self) {
        let mut candidates: Vec<std::path::PathBuf> = Vec::new();
        if let Ok(env) = std::env::var("LOOTSMITH_MISSIONS") {
            candidates.push(std::path::PathBuf::from(env));
        }
        if let Some(dir) = self.dumps_dir() {
            candidates.push(dir.join("Missions.json"));
        }
        for candidate in candidates {
            if candidate.exists() {
                if let Ok(missions) = Missions::from_path(&candidate) {
                    self.missions_data = Some(Arc::new(missions));
                    return;
                }
            }
        }
        self.missions_data = None;
    }

    fn refresh_missions(&mut self) {
        if let Some(save_game) = &self.save_game {
            self.plot_mission = save_game.plot_mission_number().unwrap_or(0);
            self.active_mission = save_game.active_mission_number().unwrap_or(0);
            let mut rows = Vec::new();
            for (pi, playthrough) in save_game.mission_playthroughs().iter().enumerate() {
                for (mi, mission) in playthrough.missions.iter().enumerate() {
                    rows.push(MissionRow {
                        playthrough: pi,
                        index: mi,
                        key: mission.mission.clone(),
                        status: mission.status,
                        orig_status: mission.status,
                    });
                }
            }
            self.mission_rows = rows;
            self.challenges = save_game
                .challenges()
                .into_iter()
                .map(|c| c.challenge)
                .collect();
        } else {
            self.plot_mission = 0;
            self.active_mission = 0;
            self.mission_rows.clear();
            self.challenges.clear();
        }
    }

    /// Rebuild the raw field list from the current savegame.
    fn refresh_raw(&mut self) {
        let mut out = Vec::new();
        if let Some(sg) = &self.save_game {
            let mut i = 0;
            while i < sg.message.fields.len() {
                let field = &sg.message.fields[i];
                let number = field.number;
                let spec = SaveGame::spec(number);
                let count = sg.message.count(number);
                let edit = match spec.map(|s| s.kind) {
                    Some(Kind::Int) => field.as_i32().map(RawEdit::Int).unwrap_or(RawEdit::None),
                    Some(Kind::Bool) => field.as_bool().map(RawEdit::Bool).unwrap_or(RawEdit::None),
                    Some(Kind::Str) => field
                        .as_str()
                        .map(|s| RawEdit::Str(s.into_owned()))
                        .unwrap_or(RawEdit::None),
                    _ => RawEdit::None,
                };
                let preview = match &edit {
                    RawEdit::None => match field.wire {
                        lootsmith_core::proto::WireType::LengthDelimited => format!(
                            "{} bytes",
                            field.as_bytes().map(|b| b.len()).unwrap_or(0)
                        ),
                        _ => field.as_varint().map(|v| v.to_string()).unwrap_or_default(),
                    },
                    _ => String::new(),
                };
                out.push(RawField {
                    number,
                    name: spec.map(|s| s.name).unwrap_or("(unknown)").to_string(),
                    kind: spec
                        .map(|s| format!("{:?}", s.kind))
                        .unwrap_or_else(|| "?".into()),
                    count,
                    preview,
                    edit,
                });
                i += count.max(1);
            }
        }
        self.raw = out;
    }

    /// Whether any mission edit differs from what is stored in the save.
    pub fn missions_dirty(&self) -> bool {
        let Some(save_game) = &self.save_game else {
            return false;
        };
        self.plot_mission != save_game.plot_mission_number().unwrap_or(0)
            || self.active_mission != save_game.active_mission_number().unwrap_or(0)
            || self
                .mission_rows
                .iter()
                .any(|r| r.status != r.orig_status)
    }

    /// Write plot/active mission numbers and changed mission statuses.
    pub fn apply_missions(&mut self) {
        let mut changed = 0usize;
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        save_game.set_plot_mission_number(self.plot_mission);
        save_game.set_active_mission_number(self.active_mission);
        for row in &mut self.mission_rows {
            if row.status != row.orig_status
                && save_game
                    .set_mission_status(row.playthrough, row.index, row.status)
                    .is_ok()
            {
                row.orig_status = row.status;
                changed += 1;
            }
        }
        self.dirty = true;
        self.status = format!("missions applied ({changed} status change(s))");
    }

    fn apply_form(&mut self) -> Option<Vec<u8>> {
        let mut save_game = self.save_game.clone()?;
        let _ = save_game.set_character_name(&self.form.character_name);
        save_game.set_applied_head_skin(&self.form.head, &self.form.skin);
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
        self.dirty = false;
        self.status = format!("saved {} bytes to {target}", bytes.len());
    }

    /// Convenience for tests/tools: write the current form to `path`.
    pub fn save_to(&mut self, path: &str) {
        self.save_as = path.to_string();
        self.save_as();
    }

    // ---- settings / theme / shortcuts ----

    /// Serialize the persisted settings (paths, theme, tab) as JSON.
    pub fn settings_json(&self) -> String {
        #[derive(serde::Serialize)]
        struct Settings<'a> {
            save: &'a str,
            data: &'a str,
            theme: &'a str,
            tab: &'a str,
        }
        let theme = match self.theme {
            Theme::Dark => "dark",
            Theme::Light => "light",
        };
        serde_json::to_string(&Settings {
            save: self.path.trim(),
            data: self.dumps.trim(),
            theme,
            tab: tab_key(self.tab),
        })
        .unwrap_or_default()
    }

    /// Restore persisted settings. Paths only fill in what is not already set.
    pub fn load_settings_json(&mut self, text: &str) {
        #[derive(serde::Deserialize)]
        struct Settings {
            #[serde(default)]
            save: String,
            #[serde(default)]
            data: String,
            #[serde(default)]
            theme: String,
            #[serde(default)]
            tab: String,
        }
        let Ok(settings) = serde_json::from_str::<Settings>(text) else {
            return;
        };
        if self.path.trim().is_empty() {
            self.path = settings.save;
        }
        if self.dumps.trim().is_empty() {
            self.dumps = settings.data;
        }
        self.theme = if settings.theme == "light" {
            Theme::Light
        } else {
            Theme::Dark
        };
        self.theme_applied = false;
        if let Some(tab) = tab_from_key(&settings.tab) {
            self.tab = tab;
        }
        if !self.dumps.trim().is_empty() {
            self.load_dumps();
        }
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.theme_applied {
            return;
        }
        self.theme_applied = true;
        let mut visuals = match self.theme {
            Theme::Dark => egui::Visuals::dark(),
            Theme::Light => egui::Visuals::light(),
        };
        visuals.selection.bg_fill = ACCENT;
        visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(28, 24, 18));
        visuals.hyperlink_color = ACCENT;
        visuals.widgets.active.bg_fill = ACCENT;
        visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, ACCENT);
        ctx.set_visuals(visuals);
        ctx.all_styles_mut(|style| {
            style.spacing.item_spacing = egui::vec2(8.0, 6.0);
            style.spacing.button_padding = egui::vec2(8.0, 4.0);
            style.spacing.interact_size.y = 22.0;
        });
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (save_as, save, open, reload) = ctx.input_mut(|input| {
            use egui::{Key, KeyboardShortcut, Modifiers};
            (
                input.consume_shortcut(&KeyboardShortcut::new(
                    Modifiers::CTRL | Modifiers::SHIFT,
                    Key::S,
                )),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::S)),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::O)),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::R)),
            )
        });
        if save_as {
            self.browse_save_as();
        }
        if save {
            self.save_in_place();
        }
        if open {
            self.browse_save();
        }
        if reload {
            self.load();
        }
    }

    fn dispatch(&mut self, action: Action) {
        match action {
            Action::Open => self.browse_save(),
            Action::Save => self.save_in_place(),
            Action::SaveAs => self.browse_save_as(),
            Action::Reload => self.load(),
            Action::LoadData => {
                if !self.dumps.trim().is_empty() {
                    self.load_dumps();
                }
            }
            Action::PickData => self.browse_dumps_dir(),
            Action::ToggleTheme => {
                self.theme = match self.theme {
                    Theme::Dark => Theme::Light,
                    Theme::Light => Theme::Dark,
                };
                self.theme_applied = false;
            }
        }
    }

    fn browse_save_as(&mut self) {
        let name = if self.path.trim().is_empty() {
            "Save0001.sav".to_string()
        } else {
            PathBuf::from(self.path.trim())
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "Save0001.sav".to_string())
        };
        if let Some(path) = rfd::FileDialog::new()
            .set_title("Save As")
            .set_file_name(name)
            .add_filter("Borderlands 2 save", &["sav"])
            .save_file()
        {
            self.save_to(&path.display().to_string());
        }
    }

    fn browse_dumps_dir(&mut self) {
        if let Some(dir) = rfd::FileDialog::new()
            .set_title("Select the folder containing Asset Library Manager.json")
            .pick_folder()
        {
            self.dumps = dir.display().to_string();
            self.load_dumps();
        }
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

    /// Build the full UI. Called by both the native app and the test harness.
    pub fn build(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.handle_shortcuts(&ctx);

        if !self.loaded_once {
            self.loaded_once = true;
            if !self.path.trim().is_empty() {
                self.load();
            }
        }

        let mut action: Option<Action> = None;
        egui::Panel::top("toolbar").show(ui, |ui| {
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if ui.button("Open…").clicked() {
                    action = Some(Action::Open);
                }
                ui.add_enabled_ui(self.save_game.is_some(), |ui| {
                    if ui.button("Save").clicked() {
                        action = Some(Action::Save);
                    }
                    if ui.button("Save As…").clicked() {
                        action = Some(Action::SaveAs);
                    }
                    if ui.button("Reload").clicked() {
                        action = Some(Action::Reload);
                    }
                });
                ui.separator();
                if ui.button("Game data…").clicked() {
                    action = Some(Action::PickData);
                }
                match (&self.manager, &self.save_game) {
                    (Some(_), _) => {
                        ui.colored_label(ACCENT, "game data: loaded");
                    }
                    (None, _) => {
                        ui.colored_label(
                            egui::Color32::from_rgb(214, 122, 60),
                            "no game data — item parts show raw ids",
                        );
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let label = if self.theme == Theme::Dark { "☀" } else { "☾" };
                    if ui.button(label).on_hover_text("Toggle light/dark").clicked() {
                        action = Some(Action::ToggleTheme);
                    }
                    if self.dirty {
                        ui.colored_label(ACCENT, "● unsaved");
                    }
                });
            });
            ui.add_space(3.0);
            egui::Grid::new("paths")
                .num_columns(3)
                .spacing([10.0, 5.0])
                .show(ui, |ui| {
                    ui.label("Save file");
                    ui.add_sized(
                        [480.0, 22.0],
                        egui::TextEdit::singleline(&mut self.path)
                            .hint_text("/path/to/Save0001.sav"),
                    );
                    if ui.button("Load").clicked() {
                        action = Some(Action::Reload);
                    }
                    ui.end_row();

                    ui.label("Game data");
                    ui.add_sized(
                        [480.0, 22.0],
                        egui::TextEdit::singleline(&mut self.dumps)
                            .hint_text("folder containing Asset Library Manager.json"),
                    );
                    if ui.button("Load data").clicked() {
                        action = Some(Action::LoadData);
                    }
                    ui.end_row();
                });
            ui.add_space(4.0);
        });
        if let Some(action) = action {
            self.dispatch(action);
        }

        egui::Panel::bottom("status").show(ui, |ui| {
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                ui.label(format!("Status: {}", self.status));
            });
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.save_game.is_none() {
                ui.vertical_centered(|ui| {
                    ui.add_space(48.0);
                    ui.heading("Lootsmith");
                    ui.add_space(4.0);
                    ui.label("A cross-platform Borderlands 2 save editor.");
                    ui.add_space(12.0);
                    ui.label("Open a save with the toolbar, then pick your game-data folder so");
                    ui.label("item parts, missions, and travel stations resolve to real names.");
                });
                return;
            }

            ui.add_space(2.0);
            ui.horizontal(|ui| {
                for (tab, label) in [
                    (Tab::General, "General"),
                    (Tab::Character, "Character"),
                    (Tab::Currency, "Currency"),
                    (Tab::Skills, "Skills"),
                    (Tab::Travel, "Fast Travel"),
                    (Tab::Vehicles, "Vehicles"),
                    (Tab::Missions, "Missions"),
                    (Tab::Backpack, "Backpack"),
                    (Tab::Bank, "Bank"),
                    (Tab::Raw, "Raw"),
                ] {
                    ui.selectable_value(&mut self.tab, tab, label);
                }
            });
            ui.separator();

            match self.tab {
                Tab::General => self.ui_general(ui),
                Tab::Character => self.ui_character(ui),
                Tab::Currency => self.ui_currency(ui),
                Tab::Skills => self.ui_skills(ui),
                Tab::Travel => self.ui_travel(ui),
                Tab::Vehicles => self.ui_vehicles(ui),
                Tab::Missions => self.ui_missions(ui),
                Tab::Backpack => self.ui_container(ui, false),
                Tab::Bank => self.ui_container(ui, true),
                Tab::Raw => self.ui_raw(ui),
            }
        });
    }

    fn ui_general(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.heading("General");
        ui.add_space(4.0);
        egui::Grid::new("general")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                let mut changed = false;
                ui.label("Level");
                changed |= ui.add(egui::DragValue::new(&mut self.form.level)).changed();
                ui.end_row();
                ui.label("Experience");
                changed |= ui.add(egui::DragValue::new(&mut self.form.exp_points)).changed();
                ui.end_row();
                ui.label("General skill points");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.general_skill_points))
                    .changed();
                ui.end_row();
                ui.label("Specialist skill points");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.specialist_skill_points))
                    .changed();
                ui.end_row();
                ui.label("Playthroughs");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.playthroughs))
                    .changed();
                ui.end_row();
                ui.label("Total play time (s)");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.total_play_time))
                    .changed();
                ui.end_row();
                ui.label("Badass mode");
                changed |= ui.checkbox(&mut self.form.badass, "").changed();
                ui.end_row();
                ui.label("Golden keys");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.golden_keys))
                    .changed();
                ui.end_row();
                ui.label("Bank slots");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.bank_slots))
                    .changed();
                ui.end_row();
                if changed {
                    self.dirty = true;
                }
            });
        ui.add_space(8.0);
        self.ui_save_buttons(ui);
    }

    fn ui_character(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.heading("Character");
        ui.add_space(4.0);
        let customizations = self.customizations.clone();
        let usage = lootsmith_core::usage_for_player_class(&self.read_info.player_class);
        let mut changed = false;
        egui::Grid::new("character")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                ui.label("Name");
                changed |= ui
                    .add_sized(
                        [240.0, 22.0],
                        egui::TextEdit::singleline(&mut self.form.character_name),
                    )
                    .changed();
                ui.end_row();
                ui.label("Class");
                ui.label(&self.read_info.player_class);
                ui.end_row();
                ui.label("Level");
                changed |= ui.add(egui::DragValue::new(&mut self.form.level)).changed();
                ui.end_row();
                ui.label("Experience");
                changed |= ui.add(egui::DragValue::new(&mut self.form.exp_points)).changed();
                ui.end_row();
                ui.label("General skill points");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.general_skill_points))
                    .changed();
                ui.end_row();
                ui.label("Specialist skill points");
                changed |= ui
                    .add(egui::DragValue::new(&mut self.form.specialist_skill_points))
                    .changed();
                ui.end_row();
                ui.label("Head");
                customization_combo(
                    ui,
                    &customizations,
                    usage,
                    "Head",
                    &mut self.form.head,
                    "char_head",
                );
                ui.end_row();
                ui.label("Skin");
                customization_combo(
                    ui,
                    &customizations,
                    usage,
                    "Skin",
                    &mut self.form.skin,
                    "char_skin",
                );
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
        if changed {
            self.dirty = true;
        }
        if customizations.is_none() {
            ui.add_space(4.0);
            ui.label("Load the game-data folder to pick heads and skins by name.");
        }
        ui.add_space(8.0);
        self.ui_save_buttons(ui);
    }

    fn ui_currency(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.heading("Currency");
        ui.add_space(2.0);
        ui.label("Index 0 is money. Other slots are eridium, seraph crystals, Torgue tokens, ...");
        ui.add_space(4.0);
        let mut changed = false;
        egui::ScrollArea::vertical()
            .id_salt("currency")
            .max_height(420.0)
            .show(ui, |ui| {
                egui::Grid::new("currency_grid")
                    .num_columns(2)
                    .spacing([16.0, 6.0])
                    .show(ui, |ui| {
                        for (i, value) in self.form.currency.iter_mut().enumerate() {
                            ui.label(format!("Slot {i}"));
                            changed |= ui.add(egui::DragValue::new(value)).changed();
                            ui.end_row();
                        }
                    });
            });
        if changed {
            self.dirty = true;
        }
        ui.add_space(8.0);
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

    fn ui_vehicles(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Vehicle steering mode");
            ui.add(egui::DragValue::new(&mut self.steering_mode));
        });
        if self.customizations.is_some() {
            ui.label("Skin names resolved from Customizations.json.");
        } else {
            ui.label("Load Customizations.json next to the Asset Library to pick skins by name.");
        }
        ui.separator();

        let customizations = self.customizations.clone();
        for row in self.vehicles.iter_mut() {
            ui.horizontal(|ui| {
                ui.strong(format!("{}:", row.usage));
                for (slot, value) in [("Skin 1", &mut row.skin1), ("Skin 2", &mut row.skin2)] {
                    let current = match (&customizations, value.as_str()) {
                        (Some(_), "None") => "None".to_string(),
                        (Some(c), key) => c.label_for(key),
                        (None, key) => key.to_string(),
                    };
                    egui::ComboBox::from_id_salt(format!("{slot}_{}", row.usage))
                        .selected_text(current)
                        .width(240.0)
                        .show_ui(ui, |ui| {
                            if ui.selectable_label(value == "None", "None").clicked() {
                                *value = "None".to_string();
                            }
                            if let Some(c) = &customizations {
                                for (key, name) in c.skin_options(&row.usage) {
                                    if ui.selectable_label(value == &key, &name).clicked() {
                                        *value = key;
                                    }
                                }
                            }
                        });
                }
            });
        }

        let dirty = self.vehicles_dirty();
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_enabled_ui(dirty, |ui| {
                if ui.button("Apply vehicle changes").clicked() {
                    self.apply_vehicles();
                }
            });
            if dirty {
                ui.label("(unsaved changes)");
            }
        });
        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_missions(&mut self, ui: &mut egui::Ui) {
        let missions_data = self.missions_data.clone();
        let name_for = |number: i32| match &missions_data {
            Some(m) => m
                .label_for_number(number)
                .unwrap_or_else(|| format!("(unknown {number})")),
            None => number.to_string(),
        };

        ui.horizontal(|ui| {
            ui.label("Plot mission");
            ui.add(egui::DragValue::new(&mut self.plot_mission));
            ui.label(name_for(self.plot_mission));
        });
        ui.horizontal(|ui| {
            ui.label("Active mission");
            ui.add(egui::DragValue::new(&mut self.active_mission));
            ui.label(name_for(self.active_mission));
        });
        ui.separator();

        ui.label(format!("Missions ({})", self.mission_rows.len()));
        egui::ScrollArea::vertical()
            .id_salt("missions")
            .max_height(280.0)
            .show(ui, |ui| {
                egui::Grid::new("missions_grid")
                    .num_columns(3)
                    .striped(true)
                    .show(ui, |ui| {
                        for row in self.mission_rows.iter_mut() {
                            let label = match &missions_data {
                                Some(m) => m.label_for(&row.key),
                                None => row.key.clone(),
                            };
                            ui.label(format!("P{}", row.playthrough + 1));
                            ui.label(label);
                            egui::ComboBox::from_id_salt(format!(
                                "mission_status_{}_{}",
                                row.playthrough, row.index
                            ))
                            .selected_text(mission_status_name(row.status))
                            .width(200.0)
                            .show_ui(ui, |ui| {
                                for (value, name) in MISSION_STATUSES.iter().enumerate() {
                                    let selected = row.status == value as i32;
                                    if ui.selectable_label(selected, *name).clicked() {
                                        row.status = value as i32;
                                    }
                                }
                            });
                            ui.end_row();
                        }
                    });
            });

        let dirty = self.missions_dirty();
        ui.separator();
        ui.horizontal(|ui| {
            ui.add_enabled_ui(dirty, |ui| {
                if ui.button("Apply mission changes").clicked() {
                    self.apply_missions();
                }
            });
            if dirty {
                ui.label("(unsaved changes)");
            }
        });
        ui.separator();

        ui.label(format!("Challenges ({})", self.challenges.len()));
        egui::ScrollArea::vertical()
            .id_salt("challenges")
            .max_height(140.0)
            .show(ui, |ui| {
                for (i, challenge) in self.challenges.iter().enumerate() {
                    ui.label(format!("[{i:>3}] {challenge}"));
                }
            });
        ui.separator();
        self.ui_save_buttons(ui);
    }

    fn ui_raw(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.heading("Raw fields");
        ui.add_space(2.0);
        ui.label("Edit scalar top-level fields directly. Lists, messages and byte blobs are read-only here.");
        ui.add_space(4.0);
        let mut changed = false;
        egui::ScrollArea::vertical()
            .id_salt("raw")
            .show(ui, |ui| {
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
                            ui.label(field.number.to_string());
                            ui.label(&field.name);
                            ui.label(&field.kind);
                            ui.label(field.count.to_string());
                            match &mut field.edit {
                                RawEdit::Int(v) => {
                                    changed |= ui.add(egui::DragValue::new(v)).changed();
                                }
                                RawEdit::Bool(v) => {
                                    changed |= ui.checkbox(v, "").changed();
                                }
                                RawEdit::Str(v) => {
                                    changed |= ui
                                        .add_sized([300.0, 20.0], egui::TextEdit::singleline(v))
                                        .changed();
                                }
                                RawEdit::None => {
                                    ui.label(&field.preview);
                                }
                            }
                            ui.end_row();
                        }
                    });
            });
        if changed {
            let updates: Vec<(u32, RawEdit)> =
                self.raw.iter().map(|f| (f.number, f.edit.clone())).collect();
            if let Some(save_game) = self.save_game.as_mut() {
                for (number, edit) in updates {
                    match edit {
                        RawEdit::Int(v) => save_game.set_i32(number, v),
                        RawEdit::Bool(v) => save_game.set_bool(number, v),
                        RawEdit::Str(v) => save_game.set_str(number, &v),
                        RawEdit::None => {}
                    }
                }
            }
            self.dirty = true;
        }
        ui.add_space(8.0);
        self.ui_save_buttons(ui);
    }

    fn ui_container(&mut self, ui: &mut egui::Ui, bank: bool) {
        ui.add_space(4.0);
        let mut item_action: Option<ItemAction> = None;
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
            ui.separator();
            if ui.button("Sync levels (all)").clicked() {
                item_action = Some(ItemAction::SyncAll);
            }
            if ui.button("Sync equipped").clicked() {
                item_action = Some(ItemAction::SyncEquipped);
            }
        });
        ui.horizontal(|ui| {
            ui.label("Item code");
            ui.add_sized(
                [520.0, 22.0],
                egui::TextEdit::singleline(&mut self.item_code)
                    .hint_text("paste BL2(...) codes here"),
            );
            if ui.button("Insert code(s)").clicked() {
                item_action = Some(ItemAction::InsertCodes);
            }
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
        ui.add_space(6.0);

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
                    ui.label("Mark");
                    egui::ComboBox::from_id_salt("mark_item")
                        .selected_text(player_mark_name(edit.mark))
                        .show_ui(ui, |ui| {
                            for (value, name) in PLAYER_MARKS.iter().enumerate() {
                                if ui.selectable_label(edit.mark == value as i32, *name).clicked() {
                                    edit.mark = value as i32;
                                    edit.dirty = true;
                                }
                            }
                        });
                });
            } else if edit.container == Container::BackpackWeapon {
                ui.horizontal(|ui| {
                    ui.label("Quick slot");
                    if ui.add(egui::DragValue::new(&mut edit.quick_slot)).changed() {
                        edit.dirty = true;
                    }
                    ui.label("Mark");
                    egui::ComboBox::from_id_salt("mark_weapon")
                        .selected_text(player_mark_name(edit.mark))
                        .show_ui(ui, |ui| {
                            for (value, name) in PLAYER_MARKS.iter().enumerate() {
                                if ui.selectable_label(edit.mark == value as i32, *name).clicked() {
                                    edit.mark = value as i32;
                                    edit.dirty = true;
                                }
                            }
                        });
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
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("lootsmith", self.settings_json());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.build(ui);
    }
}
