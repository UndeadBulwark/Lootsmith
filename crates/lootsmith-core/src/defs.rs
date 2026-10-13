//! Game definitions that drive the item/weapon part pickers.
//!
//! Ports the part-list resolution from Gibbed's
//! `GameInfo/WeaponBalanceDefinition.cs`, `ItemBalanceDefinition.cs`,
//! `Loaders/WeaponTypeDefinitionLoader.cs`, `Loaders/ItemDefinitionLoader.cs`
//! and the raw `WeaponBalancePartCollection` / `ItemBalancePartCollection`
//! shapes.
//!
//! The legal part set for a balance is its `base` chain (oldest first) merged
//! with each balance's `mode`
//! (`GameInfo/PartReplacementMode.cs`):
//!
//! * `Additive`  — append the listed parts to the accumulated list;
//! * `Selective` — replace the slot's list when the balance lists it (a
//!   missing slot keeps the accumulated list);
//! * `Complete`  — clear the slot, then add the listed parts. A balance that
//!   does not list the slot still clears it, matching
//!   `AddPartList(null, Complete, ...)`.
//!
//! A weapon type / item definition additionally owns its `titles`/`prefixes`
//! and, for items, the base part lists that a balance may override.

use std::collections::{BTreeMap, HashMap};
use std::path::Path;

use crate::error::{Error, Result};
use crate::gameinfo::AssetLibraryManager;
use crate::items::PackedData;

/// Resolve a named slot's `package.asset` path through the manager, or `None`
/// when the slot is unset or cannot be resolved. Shared by the CLI and GUI so
/// they agree on an item's `type`/`balance` before asking `GameDefs`.
pub fn slot_asset_path(
    manager: &AssetLibraryManager,
    decoded: &PackedData,
    slot: &str,
) -> Option<String> {
    let (reference, group) = decoded.part(slot)?;
    if reference.is_none() {
        return None;
    }
    manager
        .lookup_asset(
            group,
            decoded.asset_library_set_id,
            reference.asset_index,
            reference.sublibrary_index,
            reference.use_set_id,
        )
        .ok()
}

/// The part slots a weapon balance can list, in editor order.
pub const WEAPON_PART_SLOTS: [&str; 9] = [
    "body",
    "grip",
    "barrel",
    "sight",
    "stock",
    "elemental",
    "accessory1",
    "accessory2",
    "material",
];

/// The part slots an item balance can list, in editor order.
pub const ITEM_PART_SLOTS: [&str; 9] = [
    "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta", "material",
];

/// `PartReplacementMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Mode {
    #[default]
    Unknown,
    Additive,
    Selective,
    Complete,
}

impl Mode {
    fn parse(raw: &str) -> Mode {
        match raw.to_ascii_lowercase().as_str() {
            "additive" => Mode::Additive,
            "selective" => Mode::Selective,
            "complete" => Mode::Complete,
            _ => Mode::Unknown,
        }
    }
}

/// A resolved per-slot part list (always present, possibly empty).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartCollection {
    slots: BTreeMap<String, Vec<String>>,
}

impl PartCollection {
    /// The list for `slot`; empty when the slot is unset.
    pub fn get(&self, slot: &str) -> &[String] {
        self.slots.get(slot).map(Vec::as_slice).unwrap_or(&[])
    }

    /// Whether the slot is explicitly covered (lists at least one part).
    pub fn has(&self, slot: &str) -> bool {
        self.slots.get(slot).is_some_and(|v| !v.is_empty())
    }

    /// Seed a slot with a base list (used by the item-definition lists).
    fn set(&mut self, slot: &str, list: Vec<String>) {
        self.slots.insert(slot.to_string(), list);
    }

    /// Merge one balance's raw part collection into `self`, over the given
    /// slot universe. Mirrors `WeaponBalanceDefinition.AddPartList`.
    fn merge(&mut self, raw: &RawPartCollection, slots: &[&str]) {
        for &slot in slots {
            let source = raw.list(slot);
            let dest = self.slots.entry(slot.to_string()).or_default();
            match raw.mode {
                Mode::Additive => {
                    if let Some(list) = source {
                        dest.extend(list.iter().cloned());
                    }
                }
                Mode::Selective => {
                    if let Some(list) = source {
                        *dest = list.clone();
                    }
                }
                Mode::Complete => {
                    dest.clear();
                    if let Some(list) = source {
                        dest.extend(list.iter().cloned());
                    }
                }
                Mode::Unknown => {}
            }
        }
    }
}

/// One `Weapon/Item Balance Part Lists` record.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RawPartCollection {
    pub mode: Mode,
    slots: HashMap<String, Vec<String>>,
}

impl RawPartCollection {
    /// The list for a slot, or `None` when the dump omits it (distinct from an
    /// empty list for `Selective`).
    fn list(&self, slot: &str) -> Option<&Vec<String>> {
        self.slots.get(slot)
    }

    fn insert(&mut self, slot: &str, list: Vec<String>) {
        self.slots.insert(slot.to_string(), list);
    }
}

/// A weapon type (`Weapon Types.json` + `Weapon Part Lists.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WeaponTypeDef {
    pub resource_path: String,
    pub kind: String,
    pub name: String,
    pub titles: Vec<String>,
    pub prefixes: Vec<String>,
    parts: BTreeMap<String, Vec<String>>,
}

impl WeaponTypeDef {
    pub fn part_list(&self, slot: &str) -> &[String] {
        self.parts.get(slot).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// An item definition (`Items.json` + `Item Part Lists.json`).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemDef {
    pub resource_path: String,
    pub kind: String,
    pub name: String,
    pub titles: Vec<String>,
    pub prefixes: Vec<String>,
    parts: BTreeMap<String, Vec<String>>,
}

impl ItemDef {
    pub fn is_class_mod(&self) -> bool {
        self.kind.to_ascii_lowercase().contains("classmod")
    }

    pub fn part_list(&self, slot: &str) -> &[String] {
        self.parts.get(slot).map(Vec::as_slice).unwrap_or(&[])
    }
}

/// A weapon balance (`Weapon Balance.json`), linked to its `base` and parts.
#[derive(Clone, Debug, Default)]
pub struct WeaponBalanceDef {
    pub resource_path: String,
    pub base: Option<String>,
    pub weapon_type: Option<String>,
    pub manufacturers: Vec<String>,
    pub parts: Option<RawPartCollection>,
}

/// An item balance (`Item Balance.json`).
#[derive(Clone, Debug, Default)]
pub struct ItemBalanceDef {
    pub resource_path: String,
    pub base: Option<String>,
    pub item: Option<String>,
    pub items: Vec<String>,
    pub manufacturers: Vec<String>,
    pub parts: Option<RawPartCollection>,
}

/// Every definition table the part pickers need.
#[derive(Clone, Debug, Default)]
pub struct GameDefs {
    weapon_types: HashMap<String, WeaponTypeDef>,
    item_defs: HashMap<String, ItemDef>,
    weapon_balances: HashMap<String, WeaponBalanceDef>,
    item_balances: HashMap<String, ItemBalanceDef>,
}

impl GameDefs {
    /// Load every table from a game-data directory (Gibbed's space-named
    /// dumps). Missing balance tables are tolerated; the call fails only when
    /// the weapon-type / item-definition tables are absent.
    pub fn from_dir(dir: &Path) -> Result<GameDefs> {
        let read = |name: &str| -> Option<Result<String>> {
            let path = dir.join(name);
            path.is_file()
                .then(|| std::fs::read_to_string(&path).map_err(|e| Error::Io(e.to_string())))
        };
        let required = |name: &str| -> Result<String> {
            let path = dir.join(name);
            std::fs::read_to_string(&path).map_err(|e| Error::Io(e.to_string()))
        };
        let opt = |name: &str| -> Result<Option<String>> {
            match read(name) {
                Some(r) => r.map(Some),
                None => Ok(None),
            }
        };
        Self::from_parts(
            &required("Weapon Types.json")?,
            &required("Weapon Part Lists.json")?,
            &required("Items.json")?,
            &required("Item Part Lists.json")?,
            opt("Weapon Balance.json")?.as_deref(),
            opt("Weapon Balance Part Lists.json")?.as_deref(),
            opt("Item Balance.json")?.as_deref(),
            opt("Item Balance Part Lists.json")?.as_deref(),
        )
    }

    /// Build from already-read table texts. The four required tables must
    /// parse; the four balance tables are optional.
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        weapon_types: &str,
        weapon_part_lists: &str,
        items: &str,
        item_part_lists: &str,
        weapon_balances: Option<&str>,
        weapon_balance_part_lists: Option<&str>,
        item_balances: Option<&str>,
        item_balance_part_lists: Option<&str>,
    ) -> Result<GameDefs> {
        let wpl: HashMap<String, Vec<String>> = parse(weapon_part_lists, "Weapon Part Lists")?;
        let ipl: HashMap<String, Vec<String>> = parse(item_part_lists, "Item Part Lists")?;

        let mut weapon_raw: HashMap<String, RawPartCollection> = HashMap::new();
        if let Some(text) = weapon_balance_part_lists {
            let raw: HashMap<String, RawPartCollectionJson> =
                parse(text, "Weapon Balance Part Lists")?;
            for (key, record) in raw {
                let mut collection = RawPartCollection {
                    mode: Mode::parse(record.mode.as_deref().unwrap_or("")),
                    ..Default::default()
                };
                for slot in WEAPON_PART_SLOTS {
                    if let Some(list) = record.slot(slot) {
                        collection.insert(slot, list);
                    }
                }
                weapon_raw.insert(key, collection);
            }
        }
        let mut item_raw: HashMap<String, RawPartCollection> = HashMap::new();
        if let Some(text) = item_balance_part_lists {
            let raw: HashMap<String, RawPartCollectionJson> =
                parse(text, "Item Balance Part Lists")?;
            for (key, record) in raw {
                let mut collection = RawPartCollection {
                    mode: Mode::parse(record.mode.as_deref().unwrap_or("")),
                    ..Default::default()
                };
                for slot in ITEM_PART_SLOTS {
                    if let Some(list) = record.slot(slot) {
                        collection.insert(slot, list);
                    }
                }
                item_raw.insert(key, collection);
            }
        }

        let mut defs = GameDefs {
            weapon_types: load_weapon_types(weapon_types, &wpl)?,
            item_defs: load_item_defs(items, &ipl)?,
            weapon_balances: HashMap::new(),
            item_balances: HashMap::new(),
        };

        if let Some(text) = weapon_balances {
            let raw: HashMap<String, RawBalanceJson> = parse(text, "Weapon Balance")?;
            for (key, record) in raw {
                defs.weapon_balances.insert(
                    key.clone(),
                    WeaponBalanceDef {
                        resource_path: key,
                        base: record.base,
                        weapon_type: record.weapon_type,
                        manufacturers: clean_or_default(record.manufacturers),
                        parts: record
                            .parts
                            .as_ref()
                            .and_then(|p| weapon_raw.get(p).cloned()),
                    },
                );
            }
        }
        if let Some(text) = item_balances {
            let raw: HashMap<String, RawItemBalanceJson> = parse(text, "Item Balance")?;
            for (key, record) in raw {
                defs.item_balances.insert(
                    key.clone(),
                    ItemBalanceDef {
                        resource_path: key,
                        base: record.base,
                        item: record.item,
                        items: clean_or_default(record.items),
                        manufacturers: clean_or_default(record.manufacturers),
                        parts: record.parts.as_ref().and_then(|p| item_raw.get(p).cloned()),
                    },
                );
            }
        }
        Ok(defs)
    }

    /// True when no table carried any entry (a user folder that yielded
    /// nothing must not shadow the bundled tables).
    pub fn is_empty(&self) -> bool {
        self.weapon_types.is_empty()
            && self.item_defs.is_empty()
            && self.weapon_balances.is_empty()
            && self.item_balances.is_empty()
    }

    pub fn weapon_type(&self, path: &str) -> Option<&WeaponTypeDef> {
        self.weapon_types.get(path)
    }

    pub fn item_def(&self, path: &str) -> Option<&ItemDef> {
        self.item_defs.get(path)
    }

    pub fn weapon_balance(&self, path: &str) -> Option<&WeaponBalanceDef> {
        self.weapon_balances.get(path)
    }

    pub fn item_balance(&self, path: &str) -> Option<&ItemBalanceDef> {
        self.item_balances.get(path)
    }

    /// The weapon balance `base` chain, oldest first (guards cycles).
    fn weapon_chain(&self, balance: &str) -> Vec<&WeaponBalanceDef> {
        let mut chain = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut current = self.weapon_balances.get(balance);
        while let Some(def) = current {
            if !seen.insert(def.resource_path.as_str()) {
                break;
            }
            chain.push(def);
            current = def
                .base
                .as_deref()
                .and_then(|b| self.weapon_balances.get(b));
        }
        chain.reverse();
        chain
    }

    fn item_chain(&self, balance: &str) -> Vec<&ItemBalanceDef> {
        let mut chain = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let mut current = self.item_balances.get(balance);
        while let Some(def) = current {
            if !seen.insert(def.resource_path.as_str()) {
                break;
            }
            chain.push(def);
            current = def.base.as_deref().and_then(|b| self.item_balances.get(b));
        }
        chain.reverse();
        chain
    }

    /// Whether `balance` can be used with `weapon_type` (Gibbed
    /// `WeaponBalanceDefinition.IsSuitableFor`).
    pub fn weapon_is_suitable(&self, balance: &str, weapon_type: &str) -> bool {
        for def in self.weapon_chain(balance) {
            if let Some(wt) = &def.weapon_type {
                return wt == weapon_type;
            }
        }
        false
    }

    /// The merged part collection for a weapon balance (`Create`).
    pub fn weapon_parts(&self, balance: &str, weapon_type: &str) -> PartCollection {
        let mut result = PartCollection::default();
        if !self.weapon_is_suitable(balance, weapon_type) {
            return result;
        }
        for def in self.weapon_chain(balance) {
            if let Some(parts) = &def.parts {
                result.merge(parts, &WEAPON_PART_SLOTS);
            }
        }
        result
    }

    /// The manufacturer choices for a weapon balance (last non-empty in the
    /// chain).
    pub fn weapon_manufacturers(&self, balance: &str) -> Vec<String> {
        let mut out = Vec::new();
        for def in self.weapon_chain(balance) {
            if !def.manufacturers.is_empty() {
                out = def.manufacturers.clone();
            }
        }
        out
    }

    /// Choices for one weapon part slot, or `None` when the slot is not a part
    /// list (`type`/`balance`). Mirrors `BaseWeaponViewModel.BuildPartAssets`.
    pub fn weapon_slot_choices(
        &self,
        weapon_type: &str,
        balance: &str,
        slot: &str,
    ) -> Option<Vec<String>> {
        let ty = self.weapon_types.get(weapon_type)?;
        match slot {
            "prefix" => Some(ty.prefixes.clone()),
            "title" => Some(ty.titles.clone()),
            "manufacturer" | "mfr" => Some(self.weapon_manufacturers(balance)),
            _ if WEAPON_PART_SLOTS.contains(&slot) => {
                Some(self.weapon_parts(balance, weapon_type).get(slot).to_vec())
            }
            _ => None,
        }
    }

    /// The item `base` chain merged with the item definition's own part lists
    /// (`ItemBalanceDefinition.Create`). `want_balance_parts` is true for class
    /// mods or when the balance's item is exactly this item.
    pub fn item_parts(&self, item: &str, balance: &str) -> PartCollection {
        let mut result = PartCollection::default();
        if let Some(def) = self.item_defs.get(item) {
            for &slot in &ITEM_PART_SLOTS {
                result.set(slot, def.part_list(slot).to_vec());
            }
        }
        let chain = self.item_chain(balance);
        let balance_item = chain.iter().rev().find_map(|d| d.item.clone());
        let want_balance_parts = self.item_defs.get(item).is_some_and(ItemDef::is_class_mod)
            || balance_item.as_deref() == Some(item);
        if want_balance_parts {
            for def in chain {
                if let Some(parts) = &def.parts {
                    result.merge(parts, &ITEM_PART_SLOTS);
                }
            }
        }
        result
    }

    pub fn item_manufacturers(&self, balance: &str) -> Vec<String> {
        let mut out = Vec::new();
        for def in self.item_chain(balance) {
            if !def.manufacturers.is_empty() {
                out = def.manufacturers.clone();
            }
        }
        out
    }

    /// Whether `balance` can be used with `item` (`IsSuitableFor`).
    pub fn item_is_suitable(&self, balance: &str, item: &str) -> bool {
        for def in self.item_chain(balance) {
            if def.item.is_some() || !def.items.is_empty() {
                return def.item.as_deref() == Some(item) || def.items.iter().any(|i| i == item);
            }
        }
        false
    }

    /// Whether `name` is one of the resolved candidates for `slot` on a
    /// `(type, balance)` pair. `type_path`/unknown data cannot be judged and
    /// are treated as legal, so callers only warn on a definite mismatch
    /// ("None" is always allowed).
    pub fn is_legal_part(
        &self,
        is_weapon: bool,
        type_path: Option<&str>,
        balance: &str,
        slot: &str,
        name: &str,
    ) -> bool {
        if name == "None" {
            return true;
        }
        let Some(ty) = type_path else {
            return true;
        };
        let choices = if is_weapon {
            self.weapon_slot_choices(ty, balance, slot)
        } else {
            self.item_slot_choices(ty, balance, slot)
        };
        match choices {
            // Not a part list (`type`/`balance`), or unknown: cannot judge.
            None => true,
            // The balance resolves no parts for this slot: only "None" is legal.
            Some(list) if list.is_empty() => false,
            Some(list) => list.iter().any(|c| c == name),
        }
    }

    /// Choices for one item part slot, or `None` for `item`/`balance`.
    pub fn item_slot_choices(&self, item: &str, balance: &str, slot: &str) -> Option<Vec<String>> {
        let def = self.item_defs.get(item)?;
        match slot {
            "prefix" => Some(def.prefixes.clone()),
            "title" => Some(def.titles.clone()),
            "manufacturer" | "mfr" => Some(self.item_manufacturers(balance)),
            _ if ITEM_PART_SLOTS.contains(&slot) => {
                Some(self.item_parts(item, balance).get(slot).to_vec())
            }
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// JSON parsing
// ---------------------------------------------------------------------------

fn parse<T: serde::de::DeserializeOwned>(text: &str, what: &str) -> Result<T> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    serde_json::from_str(text).map_err(|e| Error::GameData(format!("bad {what} JSON: {e}")))
}

/// A JSON list that may be absent, `null`, or contain `null` elements (the
/// dumps are full of those). Absent/`null` stays `None` (the slot is not
/// listed); a present list is cleaned of `null`s.
type MaybeList = Option<Vec<Option<String>>>;

fn clean(list: MaybeList) -> Option<Vec<String>> {
    list.map(|v| v.into_iter().flatten().collect())
}

fn clean_or_default(list: MaybeList) -> Vec<String> {
    clean(list).unwrap_or_default()
}

#[derive(serde::Deserialize)]
struct RawPartCollectionJson {
    #[serde(default)]
    mode: Option<String>,
    #[serde(default)]
    body: MaybeList,
    #[serde(default)]
    grip: MaybeList,
    #[serde(default)]
    barrel: MaybeList,
    #[serde(default)]
    sight: MaybeList,
    #[serde(default)]
    stock: MaybeList,
    #[serde(default)]
    elemental: MaybeList,
    #[serde(default)]
    accessory1: MaybeList,
    #[serde(default)]
    accessory2: MaybeList,
    #[serde(default)]
    material: MaybeList,
    #[serde(default)]
    alpha: MaybeList,
    #[serde(default)]
    beta: MaybeList,
    #[serde(default)]
    gamma: MaybeList,
    #[serde(default)]
    delta: MaybeList,
    #[serde(default)]
    epsilon: MaybeList,
    #[serde(default)]
    zeta: MaybeList,
    #[serde(default)]
    eta: MaybeList,
    #[serde(default)]
    theta: MaybeList,
}

impl RawPartCollectionJson {
    fn slot(&self, slot: &str) -> Option<Vec<String>> {
        clean(match slot {
            "body" => self.body.clone(),
            "grip" => self.grip.clone(),
            "barrel" => self.barrel.clone(),
            "sight" => self.sight.clone(),
            "stock" => self.stock.clone(),
            "elemental" => self.elemental.clone(),
            "accessory1" => self.accessory1.clone(),
            "accessory2" => self.accessory2.clone(),
            "material" => self.material.clone(),
            "alpha" => self.alpha.clone(),
            "beta" => self.beta.clone(),
            "gamma" => self.gamma.clone(),
            "delta" => self.delta.clone(),
            "epsilon" => self.epsilon.clone(),
            "zeta" => self.zeta.clone(),
            "eta" => self.eta.clone(),
            "theta" => self.theta.clone(),
            _ => None,
        })
    }
}

#[derive(serde::Deserialize)]
struct RawBalanceJson {
    #[serde(default)]
    base: Option<String>,
    #[serde(default)]
    weapon_type: Option<String>,
    #[serde(default)]
    manufacturers: MaybeList,
    #[serde(default)]
    parts: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawItemBalanceJson {
    #[serde(default)]
    base: Option<String>,
    #[serde(default)]
    item: Option<String>,
    #[serde(default)]
    items: MaybeList,
    #[serde(default)]
    manufacturers: MaybeList,
    #[serde(default)]
    parts: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawWeaponTypeJson {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    titles: MaybeList,
    #[serde(default)]
    prefixes: MaybeList,
    #[serde(default)]
    body_parts: Option<String>,
    #[serde(default)]
    grip_parts: Option<String>,
    #[serde(default)]
    barrel_parts: Option<String>,
    #[serde(default)]
    sight_parts: Option<String>,
    #[serde(default)]
    stock_parts: Option<String>,
    #[serde(default)]
    elemental_parts: Option<String>,
    #[serde(default)]
    accessory1_parts: Option<String>,
    #[serde(default)]
    accessory2_parts: Option<String>,
    #[serde(default)]
    material_parts: Option<String>,
}

#[derive(serde::Deserialize)]
struct RawItemJson {
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    titles: MaybeList,
    #[serde(default)]
    prefixes: MaybeList,
    #[serde(default)]
    alpha_parts: Option<String>,
    #[serde(default)]
    beta_parts: Option<String>,
    #[serde(default)]
    gamma_parts: Option<String>,
    #[serde(default)]
    delta_parts: Option<String>,
    #[serde(default)]
    epsilon_parts: Option<String>,
    #[serde(default)]
    zeta_parts: Option<String>,
    #[serde(default)]
    eta_parts: Option<String>,
    #[serde(default)]
    theta_parts: Option<String>,
    #[serde(default)]
    material_parts: Option<String>,
}

fn resolve(part_lists: &HashMap<String, Vec<String>>, path: &Option<String>) -> Vec<String> {
    path.as_deref()
        .and_then(|p| part_lists.get(p))
        .cloned()
        .unwrap_or_default()
}

fn load_weapon_types(
    text: &str,
    part_lists: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, WeaponTypeDef>> {
    let raw: HashMap<String, RawWeaponTypeJson> = parse(text, "Weapon Types")?;
    let mut out = HashMap::new();
    for (path, r) in raw {
        let mut parts = BTreeMap::new();
        parts.insert("body".to_string(), resolve(part_lists, &r.body_parts));
        parts.insert("grip".to_string(), resolve(part_lists, &r.grip_parts));
        parts.insert("barrel".to_string(), resolve(part_lists, &r.barrel_parts));
        parts.insert("sight".to_string(), resolve(part_lists, &r.sight_parts));
        parts.insert("stock".to_string(), resolve(part_lists, &r.stock_parts));
        parts.insert(
            "elemental".to_string(),
            resolve(part_lists, &r.elemental_parts),
        );
        parts.insert(
            "accessory1".to_string(),
            resolve(part_lists, &r.accessory1_parts),
        );
        parts.insert(
            "accessory2".to_string(),
            resolve(part_lists, &r.accessory2_parts),
        );
        parts.insert(
            "material".to_string(),
            resolve(part_lists, &r.material_parts),
        );
        out.insert(
            path.clone(),
            WeaponTypeDef {
                resource_path: path,
                kind: r.kind.unwrap_or_default(),
                name: r.name.unwrap_or_default(),
                titles: clean_or_default(r.titles),
                prefixes: clean_or_default(r.prefixes),
                parts,
            },
        );
    }
    Ok(out)
}

fn load_item_defs(
    text: &str,
    part_lists: &HashMap<String, Vec<String>>,
) -> Result<HashMap<String, ItemDef>> {
    let raw: HashMap<String, RawItemJson> = parse(text, "Items")?;
    let mut out = HashMap::new();
    for (path, r) in raw {
        let mut parts = BTreeMap::new();
        for (slot, field) in [
            ("alpha", &r.alpha_parts),
            ("beta", &r.beta_parts),
            ("gamma", &r.gamma_parts),
            ("delta", &r.delta_parts),
            ("epsilon", &r.epsilon_parts),
            ("zeta", &r.zeta_parts),
            ("eta", &r.eta_parts),
            ("theta", &r.theta_parts),
            ("material", &r.material_parts),
        ] {
            parts.insert(slot.to_string(), resolve(part_lists, field));
        }
        out.insert(
            path.clone(),
            ItemDef {
                resource_path: path,
                kind: r.kind.unwrap_or_default(),
                name: r.name.unwrap_or_default(),
                titles: clean_or_default(r.titles),
                prefixes: clean_or_default(r.prefixes),
                parts,
            },
        );
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PART_LISTS: &str = r#"{"PL.Body":["body_a"],"PL.Grip":["grip_a"]}"#;
    const WEAPON_TYPES: &str = r#"{
      "WT.Pistol": {
        "type": "Pistol",
        "name": "Pistol",
        "titles": ["T.Title_A"],
        "prefixes": ["P.Prefix_A"],
        "body_parts": "PL.Body",
        "grip_parts": "PL.Grip"
      }
    }"#;
    const ITEMS: &str = r#"{"IT.Shield": {"type":"Shield","name":"Shield"}}"#;
    const ITEM_PART_LISTS: &str = "{}";

    fn defs(weapon_balance: &str, weapon_part_lists: &str) -> GameDefs {
        GameDefs::from_parts(
            WEAPON_TYPES,
            PART_LISTS,
            ITEMS,
            ITEM_PART_LISTS,
            Some(weapon_balance),
            Some(weapon_part_lists),
            None,
            None,
        )
        .expect("build defs")
    }

    #[test]
    fn base_chain_merges_with_mode_semantics() {
        // R: Additive body+grip. L: Selective grip overrides. C: Complete body
        // clears grip (slot absent) and sets body.
        let balances = r#"{
          "R.Bal": {"weapon_type":"WT.Pistol","parts":"R.PL"},
          "L.Bal": {"base":"R.Bal","parts":"L.PL"},
          "C.Bal": {"base":"L.Bal","parts":"C.PL"}
        }"#;
        let part_lists = r#"{
          "R.PL": {"mode":"Additive","body":["base_body"],"grip":["base_grip"]},
          "L.PL": {"mode":"Selective","grip":["leaf_grip"]},
          "C.PL": {"mode":"Complete","body":["done_body"]}
        }"#;
        let defs = defs(balances, part_lists);

        let l = defs.weapon_parts("L.Bal", "WT.Pistol");
        assert_eq!(
            l.get("body"),
            ["base_body"],
            "Selective inherits unlisted slots"
        );
        assert_eq!(
            l.get("grip"),
            ["leaf_grip"],
            "Selective overrides the listed slot"
        );

        let c = defs.weapon_parts("C.Bal", "WT.Pistol");
        assert_eq!(c.get("body"), ["done_body"]);
        assert!(
            c.get("grip").is_empty(),
            "Complete clears a slot it does not list"
        );

        assert!(defs.weapon_is_suitable("C.Bal", "WT.Pistol"));
    }

    #[test]
    fn weapon_slot_choices_cover_accessory2_and_prefix_title() {
        let balances = r#"{"B": {"weapon_type":"WT.Pistol","parts":"B.PL"}}"#;
        let part_lists = r#"{"B.PL": {"mode":"Additive","body":["b"],"grip":["g"]}}"#;
        let defs = defs(balances, part_lists);

        // accessory2 is a real slot even when the balance lists no parts.
        assert_eq!(
            defs.weapon_slot_choices("WT.Pistol", "B", "accessory2"),
            Some(vec![])
        );
        assert_eq!(
            defs.weapon_slot_choices("WT.Pistol", "B", "prefix"),
            Some(vec!["P.Prefix_A".to_string()])
        );
        assert_eq!(
            defs.weapon_slot_choices("WT.Pistol", "B", "title"),
            Some(vec!["T.Title_A".to_string()])
        );
        // The weapon type is not a part list.
        assert_eq!(defs.weapon_slot_choices("WT.Pistol", "B", "type"), None);
    }

    #[test]
    fn unsuitable_balance_yields_no_parts() {
        let balances = r#"
          {"A": {"weapon_type":"WT.Pistol","parts":"A.PL"},
           "B": {"weapon_type":"WT.Other","parts":"B.PL"}}"#;
        let part_lists =
            r#"{"A.PL":{"mode":"Additive","body":["a"]},"B.PL":{"mode":"Additive","body":["b"]}}"#;
        let defs = defs(balances, part_lists);
        assert!(defs.weapon_parts("A", "WT.Pistol").has("body"));
        assert!(!defs.weapon_is_suitable("B", "WT.Pistol"));
        assert!(defs.weapon_parts("B", "WT.Pistol").get("body").is_empty());
    }

    #[test]
    fn legality_rejects_parts_outside_the_resolved_list() {
        let balances = r#"{"B": {"weapon_type":"WT.Pistol","parts":"B.PL"}}"#;
        let part_lists = r#"{"B.PL": {"mode":"Additive","body":["b1"],"grip":["g1","g2"]}}"#;
        let defs = defs(balances, part_lists);
        let legal = |slot, name| defs.is_legal_part(true, Some("WT.Pistol"), "B", slot, name);
        assert!(legal("grip", "g1"));
        assert!(legal("grip", "None"));
        assert!(!legal("grip", "g9"));
        // A slot the balance resolves no parts for: only "None" is legal.
        assert!(!legal("stock", "s1"));
        assert!(legal("stock", "None"));
        // Non-part slots and unknown types are never flagged.
        assert!(legal("type", "WT.Pistol"));
        assert!(defs.is_legal_part(true, None, "B", "grip", "g9"));
    }
}
