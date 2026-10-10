//! Item display names, from Gibbed's `Name Parts` dumps
//! (`Weapon Name Parts.json` / `Item Name Parts.json`).
//!
//! Each record maps a prefix/title part asset to a display-word (for example
//! `...Prefix.Dahl.Prefix_Bayonet1 → "Mama"`, `...Title_Dahl.Title_Barrel_...
//! → "Strike"`). A weapon's display name is its prefix + title words; item
//! flavour (`Items.json`) gives type names like "Combat Rifle Ammo".

use std::collections::HashMap;
use std::path::Path;

use crate::error::{Error, Result};

#[derive(Clone, Debug, Default)]
pub struct NameParts {
    names: HashMap<String, String>,
    /// `Items.json`: balance/type asset → flavour name ("Combat Rifle Ammo").
    item_names: HashMap<String, String>,
}

impl NameParts {
    /// Parse one name-parts JSON (weapon or item flavour).
    pub fn from_json_str(text: &str) -> Result<NameParts> {
        let mut out = NameParts::default();
        out.add_json_str(text)?;
        Ok(out)
    }

    /// Merge another name-parts JSON into this one. Records that look like
    /// `Items.json` flavour entries (objects with `name` + `type`) are stored
    /// as type names, part records (objects with `name` plus flags) as words.
    pub fn add_json_str(&mut self, text: &str) -> Result<()> {
        // The Gibbed dumps carry a UTF-8 BOM; strip it before parsing.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let raw: HashMap<String, serde_json::Value> = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("bad name parts JSON: {e}")))?;
        for (key, record) in raw {
            let Some(name) = record.get("name").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if record
                .get("type")
                .and_then(serde_json::Value::as_str)
                .is_some()
            {
                self.item_names.insert(key, name.to_string());
            } else {
                self.names.insert(key, name.to_string());
            }
        }
        Ok(())
    }

    /// Load name parts from a single file or directory (any of the flavours).
    pub fn from_path(path: &Path) -> Result<NameParts> {
        if path.is_dir() {
            return Self::from_dir(path);
        }
        Self::from_json_str(&std::fs::read_to_string(path).map_err(|e| Error::Io(e.to_string()))?)
    }

    /// Load both name-part files (and the item flavour list) from a dumps dir.
    pub fn from_dir(dir: &Path) -> Result<NameParts> {
        let mut out = NameParts::default();
        for name in [
            "Weapon Name Parts.json",
            "Item Name Parts.json",
            "Items.json",
        ] {
            let path = dir.join(name);
            if path.is_file() {
                out.add_json_str(
                    &std::fs::read_to_string(&path).map_err(|e| Error::Io(e.to_string()))?,
                )?;
            }
        }
        Ok(out)
    }

    /// The display word for a prefix/title part asset.
    pub fn part_name(&self, asset: &str) -> Option<&str> {
        self.names.get(asset).map(String::as_str)
    }

    /// The flavour name for a type/balance asset ("Combat Rifle Ammo").
    pub fn type_name(&self, asset: &str) -> Option<&str> {
        self.item_names.get(asset).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.names.len() + self.item_names.len()
    }

    pub fn is_empty(&self) -> bool {
        self.names.is_empty() && self.item_names.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXAMPLE: &str = r#"{
      "GD_Anemone_Weap_SniperRifles.Name.Prefix_Dahl.Prefix_Bayonet1": {"unique": true, "name": "Mama"},
      "GD_Anemone_Weap_SniperRifles.Name.Title_Dahl.Title_Barrel_Jakobs_SlowDamage": {"name": "Terror"},
      "GD_Ammodrops.Pickups.AmmoDrop_Assault_Rifle_Bullets": {"name": "Combat Rifle Ammo", "has_full_name": true, "type": "UsableItem"}
    }"#;

    #[test]
    fn parses_part_words_and_type_names() {
        let parts = NameParts::from_json_str(EXAMPLE).expect("parse");
        assert_eq!(
            parts.part_name("GD_Anemone_Weap_SniperRifles.Name.Prefix_Dahl.Prefix_Bayonet1"),
            Some("Mama")
        );
        assert_eq!(
            parts.type_name("GD_Ammodrops.Pickups.AmmoDrop_Assault_Rifle_Bullets"),
            Some("Combat Rifle Ammo")
        );
        assert_eq!(parts.part_name("GD_Unknown.Name.Title"), None);
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn real_dumps_parse() {
        // The dumps folder is optional on machines without game data.
        let Some(home) = std::env::var_os("HOME") else {
            return;
        };
        let dir = Path::new(&home).join("Projects/bl2-rogue/data/gibbed");
        if !dir.is_dir() {
            return;
        }
        let parts = NameParts::from_dir(&dir).expect("load real name parts");
        assert!(
            parts.len() > 1000,
            "expected the real dumps to carry many names, got {}",
            parts.len()
        );
    }
}
