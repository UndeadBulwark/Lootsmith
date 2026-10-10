//! Game data: the asset library manager configuration.
//!
//! The packed item codec only needs the *bit widths* per asset group (which
//! derive from the game's Asset Library Manager version). The full set tables
//! (for name lookup) are Phase 3 and are deliberately not required here.
//!
//! Game data is not bundled; load it from a dumps file at runtime, or use
//! [`AssetLibraryConfigs::pc_default`] for the known current values.

use crate::error::{Error, Result};
use std::collections::HashMap;

/// One asset group, keyed by the same names the dumps use.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AssetGroup {
    ItemTypes,
    ItemParts,
    Manufacturers,
    BalanceDefs,
    WeaponTypes,
    WeaponParts,
}

impl AssetGroup {
    pub const ALL: [AssetGroup; 6] = [
        AssetGroup::ItemTypes,
        AssetGroup::ItemParts,
        AssetGroup::Manufacturers,
        AssetGroup::BalanceDefs,
        AssetGroup::WeaponTypes,
        AssetGroup::WeaponParts,
    ];

    pub fn key(self) -> &'static str {
        match self {
            AssetGroup::ItemTypes => "ItemTypes",
            AssetGroup::ItemParts => "ItemParts",
            AssetGroup::Manufacturers => "Manufacturers",
            AssetGroup::BalanceDefs => "BalanceDefs",
            AssetGroup::WeaponTypes => "WeaponTypes",
            AssetGroup::WeaponParts => "WeaponParts",
        }
    }
}

/// Bit widths for one asset group.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssetConfig {
    pub sublibrary_bits: u32,
    pub asset_bits: u32,
}

impl AssetConfig {
    pub const fn new(sublibrary_bits: u32, asset_bits: u32) -> Self {
        Self {
            sublibrary_bits,
            asset_bits,
        }
    }

    pub fn total_bits(&self) -> u32 {
        self.sublibrary_bits + self.asset_bits
    }

    pub fn asset_mask(&self) -> u32 {
        if self.asset_bits >= 32 {
            u32::MAX
        } else {
            (1u32 << self.asset_bits) - 1
        }
    }

    pub fn sublibrary_mask(&self) -> u32 {
        if self.sublibrary_bits == 0 {
            0
        } else {
            (1u32 << (self.sublibrary_bits - 1)) - 1
        }
    }

    pub fn use_set_id_mask(&self) -> u32 {
        if self.sublibrary_bits == 0 {
            0
        } else {
            1u32 << (self.sublibrary_bits - 1)
        }
    }

    pub fn none_index(&self) -> u32 {
        if self.total_bits() >= 32 {
            u32::MAX
        } else {
            (1u32 << self.total_bits()) - 1
        }
    }
}

/// The asset library manager configuration (version + per-group bit widths).
#[derive(Clone, Debug)]
pub struct AssetLibraryConfigs {
    pub version: u8,
    configs: HashMap<AssetGroup, AssetConfig>,
}

impl AssetLibraryConfigs {
    pub fn config(&self, group: AssetGroup) -> AssetConfig {
        self.configs
            .get(&group)
            .copied()
            .unwrap_or(AssetConfig::new(0, 0))
    }

    /// The values observed in the current community dumps (Asset Library
    /// Manager version 7). Correct for Borderlands 2 as of 2026; newer game
    /// patches could change them, in which case load a fresh dumps file.
    pub fn pc_default() -> Self {
        let mut configs = HashMap::new();
        configs.insert(AssetGroup::ItemTypes, AssetConfig::new(9, 8));
        configs.insert(AssetGroup::ItemParts, AssetConfig::new(6, 10));
        configs.insert(AssetGroup::Manufacturers, AssetConfig::new(4, 7));
        configs.insert(AssetGroup::BalanceDefs, AssetConfig::new(10, 10));
        configs.insert(AssetGroup::WeaponTypes, AssetConfig::new(7, 6));
        configs.insert(AssetGroup::WeaponParts, AssetConfig::new(6, 11));
        Self {
            version: 7,
            configs,
        }
    }

    /// Parse a `Asset Library Manager.json` dumps file. Only `version` and
    /// `configs` are read; the large `sets` table is ignored.
    pub fn from_json_str(text: &str) -> Result<Self> {
        #[derive(serde::Deserialize)]
        struct Root {
            version: u8,
            configs: HashMap<String, Cfg>,
        }
        #[derive(serde::Deserialize)]
        struct Cfg {
            sublibrary_bits: u32,
            asset_bits: u32,
        }

        let text = text.trim_start_matches('\u{feff}');
        let root: Root = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("invalid Asset Library Manager JSON: {e}")))?;

        let mut configs = HashMap::new();
        for group in AssetGroup::ALL {
            if let Some(cfg) = root.configs.get(group.key()) {
                configs.insert(group, AssetConfig::new(cfg.sublibrary_bits, cfg.asset_bits));
            }
        }
        Ok(Self {
            version: root.version,
            configs,
        })
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }
}

// ---------------------------------------------------------------------------
// Full asset library manager (version + configs + set tables) for name lookup.
// ---------------------------------------------------------------------------

struct AssetSublibrary {
    package: String,
    assets: Vec<String>,
}

struct AssetLibraryDefinition {
    sublibraries: Vec<AssetSublibrary>,
}

struct AssetLibrarySet {
    libraries: HashMap<AssetGroup, AssetLibraryDefinition>,
}

/// The full Asset Library Manager: bit-width configs plus the set tables used
/// to turn asset indices into names (and back).
pub struct AssetLibraryManager {
    pub configs: AssetLibraryConfigs,
    sets: HashMap<i32, AssetLibrarySet>,
}

impl AssetLibraryManager {
    /// Parse an `Asset Library Manager.json` dumps file (version, configs, sets).
    pub fn from_json_str(text: &str) -> Result<Self> {
        #[derive(serde::Deserialize)]
        struct Root {
            version: u8,
            configs: HashMap<String, Cfg>,
            #[serde(default)]
            sets: Vec<SetJson>,
        }
        #[derive(serde::Deserialize)]
        struct Cfg {
            sublibrary_bits: u32,
            asset_bits: u32,
        }
        #[derive(serde::Deserialize)]
        struct SetJson {
            id: i32,
            #[serde(default)]
            libraries: HashMap<String, LibJson>,
        }
        #[derive(serde::Deserialize)]
        struct LibJson {
            #[serde(default, rename = "type")]
            _type: Option<String>,
            #[serde(default)]
            sublibraries: Vec<SubJson>,
        }
        #[derive(serde::Deserialize)]
        struct SubJson {
            #[serde(default)]
            package: String,
            #[serde(default)]
            assets: Vec<String>,
        }

        let text = text.trim_start_matches('\u{feff}');
        let root: Root = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("invalid Asset Library Manager JSON: {e}")))?;

        let mut configs = HashMap::new();
        for group in AssetGroup::ALL {
            if let Some(cfg) = root.configs.get(group.key()) {
                configs.insert(group, AssetConfig::new(cfg.sublibrary_bits, cfg.asset_bits));
            }
        }

        let mut sets = HashMap::new();
        for set_json in root.sets {
            let mut libraries = HashMap::new();
            for group in AssetGroup::ALL {
                let Some(lib) = set_json.libraries.get(group.key()) else {
                    continue;
                };
                let sublibraries = lib
                    .sublibraries
                    .iter()
                    .map(|sub| AssetSublibrary {
                        package: sub.package.clone(),
                        assets: sub.assets.clone(),
                    })
                    .collect();
                libraries.insert(group, AssetLibraryDefinition { sublibraries });
            }
            sets.insert(set_json.id, AssetLibrarySet { libraries });
        }

        Ok(Self {
            configs: AssetLibraryConfigs {
                version: root.version,
                configs,
            },
            sets,
        })
    }

    pub fn from_path(path: &std::path::Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)?;
        Self::from_json_str(&text)
    }

    pub fn version(&self) -> u8 {
        self.configs.version
    }

    /// Resolve a packed reference to its `package.asset` name.
    ///
    /// `set_id` is the owning item's asset library set id. `None` references
    /// resolve to `"None"`. PC saves have no platform remapping (only X360 does).
    pub fn lookup_asset(
        &self,
        group: AssetGroup,
        set_id: i32,
        asset_index: i32,
        sublibrary_index: i32,
        use_set_id: bool,
    ) -> Result<String> {
        if asset_index < 0 && sublibrary_index < 0 {
            return Ok("None".to_string());
        }
        let actual_set = if use_set_id { set_id } else { 0 };
        let set = self
            .sets
            .get(&actual_set)
            .ok_or_else(|| Error::GameData(format!("unknown asset library set {actual_set}")))?;
        let library = set.libraries.get(&group).ok_or_else(|| {
            Error::GameData(format!("set {actual_set} has no {} library", group.key()))
        })?;
        if sublibrary_index < 0 || sublibrary_index as usize >= library.sublibraries.len() {
            return Err(Error::GameData(format!(
                "invalid sublibrary index {sublibrary_index} in set {actual_set}"
            )));
        }
        let sub = &library.sublibraries[sublibrary_index as usize];
        if asset_index < 0 || asset_index as usize >= sub.assets.len() {
            return Err(Error::GameData(format!(
                "invalid asset index {asset_index} in sublibrary {}",
                sub.package
            )));
        }
        Ok(format!(
            "{}.{}",
            sub.package, sub.assets[asset_index as usize]
        ))
    }

    /// Find the packed indices for a `package.asset` name. Falls back to set 0.
    /// Returns `(asset_index, sublibrary_index, use_set_id)`.
    pub fn find_asset(
        &self,
        group: AssetGroup,
        set_id: i32,
        name: &str,
    ) -> Result<(i32, i32, bool)> {
        if name == "None" {
            return Ok((-1, -1, false));
        }
        let (package, asset) = name.split_once('.').ok_or_else(|| {
            Error::GameData(format!("asset name '{name}' is not 'package.asset'"))
        })?;

        for &candidate in &[set_id, 0] {
            let Some(set) = self.sets.get(&candidate) else {
                continue;
            };
            let Some(library) = set.libraries.get(&group) else {
                continue;
            };
            let found = library
                .sublibraries
                .iter()
                .enumerate()
                .find_map(|(si, sub)| {
                    if sub.package == package {
                        sub.assets
                            .iter()
                            .position(|a| a == asset)
                            .map(|ai| (ai as i32, si as i32))
                    } else {
                        None
                    }
                });
            if let Some((asset_index, sublibrary_index)) = found {
                return Ok((asset_index, sublibrary_index, candidate != 0));
            }
        }
        Err(Error::GameData(format!("unsupported asset '{name}'")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_masks() {
        let configs = AssetLibraryConfigs::pc_default();
        let item_parts = configs.config(AssetGroup::ItemParts);
        assert_eq!(item_parts.total_bits(), 16);
        assert_eq!(item_parts.asset_mask(), 0x3FF);
        assert_eq!(item_parts.sublibrary_mask(), 0x1F);
        assert_eq!(item_parts.use_set_id_mask(), 0x20);
        assert_eq!(item_parts.none_index(), 0xFFFF);
    }

    #[test]
    fn parse_minimal_json() {
        let text = r#"{"version":7,"configs":{"ItemParts":{"sublibrary_bits":6,"asset_bits":10,"type":"x"}},"sets":[]}"#;
        let configs = AssetLibraryConfigs::from_json_str(text).unwrap();
        assert_eq!(configs.version, 7);
        assert_eq!(
            configs.config(AssetGroup::ItemParts),
            AssetConfig::new(6, 10)
        );
    }
}
