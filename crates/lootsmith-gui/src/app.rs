//! Lootsmith GUI application state and UI.
//!
//! The UI lives in [`LootsmithApp::build`], independent of the native window, so
//! that both the `eframe` app and the `egui_kittest` harness run identical code.

pub(crate) use eframe::egui;
pub(crate) use lootsmith_core::savegame::Kind;
pub(crate) use lootsmith_core::{
    container, items, mission_status_name, player_mark_name, AssetGroup, AssetLibraryConfigs,
    AssetLibraryManager, Container, Customizations, Missions, NameParts, Resource, PackedEntry, PartLists, Platform,
    Profile, SaveCandidate, SaveGame, TravelStations, DlcTitles, VehicleCustomization, MISSION_STATUSES,
    PLAYER_MARKS, STAT_NAMES, VEHICLE_FAMILIES,
};
pub(crate) use std::collections::HashMap;
pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;

pub(crate) use crate::theme::{self, Tokens};
pub(crate) use egui_extras::{Column, TableBuilder};

pub use crate::theme::Theme;

mod item_editor;
mod widgets;
pub mod tabs;

pub(crate) use item_editor::*;
pub(crate) use widgets::*;

pub use item_editor::slots_for;

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Tab {
    General,
    Character,
    Currency,
    Ammo,
    Skills,
    Travel,
    Vehicles,
    Missions,
    Backpack,
    Bank,
    Raw,
    About,
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
    pub kind: String,
    pub id: i32,
    pub balance: String,
    /// Composed display name (prefix + title words), when a part list maps it.
    pub name: String,
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
    /// Elements of an int/string list field, editable individually.
    pub elements: Vec<RawElem>,
    /// A pending list-level change (element added/removed).
    apply: bool,
}

/// One element of a list field (int or string).
#[derive(Clone, Debug)]
pub struct RawElem {
    pub edit: RawEdit,
    /// Edited since the last apply.
    pub changed: bool,
}

/// A pending add/remove of one list element in the raw view.
pub(crate) enum RawListOp {
    Add,
    Remove(usize),
}

impl RawField {
    /// An edited element or a pending list-level change.
    fn list_needs_apply(&self) -> bool {
        self.apply || self.elements.iter().any(|e| e.changed)
    }
}

fn tab_key(tab: Tab) -> &'static str {
    match tab {
        Tab::General => "general",
        Tab::Character => "character",
        Tab::Currency => "currency",
        Tab::Ammo => "ammo",
        Tab::Skills => "skills",
        Tab::Travel => "travel",
        Tab::Vehicles => "vehicles",
        Tab::Missions => "missions",
        Tab::Backpack => "backpack",
        Tab::Bank => "bank",
        Tab::Raw => "raw",
        Tab::About => "about",
    }
}

fn tab_from_key(key: &str) -> Option<Tab> {
    match key {
        "general" => Some(Tab::General),
        "character" => Some(Tab::Character),
        "currency" => Some(Tab::Currency),
        "ammo" => Some(Tab::Ammo),
        "skills" => Some(Tab::Skills),
        "travel" => Some(Tab::Travel),
        "vehicles" => Some(Tab::Vehicles),
        "missions" => Some(Tab::Missions),
        "backpack" => Some(Tab::Backpack),
        "bank" => Some(Tab::Bank),
        "raw" => Some(Tab::Raw),
        "about" => Some(Tab::About),
        _ => None,
    }
}

/// A toolbar/shortcut action, dispatched after the panel closure.
#[derive(Clone, Copy)]
enum Action {
    Open,
    FindSaves,
    Save,
    SaveAs,
    Reload,
    New,
    Import,
    Profile,
    PickData,
    ToggleTheme,
}

/// Which auxiliary window is open.
#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Modal {
    #[default]
    None,
    NewSave,
    Import,
    Profile,
}

/// Editable values for the `profile.bin` window.
#[derive(Default, Clone)]
pub struct ProfileForm {
    pub loaded: bool,
    pub golden_keys: i64,
    pub badass_rank: i64,
    pub badass_tokens: i64,
    pub unlocked: bool,
    pub stats: [f64; 14],
}

/// Sections the Import dialog can copy from another save.
#[derive(Default, Clone, Copy)]
pub struct ImportSel {
    pub skills: bool,
    pub missions: bool,
    pub challenges: bool,
    pub travel: bool,
    pub vehicles: bool,
    pub dlc: bool,
    pub bank_slots: bool,
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

/// The tab list, in order.
const TABS: [(Tab, &str); 12] = [
    (Tab::General, "General"),
    (Tab::Character, "Character"),
    (Tab::Currency, "Currency"),
    (Tab::Ammo, "Ammo"),
    (Tab::Skills, "Skills"),
    (Tab::Travel, "Fast Travel"),
    (Tab::Vehicles, "Vehicles"),
    (Tab::Missions, "Missions"),
    (Tab::Backpack, "Backpack"),
    (Tab::Bank, "Bank"),
    (Tab::Raw, "Raw"),
    (Tab::About, "About"),
];

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
    /// Balance → allowed parts, for balance-constrained part pickers.
    pub part_lists: Option<Arc<PartLists>>,
    /// Prefix/title part words, for item display names.
    pub name_parts: Option<Arc<NameParts>>,
    /// DLC chapter titles, for grouping fast-travel stations.
    pub dlc_titles: Option<Arc<DlcTitles>>,
    /// Ammo/resource pools (field 11) and their origin snapshot.
    pub ammo: Vec<Resource>,
    ammo_orig: Vec<(f32, i32)>,
    /// Black Market SDU levels (field 36), mirrored for editing.
    pub sdus: [i32; 9],
    sdu_orig: [i32; 9],
    /// Desired backpack/bank slot sizes (consistent with `sdus`)
    pub backpack_slots: i32,
    pub bank_size: i32,
    /// Missions (field 18) and challenge list (field 38).
    pub mission_rows: Vec<MissionRow>,
    pub plot_mission: i32,
    pub active_mission: i32,
    pub challenges: Vec<String>,
    pub missions_data: Option<Arc<Missions>>,
    /// Every top-level field, for the raw view.
    pub raw: Vec<RawField>,
    /// Search text for the raw view.
    pub raw_filter: String,
    /// Show the trailing reserved/unused currency slots.
    pub currency_show_unused: bool,
    pub theme: Theme,
    theme_applied: bool,
    /// There are edits in memory that have not been written to disk.
    pub dirty: bool,
    pub status: String,
    pub save_as: String,
    /// The `BL2(base64)` item-code box used for copy/paste.
    pub item_code: String,
    /// Filter text for the backpack/bank table.
    pub filter: String,
    /// Decoded skill icon textures (lazy).
    skill_icons: HashMap<String, egui::TextureHandle>,
    /// Saves discovered on disk for the "Find saves" picker.
    pub saves: Vec<SaveCandidate>,
    pub show_saves: bool,
    /// Container format (endian/compression) used when writing via Save As.
    pub save_platform: Platform,
    /// The `New…` / `Import…` window, when open.
    modal: Modal,
    /// New-save picker state: class index and blank-vs-clone choice.
    pub new_class: usize,
    pub new_blank: bool,
    /// Import dialog state: other save's path and chosen sections.
    pub import_path: String,
    pub import_sel: ImportSel,
    /// Profile.bin editor state: file path and editable values.
    pub profile_path: String,
    pub profile_form: ProfileForm,
    loaded_once: bool,
    pub tab: Tab,
    /// Selected entry as (container, occurrence).
    pub selected: Option<(Container, usize)>,
    item_edit: Option<ItemEdit>,
    choices: HashMap<AssetGroup, Vec<String>>,
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
            part_lists: None,
            name_parts: None,
            dlc_titles: None,
            ammo: Vec::new(),
            ammo_orig: Vec::new(),
            sdus: [0; 9],
            sdu_orig: [0; 9],
            backpack_slots: 12,
            bank_size: 6,
            mission_rows: Vec::new(),
            plot_mission: 0,
            active_mission: 0,
            challenges: Vec::new(),
            missions_data: None,
            raw: Vec::new(),
            raw_filter: String::new(),
            currency_show_unused: false,
            theme: Theme::Dark,
            theme_applied: false,
            dirty: false,
            status: "Open a save to begin (Browse… or --open).".to_string(),
            save_as: String::new(),
            item_code: String::new(),
            filter: String::new(),
            skill_icons: HashMap::new(),
            saves: Vec::new(),
            show_saves: false,
            save_platform: Platform::PC,
            modal: Modal::None,
            new_class: 0,
            new_blank: false,
            import_path: String::new(),
            import_sel: ImportSel::default(),
            profile_path: String::new(),
            profile_form: ProfileForm::default(),
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

    /// The current item-pack/asset configs (also used by tests and tools).
    pub fn configs(&self) -> AssetLibraryConfigs {
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
        let status = format!(
            "loaded {} ({} bytes, crc32 {})",
            self.path,
            data.len(),
            if save.crc32_valid() { "ok" } else { "MISMATCH" }
        );
        self.accept_save(save_game, status);
    }

    /// Fill every view (form, rows, skills, …) from a parsed save.
    pub(crate) fn accept_save(&mut self, save_game: SaveGame, status: String) {
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
        self.status = status;
        self.refresh_rows();
        self.refresh_skills();
        self.refresh_travel();
        self.refresh_vehicles();
        self.refresh_ammo();
        self.refresh_sdus();
        self.refresh_missions();
        self.refresh_raw();
        self.dirty = false;
    }

    /// The directory holding the dumps files (Asset Library Manager, Travel
    /// Stations, Customizations, Missions).
    pub(crate) fn dumps_dir(&self) -> Option<PathBuf> {
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
        self.part_lists = self.load_part_lists();
        self.name_parts = self.load_name_parts();
        self.dlc_titles = self.load_dlc_titles();
        self.refresh_rows();
        self.load_travel_stations();
        self.load_customizations();
        self.load_missions_data();
    }

    /// The part choices for one slot: constrained by the balance's part list
    /// when the game data has one for the selected balance ("Complete" =
    /// only these; "Selective" = these first, then the library), otherwise
    /// the plain library choices.
    pub fn balanced_choices(
        &self,
        _set: &i32,
        balance: &str,
        group: AssetGroup,
        slot: &str,
    ) -> Vec<String> {
        if let (Some(lists), Some(all)) = (self.part_lists.as_ref(), self.choices.get(&group)) {
            let mut constrained: Vec<String> = Vec::new();
            if let Some((complete, names)) = lists.choices_for(balance, slot) {
                if complete {
                    constrained = names;
                } else {
                    constrained = names.clone();
                    for choice in all {
                        if !constrained.contains(choice) {
                            constrained.push(choice.clone());
                        }
                    }
                }
            }
            if !constrained.is_empty() {
                return constrained;
            }
        }
        self.choices.get(&group).cloned().unwrap_or_default()
    }

    /// Load DLC chapter titles for the travel-tab grouping.
    fn load_dlc_titles(&self) -> Option<Arc<DlcTitles>> {
        let dir = self.dumps_dir()?;
        DlcTitles::from_path(&dir.join("Downloadable Contents.json"))
            .ok()
            .filter(|t| !t.is_empty())
            .map(Arc::new)
    }

    /// Load the name parts (from the game-data folder or
    /// `LOOTSMITH_NAME_PARTS`) for item display names.
    fn load_name_parts(&self) -> Option<Arc<NameParts>> {
        if let Some(dir) = self.dumps_dir() {
            if let Ok(parts) = NameParts::from_dir(&dir) {
                return Some(Arc::new(parts));
            }
        }
        if let Ok(path) = std::env::var("LOOTSMITH_NAME_PARTS") {
            if !path.trim().is_empty() {
                if let Ok(parts) = NameParts::from_dir(std::path::Path::new(&path)) {
                    return Some(Arc::new(parts));
                }
            }
        }
        None
    }

    /// Load the balance part lists (from the game-data folder or
    /// `LOOTSMITH_PART_LISTS`) for the balance-constrained pickers.
    fn load_part_lists(&self) -> Option<Arc<PartLists>> {
        if let Ok(path) = std::env::var("LOOTSMITH_PART_LISTS") {
            if !path.trim().is_empty() {
                if let Ok(lists) = PartLists::from_path(std::path::Path::new(&path)) {
                    return Some(Arc::new(lists));
                }
            }
        }
        if let Some(dir) = self.dumps_dir() {
            if let Ok(lists) = PartLists::from_dir(&dir) {
                return Some(Arc::new(lists));
            }
        }
        None
    }

    pub(crate) fn refresh_rows(&mut self) {
        let configs = self.configs();
        let manager = self.manager.clone();
        let name_parts = self.name_parts.clone();
        let mut backpack = Vec::new();
        let mut bank = Vec::new();
        if let Some(save_game) = &self.save_game {
            for (i, entry) in save_game.packed_entries().iter().enumerate() {
                backpack.push(row_for(entry, i + 1, &manager, &configs, name_parts.as_deref()));
            }
            for (i, entry) in save_game.bank_entries().iter().enumerate() {
                bank.push(row_for(entry, i + 1, &manager, &configs, name_parts.as_deref()));
            }
        }
        self.backpack = backpack;
        self.bank = bank;
    }

    pub(crate) fn refresh_skills(&mut self) {
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

    /// Snapshot the ammo pools (field 11).
    pub(crate) fn refresh_sdus(&mut self) {
        let levels = self
            .save_game
            .as_ref()
            .map(SaveGame::black_market_upgrades)
            .unwrap_or([0; 9]);
        self.sdus = levels;
        self.sdu_orig = levels;
        let (backpack, ..) = self
            .save_game
            .as_ref()
            .and_then(SaveGame::inventory_slots)
            .unwrap_or((12, 0, 0));
        self.backpack_slots = backpack;
        self.bank_size = self
            .save_game
            .as_ref()
            .and_then(SaveGame::max_bank_slots)
            .unwrap_or(6);
        // The form drives MaxBankSlots (56) on save; keep it in sync so the
        // SDU editors' writes survive `apply_form`.
        self.form.bank_slots = self.bank_size;
    }

    /// Set one SDU level (1-based `pool`), consistent slot counts included.
    pub fn set_sdu(&mut self, pool: usize, level: i32) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        if let Err(e) = save_game.set_sdu(pool.max(1) - 1, level) {
            self.status = format!("sdu error: {e}");
            return;
        }
        self.refresh_sdus();
        self.dirty = true;
    }

    /// Set the backpack's slot size, deriving the SDU level (12 + 3/level).
    pub fn set_backpack_slots(&mut self, slots: i32) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let sdus = ((slots as f32 - 12.0) / 3.0).ceil().max(0.0) as i32;
        if let Err(e) = save_game.set_sdu(7, sdus) {
            self.status = format!("sdu error: {e}");
            return;
        }
        self.refresh_sdus();
        self.dirty = true;
    }

    /// Set the bank's slot size, deriving the SDU level (6 + 2/level).
    pub fn set_bank_size(&mut self, slots: i32) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let sdus = ((slots as f32 - 6.0) / 2.0).ceil().clamp(0.0, 255.0) as i32;
        if let Err(e) = save_game.set_sdu(8, sdus) {
            self.status = format!("sdu error: {e}");
            return;
        }
        self.refresh_sdus();
        self.dirty = true;
    }

    /// Set every SDU to its vanilla cap (backpack 39, bank 24).
    pub fn max_all_sdus(&mut self) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        if let Err(e) = save_game.max_all_sdus() {
            self.status = format!("sdu error: {e}");
            return;
        }
        self.refresh_sdus();
        self.dirty = true;
        self.status = "all SDUs maxed (backpack 39, bank 24)".into();
    }

    pub(crate) fn refresh_ammo(&mut self) {
        self.ammo = self
            .save_game
            .as_ref()
            .map(SaveGame::resources)
            .unwrap_or_default();
        self.ammo_orig = self
            .ammo
            .iter()
            .map(|r| (r.amount, r.upgrade_level))
            .collect();
    }

    /// Set one ammo pool's amount / SDU level (1-based `pool` as shown).
    pub fn set_ammo(&mut self, pool: usize, amount: Option<f32>, sdu: Option<i32>) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let index = pool.max(1) - 1;
        if let Err(e) = save_game.mutate_resource(index, |resource| {
            if let Some(amount) = amount {
                resource.amount = amount;
            }
            if let Some(sdu) = sdu {
                resource.upgrade_level = sdu;
            }
        }) {
            self.status = format!("ammo error: {e}");
            return;
        }
        self.refresh_ammo();
        self.dirty = true;
    }

    pub(crate) fn refresh_travel(&mut self) {
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

    pub(crate) fn refresh_vehicles(&mut self) {
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

    pub(crate) fn refresh_missions(&mut self) {
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
    pub(crate) fn refresh_raw(&mut self) {
        let mut out = Vec::new();
        if let Some(sg) = &self.save_game {
            let mut i = 0;
            while i < sg.message.fields.len() {
                let field = &sg.message.fields[i];
                let number = field.number;
                let spec = SaveGame::spec(number);
                let count = sg.message.count(number);
                let mut elements = Vec::new();
                match spec.map(|s| s.kind) {
                    Some(Kind::IntList) => {
                        for value in sg.get_int_list(number) {
                            elements.push(RawElem {
                                edit: RawEdit::Int(value),
                                changed: false,
                            });
                        }
                    }
                    Some(Kind::StrList) => {
                        for value in sg.get_str_list(number) {
                            elements.push(RawElem {
                                edit: RawEdit::Str(value),
                                changed: false,
                            });
                        }
                    }
                    _ => {}
                }
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
                    preview: preview.clone(),
                    edit: edit.clone(),
                    elements,
                    apply: false,
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
        match container::serialize(&save_game.to_bytes(), self.save_platform) {
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

    // ---- new saves & section import ----

    /// Start a fresh character by cloning the open save and resetting
    /// progress for the chosen class. The save is only written when the user
    /// uses Save As….
    pub fn new_clone_save(&mut self) {
        let Some(save_game) = self.save_game.clone() else {
            self.modal = Modal::None;
            self.status = "nothing to clone — open or locate a save first".into();
            return;
        };
        let class = self.new_class_choice();
        let mut save = save_game;
        save.make_fresh_character(class);
        let note = format!(
            "new save — fresh {} cloned; use Save As… to write it",
            crate::skills::class_display_name(class)
        );
        self.path.clear();
        self.accept_save(save, note);
        self.dirty = true;
        self.modal = Modal::None;
        self.tab = Tab::General;
    }

    /// Start an experimental from-scratch save (spec-driven blank). Only the
    /// baseline fields are written; the game may be strict about this path,
    /// so in-game verification is recommended.
    pub fn new_blank_save(&mut self) {
        let class = self.new_class_choice();
        let save = SaveGame::blank(class);
        let note = format!(
            "new blank save ({} — experimental; verify in game); use Save As… to write it",
            crate::skills::class_display_name(class)
        );
        self.path.clear();
        self.accept_save(save, note);
        self.dirty = true;
        self.modal = Modal::None;
        self.tab = Tab::General;
    }

    pub(crate) fn new_class_choice(&self) -> &'static str {
        let classes = crate::skills::class_ids();
        let idx = self.new_class.min(classes.len().saturating_sub(1));
        classes[idx]
    }

    /// Copy the selected sections from another save into the open save.
    pub fn do_import(&mut self) {
        if self.import_path.trim().is_empty() {
            self.status = "enter or browse the other save's path first".into();
            return;
        }
        let data = match std::fs::read(self.import_path.trim()) {
            Ok(data) => data,
            Err(e) => {
                self.status = format!("import read error: {e}");
                return;
            }
        };
        let other = match container::unpack(&data, Platform::PC)
            .and_then(|file| file.save_game())
        {
            Ok(save) => save,
            Err(e) => {
                self.status = format!("import parse error: {e}");
                return;
            }
        };
        let sel = self.import_sel;
        let mut numbers: Vec<u32> = Vec::new();
        let mut groups: Vec<&str> = Vec::new();
        if sel.skills {
            numbers.push(8);
            groups.push("skills");
        }
        if sel.missions {
            numbers.push(18);
            groups.push("missions");
        }
        if sel.challenges {
            numbers.extend([38, 39, 40, 42]);
            groups.push("challenges");
        }
        if sel.travel {
            numbers.extend([16, 17]);
            groups.push("fast travel");
        }
        if sel.vehicles {
            numbers.extend([57, 58]);
            groups.push("vehicles");
        }
        if sel.dlc {
            numbers.push(27);
            groups.push("DLC expansions");
        }
        if sel.bank_slots {
            numbers.push(41);
            groups.insert(0, "bank slots");
        }
        if numbers.is_empty() {
            self.status = "select at least one section to import".into();
            return;
        }
        let Some(save) = self.save_game.take() else {
            return;
        };
        let mut save = save;
        save.import_fields_from(&other, &numbers);
        let name = std::path::Path::new(self.import_path.trim())
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.import_path.clone());
        self.accept_save(save, format!("imported {} from {name}", groups.join(", ")));
        self.dirty = true;
        self.modal = Modal::None;
    }

    // ---- profile.bin (golden keys, badass, customizations) ----

    /// Open the profile window, suggesting a profile.bin next to the save.
    pub fn open_profile_modal(&mut self) {
        self.modal = Modal::Profile;
        if self.profile_path.trim().is_empty() {
            self.profile_path = self.suggested_profile().unwrap_or_default();
        }
        self.load_profile();
    }

    /// The `profile.bin` next to the open save, else any we can find.
    pub(crate) fn suggested_profile(&self) -> Option<String> {
        if !self.path.trim().is_empty() {
            let profile = PathBuf::from(self.path.trim())
                .parent()?
                .join("profile.bin");
            if profile.exists() {
                return Some(profile.display().to_string());
            }
        }
        for candidate in lootsmith_core::locate::find_saves() {
            let profile = candidate.path.parent()?.join("profile.bin");
            if profile.exists() {
                return Some(profile.display().to_string());
            }
        }
        None
    }

    /// Read the profile's values into the editor form.
    pub fn load_profile(&mut self) {
        if self.profile_path.trim().is_empty() {
            self.status = "no profile.bin path — install/launch the game once".into();
            return;
        }
        match Profile::from_path(std::path::Path::new(self.profile_path.trim())) {
            Ok(profile) => {
                self.profile_form = ProfileForm {
                    loaded: true,
                    golden_keys: profile.golden_keys().unwrap_or(0),
                    badass_rank: profile.badass_rank().unwrap_or(0) as i64,
                    badass_tokens: profile.badass_tokens().unwrap_or(0) as i64,
                    unlocked: profile.all_customizations_unlocked(),
                    stats: profile.stats().unwrap_or([0.0; 14]),
                };
                self.status = format!("loaded profile {}", self.profile_path.trim());
            }
            Err(e) => {
                self.profile_form.loaded = false;
                self.status = format!("profile error: {e}");
            }
        }
    }

    /// Write the edited values back into profile.bin (with a .bak backup).
    pub fn save_profile(&mut self) {
        let path = self.profile_path.trim().to_string();
        if path.is_empty() {
            self.status = "no profile.bin path".into();
            return;
        }
        let Ok(mut profile) = Profile::from_path(std::path::Path::new(&path)) else {
            self.status = "profile.bin could not be re-read; reload first".into();
            return;
        };
        let form = self.profile_form.clone();
        profile.set_golden_keys(form.golden_keys.max(0));
        profile.set_badass_rank(form.badass_rank.max(0) as u32);
        profile.set_badass_tokens(form.badass_tokens.max(0) as u32);
        if form.unlocked && !profile.all_customizations_unlocked() {
            let _ = profile.unlock_all_customizations();
        }
        // Only write stats when the profile already carries a stats entry
        // (fresh profiles with badass rank 0 legitimately have none).
        if profile.has_stats() {
            if let Err(e) = profile.set_stats(&form.stats) {
                self.status = format!("profile stats error: {e}");
                return;
            }
        }
        let backup = format!("{path}.bak");
        if !std::path::Path::new(&backup).exists() {
            let _ = std::fs::copy(&path, &backup);
        }
        let bytes = profile.to_bytes();
        match std::fs::write(&path, &bytes) {
            Ok(()) => {
                self.status = format!("saved profile {path} ({} bytes, backup {backup})", bytes.len());
            }
            Err(e) => self.status = format!("profile write error: {e}"),
        }
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
        theme::apply(ctx, self.theme);
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
            Action::FindSaves => {
                self.scan_saves();
                self.show_saves = true;
            }
            Action::Save => self.save_in_place(),
            Action::SaveAs => self.browse_save_as(),
            Action::Reload => self.load(),
            Action::New => self.modal = Modal::NewSave,
            Action::Import => {
                self.modal = Modal::Import;
                if self.import_path.trim().is_empty() {
                    self.import_path = self.path.clone();
                }
            }
            Action::Profile => self.open_profile_modal(),
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

    /// Scan the usual locations for saves (for the picker / first run).
    pub fn scan_saves(&mut self) {
        self.saves = lootsmith_core::locate::find_saves();
        self.status = format!("found {} save(s) on disk", self.saves.len());
    }

    /// Decode and cache a bundled skill icon as a texture.
    pub(crate) fn skill_texture(&mut self, ctx: &egui::Context, filename: &str) -> Option<egui::TextureHandle> {
        if let Some(handle) = self.skill_icons.get(filename) {
            return Some(handle.clone());
        }
        let bytes = crate::skills::icon_bytes(filename)
            .or_else(|| crate::skills::portrait_bytes(filename))?;
        let data = eframe::icon_data::from_png_bytes(bytes).ok()?;
        let image = egui::ColorImage::from_rgba_unmultiplied(
            [data.width as usize, data.height as usize],
            &data.rgba,
        );
        let handle = ctx.load_texture(filename.to_string(), image, egui::TextureOptions::LINEAR);
        self.skill_icons.insert(filename.to_string(), handle.clone());
        Some(handle)
    }

    /// Build the full UI. Called by both the native app and the test harness.
    pub fn build(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.handle_shortcuts(&ctx);

        if !self.loaded_once {
            self.loaded_once = true;
            if self.path.trim().is_empty() {
                // First run: auto-open the newest save we can find.
                self.scan_saves();
                if let Some(newest) = self.saves.first() {
                    self.path = newest.path.display().to_string();
                }
            }
            if !self.path.trim().is_empty() {
                self.load();
            }
        }

        let mut action: Option<Action> = None;
        egui::Panel::top("toolbar")
            .frame(
                egui::Frame::default()
                    .fill(Tokens::SURFACE)
                    .inner_margin(egui::Margin::symmetric(12, 8)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.button("Open…").clicked() {
                        action = Some(Action::Open);
                    }
                    if ui.button("Find saves").clicked() {
                        action = Some(Action::FindSaves);
                    }
                    toolbar_sep(ui);
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
                    toolbar_sep(ui);
                    if ui
                        .button("New…")
                        .on_hover_text("Create a fresh character save (clone or blank)")
                        .clicked()
                    {
                        action = Some(Action::New);
                    }
                    if ui
                        .button("Import…")
                        .on_hover_text("Copy sections (skills, missions, …) from another save")
                        .clicked()
                    {
                        action = Some(Action::Import);
                    }
                    if ui
                        .button("Profile…")
                        .on_hover_text("Edit profile.bin: golden keys, badass bonuses, customizations")
                        .clicked()
                    {
                        action = Some(Action::Profile);
                    }
                    egui::ComboBox::from_id_salt("save_platform")
                        .selected_text(self.save_platform.id().to_uppercase())
                        .width(80.0)
                        .show_ui(ui, |ui| {
                            for platform in [
                                Platform::PC,
                                Platform::X360,
                                Platform::PS3,
                                Platform::PSVita,
                                Platform::Shield,
                                Platform::Switch,
                            ] {
                                ui.selectable_value(
                                    &mut self.save_platform,
                                    platform,
                                    platform.id().to_uppercase(),
                                );
                            }
                        });
                    toolbar_sep(ui);
                    if ui.button("Game data…").clicked() {
                        action = Some(Action::PickData);
                    }
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let label = if self.theme == Theme::Dark { "☀" } else { "☾" };
                        if ui.button(label).on_hover_text("Toggle light/dark").clicked() {
                            action = Some(Action::ToggleTheme);
                        }
                        if self.dirty {
                            ui.colored_label(Tokens::ACCENT, "● unsaved");
                        }
                    });
                });
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    status_dot(
                        ui,
                        if self.save_game.is_some() {
                            Tokens::OK
                        } else {
                            Tokens::TEXT_FAINT
                        },
                    );
                    ui.label(egui::RichText::new("Save file").color(Tokens::TEXT_DIM).size(11.5));
                    ui.add_sized(
                        [430.0, 24.0],
                        egui::TextEdit::singleline(&mut self.path)
                            .hint_text("/path/to/Save0001.sav"),
                    );
                    toolbar_sep(ui);
                    status_dot(
                        ui,
                        if self.manager.is_some() {
                            Tokens::OK
                        } else {
                            Tokens::WARN
                        },
                    );
                    ui.label(egui::RichText::new("Game data").color(Tokens::TEXT_DIM).size(11.5));
                    ui.add_sized(
                        [300.0, 24.0],
                        egui::TextEdit::singleline(&mut self.dumps)
                            .hint_text("folder with Asset Library Manager.json"),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if self.manager.is_some() {
                            ui.colored_label(Tokens::TEXT_FAINT, "names resolved");
                        } else {
                            ui.colored_label(Tokens::WARN, "no game data — parts show raw ids");
                        }
                    });
                });
            });
        if let Some(action) = action {
            self.dispatch(action);
        }

        egui::Panel::bottom("status")
            .frame(
                egui::Frame::default()
                    .fill(Tokens::SURFACE)
                    .inner_margin(egui::Margin::symmetric(12, 4)),
            )
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    status_dot(
                        ui,
                        if self.dirty {
                            Tokens::ACCENT
                        } else {
                            Tokens::OK
                        },
                    );
                    ui.label(egui::RichText::new(self.status.clone()).size(12.0));
                });
            });

        egui::CentralPanel::default().show(ui, |ui| {
            if self.save_game.is_none() {
                empty_state(ui);
                return;
            }
            egui::Panel::top("tabs")
                .frame(
                    egui::Frame::default()
                        .fill(Tokens::SURFACE)
                        .inner_margin(egui::Margin::symmetric(10, 0)),
                )
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        for (tab, label) in TABS {
                            let text = match tab {
                                Tab::Backpack => format!("Backpack ({})", self.backpack.len()),
                                Tab::Bank => format!("Bank ({})", self.bank.len()),
                                _ => label.to_string(),
                            };
                            if theme::tab(ui, self.tab == tab, &text) {
                                self.tab = tab;
                            }
                        }
                    });
                });
            match self.tab {
                Tab::General => self.ui_general(ui),
                Tab::Character => self.ui_character(ui),
                Tab::Currency => self.ui_currency(ui),
                Tab::Ammo => self.ui_ammo(ui),
                Tab::Skills => self.ui_skills(ui),
                Tab::Travel => self.ui_travel(ui),
                Tab::Vehicles => self.ui_vehicles(ui),
                Tab::Missions => self.ui_missions(ui),
                Tab::Backpack => self.ui_container(ui, false),
                Tab::Bank => self.ui_container(ui, true),
                Tab::Raw => self.ui_raw(ui),
                Tab::About => tabs::about::ui_about(ui),
            }
        });

        if self.show_saves {
            let saves = self.saves.clone();
            let mut pick: Option<String> = None;
            let mut rescan = false;
            let mut open = true;
            egui::Window::new("Saves found")
                .collapsible(false)
                .resizable(true)
                .default_width(760.0)
                .open(&mut open)
                .show(ui.ctx(), |ui| {
                    if saves.is_empty() {
                        ui.label("No Borderlands 2 saves found in the usual locations.");
                        ui.label("Use Open… to pick one manually, or set LOOTSMITH_SAVE_ROOTS.");
                    } else {
                        ui.label("Pick a save to open (newest first):");
                        ui.add_space(4.0);
                        for candidate in &saves {
                            let text = format!(
                                "{}   [{}]",
                                candidate.path.display(),
                                candidate.modified_text()
                            );
                            if ui.selectable_label(false, text).clicked() {
                                pick = Some(candidate.path.display().to_string());
                            }
                        }
                    }
                    ui.add_space(6.0);
                    if ui.button("Rescan").clicked() {
                        rescan = true;
                    }
                });
            if !open {
                self.show_saves = false;
            }
            if rescan {
                self.scan_saves();
            }
            if let Some(path) = pick {
                self.path = path;
                self.show_saves = false;
                self.load();
            }
        }

        if self.modal == Modal::NewSave {
            let classes = crate::skills::class_ids();
            let mut new_class = self.new_class;
            let mut new_blank = self.new_blank;
            let mut open = true;
            let mut create = false;
            let mut cancel = false;
            egui::Window::new("New save")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .default_width(430.0)
                .show(ui.ctx(), |ui| {
                    ui.label("Create a fresh character. Nothing is written until you use Save As…");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("Class");
                        egui::ComboBox::from_id_salt("new_class_choice")
                            .selected_text(crate::skills::class_display_name(
                                classes[new_class.min(classes.len() - 1)],
                            ))
                            .show_ui(ui, |ui| {
                                for (i, id) in classes.iter().enumerate() {
                                    ui.selectable_value(
                                        &mut new_class,
                                        i,
                                        crate::skills::class_display_name(id),
                                    );
                                }
                            });
                    });
                    ui.add_space(6.0);
                    ui.radio_value(&mut new_blank, false, "Clone the open save, reset progress");
                    ui.radio_value(&mut new_blank, true, "Blank save (spec-driven, experimental)");
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Create").clicked() {
                            create = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });
            if !open {
                self.modal = Modal::None;
            }
            if let Some(blank) = {
                self.new_class = new_class;
                self.new_blank = new_blank;
                let mut taken = None;
                if cancel {
                    taken = Some(false);
                } else if create {
                    taken = Some(new_blank);
                }
                taken
            } {
                if blank {
                    self.new_blank_save();
                } else {
                    self.new_clone_save();
                }
            }
        }

        if self.modal == Modal::Profile {
            let mut path = self.profile_path.clone();
            let mut form = self.profile_form.clone();
            let mut open = true;
            // 1 = load from disk, 2 = save to disk
            let mut act: Option<u8> = None;
            egui::Window::new("Profile")
                .collapsible(false)
                .resizable(true)
                .open(&mut open)
                .default_width(470.0)
                .show(ui.ctx(), |ui| {
                    ui.label(
                        "Edits profile.bin (shared account data): golden keys, badass rank & \
                         tokens, stat bonuses, customization unlocks.",
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("File");
                        ui.add_sized([360.0, 24.0], egui::TextEdit::singleline(&mut path));
                    });
                    ui.add_space(6.0);
                    if !form.loaded {
                        theme::muted(
                            ui,
                            "Profile not loaded yet — set the path and click Load.",
                        );
                    }
                    egui::Grid::new("profile_grid")
                        .num_columns(2)
                        .spacing([16.0, 6.0])
                        .show(ui, |ui| {
                            ui.label("Golden keys");
                            ui.add(
                                egui::DragValue::new(&mut form.golden_keys)
                                    .range(0..=100_000),
                            );
                            ui.end_row();
                            ui.label("Badass rank");
                            ui.add(
                                egui::DragValue::new(&mut form.badass_rank)
                                    .range(0..=2_000_000_000),
                            );
                            ui.end_row();
                            ui.label("Badass tokens");
                            ui.add(
                                egui::DragValue::new(&mut form.badass_tokens)
                                    .range(0..=500),
                            );
                            ui.end_row();
                            ui.label("Unlock all customizations");
                            ui.checkbox(&mut form.unlocked, "");
                            ui.end_row();
                        });
                    ui.add_space(6.0);
                    theme::section(ui, "Badass bonus stats");
                    egui::Grid::new("profile_stats")
                        .num_columns(2)
                        .striped(true)
                        .spacing([12.0, 4.0])
                        .show(ui, |ui| {
                            for (i, (name, value)) in
                                STAT_NAMES.iter().zip(form.stats.iter_mut()).enumerate()
                            {
                                ui.label(format!("{i}: {name}"));
                                ui.add(
                                    egui::DragValue::new(value)
                                        .range(0.0..=9_975_792.3)
                                        .speed(1.0),
                                );
                                ui.end_row();
                            }
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button("Load")
                            .on_hover_text("Re-read profile.bin from disk")
                            .clicked()
                        {
                            act = Some(1);
                        }
                        if ui
                            .button("Save profile")
                            .on_hover_text("Write profile.bin (with a .bak backup)")
                            .clicked()
                        {
                            act = Some(2);
                        }
                    });
                });
            if !open {
                self.modal = Modal::None;
            }
            self.profile_path = path;
            if let Some(one) = act {
                if one == 1 {
                    self.profile_form = form;
                    self.load_profile();
                } else {
                    self.profile_form = form;
                    self.save_profile();
                }
            }
        }

        if self.modal == Modal::Import {
            let mut path = self.import_path.clone();
            let mut sel = self.import_sel;
            let mut open = true;
            let mut do_import = false;
            let mut cancel = false;
            egui::Window::new("Import sections")
                .collapsible(false)
                .resizable(false)
                .open(&mut open)
                .default_width(460.0)
                .show(ui.ctx(), |ui| {
                    ui.label("Copy sections from another save into the open save.");
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label("Save");
                        ui.add_sized([360.0, 24.0], egui::TextEdit::singleline(&mut path));
                    });
                    ui.add_space(8.0);
                    egui::Grid::new("import_sel")
                        .num_columns(2)
                        .spacing([16.0, 4.0])
                        .show(ui, |ui| {
                            for (flag, label) in [
                                (&mut sel.skills, "Skills (SkillData)"),
                                (&mut sel.missions, "Missions (playthroughs, statuses)"),
                                (&mut sel.challenges, "Challenges"),
                                (&mut sel.travel, "Fast travel stations"),
                                (&mut sel.vehicles, "Vehicle customizations"),
                                (&mut sel.dlc, "DLC expansion data"),
                                (&mut sel.bank_slots, "Bank slots (contents)"),
                            ] {
                                ui.checkbox(flag, "");
                                ui.label(label);
                                ui.end_row();
                            }
                        });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui
                            .button("Import")
                            .on_hover_text("Replace the selected sections in the open save")
                            .clicked()
                        {
                            do_import = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel = true;
                        }
                    });
                });
            if !open {
                self.modal = Modal::None;
            }
            if cancel {
                self.modal = Modal::None;
            }
            if do_import {
                self.import_path = path;
                self.import_sel = sel;
                self.do_import();
            }
        }
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
