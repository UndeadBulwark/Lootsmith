//! Mission definitions (`Missions.json`).
//!
//! Maps a mission resource path (e.g. `GD_Allium_Delivery.M_Delivery`) to its
//! numeric id, display name and flags. Used to label mission and plot/active
//! mission numbers.
//!
//! Game data is not bundled; load it from a dumps file at runtime.

use crate::error::{Error, Result};
use std::collections::HashMap;

/// One mission definition.
#[derive(Clone, Debug)]
pub struct Mission {
    pub key: String,
    pub number: i32,
    pub name: String,
    pub description: String,
    pub is_plot_critical: bool,
    pub can_be_failed: bool,
}

/// The parsed `Missions.json`, indexed by resource path and by number.
pub struct Missions {
    items: Vec<Mission>,
    by_key: HashMap<String, usize>,
    by_number: HashMap<i32, usize>,
}

impl Missions {
    pub fn from_json_str(text: &str) -> Result<Self> {
        #[derive(serde::Deserialize)]
        struct Def {
            #[serde(default)]
            number: i32,
            #[serde(default)]
            name: String,
            #[serde(default)]
            description: String,
            #[serde(default)]
            is_plot_critical: bool,
            #[serde(default)]
            can_be_failed: bool,
        }

        let text = text.trim_start_matches('\u{feff}');
        let raw: HashMap<String, Def> = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("invalid Missions JSON: {e}")))?;

        let mut items: Vec<Mission> = raw
            .into_iter()
            .map(|(key, def)| Mission {
                key,
                number: def.number,
                name: def.name,
                description: def.description,
                is_plot_critical: def.is_plot_critical,
                can_be_failed: def.can_be_failed,
            })
            .collect();
        items.sort_by(|a, b| a.key.cmp(&b.key));

        let mut by_key = HashMap::new();
        let mut by_number = HashMap::new();
        for (i, mission) in items.iter().enumerate() {
            by_key.insert(mission.key.clone(), i);
            by_number.entry(mission.number).or_insert(i);
        }

        Ok(Self {
            items,
            by_key,
            by_number,
        })
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

    pub fn find(&self, key: &str) -> Option<&Mission> {
        self.by_key.get(key).map(|&i| &self.items[i])
    }

    pub fn find_number(&self, number: i32) -> Option<&Mission> {
        self.by_number.get(&number).map(|&i| &self.items[i])
    }

    /// A display label for a mission key (its name, else the key).
    pub fn label_for(&self, key: &str) -> String {
        match self.find(key) {
            Some(m) if !m.name.is_empty() => m.name.clone(),
            _ => key.to_string(),
        }
    }

    /// A display label for a `PlotMissionNumber`/`ActiveMissionNumber`.
    pub fn label_for_number(&self, number: i32) -> Option<String> {
        self.find_number(number)
            .map(|m| format!("{} ({} / {})", m.name, m.key, number))
    }

    /// All missions sorted by number, as `(number, key, name)`.
    pub fn by_number_sorted(&self) -> Vec<(i32, &str, &str)> {
        let mut out: Vec<(i32, &str, &str)> = self
            .items
            .iter()
            .map(|m| (m.number, m.key.as_str(), m.name.as_str()))
            .collect();
        out.sort();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_resolves_by_key_and_number() {
        let text = r#"{
            "GD_X.M_One": { "number": 5, "name": "One", "is_plot_critical": true },
            "GD_X.M_Two": { "number": 9, "name": "Two", "can_be_failed": true }
        }"#;
        let m = Missions::from_json_str(text).unwrap();
        assert_eq!(m.len(), 2);
        assert_eq!(m.label_for("GD_X.M_One"), "One");
        assert_eq!(m.find_number(9).unwrap().name, "Two");
        assert!(m.label_for_number(5).unwrap().starts_with("One"));
        assert_eq!(m.label_for_number(999), None);
    }
}
