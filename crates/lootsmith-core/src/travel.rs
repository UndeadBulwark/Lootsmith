//! Fast-travel station definitions (`Travel Stations.json`).
//!
//! A save records visited teleporters (field 16) and the last one (field 17) by
//! station `resource_name` (e.g. `"Sanctuary"`). This table maps that name to the
//! full station definition and its player-facing display name.
//!
//! Game data is not bundled; load it from a dumps file at runtime.

use crate::error::{Error, Result};
use std::collections::HashMap;

/// DLC expansion titles (`Downloadable Contents.json`, type "Expansion"),
/// used to group fast-travel stations by chapter in the UI.
#[derive(Clone, Debug, Default)]
pub struct DlcTitles {
    titles: std::collections::HashMap<String, String>,
}

impl DlcTitles {
    /// Parse a `Downloadable Contents.json` string, keeping Expansion entries.
    pub fn from_json_str(text: &str) -> Result<DlcTitles> {
        // The Gibbed dumps carry a UTF-8 BOM; strip it before parsing.
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        let raw: std::collections::HashMap<String, serde_json::Value> = serde_json::from_str(text)
            .map_err(|e| crate::error::Error::GameData(format!("bad DLC contents JSON: {e}")))?;
        let mut titles = std::collections::HashMap::new();
        for (key, record) in raw {
            if record.get("type").and_then(serde_json::Value::as_str) != Some("Expansion") {
                continue;
            }
            if let Some(name) = record.get("name").and_then(serde_json::Value::as_str) {
                titles.insert(key, name.to_string());
            }
        }
        Ok(DlcTitles { titles })
    }

    /// Load `Downloadable Contents.json` from the game-data folder (or the
    /// file directly).
    pub fn from_path(path: &std::path::Path) -> Result<DlcTitles> {
        Self::from_json_str(
            &std::fs::read_to_string(path).map_err(|e| crate::error::Error::Io(e.to_string()))?,
        )
    }

    pub fn title(&self, expansion_key: &str) -> Option<&str> {
        self.titles.get(expansion_key).map(String::as_str)
    }

    pub fn len(&self) -> usize {
        self.titles.len()
    }

    pub fn is_empty(&self) -> bool {
        self.titles.is_empty()
    }
}

/// One `FastTravelStationDefinition`.
#[derive(Clone, Debug)]
pub struct TravelStation {
    /// The top-level key, e.g. `GD_FastTravelStations.Sanctuary.Sanctuary`.
    pub id: String,
    /// The `resource_name` stored in the save, e.g. `Sanctuary`.
    pub resource_name: String,
    /// Player-facing name, e.g. `Sanctuary` or `Southern Shelf`.
    pub display_name: String,
    pub level_name: String,
    pub dlc_expansion: Option<String>,
    pub initially_active: bool,
    pub send_only: bool,
}

impl TravelStation {
    /// Whether this station is one the player can fast-travel *to* (excludes the
    /// send-only and one-way stations that only appear as level exits).
    pub fn is_destination(&self) -> bool {
        !self.send_only
    }

    /// A readable label: display name plus the resource name when they differ.
    pub fn label(&self) -> String {
        if self.display_name.is_empty() {
            self.resource_name.clone()
        } else if self.display_name == self.resource_name {
            self.display_name.clone()
        } else {
            format!("{} ({})", self.display_name, self.resource_name)
        }
    }
}

/// The parsed `Travel Stations.json`, indexed by `resource_name`.
pub struct TravelStations {
    stations: Vec<TravelStation>,
    by_resource: HashMap<String, usize>,
}

impl TravelStations {
    pub fn from_json_str(text: &str) -> Result<Self> {
        #[derive(serde::Deserialize)]
        struct Def {
            #[serde(default)]
            resource_name: String,
            #[serde(default)]
            station_display_name: String,
            #[serde(default)]
            level_name: String,
            #[serde(default)]
            dlc_expansion: Option<String>,
            #[serde(default)]
            initially_active: bool,
            #[serde(default)]
            send_only: bool,
        }

        let text = text.trim_start_matches('\u{feff}');
        let raw: HashMap<String, Def> = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("invalid Travel Stations JSON: {e}")))?;

        let mut stations: Vec<TravelStation> = raw
            .into_iter()
            .filter(|(_, def)| !def.resource_name.is_empty())
            .map(|(id, def)| TravelStation {
                id,
                resource_name: def.resource_name,
                display_name: def.station_display_name,
                level_name: def.level_name,
                dlc_expansion: def.dlc_expansion,
                initially_active: def.initially_active,
                send_only: def.send_only,
            })
            .collect();
        stations.sort_by(|a, b| a.resource_name.cmp(&b.resource_name));

        let mut by_resource = HashMap::new();
        for (i, station) in stations.iter().enumerate() {
            // Keep the first definition for a duplicated resource name.
            by_resource.entry(station.resource_name.clone()).or_insert(i);
        }

        Ok(Self {
            stations,
            by_resource,
        })
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }

    pub fn all(&self) -> &[TravelStation] {
        &self.stations
    }

    pub fn len(&self) -> usize {
        self.stations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stations.is_empty()
    }

    /// Look up a save's teleporter string (a `resource_name`).
    pub fn find(&self, resource_name: &str) -> Option<&TravelStation> {
        self.by_resource
            .get(resource_name)
            .map(|&i| &self.stations[i])
    }

    /// A label for a save string, falling back to the raw string when unknown.
    pub fn label_for(&self, resource_name: &str) -> String {
        match self.find(resource_name) {
            Some(station) => station.label(),
            None => resource_name.to_string(),
        }
    }

    /// Distinct destination labels (for an add dropdown), sorted by label.
    pub fn destination_labels(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .stations
            .iter()
            .filter(|s| s.is_destination())
            .map(|s| s.resource_name.clone())
            .collect();
        out.sort();
        out.dedup();
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_resolves() {
        let text = r#"{
            "GD_FastTravelStations.Sanctuary.Sanctuary": {
                "resource_name": "Sanctuary",
                "station_display_name": "Sanctuary",
                "level_name": "Sanctuary_P",
                "initially_active": true,
                "send_only": false
            },
            "GD_FastTravelStations.Zone1.SouthernShelfTown": {
                "resource_name": "SouthernShelfTown",
                "station_display_name": "Southern Shelf",
                "level_name": "SouthernShelf_P",
                "initially_active": false,
                "send_only": false
            }
        }"#;
        let stations = TravelStations::from_json_str(text).unwrap();
        assert_eq!(stations.len(), 2);
        assert_eq!(
            stations.find("SouthernShelfTown").unwrap().display_name,
            "Southern Shelf"
        );
        assert_eq!(stations.label_for("Sanctuary"), "Sanctuary");
        assert_eq!(stations.label_for("DoesNotExist"), "DoesNotExist");
    }
}
