//! Lootsmith GUI application state and UI.
//!
//! The UI lives in [`LootsmithApp::build`], independent of the native window, so
//! that both the `eframe` app and the `egui_kittest` harness run identical code.

use eframe::egui;
use lootsmith_core::savegame::Kind;
use lootsmith_core::{
    container, items, mission_status_name, player_mark_name, AssetGroup, AssetLibraryConfigs,
    AssetLibraryManager, Container, Customizations, Missions, PackedEntry, PartLists, Platform,
    Profile, SaveCandidate, SaveGame, TravelStations, VehicleCustomization, MISSION_STATUSES,
    PLAYER_MARKS, STAT_NAMES, VEHICLE_FAMILIES,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use crate::theme::{self, Tokens};
use egui_extras::{Column, TableBuilder};

pub use crate::theme::Theme;

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
    pub kind: String,
    pub id: i32,
    pub balance: String,
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
enum RawListOp {
    Add,
    Remove(usize),
}

impl RawField {
    /// An edited element or a pending list-level change.
    fn list_needs_apply(&self) -> bool {
        self.apply || self.elements.iter().any(|e| e.changed)
    }
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

/// Blend `accent` into `base` by `t` (0..1).
fn tint(base: egui::Color32, accent: egui::Color32, t: f32) -> egui::Color32 {
    let mix = |a: u8, b: u8| (a as f32 * (1.0 - t) + b as f32 * t).round() as u8;
    egui::Color32::from_rgb(
        mix(base.r(), accent.r()),
        mix(base.g(), accent.g()),
        mix(base.b(), accent.b()),
    )
}

/// One tree plate: a 3-slot-per-row tier pyramid, uniformly tinted with the
/// tree colour like the community calculator. A tier's single skill centres,
/// a pair spreads to the outer columns, three fill the row — no empty boxes.
fn tree_card(
    ui: &mut egui::Ui,
    tree: &crate::skills::SkillTree,
    grades: &[i32],
    grade_of: &impl Fn(&str) -> Option<usize>,
    textures: &std::collections::HashMap<String, Option<egui::TextureHandle>>,
    edit: &mut Option<(usize, i32)>,
) {
    const TILE: f32 = 62.0;
    const GAP: f32 = 6.0;
    const COLS: usize = 3;

    let accent = crate::skills::tree_accent(&tree.color);
    let tree_total: i32 = tree
        .tiers
        .iter()
        .flat_map(|t| &t.skills)
        .filter_map(|n| grade_of(&n.skill))
        .map(|i| grades[i].max(0))
        .sum();
    let grid_w = COLS as f32 * TILE + (COLS as f32 - 1.0) * GAP;

    egui::Frame::default()
        .fill(tint(Tokens::SURFACE, accent, 0.26))
        .stroke(egui::Stroke::new(1.0, Tokens::BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(grid_w);
                ui.spacing_mut().item_spacing = egui::vec2(GAP, GAP);

                for tier in &tree.tiers {
                    let unlocked = crate::skills::tier_unlocked(tree_total, tier.level);
                    let slots: [Option<&crate::skills::SkillNode>; COLS] =
                        match tier.skills.as_slice() {
                            [only] => [None, Some(only), None],
                            [a, b] => [Some(a), None, Some(b)],
                            [a, b, c, ..] => [Some(a), Some(b), Some(c)],
                            _ => [None, None, None],
                        };
                    ui.horizontal(|ui| {
                        for slot in slots {
                            match slot {
                                Some(node) => {
                                    let idx = grade_of(&node.skill);
                                    let grade = idx.map(|i| grades[i]).unwrap_or(0);
                                    let max = node.max.max(1);
                                    let tex = textures.get(&node.icon).cloned().flatten();
                                    let response = skill_tile(
                                        ui,
                                        &TileState {
                                            node,
                                            texture: tex.as_ref(),
                                            grade,
                                            max,
                                            unlocked,
                                            accent,
                                            size: TILE,
                                        },
                                    );
                                    if unlocked {
                                        if let Some(i) = idx {
                                            if response.clicked() && grade < max {
                                                *edit = Some((i, grade + 1));
                                            } else if response.secondary_clicked() && grade > 0 {
                                                *edit = Some((i, grade - 1));
                                            }
                                        }
                                    }
                                }
                                // Invisible spacer so the pyramid stays centred.
                                None => {
                                    ui.allocate_exact_size(
                                        egui::vec2(TILE, TILE),
                                        egui::Sense::hover(),
                                    );
                                }
                            }
                        }
                    });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(tree.name.to_uppercase())
                            .strong()
                            .size(13.0)
                            .color(accent),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(tree_total.to_string())
                                .strong()
                                .size(13.0)
                                .color(accent),
                        );
                    });
                });
            });
        });
}

/// One skill tile: icon-centred with a metal frame and a rank badge, matching the
/// community calculator. Partial/full/locked/hover states are all visualised.
struct TileState<'a> {
    node: &'a crate::skills::SkillNode,
    texture: Option<&'a egui::TextureHandle>,
    grade: i32,
    max: i32,
    unlocked: bool,
    accent: egui::Color32,
    size: f32,
}

fn skill_tile(ui: &mut egui::Ui, t: &TileState) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(t.size, t.size), egui::Sense::click());
    let hovered = response.hovered();
    let painter = ui.painter_at(rect);
    let full = t.max > 1 && t.grade >= t.max;
    let unlocked = t.unlocked;
    let accent = t.accent;

    // Soft glow when maxed.
    if full && unlocked {
        painter.rect_filled(
            rect.expand(2.0),
            egui::CornerRadius::same(10),
            accent.gamma_multiply(0.30),
        );
    }
    // Metal edge + inner plate (brightens on hover).
    painter.rect_filled(rect, egui::CornerRadius::same(8), Tokens::BG);
    let plate = if !unlocked {
        Tokens::BG
    } else if hovered {
        tint(Tokens::SURFACE, accent, 0.06)
    } else {
        Tokens::SURFACE_ALT
    };
    painter.rect_filled(rect.shrink(2.0), egui::CornerRadius::same(7), plate);

    // Border: accent when invested, dim when locked.
    let (width, color) = if !unlocked {
        (1.0, Tokens::SURFACE_ALT)
    } else if t.grade > 0 {
        (2.0, accent)
    } else {
        (1.0, Tokens::BORDER)
    };
    painter.rect_stroke(
        rect.shrink(width / 2.0),
        egui::CornerRadius::same(8),
        egui::Stroke::new(width, color),
        egui::StrokeKind::Inside,
    );

    // Icon.
    if let Some(texture) = t.texture {
        let icon = egui::Rect::from_center_size(
            rect.center() - egui::vec2(0.0, 3.0),
            egui::vec2(t.size - 18.0, t.size - 18.0),
        );
        let icon_tint = if unlocked {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_gray(88)
        };
        painter.image(
            texture.id(),
            icon,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            icon_tint,
        );
    }
    // Locked marker.
    if !unlocked {
        painter.circle_filled(
            rect.left_top() + egui::vec2(10.0, 10.0),
            3.0,
            Tokens::TEXT_FAINT,
        );
    }

    // Rank badge, bottom-right.
    let badge_color = if !unlocked {
        Tokens::TEXT_FAINT
    } else if full {
        accent
    } else {
        Tokens::TEXT
    };
    let galley = painter.layout_no_wrap(
        format!("{}/{}", t.grade, t.max),
        egui::FontId::proportional(11.5),
        badge_color,
    );
    let pad = egui::vec2(4.0, 2.0);
    let badge_size = galley.size() + pad * 2.0;
    let badge = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - badge_size.x - 3.0,
            rect.bottom() - badge_size.y - 3.0,
        ),
        badge_size,
    );
    painter.rect_filled(
        badge,
        egui::CornerRadius::same(3),
        egui::Color32::from_black_alpha(210),
    );
    painter.galley(badge.min + pad, galley, badge_color);

    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, unlocked, &t.node.skill)
    });
    if t.node.desc.is_empty() {
        response
    } else {
        response.on_hover_text(&t.node.desc)
    }
}

/// Display label for a currency slot.
fn currency_name(index: usize) -> String {
    match index {
        0 => "Money".to_string(),
        1 => "Eridium".to_string(),
        2 => "Seraph crystals".to_string(),
        3 => "Torgue tokens".to_string(),
        n => format!("Slot {n}"),
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
const TABS: [(Tab, &str); 10] = [
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
];

/// A vertical rule between toolbar groups.
fn toolbar_sep(ui: &mut egui::Ui) {
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover());
    ui.painter()
        .vline(rect.center().x, rect.y_range(), egui::Stroke::new(1.0, Tokens::BORDER));
    ui.add_space(4.0);
}

/// A filled status dot.
fn status_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

/// A grouped surface card of a fixed width.
fn card(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    ui.allocate_ui_with_layout(
        egui::vec2(width, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::Frame::group(ui.style())
                .fill(Tokens::SURFACE)
                .stroke(egui::Stroke::new(1.0, Tokens::BORDER))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(width - 28.0);
                    add(ui);
                });
        },
    );
}

/// The empty state shown before a save is open.
fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(64.0);
        ui.label(
            egui::RichText::new("Lootsmith")
                .size(26.0)
                .strong()
                .color(Tokens::ACCENT),
        );
        ui.add_space(6.0);
        theme::muted(ui, "A cross-platform Borderlands 2 save editor.");
        ui.add_space(14.0);
        theme::faint(ui, "Open a save (or use Find saves), then pick your game-data folder");
        theme::faint(ui, "so item parts, missions and travel stations resolve to names.");
    });
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
    /// Balance → allowed parts, for balance-constrained part pickers.
    pub part_lists: Option<Arc<PartLists>>,
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
            let balance = resolve_balance(manager, &decoded);
            Row {
                container: entry.container,
                occurrence: entry.occurrence,
                text: format!(
                    "{kind} #{display} id={} balance={balance}{meta}{mark}",
                    decoded.unique_id,
                ),
                kind: kind.to_string(),
                id: decoded.unique_id,
                balance,
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
            kind: "?".to_string(),
            id: 0,
            balance: format!("decode error: {e}"),
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
            part_lists: None,
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
        let status = format!(
            "loaded {} ({} bytes, crc32 {})",
            self.path,
            data.len(),
            if save.crc32_valid() { "ok" } else { "MISMATCH" }
        );
        self.accept_save(save_game, status);
    }

    /// Fill every view (form, rows, skills, …) from a parsed save.
    fn accept_save(&mut self, save_game: SaveGame, status: String) {
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
        self.part_lists = self.load_part_lists();
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

    fn new_class_choice(&self) -> &'static str {
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
    fn suggested_profile(&self) -> Option<String> {
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
    fn skill_texture(&mut self, ctx: &egui::Context, filename: &str) -> Option<egui::TextureHandle> {
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
                            if theme::tab(ui, self.tab == tab, label) {
                                self.tab = tab;
                            }
                        }
                    });
                });
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

    fn ui_general(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        card(ui, 560.0, |ui| {
        theme::section(ui, "General");
        egui::Grid::new("general")
            .num_columns(2)
            .spacing([16.0, 6.0])
            .show(ui, |ui| {
                let mut changed = false;
                ui.label("Level");
                ui.horizontal(|ui| {
                    changed |= ui
                        .add(
                            egui::DragValue::new(&mut self.form.level)
                                .range(1..=80)
                                .speed(0.2),
                        )
                        .changed();
                    if ui
                        .button("Max")
                        .on_hover_text("Vanilla cap: level 72")
                        .clicked()
                    {
                        self.form.level = 72;
                        changed = true;
                    }
                    if ui
                        .button("OP max")
                        .on_hover_text("UCP/Overpower cap: level 80")
                        .clicked()
                    {
                        self.form.level = 80;
                        changed = true;
                    }
                });
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
        });
    }

    fn ui_character(&mut self, ui: &mut egui::Ui) {
        ui.add_space(10.0);
        let customizations = self.customizations.clone();
        let usage = lootsmith_core::usage_for_player_class(&self.read_info.player_class);
        let mut changed = false;
        card(ui, 620.0, |ui| {
        theme::section(ui, "Character");
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
        });
        if changed {
            self.dirty = true;
        }
        if customizations.is_none() {
            ui.add_space(6.0);
            theme::muted(ui, "Load the game-data folder to pick heads and skins by name.");
        }
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
                            ui.label(currency_name(i));
                            changed |= ui.add(egui::DragValue::new(value)).changed();
                            ui.end_row();
                        }
                    });
            });
        if changed {
            self.dirty = true;
        }
    }

    fn ui_skills(&mut self, ui: &mut egui::Ui) {
        let Some(class) = crate::skills::for_player_class(&self.read_info.player_class) else {
            theme::muted(ui, "Unknown player class — the skill tree is unavailable.");
            return;
        };

        // Snapshot grades and index the save's skills by normalized leaf name.
        let grades: Vec<i32> = self.skills.iter().map(|r| r.grade).collect();
        let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, row) in self.skills.iter().enumerate() {
            let leaf = row.name.rsplit('.').next().unwrap_or(&row.name);
            index.insert(crate::skills::norm(leaf), i);
        }
        let grade_of = |name: &str| index.get(&crate::skills::norm(name)).copied();

        let spent: i32 = grades.iter().map(|g| (*g).max(0)).sum();
        let available = self.form.general_skill_points;
        let portrait = self.skill_texture(ui.ctx(), &class.portrait);

        // Decode/cache every icon this class needs, once.
        let mut textures: std::collections::HashMap<String, Option<egui::TextureHandle>> =
            std::collections::HashMap::new();
        for tree in &class.trees {
            for tier in &tree.tiers {
                for node in &tier.skills {
                    if !node.icon.is_empty() && !textures.contains_key(&node.icon) {
                        let handle = self.skill_texture(ui.ctx(), &node.icon);
                        textures.insert(node.icon.clone(), handle);
                    }
                }
            }
        }

        // Header.
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label(
                    egui::RichText::new("BORDERLANDS 2 SKILL CALCULATOR")
                        .strong()
                        .size(18.0)
                        .color(Tokens::ACCENT),
                );
                ui.add_space(2.0);
                ui.label(
                    egui::RichText::new(&class.name)
                        .strong()
                        .size(28.0)
                        .color(Tokens::ACCENT),
                );
                ui.label(
                    egui::RichText::new(format!(
                        "LEVEL {} {}",
                        self.form.level,
                        class.class_name.to_uppercase()
                    ))
                    .size(14.0),
                );
                ui.add_space(6.0);
                ui.label(
                    egui::RichText::new(format!("Spent {spent}     Available {available}"))
                        .color(Tokens::TEXT_DIM)
                        .size(12.0),
                );
                theme::faint(ui, "Left-click adds a rank, right-click removes.");
                theme::faint(ui, "A tier unlocks every 5 points invested.");
            });
        });
        ui.add_space(6.0);

        let mut edit: Option<(usize, i32)> = None;

        egui::ScrollArea::vertical()
            .id_salt("skill_trees")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    if let Some(texture) = &portrait {
                        ui.add(
                            egui::Image::new(texture)
                                .max_height(430.0)
                                .max_width(300.0),
                        );
                        ui.add_space(16.0);
                    }
                    for tree in class.trees.iter() {
                        tree_card(ui, tree, &grades, &grade_of, &textures, &mut edit);
                        ui.add_space(20.0);
                    }
                });

                // Save skills not matched to a tree node (e.g. the action skill).
                let matched: std::collections::HashSet<usize> = class
                    .trees
                    .iter()
                    .flat_map(|t| &t.tiers)
                    .flat_map(|t| &t.skills)
                    .filter_map(|n| grade_of(&n.skill))
                    .collect();
                let extras: Vec<usize> = (0..grades.len())
                    .filter(|i| !matched.contains(i))
                    .collect();
                if !extras.is_empty() {
                    ui.add_space(10.0);
                    theme::section(ui, "Other skills");
                    ui.horizontal_wrapped(|ui| {
                        for i in extras {
                            let leaf = self.skills[i]
                                .name
                                .rsplit('.')
                                .next()
                                .unwrap_or("?")
                                .to_string();
                            let display = if leaf.starts_with("Skill_") {
                                "Action Skill".to_string()
                            } else {
                                leaf
                            };
                            let grade = grades[i];
                            let label =
                                egui::RichText::new(format!("{display}   {grade}/1")).size(12.0);
                            let label = if grade > 0 {
                                label.color(Tokens::ACCENT)
                            } else {
                                label
                            };
                            if ui
                                .button(label)
                                .on_hover_text(&self.skills[i].name)
                                .clicked()
                                && grade < 1
                            {
                                edit = Some((i, grade + 1));
                            }
                        }
                    });
                }
            });

        // Apply a click: set the grade and keep the point budget consistent.
        if let Some((i, new_grade)) = edit {
            let old = self.skills[i].grade;
            if new_grade != old {
                self.skills[i].grade = new_grade;
                if new_grade > old && self.form.general_skill_points > 0 {
                    self.form.general_skill_points -= 1;
                } else if new_grade < old {
                    self.form.general_skill_points += 1;
                }
                self.apply_skills();
            }
        }
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
    }

    fn ui_raw(&mut self, ui: &mut egui::Ui) {
        ui.add_space(4.0);
        ui.heading("Raw fields");
        ui.add_space(2.0);
        ui.label(
            "Edit scalar fields and int/string list elements directly. Nested \
             messages and byte blobs are read-only here (use the CLI json dump).",
        );
        ui.add_space(4.0);
        let mut changed = false;
        let mut list_ops: Vec<(u32, RawListOp)> = Vec::new();
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
                                RawEdit::None if !field.elements.is_empty() => {
                                    let number = field.number;
                                    let ops = &mut list_ops;
                                    ui.vertical(|ui| {
                                        for i in 0..field.elements.len() {
                                            ui.horizontal(|ui| {
                                                match &mut field.elements[i].edit {
                                                    RawEdit::Int(v) => {
                                                        if ui
                                                            .add(egui::DragValue::new(v))
                                                            .changed()
                                                        {
                                                            field.elements[i].changed = true;
                                                        }
                                                    }
                                                    RawEdit::Str(v) => {
                                                        if ui
                                                            .add(
                                                                egui::TextEdit::singleline(v)
                                                                    .desired_width(260.0),
                                                            )
                                                            .changed()
                                                        {
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
                                    ui.label(&field.preview);
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

    fn ui_container(&mut self, ui: &mut egui::Ui, bank: bool) {
        let mut item_action: Option<ItemAction> = None;
        let mut clicked: Option<(Container, usize)> = None;
        let mut apply = false;
        let mut move_to: Option<Container> = None;
        let mut remove = false;

        // ---- action bar ----
        egui::Panel::top("bag_actions")
            .frame(
                egui::Frame::default()
                    .fill(Tokens::BG)
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
                    ui.label(egui::RichText::new("Item code").color(Tokens::TEXT_DIM).size(11.5));
                    ui.add_sized(
                        [520.0, 24.0],
                        egui::TextEdit::singleline(&mut self.item_code)
                            .hint_text("paste BL2(...) codes here"),
                    );
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
                        .fill(Tokens::SURFACE)
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
                        ui.label(egui::RichText::new(title).strong().size(14.0));
                        ui.label(theme::mono(format!("id={}", edit.decoded.unique_id)));
                    });
                    ui.add_space(6.0);

                    if edit.container == Container::BackpackItem
                        || edit.container == Container::BackpackWeapon
                    {
                        ui.horizontal(|ui| {
                            if edit.container == Container::BackpackItem {
                                ui.label("Quantity");
                                if ui.add(egui::DragValue::new(&mut edit.quantity)).changed() {
                                    edit.dirty = true;
                                }
                                if ui.checkbox(&mut edit.equipped, "Equipped").changed() {
                                    edit.dirty = true;
                                }
                            } else {
                                ui.label("Quick slot");
                                if ui.add(egui::DragValue::new(&mut edit.quick_slot)).changed() {
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
                    }

                    let is_weapon = edit.decoded.is_weapon;
                    let set = edit.decoded.asset_library_set_id;

                    egui::Panel::bottom("inspector_actions")
                        .frame(
                            egui::Frame::default()
                                .fill(Tokens::SURFACE)
                                .inner_margin(egui::Margin::symmetric(0, 8)),
                        )
                        .show(ui, |ui| {
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
                        });

                    theme::section(ui, if is_weapon { "Weapon parts" } else { "Item parts" });
                    egui::ScrollArea::vertical()
                        .id_salt("inspector_parts")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            egui::Grid::new("parts_grid")
                                .num_columns(2)
                                .spacing([10.0, 6.0])
                                .show(ui, |ui| {
                                    for (slot, label) in slots_for(is_weapon) {
                                        ui.label(
                                            egui::RichText::new(*label)
                                                .color(Tokens::TEXT_DIM)
                                                .size(12.0),
                                        );
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
                                            .selected_text(current)
                                            .width(270.0)
                                            .show_ui(ui, |ui| {
                                                if ui.selectable_label(false, "None").clicked() {
                                                    let _ =
                                                        edit.decoded.set_part(manager, slot, "None");
                                                    edit.dirty = true;
                                                }
                                                for choice in &choices {
                                                        if ui
                                                            .selectable_label(false, choice)
                                                            .clicked()
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
            .frame(egui::Frame::default().fill(Tokens::BG))
            .show(ui, |ui| {
                if visible.is_empty() {
                    ui.add_space(12.0);
                    theme::muted(ui, if bank { "(bank is empty)" } else { "(backpack is empty)" });
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
                        for name in ["#", "Type", "ID", "Qty", "Eq", "Quick", "Mark", "Balance / Name"] {
                            header.col(|ui| {
                                ui.label(
                                    egui::RichText::new(name)
                                        .strong()
                                        .size(11.5)
                                        .color(Tokens::TEXT_DIM),
                                );
                            });
                        }
                    })
                    .body(|mut body| {
                    for (i, row) in visible.iter().enumerate() {
                        let is_selected = selected == Some((row.container, row.occurrence));
                        body.row(24.0, |mut tr| {
                            tr.set_selected(is_selected);
                            tr.col(|ui| {
                                ui.label((i + 1).to_string());
                            });
                            tr.col(|ui| {
                                ui.label(
                                    egui::RichText::new(&row.kind)
                                        .color(if row.is_weapon {
                                            Tokens::ACCENT
                                        } else {
                                            Tokens::TEXT_DIM
                                        })
                                        .size(12.0),
                                );
                            });
                            tr.col(|ui| {
                                if ui
                                    .selectable_label(is_selected, theme::mono(format!("id={}", row.id)))
                                    .clicked()
                                {
                                    clicked = Some((row.container, row.occurrence));
                                }
                            });
                            tr.col(|ui| {
                                if row.is_weapon || row.container == Container::Bank {
                                    ui.label("");
                                } else {
                                    ui.label(row.quantity.to_string());
                                }
                            });
                            tr.col(|ui| {
                                if row.is_weapon || row.container == Container::Bank {
                                    ui.label("");
                                } else {
                                    ui.label(if row.equipped { "✓" } else { "" });
                                }
                            });
                            tr.col(|ui| {
                                if row.is_weapon {
                                    ui.label(row.quick_slot.to_string());
                                } else {
                                    ui.label("");
                                }
                            });
                            tr.col(|ui| {
                                ui.label(player_mark_name(row.mark));
                            });
                            tr.col(|ui| {
                                let text = if row.empty {
                                    egui::RichText::new(&row.balance)
                                        .color(Tokens::TEXT_FAINT)
                                        .italics()
                                } else {
                                    egui::RichText::new(&row.balance).color(Tokens::TEXT_DIM)
                                };
                                ui.label(text).on_hover_text(&row.balance);
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

impl eframe::App for LootsmithApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("lootsmith", self.settings_json());
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.build(ui);
    }
}
