//! Balance → part list mappings, from Gibbed's `Balance Part Lists` dumps
//! (`Item Balance Part Lists.json` / `Weapon Balance Part Lists.json`).
//!
//! Each record is keyed `<balance>.PartList` with the part names allowed per
//! slot. `mode` is "Complete" (only these parts for the covered slots) or
//! "Selective" (these parts plus anything else in the library). Records
//! without the balance being edited are simply not consulted.

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct BalancePartList {
    /// Raw mode ("Complete" / "Selective").
    pub mode: String,
    pub slots: HashMap<String, Vec<String>>,
}

impl BalancePartList {
    /// "Complete" records constrain a slot to exactly their list.
    pub fn is_complete(&self) -> bool {
        self.mode.eq_ignore_ascii_case("Complete")
    }
}

#[derive(Clone, Debug, Default)]
pub struct PartLists {
    /// Record id (`<balance>.PartList`, suffix kept) → part list.
    records: HashMap<String, BalancePartList>,
}

impl PartLists {
    /// Parse one part lists JSON (either item or weapon flavour).
    pub fn from_json_str(text: &str) -> Result<PartLists> {
        let mut out = PartLists::default();
        out.add_json_str(text)?;
        Ok(out)
    }

    /// Merge another part lists JSON into this one (item + weapon files).
    pub fn add_json_str(&mut self, text: &str) -> Result<()> {
        let raw: HashMap<String, serde_json::Value> = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("bad part lists JSON: {e}")))?;
        for (key, record) in raw {
            let record = record.as_object().ok_or_else(|| {
                Error::GameData(format!("part list record {key} is not an object"))
            })?;
            let mode = record
                .get("mode")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Selective")
                .to_string();
            let mut slots = HashMap::new();
            let lists = record
                .iter()
                .filter(|(field, _)| field.as_str() != "mode")
                .filter_map(|(field, parts)| {
                    let names: Vec<String> = parts
                        .as_array()?
                        .iter()
                        .filter_map(serde_json::Value::as_str)
                        .map(str::to_string)
                        .collect();
                    (matches!(parts.as_array(), Some(array) if !array.is_empty()))
                        .then_some((field.clone(), names))
                });
            for (field, names) in lists {
                slots.insert(field, names);
            }
            self.records.insert(key, BalancePartList { mode, slots });
        }
        Ok(())
    }

    /// Load one (or both, as a JSON array) part list files from disk.
    pub fn from_path(path: &Path) -> Result<PartLists> {
        let text = std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?;
        Self::from_json_str(&text)
    }

    /// Load `Item Balance Part Lists.json` and `Weapon Balance Part Lists.json`
    /// from the game-data directory, when present.
    pub fn from_dir(dir: &Path) -> Result<PartLists> {
        let mut lists = PartLists::default();
        for name in [
            "Item Balance Part Lists.json",
            "Weapon Balance Part Lists.json",
        ] {
            let path = dir.join(name);
            if path.is_file() {
                lists.add_json_str(
                    &std::fs::read_to_string(&path).map_err(|e| Error::Io(e.to_string()))?,
                )?;
            }
        }
        Ok(lists)
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// The record for a balance asset name (`<balance>.PartList` accepted).
    pub fn for_balance(&self, balance: &str) -> Option<&BalancePartList> {
        if let Some(record) = self.records.get(balance) {
            return Some(record);
        }
        self.records.get(&format!("{balance}.PartList"))
    }

    /// The constrained choices for `slot` on `balance`: `(complete, names)`.
    /// `complete` means the list is exhaustive; otherwise it is a hint that
    /// should be offered first, merged with the library-wide choices.
    pub fn choices_for(&self, balance: &str, slot: &str) -> Option<(bool, Vec<String>)> {
        let record = self.for_balance(balance)?;
        let names = record.slots.get(slot)?;
        Some((record.is_complete(), names.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"{
      "GD_Aster_Weapons.SMGs.SMG_Bandit_3_Orc.PartList": {
        "mode": "Complete",
        "body": ["GD_Weap_SMG.Body.SMG_Body_Bandit"],
        "barrel": ["GD_Weap_SMG.Barrel.SMG_Barrel_Bandit_A"]
      },
      "GD_Anemone_Weapons.SMGs.SMG_Maliwan_5_HellFire.PartList": {
        "mode": "Selective",
        "barrel": ["GD_Weap_SMG.Barrel.SMG_Barrel_Maliwan_HellFire"]
      },
      "GD_Anemone_GrenadeMods.A_Item.GrenadeMod_Standard": {
        "item": "GD_Anemone_GrenadeMods.A_Item.GrenadeMod_Standard",
        "mode": "Selective",
        "gamma": ["GD_Anemone_GrenadeMods.Trigger.Trigger_Grade0"]
      }
    }"#;

    #[test]
    fn parses_both_record_shapes() {
        let lists = PartLists::from_json_str(EXAMPLE).expect("parse");
        assert_eq!(lists.len(), 3);
    }

    #[test]
    fn balance_lookup_accepts_suffix_or_not() {
        let lists = PartLists::from_json_str(EXAMPLE).expect("parse");
        let with_suffix = lists.for_balance("GD_Aster_Weapons.SMGs.SMG_Bandit_3_Orc.PartList");
        let bare = lists.for_balance("GD_Aster_Weapons.SMGs.SMG_Bandit_3_Orc");
        assert_eq!(
            with_suffix.map(|r| r.mode.clone()),
            bare.map(|r| r.mode.clone())
        );
        assert!(bare.is_some());
        assert!(lists.for_balance("GD_Unknown.Balance").is_none());
    }

    #[test]
    fn complete_records_constrain_selective_hint() {
        let lists = PartLists::from_json_str(EXAMPLE).expect("parse");
        let (complete, body) = lists
            .choices_for("GD_Aster_Weapons.SMGs.SMG_Bandit_3_Orc", "body")
            .expect("body list");
        assert!(complete);
        assert_eq!(body, ["GD_Weap_SMG.Body.SMG_Body_Bandit"]);

        let (complete, barrel) = lists
            .choices_for("GD_Anemone_Weapons.SMGs.SMG_Maliwan_5_HellFire", "barrel")
            .expect("barrel list");
        assert!(!complete);
        assert_eq!(barrel, ["GD_Weap_SMG.Barrel.SMG_Barrel_Maliwan_HellFire"]);

        // Slot not covered by a Selective record → no constraint.
        assert!(lists
            .choices_for("GD_Anemone_Weapons.SMGs.SMG_Maliwan_5_HellFire", "stock")
            .is_none());
    }
}