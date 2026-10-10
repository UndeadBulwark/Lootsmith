//! The backpack/bank item model & editor helpers (moved verbatim from the
//! former single-file `app.rs`).

use super::*;
use eframe::egui;

use crate::app::LootsmithApp;

pub struct ItemEdit {
    pub container: Container,
    pub occurrence: usize,
    pub decoded: items::PackedData,
    pub quantity: i32,
    pub equipped: bool,
    pub mark: i32,
    pub quick_slot: i32,
    pub orig_quantity: i32,
    pub orig_equipped: bool,
    pub orig_mark: i32,
    pub orig_quick_slot: i32,
    pub dirty: bool,
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

pub(crate) fn resolve_balance(
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

/// Compose an item's display name from its prefix/title part words, when the
/// game data's name parts map them (`Weapon/Item Name Parts.json`).
pub(crate) fn item_name(
    decoded: &items::PackedData,
    manager: &Option<Arc<AssetLibraryManager>>,
    name_parts: Option<&NameParts>,
) -> String {
    let (Some(manager), Some(name_parts)) = (manager.as_ref(), name_parts) else {
        return String::new();
    };
    let set = decoded.asset_library_set_id;
    let mut words: Vec<&str> = Vec::new();
    for slot in ["prefix", "title"] {
        let Some((reference, group)) = decoded.part(slot) else {
            continue;
        };
        if reference == items::PackedAssetReference::NONE {
            continue;
        }
        let Ok(asset) = manager.lookup_asset(
            group,
            set,
            reference.asset_index,
            reference.sublibrary_index,
            reference.use_set_id,
        ) else {
            continue;
        };
        if let Some(word) = name_parts.part_name(&asset) {
            words.push(word);
        }
    }
    words.join(" ")
}

pub(crate) fn row_for(
    entry: &PackedEntry,
    display: usize,
    manager: &Option<Arc<AssetLibraryManager>>,
    configs: &AssetLibraryConfigs,
    name_parts: Option<&NameParts>,
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
            let mark = if decoded.unique_id == 0 {
                " (empty)"
            } else {
                ""
            };
            let balance = resolve_balance(manager, &decoded);
            let name = item_name(&decoded, manager, name_parts);
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
                name,
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
            name: String::new(),
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

    /// Mutate the currently selected item's editor (tests/tools); `None`
    /// when nothing is selected or game data is not loaded.
    pub fn selected_item_editor(&mut self, change: impl FnOnce(&mut ItemEdit)) {
        if let Some(edit) = self.item_edit.as_mut() {
            change(edit);
        }
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
        if let Err(e) =
            save_game.replace_serial(edit.container.field_number(), edit.occurrence, &serial)
        {
            self.status = format!("update error: {e}");
            return;
        }
        if edit.container == Container::BackpackItem {
            if edit.quantity != edit.orig_quantity {
                let _ =
                    save_game.set_entry_quantity(edit.container, edit.occurrence, edit.quantity);
            }
            if edit.equipped != edit.orig_equipped {
                let _ =
                    save_game.set_entry_equipped(edit.container, edit.occurrence, edit.equipped);
            }
        }
        if edit.container == Container::BackpackWeapon && edit.quick_slot != edit.orig_quick_slot {
            let _ =
                save_game.set_entry_quick_slot(edit.container, edit.occurrence, edit.quick_slot);
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
}
