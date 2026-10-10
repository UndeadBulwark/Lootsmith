//! Customization definitions (`Customizations.json`).
//!
//! Maps a customization resource path (e.g. `GD_AllCustoms_allium.Assassin.Head_Snowman`)
//! to its display name, type (`Head`/`Skin`) and the usages it applies to
//! (character classes, and the vehicle families `Runner`/`BanditTech`/`Hovercraft`/
//! `FanBoat`). Vehicles pick **Skin** customizations for their usage.
//!
//! Game data is not bundled; load it from a dumps file at runtime.

use crate::error::{Error, Result};
use std::collections::HashMap;

/// One customization definition.
#[derive(Clone, Debug)]
pub struct Customization {
    pub key: String,
    pub name: String,
    pub kind: String,
    pub usage: Vec<String>,
    pub dlc: Option<String>,
}

impl Customization {
    pub fn is_skin(&self) -> bool {
        self.kind == "Skin"
    }
}

/// The parsed `Customizations.json`, indexed by resource path.
pub struct Customizations {
    items: Vec<Customization>,
    by_key: HashMap<String, usize>,
}

impl Customizations {
    pub fn from_json_str(text: &str) -> Result<Self> {
        #[derive(serde::Deserialize)]
        struct Def {
            #[serde(default)]
            name: String,
            #[serde(default, rename = "type")]
            kind: String,
            #[serde(default)]
            usage: Vec<String>,
            #[serde(default)]
            dlc: Option<String>,
        }

        let text = text.trim_start_matches('\u{feff}');
        let raw: HashMap<String, Def> = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("invalid Customizations JSON: {e}")))?;

        let mut items: Vec<Customization> = raw
            .into_iter()
            .map(|(key, def)| Customization {
                key,
                name: def.name,
                kind: def.kind,
                usage: def.usage,
                dlc: def.dlc,
            })
            .collect();
        items.sort_by(|a, b| a.key.cmp(&b.key));

        let mut by_key = HashMap::new();
        for (i, item) in items.iter().enumerate() {
            by_key.insert(item.key.clone(), i);
        }

        Ok(Self { items, by_key })
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn find(&self, key: &str) -> Option<&Customization> {
        self.by_key.get(key).map(|&i| &self.items[i])
    }

    /// A display label for a customization key (its name, else the key).
    pub fn label_for(&self, key: &str) -> String {
        match self.find(key) {
            Some(c) if !c.name.is_empty() => c.name.clone(),
            _ => key.to_string(),
        }
    }

    /// The **Skin** customization keys for a vehicle `usage`, as `(key, name)`
    /// pairs sorted by display name.
    pub fn skin_options(&self, usage: &str) -> Vec<(String, String)> {
        let mut out: Vec<(&str, &str)> = self
            .items
            .iter()
            .filter(|c| c.is_skin() && c.usage.iter().any(|u| u == usage))
            .map(|c| (c.name.as_str(), c.key.as_str()))
            .collect();
        out.sort();
        out.dedup();
        out.into_iter()
            .map(|(name, key)| (key.to_string(), name.to_string()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_filters_skins() {
        let text = r#"{
            "GD_X.Assassin.Head_Snowman": {
                "name": "Chill 0ut", "type": "Head", "usage": ["Assassin"]
            },
            "GD_X.Runner.Skin_A": {
                "name": "A", "type": "Skin", "usage": ["Runner", "Soldier"]
            },
            "GD_X.Runner.Skin_B": {
                "name": "B", "type": "Skin", "usage": ["Runner"]
            }
        }"#;
        let c = Customizations::from_json_str(text).unwrap();
        assert_eq!(c.len(), 3);
        let runner = c.skin_options("Runner");
        assert_eq!(runner.len(), 2, "only the two Runner skins");
        assert_eq!(runner[0].1, "A");
        assert_eq!(c.find("GD_X.Runner.Skin_A").unwrap().name, "A");
        assert_eq!(c.label_for("GD_X.Unknown"), "GD_X.Unknown");
    }
}
