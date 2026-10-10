//! Lootsmith core: a Rust library for reading and writing Borderlands 2 saves.
//!
//! A Rust port based on Gibbed's Borderlands 2 Save Editor by Rick Gibbed
//! (zlib license).

pub mod bitio;
pub mod container;
pub mod crc32;
pub mod customizations;
pub mod error;
pub mod gameinfo;
pub mod huffman;
pub mod items;
pub mod locate;
pub mod missions;
pub mod platform;
pub mod proto;
pub mod savegame;
pub mod travel;

pub use container::{pack_inner, serialize, unpack, wrap, SaveFile, BLOCK_SIZE};
pub use customizations::{usage_for_player_class, Customization, Customizations};
pub use error::{Error, Result};
pub use gameinfo::{AssetConfig, AssetGroup, AssetLibraryConfigs, AssetLibraryManager};
pub use items::{pack as pack_item, unpack as unpack_item, PackedData, PackedSlot};
pub use locate::{find_saves, find_saves_in, SaveCandidate};
pub use missions::{Mission, Missions};
pub use platform::{CompressionScheme, Endian, Platform};
pub use savegame::{
    mission_status_name, player_mark_name, Challenge, Container, MissionEntry, MissionPlaythrough,
    PackedEntry, SaveGame, Skill, VehicleCustomization, MISSION_STATUSES, PLAYER_MARKS,
    VEHICLE_FAMILIES,
};
pub use travel::{TravelStation, TravelStations};
