//! Bundled game-data tables (`assets/data/`), used when the user has not
//! pointed Lootsmith at a game-data folder. A user folder always overrides
//! these files; the tables are the same ones Gibbed's editor reads, only
//! re-serialised to compact JSON.
//!
//! Licensing: see README, "Licensing". These are data tables (names, ids,
//! numbers) from the community dumps, not art or code.

use crate::error::Result;

macro_rules! bundled_text {
    ($name:literal) => {
        include_str!(concat!("../assets/data/", $name))
    };
}

/// The bundled Asset Library Manager.
pub fn asset_library() -> Result<crate::gameinfo::AssetLibraryManager> {
    crate::gameinfo::AssetLibraryManager::from_json_str(bundled_text!("asset-library-manager.json"))
}

pub fn travel_stations() -> Result<crate::travel::TravelStations> {
    crate::travel::TravelStations::from_json_str(bundled_text!("travel-stations.json"))
}

pub fn dlc_titles() -> Result<crate::travel::DlcTitles> {
    crate::travel::DlcTitles::from_json_str(bundled_text!("downloadable-contents.json"))
}

pub fn customizations() -> Result<crate::customizations::Customizations> {
    crate::customizations::Customizations::from_json_str(bundled_text!("customizations.json"))
}

pub fn missions() -> Result<crate::missions::Missions> {
    crate::missions::Missions::from_json_str(bundled_text!("missions.json"))
}

pub fn name_parts() -> Result<crate::nameparts::NameParts> {
    let mut parts =
        crate::nameparts::NameParts::from_json_str(bundled_text!("weapon-name-parts.json"))?;
    parts.add_json_str(bundled_text!("item-name-parts.json"))?;
    parts.add_json_str(bundled_text!("items.json"))?;
    Ok(parts)
}

pub fn part_lists() -> Result<crate::partlists::PartLists> {
    let mut lists =
        crate::partlists::PartLists::from_json_str(bundled_text!("item-balance-part-lists.json"))?;
    lists.add_json_str(bundled_text!("weapon-balance-part-lists.json"))?;
    Ok(lists)
}

/// The full definitions used by the part pickers (weapon types, item
/// definitions and their balance chains).
pub fn defs() -> Result<crate::defs::GameDefs> {
    crate::defs::GameDefs::from_parts(
        bundled_text!("weapon-types.json"),
        bundled_text!("weapon-part-lists.json"),
        bundled_text!("items.json"),
        bundled_text!("item-part-lists.json"),
        Some(bundled_text!("weapon-balance.json")),
        Some(bundled_text!("weapon-balance-part-lists.json")),
        Some(bundled_text!("item-balance.json")),
        Some(bundled_text!("item-balance-part-lists.json")),
    )
}
