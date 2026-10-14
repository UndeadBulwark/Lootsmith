//! Lootsmith GUI application state and UI.
//!
//! The UI lives in [`LootsmithApp::build`], independent of the native window, so
//! that both the `eframe` app and the `egui_kittest` harness run identical code.

pub(crate) use eframe::egui;
pub(crate) use lootsmith_core::savegame::Kind;
pub(crate) use lootsmith_core::{
    container, items, mission_status_name, player_mark_name, AssetGroup, AssetLibraryConfigs,
    AssetLibraryManager, ChallengeStat, ChallengeStats, Container, Customizations, DlcTitles,
    GameDefs, Missions, NameParts, PackedEntry, Platform, Profile, Resource, SaveCandidate,
    SaveGame, TravelStations, VehicleCustomization, MISSION_STATUSES, PLAYER_MARKS, STAT_NAMES,
    VEHICLE_FAMILIES,
};
pub(crate) use std::collections::HashMap;
pub(crate) use std::path::PathBuf;
pub(crate) use std::sync::Arc;

pub(crate) use crate::theme;
use crate::ui::widgets::Glyph;

pub use crate::theme::Theme;

mod item_editor;
pub mod tabs;
mod widgets;

pub(crate) use item_editor::*;
pub(crate) use widgets::*;

pub use item_editor::slots_for;

/// How the UI scale is chosen (spec §2.1 rule 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ScaleMode {
    /// Pick an integer scale from the monitor's physical size.
    #[default]
    Auto,
    /// A fixed integer scale (1..=4).
    Fixed(u32),
}

/// One palette / menu command. The Menu window and the Ctrl+K palette share this
/// single table (`PALETTE`), so every dropped toolbar action stays reachable.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Cmd {
    Open,
    FindSaves,
    Save,
    SaveAs,
    Reload,
    NewSave,
    Import,
    Profile,
    PickData,
    ToggleTheme,
    NewWeapon,
    NewItem,
    Duplicate,
    CopyCode,
    SyncLevels,
    SyncEquipped,
    InsertCodes,
    Scale(u32),
}

/// Every command, in palette order.
pub(crate) const PALETTE: &[Cmd] = &[
    Cmd::Open,
    Cmd::FindSaves,
    Cmd::Save,
    Cmd::SaveAs,
    Cmd::Reload,
    Cmd::NewSave,
    Cmd::Import,
    Cmd::Profile,
    Cmd::PickData,
    Cmd::ToggleTheme,
    Cmd::NewWeapon,
    Cmd::NewItem,
    Cmd::Duplicate,
    Cmd::CopyCode,
    Cmd::SyncLevels,
    Cmd::SyncEquipped,
    Cmd::InsertCodes,
    Cmd::Scale(1),
    Cmd::Scale(2),
    Cmd::Scale(3),
    Cmd::Scale(4),
];

impl Cmd {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Cmd::Open => "Open…",
            Cmd::FindSaves => "Find saves",
            Cmd::Save => "Save",
            Cmd::SaveAs => "Save As…",
            Cmd::Reload => "Reload",
            Cmd::NewSave => "New…",
            Cmd::Import => "Import…",
            Cmd::Profile => "Profile…",
            Cmd::PickData => "Game data…",
            Cmd::ToggleTheme => "Toggle light/dark",
            Cmd::NewWeapon => "New weapon",
            Cmd::NewItem => "New item",
            Cmd::Duplicate => "Duplicate",
            Cmd::CopyCode => "Copy code",
            Cmd::SyncLevels => "Sync levels",
            Cmd::SyncEquipped => "Sync equipped",
            Cmd::InsertCodes => "Insert code(s)",
            Cmd::Scale(1) => "Scale 1x",
            Cmd::Scale(2) => "Scale 2x",
            Cmd::Scale(3) => "Scale 3x",
            Cmd::Scale(_) => "Scale 4x",
        }
    }
}

#[derive(Clone, PartialEq, Eq, Copy, Debug)]
pub enum Tab {
    Character,
    Currency,
    Ammo,
    World,
    Skills,
    Travel,
    Vehicles,
    Missions,
    Backpack,
    Bank,
    Raw,
    About,
    Profile,
    Import,
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
    /// The item's game stage (the level the item is tuned to).
    pub level: i32,
    /// A short type name ("Pistol", "Shield", "Item", "Unknown").
    pub type_name: String,
    /// Rarity from the balance definition (never guessed; `Unknown` otherwise).
    pub rarity: lootsmith_core::Rarity,
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
        Tab::Character => "character",
        Tab::Currency => "currency",
        Tab::Ammo => "ammo",
        Tab::World => "world",
        Tab::Skills => "skills",
        Tab::Travel => "travel",
        Tab::Vehicles => "vehicles",
        Tab::Missions => "missions",
        Tab::Backpack => "backpack",
        Tab::Bank => "bank",
        Tab::Raw => "raw",
        Tab::About => "about",
        Tab::Profile => "profile",
        Tab::Import => "import",
    }
}

fn tab_from_key(key: &str) -> Option<Tab> {
    match key {
        "character" => Some(Tab::Character),
        "currency" => Some(Tab::Currency),
        "ammo" => Some(Tab::Ammo),
        "world" => Some(Tab::World),
        "skills" => Some(Tab::Skills),
        "travel" => Some(Tab::Travel),
        "vehicles" => Some(Tab::Vehicles),
        "missions" => Some(Tab::Missions),
        "backpack" => Some(Tab::Backpack),
        "bank" => Some(Tab::Bank),
        "raw" => Some(Tab::Raw),
        "about" => Some(Tab::About),
        "profile" => Some(Tab::Profile),
        "import" => Some(Tab::Import),
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
    Duplicate,
    SyncAll,
}

/// The tab list, in order.
const TABS: [(Tab, &str); 14] = [
    (Tab::Character, "Character"),
    (Tab::Currency, "Currency"),
    (Tab::Ammo, "Ammo & SDUs"),
    (Tab::World, "World"),
    (Tab::Skills, "Skills"),
    (Tab::Travel, "Fast Travel"),
    (Tab::Vehicles, "Vehicles"),
    (Tab::Missions, "Missions"),
    (Tab::Backpack, "Backpack"),
    (Tab::Bank, "Bank"),
    (Tab::Raw, "Raw"),
    (Tab::About, "About"),
    (Tab::Profile, "Profile"),
    (Tab::Import, "Import"),
];

/// Sidebar groups: label -> tabs (spec §2.2 grouped sidebar).
const NAV_GROUPS: [(&str, &[Tab]); 5] = [
    ("Character", &[Tab::Character, Tab::Skills, Tab::Currency]),
    ("Inventory", &[Tab::Backpack, Tab::Bank, Tab::Ammo]),
    ("Progress", &[Tab::Missions, Tab::Travel, Tab::World]),
    ("Account", &[Tab::Vehicles, Tab::Profile]),
    ("Tools", &[Tab::Raw, Tab::Import, Tab::About]),
];

/// The drawn sidebar glyph for a tab.
fn nav_icon_for(tab: Tab) -> crate::ui::pixel::NavIcon {
    use crate::ui::pixel::NavIcon as I;
    match tab {
        Tab::Character => I::Person,
        Tab::Skills => I::Star,
        Tab::Currency => I::Coin,
        Tab::Backpack => I::Bag,
        Tab::Bank => I::Vault,
        Tab::Ammo => I::Bullet,
        Tab::Missions => I::List,
        Tab::Travel => I::Pin,
        Tab::World => I::Globe,
        Tab::Vehicles => I::Wheel,
        Tab::Raw => I::Braces,
        Tab::About => I::Info,
        Tab::Profile => I::Badge,
        Tab::Import => I::Import,
    }
}

/// The four shell regions, in logical points, recomputed every frame. The shell
/// test asserts these rects (spec §2.2: 56 px top bar, 204 px sidebar, 34 px
/// status bar).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShellRects {
    pub top: egui::Rect,
    pub sidebar: egui::Rect,
    pub status: egui::Rect,
    pub content: egui::Rect,
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
    /// Weapon-type / item-definition / balance-chain tables that resolve the
    /// per-slot part lists (Gibbed's `Create`).
    pub defs: Option<Arc<GameDefs>>,
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
    /// World progress (fields 29/30/43/46/27), mirrored for editing.
    pub world_regions: Vec<lootsmith_core::world::RegionGameStage>,
    pub world_discoveries: Vec<lootsmith_core::world::WorldDiscovery>,
    pub world_lockouts: Vec<lootsmith_core::world::Lockout>,
    pub world_expansions: Vec<lootsmith_core::world::DlcExpansion>,
    /// Memento counts `(weapons, items)` (fields 32/33).
    pub mementos: (usize, usize),
    /// Challenge-progress stats (field 15) mirrored for editing.
    pub challenge_stats: Vec<ChallengeStat>,
    challenge_stat_orig: Vec<(u16, i32)>,
    /// True when the bundled tables are in use (no user folder).
    pub dumps_bundled: bool,
    /// Game-data folder candidates for first-run onboarding.
    pub dump_candidates: Vec<PathBuf>,
    /// Search text for the challenge stats.
    pub challenge_filter: String,
    /// Debug: active tab key (tests).
    /// `NumChallengePrestiges` (field 42).
    pub prestiges: i32,
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
    /// Skills: block a rank add when no points are available (default on).
    pub enforce_budget: bool,
    /// Skills: a pasted `LS2:` build string to apply.
    pub skill_build: String,
    /// Skills: the first Respec click arms, the second confirms.
    pub respec_armed: bool,
    pub theme: Theme,
    theme_applied: bool,
    /// UI scale: 1.0, 1.5 or 2.0 (spec §2.1 rule 7). Applied via
    /// `ctx.set_pixels_per_point`.
    pub ui_scale: f32,
    /// Cached background dither texture, keyed by dark/light.
    dither_tex: [Option<egui::TextureHandle>; 2],
    logo_tex: Option<egui::TextureHandle>,
    /// The shell rects computed by `build` (top bar, sidebar, status bar, content).
    pub shell: ShellRects,
    /// Whether the sidebar is collapsed to the icon rail.
    pub sidebar_collapsed: bool,
    /// Animated collapse progress (0 = expanded, 1 = rail).
    pub sidebar_anim: f32,
    /// Set each frame when the window is too narrow for the expanded sidebar.
    /// Kept separate so the automatic collapse never overwrites the user's choice.
    pub sidebar_narrow: bool,
    /// Settings-dialog visibility (top-bar "Menu").
    pub show_settings: bool,
    /// The Ctrl+K action-search text.
    pub action_search: String,
    /// Ctrl+K command palette visibility and selection.
    pub show_palette: bool,
    pub palette_query: String,
    pub palette_sel: usize,
    /// The "Paste BL2(…)" dialog.
    pub show_paste: bool,
    /// Test hook: force the workbench scroll offset (None = normal). Not set in
    /// normal use; lets a test jump the part list to a specific position.
    pub inspector_scroll: Option<f32>,
    /// The UI-scale mode (Auto or a fixed integer scale).
    pub scale_mode: ScaleMode,
    /// Latching auto-scale state (decides once per monitor size).
    pub(crate) auto: crate::ui::auto_scale::AutoScale,
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
    /// Test/tool isolation: when set, never read `LOOTSMITH_*` variables and
    /// never run first-run save/dump discovery (see [`LootsmithApp::new_isolated`]).
    ignore_env: bool,
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
            defs: None,
            name_parts: None,
            dlc_titles: None,
            ammo: Vec::new(),
            ammo_orig: Vec::new(),
            sdus: [0; 9],
            sdu_orig: [0; 9],
            backpack_slots: 12,
            bank_size: 6,
            world_regions: Vec::new(),
            world_discoveries: Vec::new(),
            world_lockouts: Vec::new(),
            world_expansions: Vec::new(),
            mementos: (0, 0),
            challenge_stats: Vec::new(),
            challenge_stat_orig: Vec::new(),
            dumps_bundled: true,
            dump_candidates: Vec::new(),
            challenge_filter: String::new(),
            prestiges: 0,
            mission_rows: Vec::new(),
            plot_mission: 0,
            active_mission: 0,
            challenges: Vec::new(),
            missions_data: None,
            raw: Vec::new(),
            raw_filter: String::new(),
            currency_show_unused: false,
            enforce_budget: true,
            skill_build: String::new(),
            respec_armed: false,
            theme: Theme::Light,
            theme_applied: false,
            ui_scale: 1.0,
            dither_tex: [None, None],
            logo_tex: None,
            shell: ShellRects {
                top: egui::Rect::ZERO,
                sidebar: egui::Rect::ZERO,
                status: egui::Rect::ZERO,
                content: egui::Rect::ZERO,
            },
            sidebar_collapsed: false,
            sidebar_anim: 0.0,
            sidebar_narrow: false,
            show_settings: false,
            action_search: String::new(),
            show_palette: false,
            palette_query: String::new(),
            palette_sel: 0,
            show_paste: false,
            inspector_scroll: None,
            scale_mode: ScaleMode::Auto,
            auto: crate::ui::auto_scale::AutoScale::default(),
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
            ignore_env: false,
            tab: Tab::Character,
            selected: None,
            item_edit: None,
            choices: HashMap::new(),
        };
        if !app.dumps.trim().is_empty() {
            app.load_dumps();
        }
        app
    }

    /// Construct an app for **tests and tools** that must be isolated from the
    /// host. It never reads `LOOTSMITH_*` environment variables, never runs the
    /// first-run save/dump discovery, and never touches persisted settings
    /// (those live in `main.rs` via eframe `Storage`, not here). The bundled
    /// game-data tables are always used; the save at `open` still opens.
    pub fn new_isolated(open: Option<String>) -> Self {
        let mut app = Self::new(open, None);
        app.ignore_env = true;
        // The constructor skips loading when the dumps path is empty, so load
        // here: with `ignore_env` set this resolves to the bundled tables.
        app.load_dumps();
        app
    }

    /// True when this app ignores `LOOTSMITH_*` variables (tests/tools).
    pub fn ignores_env(&self) -> bool {
        self.ignore_env
    }

    /// Read a `LOOTSMITH_*` variable, or `None` when the app is isolated. Every
    /// environment read in this crate goes through here, so `new_isolated`
    /// turns them all off in one place.
    fn env_var(&self, name: &str) -> Option<String> {
        if self.ignore_env {
            return None;
        }
        std::env::var(name).ok().filter(|v| !v.trim().is_empty())
    }

    /// Switch theme and force it to be applied on the next frame.
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
        self.theme_applied = false;
    }

    pub fn set_tab(&mut self, tab: Tab) {
        self.tab = tab;
    }

    /// The permitted integer UI scales (spec §2.1 rule 7). Fractional scales
    /// blur pixel art, so only whole multiples are offered.
    pub const UI_SCALES: [f32; 4] = [1.0, 2.0, 3.0, 4.0];

    /// Logical width the shell is designed for. A narrower window shrinks the whole
    /// UI to fit (the one fractional-scale exception to rule 7).
    pub const DESIGN_W: f32 = 1280.0;
    /// Never shrink past this: below it, content scrolls rather than getting tiny.
    pub const MIN_ZOOM: f32 = 0.6;

    /// The auto UI scale for a monitor's **physical** pixel size (spec §2.1 rule 7).
    /// Thin wrapper over [`crate::ui::auto_scale::scale_for`].
    pub fn ui_scale_for(physical_width_px: u32, physical_height_px: u32) -> u32 {
        crate::ui::auto_scale::scale_for(physical_width_px, physical_height_px)
    }

    /// Snap an arbitrary scale to the nearest permitted value (ties favour the
    /// smaller scale).
    pub fn clamp_ui_scale(scale: f32) -> f32 {
        Self::UI_SCALES
            .into_iter()
            .min_by(|a, b| {
                (scale - *a)
                    .abs()
                    .partial_cmp(&(scale - *b).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(1.0)
    }

    /// Set a fixed UI scale (switches to [`ScaleMode::Fixed`]).
    pub fn set_ui_scale(&mut self, scale: f32) {
        let v = Self::clamp_ui_scale(scale).round() as u32;
        self.scale_mode = ScaleMode::Fixed(v.clamp(1, 4));
        self.ui_scale = v.clamp(1, 4) as f32;
    }

    /// Select the scale mode (Auto or a fixed integer scale).
    pub fn set_scale_mode(&mut self, mode: ScaleMode) {
        if mode == ScaleMode::Auto {
            // Re-decide from scratch after leaving a fixed scale.
            self.auto = crate::ui::auto_scale::AutoScale::default();
        }
        self.scale_mode = mode;
        if let ScaleMode::Fixed(n) = mode {
            self.ui_scale = n.clamp(1, 4) as f32;
        }
    }

    /// Whether the active container tab is the Bank.
    pub(crate) fn is_bank(&self) -> bool {
        self.tab == Tab::Bank
    }

    /// Run a palette / menu command. The Menu window and the Ctrl+K palette share
    /// this, so every command is reachable from both.
    pub(crate) fn run_cmd(&mut self, cmd: Cmd, ctx: &egui::Context) {
        let bank = self.is_bank();
        match cmd {
            Cmd::Open => self.dispatch(Action::Open),
            Cmd::FindSaves => self.dispatch(Action::FindSaves),
            Cmd::Save => self.dispatch(Action::Save),
            Cmd::SaveAs => self.dispatch(Action::SaveAs),
            Cmd::Reload => self.dispatch(Action::Reload),
            Cmd::NewSave => self.dispatch(Action::New),
            Cmd::Import => self.dispatch(Action::Import),
            Cmd::Profile => self.dispatch(Action::Profile),
            Cmd::PickData => self.dispatch(Action::PickData),
            Cmd::ToggleTheme => self.dispatch(Action::ToggleTheme),
            Cmd::NewWeapon => {
                self.new_item(bank, true);
            }
            Cmd::NewItem => {
                self.new_item(bank, false);
            }
            Cmd::Duplicate => {
                self.duplicate_selected();
            }
            Cmd::CopyCode => {
                self.copy_selected_code(ctx);
            }
            Cmd::SyncLevels => {
                self.sync_levels(false);
            }
            Cmd::SyncEquipped => {
                self.sync_levels(true);
            }
            Cmd::InsertCodes => {
                self.show_paste = true;
            }
            Cmd::Scale(n) => {
                self.set_scale_mode(ScaleMode::Fixed(n.clamp(1, 4)));
            }
        }
        ctx.request_repaint();
    }

    /// Filtered palette commands for the current query.
    pub(crate) fn palette_items(&self) -> Vec<Cmd> {
        let q = self.palette_query.trim().to_lowercase();
        PALETTE
            .iter()
            .copied()
            .filter(|c| q.is_empty() || c.label().to_lowercase().contains(&q))
            .collect()
    }

    /// Resolve and apply the UI scale for this frame (spec §2.1 rule 7). `Auto`
    /// latches on the monitor's **physical** size so it settles once and never
    /// drives a resize loop. On top of that the whole UI is shrunk just enough to
    /// fit the window (a fractional zoom — the one exception to the whole-number
    /// rule), so no bar overlaps on a small window. The physical window size is
    /// invariant under zoom, so the fit cannot feed back either.
    fn apply_scale(&mut self, ctx: &egui::Context) {
        let native = ctx.native_pixels_per_point().unwrap_or(1.0);
        let (base, allow_fit): (f32, bool) = match self.scale_mode {
            ScaleMode::Fixed(n) => (n.clamp(1, 4) as f32, false),
            ScaleMode::Auto => {
                // `update` must be called exactly once per frame.
                let mon = ctx.input(|i| i.viewport().monitor_size).map(|m| (m.x, m.y));
                let _ = self.auto.update(mon, ctx.pixels_per_point(), native);
                (self.auto.applied().unwrap_or(1) as f32, true)
            }
        };
        let mut target = base;
        if allow_fit {
            // Physical window width (invariant under zoom) / design width.
            let screen = ctx.content_rect().size() * ctx.pixels_per_point();
            target = (screen.x / Self::DESIGN_W).min(base);
            // Snap down to 1/64: fine enough to barely over-shrink, coarse enough
            // to damp resize jitter.
            target = ((target * 64.0).floor() / 64.0).clamp(Self::MIN_ZOOM, 4.0);
        }
        if (target - ctx.pixels_per_point()).abs() > 1.0 / 128.0 {
            ctx.set_pixels_per_point(target);
        }
        self.ui_scale = target;
    }

    /// The cached background dither texture for the active theme (spec §2.5).
    /// The texture id is kept stable across a theme flip (updated in place), so
    /// no frame renders with a stale dither.
    fn dither_handle(&mut self, ctx: &egui::Context, p: &theme::Palette) -> egui::TextureHandle {
        let idx = if self.theme == Theme::Dark { 0 } else { 1 };
        for (i, palette) in [&theme::DARK, &theme::LIGHT].into_iter().enumerate() {
            if self.dither_tex[i].is_none() {
                let name = if i == 0 {
                    "lootsmith.dither.dark"
                } else {
                    "lootsmith.dither.light"
                };
                self.dither_tex[i] =
                    Some(crate::ui::pixel::dither_texture_named(ctx, name, palette));
            }
        }
        let _ = p;
        self.dither_tex[idx].clone().expect("dither texture")
    }

    /// The cached crest texture, decoded from the embedded 256 px PNG. Filtered
    /// linearly (with mipmaps) because the crest is detailed artwork, not a
    /// pixel-art widget: nearest-neighbour downscaling made it jagged and its
    /// banner text unreadable.
    fn logo(&mut self, ctx: &egui::Context) -> Option<egui::TextureHandle> {
        if self.logo_tex.is_none() {
            let ic = eframe::icon_data::from_png_bytes(include_bytes!("../assets/lootsmith.png"))
                .ok()?;
            let image = egui::ColorImage::from_rgba_unmultiplied(
                [ic.width as usize, ic.height as usize],
                &ic.rgba,
            );
            let options = egui::TextureOptions {
                mipmap_mode: Some(egui::TextureFilter::Linear),
                ..egui::TextureOptions::LINEAR
            };
            self.logo_tex = Some(ctx.load_texture("lootsmith.logo", image, options));
        }
        self.logo_tex.clone()
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
        self.refresh_challenge_stats();
        self.refresh_world();
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
            self.env_var("LOOTSMITH_ASSET_LIBRARY").unwrap_or_default()
        } else {
            self.dumps.clone()
        };
        let path = PathBuf::from(raw.trim());
        let library = if path.is_dir() {
            path.join("Asset Library Manager.json")
        } else {
            path.clone()
        };

        // A user folder always wins; otherwise use the bundled tables, so a
        // fresh install shows part names with no setup at all.
        let from_user = !raw.trim().is_empty() && library.is_file();
        let loaded = if from_user {
            lootsmith_core::gameinfo::AssetLibraryManager::from_path(&library)
                .map_err(|e| e.to_string())
        } else {
            lootsmith_core::bundled::asset_library().map_err(|e| e.to_string())
        };
        match loaded {
            Ok(m) => {
                self.manager = Some(Arc::new(m));
                self.dumps_bundled = !from_user;
                self.status = if from_user {
                    format!("game data: {}", library.display())
                } else {
                    "game data: bundled tables (point Game data at a folder to override)".into()
                };
            }
            Err(e) => {
                self.manager = None;
                self.status = format!("asset library error: {e}");
            }
        }
        self.defs = self.load_defs();
        self.name_parts = self.load_name_parts();
        self.dlc_titles = self.load_dlc_titles();
        self.refresh_rows();
        self.load_travel_stations();
        self.load_customizations();
        self.load_missions_data();
    }

    /// The part choices for one slot, from the resolved weapon-type / item
    /// balance (`GameDefs`), matching Gibbed's `Create`: the balance `base`
    /// chain merged by mode, seeded by the weapon type / item definition.
    /// Falls back to the plain library choices only when no definitions are
    /// loaded (so the dropdown is never empty).
    pub fn slot_choices(
        &self,
        is_weapon: bool,
        type_path: Option<&str>,
        balance: &str,
        slot: &str,
        group: AssetGroup,
    ) -> Vec<String> {
        if let Some(defs) = self.defs.as_ref() {
            let resolved = match type_path {
                Some(ty) if is_weapon => defs.weapon_slot_choices(ty, balance, slot),
                Some(ty) => defs.item_slot_choices(ty, balance, slot),
                // No weapon type / item definition: only "None" is legal.
                None => Some(Vec::new()),
            };
            if let Some(list) = resolved {
                return list;
            }
        }
        self.choices.get(&group).cloned().unwrap_or_default()
    }

    /// Load the weapon-type / item / balance definitions (from the game-data
    /// folder, else the bundled tables).
    fn load_defs(&self) -> Option<Arc<GameDefs>> {
        if let Some(dir) = self.dumps_dir() {
            if let Ok(defs) = GameDefs::from_dir(&dir) {
                if !defs.is_empty() {
                    return Some(Arc::new(defs));
                }
            }
        }
        lootsmith_core::bundled::defs().ok().map(Arc::new)
    }

    /// Load DLC chapter titles for the travel-tab grouping.
    fn load_dlc_titles(&self) -> Option<Arc<DlcTitles>> {
        if let Some(dir) = self.dumps_dir() {
            if let Ok(titles) = DlcTitles::from_path(&dir.join("Downloadable Contents.json")) {
                if !titles.is_empty() {
                    return Some(Arc::new(titles));
                }
            }
        }
        lootsmith_core::bundled::dlc_titles().ok().map(Arc::new)
    }

    /// Load the name parts (from the game-data folder or
    /// `LOOTSMITH_NAME_PARTS`) for item display names.
    fn load_name_parts(&self) -> Option<Arc<NameParts>> {
        if let Some(dir) = self.dumps_dir() {
            // A user folder that yields no name parts must not shadow the
            // bundled tables (the bundled files use different names).
            if let Ok(parts) = NameParts::from_dir(&dir) {
                if !parts.is_empty() {
                    return Some(Arc::new(parts));
                }
            }
        }
        if let Some(path) = self.env_var("LOOTSMITH_NAME_PARTS") {
            if let Ok(parts) = NameParts::from_path(std::path::Path::new(&path)) {
                if !parts.is_empty() {
                    return Some(Arc::new(parts));
                }
            }
        }
        // Bundled tables: a fresh run resolves item names with no setup.
        lootsmith_core::bundled::name_parts().ok().map(Arc::new)
    }

    pub(crate) fn refresh_rows(&mut self) {
        let configs = self.configs();
        let manager = self.manager.clone();
        let name_parts = self.name_parts.clone();
        let mut backpack = Vec::new();
        let mut bank = Vec::new();
        if let Some(save_game) = &self.save_game {
            for (i, entry) in save_game.packed_entries().iter().enumerate() {
                backpack.push(row_for(
                    entry,
                    i + 1,
                    &manager,
                    &configs,
                    name_parts.as_deref(),
                ));
            }
            for (i, entry) in save_game.bank_entries().iter().enumerate() {
                bank.push(row_for(
                    entry,
                    i + 1,
                    &manager,
                    &configs,
                    name_parts.as_deref(),
                ));
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
        if let Some(env) = self.env_var("LOOTSMITH_TRAVEL_STATIONS") {
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
        // Fall back to the bundled table (a user folder always overrides).
        self.stations = lootsmith_core::bundled::travel_stations()
            .ok()
            .map(Arc::new);
    }

    /// Snapshot the ammo pools (field 11).
    /// Set `NumChallengePrestiges` (field 42).
    pub fn set_prestige(&mut self, level: i32) {
        if let Some(save_game) = self.save_game.as_mut() {
            save_game.set_i32(42, level.max(0));
        }
        self.prestiges = level.max(0);
        self.dirty = true;
    }

    /// Uncovered discoveries count (for the tab header / tests).
    pub fn discoveries_uncovered(&self) -> usize {
        self.world_discoveries
            .iter()
            .filter(|d| d.uncovered)
            .count()
    }

    pub(crate) fn refresh_world(&mut self) {
        let Some(save_game) = &self.save_game else {
            self.world_regions.clear();
            self.world_discoveries.clear();
            self.world_lockouts.clear();
            self.world_expansions.clear();
            self.mementos = (0, 0);
            return;
        };
        self.world_regions = save_game.region_game_stages();
        self.world_discoveries = save_game.world_discoveries();
        self.world_lockouts = save_game.lockouts();
        self.world_expansions = save_game.dlc_expansions();
        self.mementos = save_game.memento_counts();
    }

    /// Mark every world discovery explored, then refresh the mirrors.
    pub fn mark_all_discovered(&mut self) {
        if let Some(save_game) = self.save_game.as_mut() {
            match save_game.mark_all_discovered() {
                Ok(count) => self.status = format!("marked {count} discoveries explored"),
                Err(e) => self.status = format!("discovery error: {e}"),
            }
        }
        self.refresh_world();
        self.dirty = true;
    }

    /// Zero every lockout timer, then refresh the mirrors.
    pub fn clear_all_lockouts(&mut self) {
        if let Some(save_game) = self.save_game.as_mut() {
            let count = save_game.clear_all_lockouts();
            self.status = format!("cleared {count} lockout timers");
        }
        self.refresh_world();
        self.dirty = true;
    }

    /// Drop every weapon/item memento.
    pub fn clear_mementos(&mut self) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        let (weapons, items) = save_game.clear_mementos();
        self.status = format!("cleared mementos (weapons {weapons}, items {items})");
        self.refresh_world();
        self.dirty = true;
    }

    /// Set one region's game stage (1-based as shown).
    pub fn set_world_stage(&mut self, pool: usize, stage: i32) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        if let Err(e) = save_game.set_region_game_stage(pool.max(1) - 1, stage) {
            self.status = format!("world error: {e}");
            return;
        }
        self.refresh_world();
        self.dirty = true;
    }

    /// Set one discovery by index (1-based as shown).
    pub fn set_discovery(&mut self, index: usize, uncovered: bool) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        if let Err(e) = save_game.set_discovery(index.max(1) - 1, uncovered) {
            self.status = format!("world error: {e}");
            return;
        }
        self.refresh_world();
        self.dirty = true;
    }

    pub(crate) fn refresh_challenge_stats(&mut self) {
        let stats = self
            .save_game
            .as_ref()
            .and_then(|save| save.challenge_stats().ok())
            .unwrap_or(ChallengeStats {
                unknown: 4,
                entries: Vec::new(),
            });
        self.challenge_stat_orig = stats
            .entries
            .iter()
            .map(|entry| (entry.id, entry.total_value))
            .collect();
        self.challenge_stats = stats.entries;
        self.prestiges = self
            .save_game
            .as_ref()
            .and_then(|save| save.num_challenge_prestiges())
            .unwrap_or(0);
    }

    /// Set one challenge's total value by id.
    pub fn set_challenge_value(&mut self, id: u16, total: i32) {
        let Some(save_game) = self.save_game.as_mut() else {
            return;
        };
        if let Err(e) = save_game.set_challenge_stat(id, |entry| entry.total_value = total) {
            self.status = format!("challenge error: {e}");
            return;
        }
        self.challenge_stat_orig = self
            .save_game
            .as_ref()
            .and_then(|s| s.challenge_stats().ok())
            .map(|stats| {
                stats
                    .entries
                    .iter()
                    .map(|entry| (entry.id, entry.total_value))
                    .collect()
            })
            .unwrap_or_default();
        self.challenge_stats = self
            .save_game
            .as_ref()
            .and_then(|s| s.challenge_stats().ok())
            .map(|stats| stats.entries)
            .unwrap_or_default();
        self.dirty = true;
    }

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
        if let Some(env) = self.env_var("LOOTSMITH_CUSTOMIZATIONS") {
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
        self.customizations = lootsmith_core::bundled::customizations().ok().map(Arc::new);
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
        if let Some(env) = self.env_var("LOOTSMITH_MISSIONS") {
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
        self.missions_data = lootsmith_core::bundled::missions().ok().map(Arc::new);
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
                        lootsmith_core::proto::WireType::LengthDelimited => {
                            format!("{} bytes", field.as_bytes().map(|b| b.len()).unwrap_or(0))
                        }
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
            || self.mission_rows.iter().any(|r| r.status != r.orig_status)
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
        self.tab = Tab::Character;
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
        self.tab = Tab::Character;
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
        let other = match container::unpack(&data, Platform::PC).and_then(|file| file.save_game()) {
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
                self.status = format!(
                    "saved profile {path} ({} bytes, backup {backup})",
                    bytes.len()
                );
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
            scale: String,
            collapsed: bool,
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
            scale: match self.scale_mode {
                ScaleMode::Auto => "auto".to_string(),
                ScaleMode::Fixed(n) => n.to_string(),
            },
            collapsed: self.sidebar_collapsed,
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
            #[serde(default)]
            scale: String,
            #[serde(default)]
            collapsed: bool,
        }
        let Ok(settings) = serde_json::from_str::<Settings>(text) else {
            return;
        };
        self.sidebar_collapsed = settings.collapsed;
        self.sidebar_anim = if settings.collapsed { 1.0 } else { 0.0 };
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
        self.scale_mode = match settings.scale.as_str() {
            "1" => ScaleMode::Fixed(1),
            "2" => ScaleMode::Fixed(2),
            "3" => ScaleMode::Fixed(3),
            "4" => ScaleMode::Fixed(4),
            _ => ScaleMode::Auto,
        };
        // Seed the latch so frame 0 is already at the persisted scale.
        self.auto = crate::ui::auto_scale::AutoScale::with_persisted(match self.scale_mode {
            ScaleMode::Fixed(n) => Some(n),
            ScaleMode::Auto => None,
        });
        if !self.dumps.trim().is_empty() {
            self.load_dumps();
        }
    }

    /// Collapse/expand the sidebar instantly (no animation). Tests and
    /// snapshots use this so a single `run()` lands on the final layout.
    pub fn set_sidebar_collapsed(&mut self, collapsed: bool) {
        self.sidebar_collapsed = collapsed;
        self.sidebar_anim = if collapsed { 1.0 } else { 0.0 };
    }

    fn apply_theme(&mut self, ctx: &egui::Context) {
        if self.theme_applied {
            return;
        }
        self.theme_applied = true;
        theme::apply(ctx, self.theme);
    }

    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        let (save_as, save, open, reload, palette) = ctx.input_mut(|input| {
            use egui::{Key, KeyboardShortcut, Modifiers};
            (
                input.consume_shortcut(&KeyboardShortcut::new(
                    Modifiers::CTRL | Modifiers::SHIFT,
                    Key::S,
                )),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::S)),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::O)),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::R)),
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::K)),
            )
        });
        if palette {
            self.show_palette = true;
        }
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

    /// The top-bar menu bar. Commands that need dispatch are returned; toggles
    /// (theme, scale, sidebar, skill budget) are applied in place.
    fn top_menu(&mut self, ui: &mut egui::Ui) -> Option<Action> {
        let mut action = None;
        ui.menu_button("File", |ui| {
            if ui.button("Open…").clicked() {
                action = Some(Action::Open);
                ui.close();
            }
            if ui.button("Find saves…").clicked() {
                action = Some(Action::FindSaves);
                ui.close();
            }
            if ui.button("Reload").clicked() {
                action = Some(Action::Reload);
                ui.close();
            }
            if ui.button("New…").clicked() {
                action = Some(Action::New);
                ui.close();
            }
            if ui.button("Profile…").clicked() {
                action = Some(Action::Profile);
                ui.close();
            }
            if ui.button("Import sections…").clicked() {
                action = Some(Action::Import);
                ui.close();
            }
            if ui.button("Preferences…").clicked() {
                self.show_settings = true;
                ui.close();
            }
            ui.separator();
            if ui.button("Save").clicked() {
                action = Some(Action::Save);
                ui.close();
            }
            if ui.button("Save As…").clicked() {
                action = Some(Action::SaveAs);
                ui.close();
            }
            ui.separator();
            ui.menu_button("Platform", |ui| {
                for platform in [
                    Platform::PC,
                    Platform::X360,
                    Platform::PS3,
                    Platform::PSVita,
                    Platform::Shield,
                    Platform::Switch,
                ] {
                    let selected = self.save_platform == platform;
                    if ui
                        .selectable_label(selected, platform.id().to_uppercase())
                        .clicked()
                    {
                        self.save_platform = platform;
                        ui.close();
                    }
                }
            });
            ui.separator();
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close();
            }
        });
        ui.menu_button("Edit", |ui| {
            // Placeholders until the undo stack lands; shown disabled so the
            // menu matches the real command set.
            ui.add_enabled(false, egui::Button::new("Undo"));
            ui.add_enabled(false, egui::Button::new("Redo"));
        });
        ui.menu_button("View", |ui| {
            ui.menu_button("Theme", |ui| {
                if ui
                    .selectable_label(self.theme == Theme::Dark, "Dark")
                    .clicked()
                {
                    self.set_theme(Theme::Dark);
                    ui.close();
                }
                if ui
                    .selectable_label(self.theme == Theme::Light, "Light")
                    .clicked()
                {
                    self.set_theme(Theme::Light);
                    ui.close();
                }
            });
            // Scale by size, not by number: menu text keeps the label font.
            ui.menu_button("Scale", |ui| {
                if ui
                    .selectable_label(matches!(self.scale_mode, ScaleMode::Auto), "Auto")
                    .clicked()
                {
                    self.set_scale_mode(ScaleMode::Auto);
                    ui.close();
                }
                for (value, label) in Self::UI_SCALES
                    .iter()
                    .zip(["Small", "Normal", "Large", "Huge"])
                {
                    let selected = self.scale_mode == ScaleMode::Fixed(*value as u32);
                    if ui.selectable_label(selected, label).clicked() {
                        self.set_scale_mode(ScaleMode::Fixed(*value as u32));
                        ui.close();
                    }
                }
            });
            ui.separator();
            let expanded = !self.sidebar_collapsed;
            if ui.selectable_label(expanded, "Show sidebar").clicked() {
                self.sidebar_collapsed = !expanded;
                ui.close();
            }
            if ui
                .selectable_label(self.enforce_budget, "Enforce skill budget")
                .clicked()
            {
                self.enforce_budget = !self.enforce_budget;
                ui.close();
            }
        });
        ui.menu_button("Tools", |ui| {
            if ui.button("Find an action…").clicked() {
                self.show_palette = true;
                ui.close();
            }
            if ui.button("Game data folder…").clicked() {
                action = Some(Action::PickData);
                ui.close();
            }
        });
        ui.menu_button("Help", |ui| {
            if ui.button("About").clicked() {
                self.tab = Tab::About;
                ui.close();
            }
        });
        action
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
            Action::Import => self.open_import_page(),
            Action::Profile => self.open_profile_page(),
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
    pub(crate) fn skill_texture(
        &mut self,
        ctx: &egui::Context,
        filename: &str,
    ) -> Option<egui::TextureHandle> {
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
        self.skill_icons
            .insert(filename.to_string(), handle.clone());
        Some(handle)
    }

    /// Build the full UI. Called by both the native app and the test harness.
    /// The sidebar base label and count badge for a tab.
    pub(crate) fn nav_base_count(&self, tab: Tab) -> (&'static str, Option<usize>) {
        match tab {
            Tab::Backpack => ("Backpack", Some(self.backpack.len())),
            Tab::Bank => ("Bank", Some(self.bank.len())),
            _ => (
                TABS.iter()
                    .find(|(t, _)| *t == tab)
                    .map(|(_, l)| *l)
                    .unwrap_or("?"),
                None,
            ),
        }
    }

    /// The sidebar label for a tab (Backpack/Bank carry a live count badge).
    fn nav_label(&self, tab: Tab) -> String {
        match tab {
            Tab::Backpack => format!("Backpack ({})", self.backpack.len()),
            Tab::Bank => format!("Bank ({})", self.bank.len()),
            _ => TABS
                .iter()
                .find(|(t, _)| *t == tab)
                .map(|(_, l)| (*l).to_string())
                .unwrap_or_else(|| "?".to_string()),
        }
    }

    pub fn build(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.apply_theme(&ctx);
        self.apply_scale(&ctx);
        let p = theme::active(&ctx);
        self.handle_shortcuts(&ctx);

        if !self.loaded_once {
            self.loaded_once = true;
            // Isolated apps (tests/tools) never run first-run discovery, which
            // would read `LOOTSMITH_DUMP_DIRS` / `LOOTSMITH_SAVE_ROOTS` and the
            // host's save folders. They use the bundled tables and the given
            // save only.
            if !self.ignore_env {
                if self.path.trim().is_empty() {
                    // First run: auto-open the newest save we can find.
                    self.scan_saves();
                    if let Some(newest) = self.saves.first() {
                        self.path = newest.path.display().to_string();
                    }
                }
                if self.dumps.trim().is_empty() {
                    // First run: adopt a game-data folder when one is obvious,
                    // else the bundled tables are used (no picker).
                    if let Some(dir) = lootsmith_core::find_dump_dirs().first() {
                        self.dumps = dir.display().to_string();
                    }
                }
            }
            if self.dumps.trim().is_empty() {
                self.load_dumps();
            }
            if !self.path.trim().is_empty() {
                self.load();
            }
        }

        let mut action: Option<Action> = None;
        let ppp = ctx.pixels_per_point();
        let gap = 16.0;

        // Spec §2.5: paint `bg` + the dither tile over the WHOLE root so nothing
        // ever shows the renderer's clear colour (no black bars), at any scale or
        // while resizing. Bars are then drawn flush on top.
        {
            let tex = self.dither_handle(&ctx, p);
            let screen = ctx.content_rect();
            let bg = ctx.layer_painter(egui::LayerId::background());
            bg.rect_filled(screen, 0.0, p.bg);
            crate::ui::pixel::dither(&bg, screen, &tex);
        }

        // --- Top plank bar (56 px), flush to the window edges ---
        egui::Panel::top("topbar")
            .exact_size(56.0)
            .frame(egui::Frame::default())
            .show(ui, |ui| {
                let bar = ui.max_rect();
                self.shell.top = bar;
                crate::ui::pixel::plank_rule(ui.painter(), bar, ppp, p, true);
                let row = bar.shrink2(egui::vec2(14.0, 0.0));
                ui.scope_builder(egui::UiBuilder::new().max_rect(row), |ui| {
                    crate::ui::widgets::wood_style(ui, p);
                    ui.horizontal_centered(|ui| {
                        if let Some(a) = self.top_menu(ui) {
                            action = Some(a);
                        }
                        ui.add_space(6.0);
                        // File name chip; a dot marks unsaved changes.
                        let name = std::path::Path::new(&self.path)
                            .file_name()
                            .map(|s| s.to_string_lossy().to_string())
                            .filter(|s| !s.is_empty())
                            .unwrap_or_else(|| "no file".to_string());
                        let chip_w = 210.0;
                        let name = crate::ui::widgets::fit_text(ui, &name, chip_w - 26.0, 13.0);
                        let (r, _) =
                            ui.allocate_exact_size(egui::vec2(chip_w, 24.0), egui::Sense::hover());
                        crate::ui::widgets::chip(ui, r, &name, p.fg);
                        if self.dirty {
                            ui.painter().circle_filled(
                                r.right_center() - egui::vec2(9.0, 0.0),
                                3.5,
                                p.dng,
                            );
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let (r, _) = ui
                                .allocate_exact_size(egui::vec2(66.0, 28.0), egui::Sense::click());
                            if crate::ui::widgets::button(
                                ui,
                                r,
                                "Save",
                                crate::ui::widgets::ButtonKind::Primary,
                            )
                            .clicked()
                            {
                                action = Some(Action::Save);
                            }
                            // The action search moved behind a compact icon.
                            let (sr, sresp) = ui
                                .allocate_exact_size(egui::vec2(30.0, 26.0), egui::Sense::click());
                            crate::ui::pixel::nav_icon(
                                ui.painter(),
                                sr.center(),
                                crate::ui::pixel::NavIcon::Search,
                                p.onw,
                                1.0,
                                ppp,
                            );
                            sresp.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::WidgetType::Button,
                                    true,
                                    "Find an action…  Ctrl+K",
                                )
                            });
                            if sresp.on_hover_text("Find an action…  Ctrl+K").clicked() {
                                self.show_palette = true;
                            }
                        });
                    });
                });
            });

        // --- Status plank bar (34 px), flush to the window edges ---
        egui::Panel::bottom("status")
            .exact_size(34.0)
            .frame(egui::Frame::default())
            .show(ui, |ui| {
                let bar = ui.max_rect();
                self.shell.status = bar;
                // `pan` base with a gold rule: the status text and any screen text
                // that overflows the body keep contrast on a light fill.
                ui.painter().rect_filled(bar, 0.0, p.pan);
                ui.painter().rect_filled(
                    egui::Rect::from_min_max(bar.min, egui::pos2(bar.max.x, bar.min.y + 2.0)),
                    0.0,
                    p.gold,
                );
                let row = bar.shrink2(egui::vec2(14.0, 0.0));
                ui.scope_builder(egui::UiBuilder::new().max_rect(row), |ui| {
                    crate::ui::widgets::wood_style(ui, p);
                    ui.horizontal_centered(|ui| {
                        status_dot(
                            ui,
                            if self.dirty { Glyph::Dot } else { Glyph::Check },
                            if self.dirty { p.acc } else { p.gold },
                        );
                        let status_path = crate::ui::item_table::elide_middle(&self.path, 52);
                        let level = self.form.level;
                        let saved = std::fs::metadata(&self.path)
                            .and_then(|m| m.modified())
                            .ok()
                            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                            .map(|d| {
                                let secs = d.as_secs();
                                format!("saved {:02}:{:02}", (secs / 3600) % 24, (secs / 60) % 60)
                            })
                            .unwrap_or_default();
                        ui.add(egui::Label::new(crate::ui::widgets::text_with_nums(
                            &status_path,
                            p.fg,
                            12.0,
                        )));
                        ui.add(egui::Label::new(crate::ui::widgets::text_with_nums(
                            &format!("Level {level}"),
                            p.fg,
                            12.0,
                        )));
                        if !saved.is_empty() {
                            ui.add(egui::Label::new(crate::ui::widgets::text_with_nums(
                                &saved, p.fg, 12.0,
                            )));
                        }
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let ok = self.manager.is_some();
                            let (rect, _) = ui
                                .allocate_exact_size(egui::vec2(170.0, 22.0), egui::Sense::click());
                            let hover = if self.dumps.trim().is_empty() {
                                "No game-data folder — using bundled tables".to_string()
                            } else {
                                self.dumps.clone()
                            };
                            let response = crate::ui::widgets::status_chip(
                                ui,
                                rect,
                                if ok {
                                    "Game data ready"
                                } else {
                                    "Game data missing"
                                },
                                ok,
                            )
                            .on_hover_text(hover);
                            if response.clicked() {
                                action = Some(Action::PickData);
                            }
                        });
                    });
                });
            });

        // --- Sidebar (204 px, or a 64 px rail when collapsed), inset 16 px ---
        // Auto-collapse to the rail when the window is too narrow for the expanded
        // sidebar (hysteresis keeps it stable); the manual toggle is preserved.
        const COLLAPSE_AT: f32 = 1060.0;
        let logical_w = ctx.content_rect().width();
        let threshold = if self.sidebar_narrow {
            COLLAPSE_AT + 40.0
        } else {
            COLLAPSE_AT
        };
        self.sidebar_narrow = logical_w < threshold;
        // Animate the collapse (180 ms ease-out) instead of snapping.
        let target = if self.sidebar_collapsed || self.sidebar_narrow {
            1.0
        } else {
            0.0
        };
        let dt = ui.input(|i| i.stable_dt).min(0.05);
        let step = dt / 0.18;
        self.sidebar_anim =
            (self.sidebar_anim + (target - self.sidebar_anim).clamp(-step, step)).clamp(0.0, 1.0);
        if (target - self.sidebar_anim).abs() > 0.001 {
            ui.ctx().request_repaint();
        }
        // Symmetric smoothstep on the expansion progress: 1 = expanded, 0 = rail.
        let progress = 1.0 - self.sidebar_anim;
        let reveal = (progress * progress * (3.0 - 2.0 * progress)).clamp(0.0, 1.0);
        let side_w = 64.0 + (204.0 - 64.0) * reveal;
        egui::Panel::left("sidebar")
            .show_separator_line(false)
            .exact_size(side_w + 2.0 * gap)
            .frame(egui::Frame::default().inner_margin(egui::Margin {
                left: gap as i8,
                right: gap as i8,
                top: gap as i8,
                bottom: gap as i8,
            }))
            .show(ui, |ui| {
                let bar = ui.max_rect();
                self.shell.sidebar = bar;
                crate::ui::pixel::panel(ui.painter(), bar, ppp, p);
                let inner = bar.shrink(8.0 + 6.0 * reveal);
                ui.scope_builder(egui::UiBuilder::new().max_rect(inner), |ui| {
                    ui.set_clip_rect(inner);
                    ui.spacing_mut().item_spacing.y = 0.0;
                    egui::ScrollArea::vertical()
                        .id_salt("nav_scroll")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.add_space(2.0);
                            // The logo IS the collapse toggle.
                            if let Some(tex) = self.logo(ui.ctx()) {
                                // 48 px in the rail, growing to the sidebar's full
                                // inner width when expanded so the crest and its
                                // banner text are legible.
                                let avail = ui.available_width();
                                let size = 48.0 + (avail - 48.0).max(0.0) * reveal;
                                let (lr, lresp) = ui.allocate_exact_size(
                                    egui::vec2(avail, size),
                                    egui::Sense::click(),
                                );
                                let rect = egui::Rect::from_center_size(
                                    lr.center_top() + egui::vec2(0.0, size / 2.0),
                                    egui::vec2(size, size),
                                );
                                ui.painter().image(
                                    tex.id(),
                                    rect,
                                    egui::Rect::from_min_max(
                                        egui::pos2(0.0, 0.0),
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    egui::Color32::WHITE,
                                );
                                if lresp.clicked() {
                                    self.sidebar_collapsed = !self.sidebar_collapsed;
                                }
                                lresp.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::WidgetType::Button,
                                        true,
                                        "sidebar toggle",
                                    )
                                });
                                lresp.on_hover_text(if self.sidebar_collapsed {
                                    "Expand sidebar"
                                } else {
                                    "Collapse sidebar"
                                });
                            }
                            ui.add_space(2.0 * reveal);
                            for (gi, (group, tabs)) in NAV_GROUPS.iter().enumerate() {
                                if gi > 0 {
                                    let (r, _) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 3.0 + 4.0 * reveal),
                                        egui::Sense::hover(),
                                    );
                                    if reveal > 0.5 {
                                        crate::ui::widgets::trail_divider(
                                            ui,
                                            r.left_center(),
                                            r.right_center(),
                                        );
                                    } else {
                                        // Collapsed rail: a short divider centred on the
                                        // icon column. A 19 px span is exactly three 3 px
                                        // dashes ending on a dash, so the dash/gap pattern
                                        // is symmetric about `cx`; a 24 px span would end
                                        // on a gap and read ~2 px to the left.
                                        let cx = r.center().x;
                                        crate::ui::widgets::trail_divider(
                                            ui,
                                            egui::pos2(cx - 9.5, r.center().y),
                                            egui::pos2(cx + 9.5, r.center().y),
                                        );
                                    }
                                }
                                // Group label fades out as the rail closes. Painted (not a
                                // Label) so it does not collide with a tab of the same name.
                                let (gr, _) = ui.allocate_exact_size(
                                    egui::vec2(ui.available_width(), 13.0 * reveal),
                                    egui::Sense::hover(),
                                );
                                if reveal > 0.02 {
                                    let font = egui::FontId::proportional(11.0);
                                    let gcol = p.gold.gamma_multiply(reveal);
                                    ui.painter().text(
                                        gr.left_center(),
                                        egui::Align2::LEFT_CENTER,
                                        *group,
                                        font.clone(),
                                        gcol,
                                    );
                                    // Faux-bold: a second pass half a pixel over.
                                    ui.painter().text(
                                        gr.left_center() + egui::vec2(0.6, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        *group,
                                        font,
                                        gcol,
                                    );
                                }
                                ui.add_space(1.0 * reveal);
                                for tab in tabs.iter() {
                                    let (base, count) = self.nav_base_count(*tab);
                                    let (rect, _) = ui.allocate_exact_size(
                                        egui::vec2(ui.available_width(), 24.0),
                                        egui::Sense::click(),
                                    );
                                    if crate::ui::widgets::nav_row(
                                        ui,
                                        rect,
                                        nav_icon_for(*tab),
                                        base,
                                        self.tab == *tab,
                                        count,
                                        reveal,
                                    )
                                    .clicked()
                                    {
                                        self.tab = *tab;
                                    }
                                }
                                ui.add_space(2.0 * reveal);
                            }
                        });
                });
            });

        // --- Body (+ header panel), inset 16 px ---
        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin {
                left: 0,
                right: gap as i8,
                top: gap as i8,
                bottom: gap as i8,
            }))
            .show(ui, |ui| {
                let area = ui.max_rect();
                self.shell.content = area;
                // Header panel: content inset 20 px.
                // Backpack/Bank draw their own header row inside the screen.
                let body = if matches!(self.tab, Tab::Backpack | Tab::Bank) {
                    area
                } else {
                    let header_h = 44.0;
                    let header =
                        egui::Rect::from_min_size(area.min, egui::vec2(area.width(), header_h));
                    crate::ui::pixel::panel(ui.painter(), header, ppp, p);
                    let title = self.nav_label(self.tab);
                    let ribbon_w =
                        (title.chars().count() as f32 * 9.0 + 48.0).min(header.width() - 40.0);
                    let ribbon_rect = egui::Rect::from_min_size(
                        egui::pos2(header.left() + 20.0, header.center().y - 12.0),
                        egui::vec2(ribbon_w, 24.0),
                    );
                    crate::ui::widgets::ribbon(ui, ribbon_rect, &title);
                    egui::Rect::from_min_max(header.left_bottom() + egui::vec2(0.0, 10.0), area.max)
                };
                ui.scope_builder(egui::UiBuilder::new().max_rect(body), |ui| {
                    ui.set_clip_rect(body);
                    if self.save_game.is_none() {
                        empty_state(ui);
                        return;
                    }
                    match self.tab {
                        Tab::Character => self.ui_character(ui),
                        Tab::Currency => self.ui_currency(ui),
                        Tab::Ammo => self.ui_ammo(ui),
                        Tab::World => self.ui_world(ui),
                        Tab::Skills => self.ui_skills(ui),
                        Tab::Travel => self.ui_travel(ui),
                        Tab::Vehicles => self.ui_vehicles(ui),
                        Tab::Missions => self.ui_missions(ui),
                        Tab::Backpack => self.ui_container(ui, false),
                        Tab::Bank => self.ui_container(ui, true),
                        Tab::Raw => self.ui_raw(ui),
                        Tab::About => tabs::about::ui_about(ui),
                        Tab::Profile => self.ui_profile(ui),
                        Tab::Import => self.ui_import(ui),
                    }
                });
            });

        if self.show_settings {
            let mut open = true;
            egui::Window::new("Settings")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label(egui::RichText::new("File").color(p.gold).size(12.0));
                    ui.horizontal_wrapped(|ui| {
                        if ui.button("Open…").clicked() {
                            action = Some(Action::Open);
                        }
                        if ui.button("Find saves").clicked() {
                            action = Some(Action::FindSaves);
                        }
                        if ui.button("Reload").clicked() {
                            action = Some(Action::Reload);
                        }
                        if ui.button("New…").clicked() {
                            action = Some(Action::New);
                        }
                        if ui.button("Import…").clicked() {
                            action = Some(Action::Import);
                        }
                        if ui.button("Profile…").clicked() {
                            action = Some(Action::Profile);
                        }
                    });
                    ui.separator();
                    ui.label(egui::RichText::new("Platform").color(p.gold).size(12.0));
                    egui::ComboBox::from_id_salt("settings_platform")
                        .selected_text(self.save_platform.id().to_uppercase())
                        .width(100.0)
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
                    ui.separator();
                    ui.label(egui::RichText::new("Scale").color(p.gold).size(12.0));
                    ui.horizontal(|ui| {
                        if ui
                            .selectable_label(matches!(self.scale_mode, ScaleMode::Auto), "Auto")
                            .clicked()
                        {
                            self.set_scale_mode(ScaleMode::Auto);
                        }
                        for value in Self::UI_SCALES {
                            let text = format!("{}x", value as u32);
                            let selected = self.scale_mode == ScaleMode::Fixed(value as u32);
                            if ui.selectable_label(selected, text).clicked() {
                                self.set_scale_mode(ScaleMode::Fixed(value as u32));
                            }
                        }
                    });
                    ui.separator();
                    if ui.button("Toggle light/dark").clicked() {
                        action = Some(Action::ToggleTheme);
                    }
                });
            if !open {
                self.show_settings = false;
            }
        }

        if self.show_palette {
            let mut open = true;
            let mut run: Option<Cmd> = None;
            egui::Window::new("Action search")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .default_width(440.0)
                .show(&ctx, |ui| {
                    let resp = ui.add(
                        egui::TextEdit::singleline(&mut self.palette_query)
                            .hint_text("Type an action…  (Ctrl+K)"),
                    );
                    resp.request_focus();
                    let items = self.palette_items();
                    if !items.is_empty() {
                        self.palette_sel = self.palette_sel.min(items.len() - 1);
                    }
                    let (up, down, enter, esc) = ui.input(|i| {
                        (
                            i.key_pressed(egui::Key::ArrowUp),
                            i.key_pressed(egui::Key::ArrowDown),
                            i.key_pressed(egui::Key::Enter),
                            i.key_pressed(egui::Key::Escape),
                        )
                    });
                    if down && !items.is_empty() {
                        self.palette_sel = (self.palette_sel + 1) % items.len();
                    }
                    if up && !items.is_empty() {
                        self.palette_sel = self.palette_sel.saturating_sub(1);
                    }
                    ui.separator();
                    egui::ScrollArea::vertical()
                        .max_height(260.0)
                        .show(ui, |ui| {
                            for (i, c) in items.iter().enumerate() {
                                if ui
                                    .selectable_label(i == self.palette_sel, c.label())
                                    .clicked()
                                {
                                    run = Some(*c);
                                }
                            }
                        });
                    if enter {
                        if let Some(c) = items.get(self.palette_sel) {
                            run = Some(*c);
                        }
                    }
                    if esc {
                        self.show_palette = false;
                    }
                });
            if !open {
                self.show_palette = false;
            }
            if let Some(c) = run {
                self.show_palette = false;
                self.palette_query.clear();
                self.palette_sel = 0;
                self.run_cmd(c, &ctx);
            }
        }

        if self.show_paste {
            let mut open = true;
            let mut insert = false;
            egui::Window::new("Paste BL2(…)")
                .open(&mut open)
                .collapsible(false)
                .resizable(false)
                .show(&ctx, |ui| {
                    ui.label("Paste one or more BL2(…) codes:");
                    ui.add(
                        egui::TextEdit::multiline(&mut self.item_code)
                            .desired_width(420.0)
                            .desired_rows(4),
                    );
                    if ui.button("Insert code(s)").clicked() {
                        insert = true;
                    }
                });
            if !open {
                self.show_paste = false;
            }
            if insert {
                self.show_paste = false;
                self.insert_codes(self.is_bank());
            }
        }

        if let Some(action) = action {
            self.dispatch(action);
            ctx.request_repaint();
        }

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
                    ui.radio_value(
                        &mut new_blank,
                        true,
                        "Blank save (spec-driven, experimental)",
                    );
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
    }
}

impl eframe::App for LootsmithApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("lootsmith", self.settings_json());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.build(ui);
    }

    /// Clear the window to the theme background so no black ever shows at the
    /// edges or during a resize.
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        let p = match self.theme {
            Theme::Dark => crate::theme::DARK,
            Theme::Light => crate::theme::LIGHT,
        };
        egui::Rgba::from(p.bg).to_array()
    }
}
