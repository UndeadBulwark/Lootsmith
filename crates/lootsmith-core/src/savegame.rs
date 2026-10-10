//! Typed view over `WillowTwoPlayerSaveGame`.
//!
//! Field numbers and kinds are transcribed from
//! `ProtoBufFormats/WillowTwoSave/WillowTwoPlayerSaveGame.cs`. The underlying
//! [`Message`] keeps raw bytes, so untouched fields re-emit exactly; setters
//! replace a single field with a canonical encoding.

use crate::error::{Error, Result};
use crate::proto::{Field, Message, WireType};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Str,
    Int,
    Bool,
    Bytes,
    Guid,
    IntList,
    StrList,
    Msg(&'static str),
    MsgList(&'static str),
}

pub struct FieldSpec {
    pub number: u32,
    pub name: &'static str,
    pub kind: Kind,
}

/// Top-level schema. `IsRequired`/`IsPacked` come from the C# attributes.
pub const TOP_LEVEL: &[FieldSpec] = &[
    FieldSpec {
        number: 1,
        name: "PlayerClass",
        kind: Kind::Str,
    },
    FieldSpec {
        number: 2,
        name: "ExpLevel",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 3,
        name: "ExpPoints",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 4,
        name: "GeneralSkillPoints",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 5,
        name: "SpecialistSkillPoints",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 6,
        name: "CurrencyOnHand",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 7,
        name: "PlaythroughsCompleted",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 8,
        name: "SkillData",
        kind: Kind::MsgList("SkillData"),
    },
    FieldSpec {
        number: 9,
        name: "Unknown9",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 10,
        name: "Unknown10",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 11,
        name: "ResourceData",
        kind: Kind::MsgList("ResourceData"),
    },
    FieldSpec {
        number: 12,
        name: "ItemData",
        kind: Kind::MsgList("ItemData"),
    },
    FieldSpec {
        number: 13,
        name: "InventorySlotData",
        kind: Kind::Msg("InventorySlotData"),
    },
    FieldSpec {
        number: 14,
        name: "WeaponData",
        kind: Kind::MsgList("WeaponData"),
    },
    FieldSpec {
        number: 15,
        name: "StatsData",
        kind: Kind::Bytes,
    },
    FieldSpec {
        number: 16,
        name: "VisitedTeleporters",
        kind: Kind::StrList,
    },
    FieldSpec {
        number: 17,
        name: "LastVisitedTeleporter",
        kind: Kind::Str,
    },
    FieldSpec {
        number: 18,
        name: "MissionPlaythroughs",
        kind: Kind::MsgList("MissionPlaythroughData"),
    },
    FieldSpec {
        number: 19,
        name: "UIPreferences",
        kind: Kind::Msg("UIPreferencesData"),
    },
    FieldSpec {
        number: 20,
        name: "SaveGameId",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 21,
        name: "PlotMissionNumber",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 22,
        name: "Unknown22",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 23,
        name: "UsedMarketingCodes",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 24,
        name: "MarketingCodesNeedingNotification",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 25,
        name: "TotalPlayTime",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 26,
        name: "LastSavedDate",
        kind: Kind::Str,
    },
    FieldSpec {
        number: 27,
        name: "DLCExpansionData",
        kind: Kind::MsgList("DLCExpansionData"),
    },
    FieldSpec {
        number: 28,
        name: "Unknown28",
        kind: Kind::StrList,
    },
    FieldSpec {
        number: 29,
        name: "RegionGameStages",
        kind: Kind::MsgList("RegionGameStageData"),
    },
    FieldSpec {
        number: 30,
        name: "WorldDiscoveryList",
        kind: Kind::MsgList("WorldDiscoveryData"),
    },
    FieldSpec {
        number: 31,
        name: "IsBadassModeSaveGame",
        kind: Kind::Bool,
    },
    FieldSpec {
        number: 32,
        name: "WeaponMementos",
        kind: Kind::MsgList("WeaponMemento"),
    },
    FieldSpec {
        number: 33,
        name: "ItemMementos",
        kind: Kind::MsgList("ItemMemento"),
    },
    FieldSpec {
        number: 34,
        name: "SaveGuid",
        kind: Kind::Guid,
    },
    FieldSpec {
        number: 35,
        name: "AppliedCustomizations",
        kind: Kind::StrList,
    },
    FieldSpec {
        number: 36,
        name: "BlackMarketUpgrades",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 37,
        name: "ActiveMissionNumber",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 38,
        name: "ChallengeList",
        kind: Kind::MsgList("ChallengeData"),
    },
    FieldSpec {
        number: 39,
        name: "LevelChallengeUnlocks",
        kind: Kind::IntList,
    },
    FieldSpec {
        number: 40,
        name: "OneOffLevelChallengeCompletion",
        kind: Kind::MsgList("OneOffLevelChallengeData"),
    },
    FieldSpec {
        number: 41,
        name: "BankSlots",
        kind: Kind::MsgList("BankSlot"),
    },
    FieldSpec {
        number: 42,
        name: "NumChallengePrestiges",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 43,
        name: "LockoutList",
        kind: Kind::MsgList("LockoutData"),
    },
    FieldSpec {
        number: 44,
        name: "IsDLCPlayerClass",
        kind: Kind::Bool,
    },
    FieldSpec {
        number: 45,
        name: "DLCPlayerClassPackageId",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 46,
        name: "FullyExploredAreas",
        kind: Kind::StrList,
    },
    FieldSpec {
        number: 47,
        name: "Unknown47",
        kind: Kind::MsgList("GoldenKeys"),
    },
    FieldSpec {
        number: 48,
        name: "NumGoldenKeysNotified",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 49,
        name: "LastPlaythroughNumber",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 50,
        name: "ShowNewPlaythroughNotification",
        kind: Kind::Bool,
    },
    FieldSpec {
        number: 51,
        name: "ReceivedDefaultWeapon",
        kind: Kind::Bool,
    },
    FieldSpec {
        number: 52,
        name: "QueuedTrainingMessages",
        kind: Kind::StrList,
    },
    FieldSpec {
        number: 53,
        name: "PackedItemData",
        kind: Kind::MsgList("PackedItemData"),
    },
    FieldSpec {
        number: 54,
        name: "PackedWeaponData",
        kind: Kind::MsgList("PackedWeaponData"),
    },
    FieldSpec {
        number: 55,
        name: "AwesomeSkillDisabled",
        kind: Kind::Bool,
    },
    FieldSpec {
        number: 56,
        name: "MaxBankSlots",
        kind: Kind::Int,
    },
    FieldSpec {
        number: 57,
        name: "ChosenVehicleCustomizations",
        kind: Kind::MsgList("ChosenVehicleCustomization"),
    },
    FieldSpec {
        number: 58,
        name: "VehicleSteeringMode",
        kind: Kind::Int,
    },
];

/// A decoded `WillowTwoPlayerSaveGame`.
#[derive(Clone, Debug)]
pub struct SaveGame {
    pub message: Message,
}

impl SaveGame {
    pub fn parse(bytes: &[u8]) -> Result<SaveGame> {
        Ok(SaveGame {
            message: Message::parse(bytes)?,
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        self.message.to_bytes()
    }

    pub fn spec(number: u32) -> Option<&'static FieldSpec> {
        TOP_LEVEL.iter().find(|s| s.number == number)
    }

    pub fn field_name(number: u32) -> &'static str {
        SaveGame::spec(number).map(|s| s.name).unwrap_or("?")
    }

    // ---- scalar getters ----

    pub fn get_str(&self, number: u32) -> Option<String> {
        self.message
            .get(number)
            .and_then(|f| f.as_str())
            .map(|s| s.into_owned())
    }

    pub fn get_i32(&self, number: u32) -> Option<i32> {
        self.message.get(number).and_then(|f| f.as_i32())
    }

    pub fn get_bool(&self, number: u32) -> Option<bool> {
        self.message.get(number).and_then(|f| f.as_bool())
    }

    pub fn get_bytes(&self, number: u32) -> Option<&[u8]> {
        self.message.get(number).and_then(|f| f.as_bytes())
    }

    /// Unpacked repeated varints, in order. For packed fields the single
    /// length-delimited field is expanded here.
    pub fn get_int_list(&self, number: u32) -> Vec<i32> {
        let mut out = Vec::new();
        for field in self.message.get_all(number) {
            match field.wire {
                crate::proto::WireType::Varint => {
                    if let Some(v) = field.as_i32() {
                        out.push(v);
                    }
                }
                crate::proto::WireType::LengthDelimited => {
                    if let Some(bytes) = field.as_bytes() {
                        let mut pos = 0;
                        while pos < bytes.len() {
                            match crate::proto::read_varint(bytes, &mut pos) {
                                Ok(v) => out.push(v as u32 as i32),
                                Err(_) => break,
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// Repeated string fields, in order (field 16 style: one field per value).
    pub fn get_str_list(&self, number: u32) -> Vec<String> {
        self.message
            .get_all(number)
            .filter_map(|f| f.as_str())
            .map(|s| s.into_owned())
            .collect()
    }

    // ---- scalar setters ----

    pub fn set_str(&mut self, number: u32, value: &str) {
        self.message.set(Field::string(number, value));
    }

    pub fn set_i32(&mut self, number: u32, value: i32) {
        self.message.set(Field::varint(number, value as u32 as u64));
    }

    pub fn set_bool(&mut self, number: u32, value: bool) {
        self.message.set(Field::varint(number, u64::from(value)));
    }

    pub fn set_bytes(&mut self, number: u32, value: &[u8]) {
        self.message.set(Field::bytes(number, value));
    }

    /// Replace all occurrences of a packed varint list with one packed field.
    pub fn set_int_list_packed(&mut self, number: u32, values: &[i32]) {
        let mut payload = Vec::new();
        for &v in values {
            crate::proto::write_varint(&mut payload, v as u32 as u64);
        }
        self.message.remove_all(number);
        self.message.push(Field::bytes(number, &payload));
    }

    /// Replace a repeated int list, keeping the original packed/unpacked
    /// shape: one packed field when the save had one, one varint field per
    /// value when the save had unpacked elements.
    pub fn set_int_list(&mut self, number: u32, values: &[i32]) {
        let packed = match self.message.get(number) {
            Some(field) => field.wire == WireType::LengthDelimited,
            None => true,
        };
        if packed {
            self.set_int_list_packed(number, values);
            return;
        }
        self.message.remove_all(number);
        for &v in values {
            self.message
                .push(Field::varint(number, v as u32 as u64));
        }
    }

    /// Replace all occurrences of a repeated string field with `values`.
    pub fn set_str_list(&mut self, number: u32, values: &[String]) {
        self.message.remove_all(number);
        for value in values {
            self.message.push(Field::string(number, value));
        }
    }

    // ---- named convenience accessors ----

    pub fn player_class(&self) -> Option<String> {
        self.get_str(1)
    }
    pub fn exp_level(&self) -> Option<i32> {
        self.get_i32(2)
    }
    pub fn exp_points(&self) -> Option<i32> {
        self.get_i32(3)
    }
    pub fn general_skill_points(&self) -> Option<i32> {
        self.get_i32(4)
    }
    pub fn specialist_skill_points(&self) -> Option<i32> {
        self.get_i32(5)
    }
    pub fn currency_on_hand(&self) -> Vec<i32> {
        self.get_int_list(6)
    }
    pub fn playthroughs_completed(&self) -> Option<i32> {
        self.get_i32(7)
    }
    pub fn save_game_id(&self) -> Option<i32> {
        self.get_i32(20)
    }
    pub fn total_play_time(&self) -> Option<i32> {
        self.get_i32(25)
    }
    pub fn last_saved_date(&self) -> Option<String> {
        self.get_str(26)
    }
    pub fn num_golden_keys_notified(&self) -> Option<i32> {
        self.get_i32(48)
    }
    pub fn max_bank_slots(&self) -> Option<i32> {
        self.get_i32(56)
    }

    // ---- character ----

    /// The character name (`UIPreferencesData.CharacterName`, field 19.1).
    pub fn character_name(&self) -> Option<String> {
        let prefs = self.message.get(19)?.as_message().ok().flatten()?;
        let bytes = prefs.get(1)?.as_bytes()?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    }

    pub fn set_character_name(&mut self, name: &str) -> Result<()> {
        match self
            .message
            .get(19)
            .and_then(|f| f.as_bytes().map(|b| b.to_vec()))
        {
            Some(bytes) => {
                let mut prefs = Message::parse(&bytes)?;
                prefs.set(Field::bytes(1, name.as_bytes()));
                self.message.set(Field::bytes(19, &prefs.to_bytes()));
            }
            None => {
                let mut prefs = Message::default();
                prefs.push(Field::bytes(1, name.as_bytes()));
                self.message.push(Field::bytes(19, &prefs.to_bytes()));
            }
        }
        Ok(())
    }

    /// Applied customizations (field 35): index 0 is the head, index 4 the skin.
    pub fn applied_customizations(&self) -> Vec<String> {
        self.get_str_list(35)
    }

    /// Set the head (index 0) and skin (index 4) customization paths.
    pub fn set_applied_head_skin(&mut self, head: &str, skin: &str) {
        let mut list = self.get_str_list(35);
        while list.len() < 5 {
            list.push(String::new());
        }
        list[0] = head.to_string();
        list[4] = skin.to_string();
        self.set_str_list(35, &list);
    }

    // ---- fast travel ----

    pub fn visited_teleporters(&self) -> Vec<String> {
        self.get_str_list(16)
    }

    pub fn last_visited_teleporter(&self) -> Option<String> {
        self.get_str(17)
    }

    pub fn set_visited_teleporters(&mut self, list: &[String]) {
        self.set_str_list(16, list);
    }

    pub fn set_last_visited_teleporter(&mut self, name: &str) {
        self.set_str(17, name);
    }

    /// Add a teleporter if not already visited. Returns true if it was added.
    pub fn add_visited_teleporter(&mut self, name: &str) -> bool {
        let mut list = self.visited_teleporters();
        if list.iter().any(|entry| entry == name) {
            return false;
        }
        list.push(name.to_string());
        self.set_visited_teleporters(&list);
        true
    }

    /// Remove a teleporter. Returns true if it was present.
    pub fn remove_visited_teleporter(&mut self, name: &str) -> bool {
        let mut list = self.visited_teleporters();
        let before = list.len();
        list.retain(|entry| entry != name);
        if list.len() == before {
            return false;
        }
        self.set_visited_teleporters(&list);
        true
    }

    /// One annotated line per field, in wire order.
    pub fn describe(&self) -> Vec<String> {
        let mut lines = Vec::new();
        let mut i = 0;
        while i < self.message.fields.len() {
            let field = &self.message.fields[i];
            let number = field.number;
            let count = self.message.count(number);
            let spec = SaveGame::spec(number);
            let name = spec.map(|s| s.name).unwrap_or("(unknown)");
            let kind = spec.map(|s| s.kind);
            let rendered = render_field(field, kind);
            lines.push(format!(
                "#{number:<3} {name:<32} {rendered}{}",
                if count > 1 {
                    format!("  [x{count}]")
                } else {
                    String::new()
                }
            ));
            // Skip the remaining duplicates of this number.
            i += count;
        }
        lines
    }
}

fn render_field(field: &Field, kind: Option<Kind>) -> String {
    use crate::proto::WireType;
    match kind {
        Some(Kind::Str) => match field.as_str() {
            Some(s) => format!("str = {:?}", s),
            None => "str = <invalid>".to_string(),
        },
        Some(Kind::Int) => match field.as_i32() {
            Some(v) => format!("int = {v}"),
            None => "int = <invalid>".to_string(),
        },
        Some(Kind::Bool) => match field.as_bool() {
            Some(v) => format!("bool = {v}"),
            None => "bool = <invalid>".to_string(),
        },
        Some(Kind::Bytes) => match field.as_bytes() {
            Some(b) => format!("bytes = {} bytes", b.len()),
            None => "bytes = <invalid>".to_string(),
        },
        Some(Kind::IntList) => {
            let mut vals = Vec::new();
            match field.wire {
                WireType::Varint => {
                    if let Some(v) = field.as_i32() {
                        vals.push(v);
                    }
                }
                WireType::LengthDelimited => {
                    if let Some(bytes) = field.as_bytes() {
                        let mut pos = 0;
                        while pos < bytes.len() {
                            match crate::proto::read_varint(bytes, &mut pos) {
                                Ok(v) => vals.push(v as u32 as i32),
                                Err(_) => break,
                            }
                        }
                    }
                }
                _ => {}
            }
            format!("int list = {vals:?}")
        }
        Some(Kind::StrList) => match field.as_str() {
            Some(s) => format!("str = {:?}", s),
            None => "str = <invalid>".to_string(),
        },
        Some(Kind::Guid) => match field.as_bytes() {
            Some(b) => format!("guid = {} bytes ({:02x?})", b.len(), &b[..b.len().min(16)]),
            None => "guid = <invalid>".to_string(),
        },
        Some(Kind::Msg(type_name)) | Some(Kind::MsgList(type_name)) => match field.as_bytes() {
            Some(b) => format!("{type_name} = {} bytes", b.len()),
            None => format!("{type_name} = <invalid>"),
        },
        None => match field.wire {
            WireType::Varint => format!("varint = {:?}", field.as_varint()),
            WireType::LengthDelimited => {
                format!(
                    "len-delimited = {} bytes",
                    field.as_bytes().map(|b| b.len()).unwrap_or(0)
                )
            }
            WireType::Fixed32 => format!("fixed32 = {:?}", field.as_fixed32()),
            WireType::Fixed64 => format!("fixed64 = {:?}", field.as_fixed64()),
            WireType::StartGroup | WireType::EndGroup => "group".to_string(),
        },
    }
}

/// A `SkillData` entry (field 8): one skill-tree node's rank and points.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Skill {
    pub skill: String,
    pub grade: i32,
    pub grade_points: i32,
    pub equipped_slot_index: i32,
}

/// A `ChosenVehicleCustomization` entry (field 57): a vehicle family and the
/// skin customizations chosen for it (slot 0, slot 1, then any extras).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VehicleCustomization {
    pub family: String,
    pub customizations: Vec<String>,
}

/// The vehicle families the game recognises, as `(usage, family_resource)`.
/// `usage` is the key into `Customizations.json` used to offer skin choices.
pub const VEHICLE_FAMILIES: &[(&str, &str)] = &[    (
        "Runner",
        "GD_Globals.VehicleSpawnStation.VehicleFamily_Runner",
    ),
    (
        "BanditTech",
        "GD_Globals.VehicleSpawnStation.VehicleFamily_BanditTechnical",
    ),
    (
        "Hovercraft",
        "GD_OrchidPackageDef.Vehicles.VehicleFamily_Hovercraft",
    ),
    (
        "FanBoat",
        "GD_SagePackageDef.Vehicles.VehicleFamily_FanBoat",
    ),
];

/// `MissionStatus` names, indexed by the value stored in the save.
pub const MISSION_STATUSES: &[&str] = &[
    "NotStarted",
    "Active",
    "RequiredObjectivesComplete",
    "ReadyToTurnIn",
    "Complete",
    "Failed",
];

/// The name of a `MissionStatus` value.
/// 16 pseudo-random bytes for [`SaveGame::refresh_guid`]. Seeded from the
/// clock with an xorshift step; a tiny non-cryptographic stream is fine for a
/// save identifier.
fn random_guid() -> [u8; 16] {
    let mut seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0x9E37_79B9_7F4A_7C15)
        | 1;
    let mut out = [0u8; 16];
    for chunk in out.chunks_mut(8) {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        chunk.copy_from_slice(&seed.to_le_bytes()[..chunk.len()]);
    }
    out
}

pub fn mission_status_name(value: i32) -> &'static str {
    MISSION_STATUSES
        .get(value as usize)
        .copied()
        .unwrap_or("?")
}

/// `PlayerMark` names, indexed by value.
pub const PLAYER_MARKS: &[&str] = &["Trash", "Standard", "Favorite"];

/// The name of a `PlayerMark` value.
pub fn player_mark_name(value: i32) -> &'static str {
    PLAYER_MARKS
        .get(value as usize)
        .copied()
        .unwrap_or("?")
}

/// A `ChallengeData` entry (field 38).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Challenge {
    pub challenge: String,
    pub is_from_dlc: bool,
    pub dlc_package_id: i32,
}

/// A `MissionData` entry inside a playthrough.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionEntry {
    pub mission: String,
    pub status: i32,
    pub is_from_dlc: bool,
    pub dlc_package_id: i32,
    pub active_objective_set_index: i32,
    pub needs_rewards: bool,
    pub heard_kickoff: bool,
    pub game_stage: i32,
}

/// A `MissionPlaythroughData` entry (field 18).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MissionPlaythrough {
    pub playthrough_number: Option<i32>,
    pub active_mission: String,
    pub missions: Vec<MissionEntry>,
    pub pending_rewards: usize,
    pub filtered_missions: Vec<String>,
}

/// Which top-level list a packed blob lives in.
///
/// `PackedItemData`/`PackedWeaponData` (fields 53/54) are the backpack;
/// `BankSlot` (field 41) is the shared stash. All three hold an
/// `InventorySerialNumber` blob, so an entry's real kind (item vs weapon) comes
/// from decoding the blob, not from the container.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Container {
    /// Field 53, `PackedItemData` — backpack items (quantity/equipped/mark).
    BackpackItem,
    /// Field 54, `PackedWeaponData` — backpack weapons (quick slot/mark).
    BackpackWeapon,
    /// Field 41, `BankSlot` — the shared stash (serial only).
    Bank,
}

impl Container {
    /// Containers in the order reported by [`SaveGame::entries`].
    pub const ALL: [Container; 3] = [
        Container::BackpackItem,
        Container::BackpackWeapon,
        Container::Bank,
    ];

    /// The top-level field number this container lives in.
    pub fn field_number(self) -> u32 {
        match self {
            Container::BackpackItem => 53,
            Container::BackpackWeapon => 54,
            Container::Bank => 41,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Container::BackpackItem => "backpack item",
            Container::BackpackWeapon => "backpack weapon",
            Container::Bank => "bank",
        }
    }
}

/// A packed item/weapon entry: which list it lives in, its occurrence index
/// within that list, its `InventorySerialNumber` bytes, and — for backpack
/// entries — the surrounding metadata fields.
#[derive(Clone, Debug)]
pub struct PackedEntry {
    pub container: Container,
    pub field_number: u32,
    pub occurrence: usize,
    pub serial: Vec<u8>,
    /// `PackedItemData.Quantity` (field 2); 1 for weapons/bank.
    pub quantity: i32,
    /// `PackedItemData.Equipped` (field 3); false for weapons/bank.
    pub equipped: bool,
    /// `PackedWeaponData.QuickSlot` (field 2); -1 for items/bank.
    pub quick_slot: i32,
    /// `PlayerMark` (item field 4 / weapon field 3); 0 when absent.
    pub mark: i32,
}

impl SaveGame {
    /// Packed blobs in one container, in order.
    pub fn entries_in(&self, container: Container) -> Vec<PackedEntry> {
        let field_number = container.field_number();
        let mut out = Vec::new();
        for (occurrence, field) in self.message.get_all(field_number).enumerate() {
            let Ok(Some(sub)) = field.as_message() else {
                continue;
            };
            let Some(serial) = sub.get(1).and_then(|f| f.as_bytes()) else {
                continue;
            };
            let (quantity, equipped, quick_slot, mark) = match container {
                Container::BackpackItem => (
                    sub.get(2).and_then(|f| f.as_i32()).unwrap_or(1),
                    sub.get(3).and_then(|f| f.as_bool()).unwrap_or(false),
                    -1,
                    sub.get(4).and_then(|f| f.as_i32()).unwrap_or(0),
                ),
                Container::BackpackWeapon => (
                    1,
                    false,
                    sub.get(2).and_then(|f| f.as_i32()).unwrap_or(-1),
                    sub.get(3).and_then(|f| f.as_i32()).unwrap_or(0),
                ),
                Container::Bank => (1, false, -1, 0),
            };
            out.push(PackedEntry {
                container,
                field_number,
                occurrence,
                serial: serial.to_vec(),
                quantity,
                equipped,
                quick_slot,
                mark,
            });
        }
        out
    }

    /// Backpack blobs: items (field 53) then weapons (field 54).
    pub fn packed_entries(&self) -> Vec<PackedEntry> {
        let mut out = self.entries_in(Container::BackpackItem);
        out.extend(self.entries_in(Container::BackpackWeapon));
        out
    }

    /// Bank blobs (field 41).
    pub fn bank_entries(&self) -> Vec<PackedEntry> {
        self.entries_in(Container::Bank)
    }

    /// Every packed blob: backpack (items, weapons) then bank.
    pub fn entries(&self) -> Vec<PackedEntry> {
        let mut out = self.packed_entries();
        out.extend(self.bank_entries());
        out
    }

    /// Replace the `InventorySerialNumber` (field 1) of one packed entry,
    /// rebuilding that entry's bytes while leaving every other field untouched.
    pub fn replace_serial(
        &mut self,
        field_number: u32,
        occurrence: usize,
        new_serial: &[u8],
    ) -> crate::error::Result<()> {
        self.mutate_entry(field_number, occurrence, |sub| {
            sub.set(Field::bytes(1, new_serial));
        })
    }

    /// Apply `f` to one entry's sub-message (matched by top-level field number
    /// and occurrence), rebuilding only that entry's bytes.
    pub fn mutate_entry<F: FnOnce(&mut Message)>(
        &mut self,
        field_number: u32,
        occurrence: usize,
        f: F,
    ) -> crate::error::Result<()> {
        let mut seen = 0usize;
        for field in &mut self.message.fields {
            if field.number != field_number {
                continue;
            }
            if seen == occurrence {
                let sub_bytes = field
                    .as_bytes()
                    .ok_or_else(|| {
                        crate::error::Error::Proto(
                            "packed entry is not length-delimited".into(),
                        )
                    })?
                    .to_vec();
                let mut sub = Message::parse(&sub_bytes)?;
                f(&mut sub);
                *field = Field::bytes(field_number, &sub.to_bytes());
                return Ok(());
            }
            seen += 1;
        }
        Err(crate::error::Error::Proto(format!(
            "no packed entry {occurrence} in field {field_number}"
        )))
    }

    /// Append a new entry (serial only) to `container`. Returns its occurrence.
    pub fn append_entry(
        &mut self,
        container: Container,
        serial: &[u8],
    ) -> crate::error::Result<usize> {
        let mut sub = Message::default();
        sub.push(Field::bytes(1, serial));
        self.message
            .push(Field::bytes(container.field_number(), &sub.to_bytes()));
        Ok(self.message.count(container.field_number()) - 1)
    }

    /// Remove one entry from `container`. Later entries shift down.
    pub fn remove_entry(
        &mut self,
        container: Container,
        occurrence: usize,
    ) -> crate::error::Result<()> {
        let field_number = container.field_number();
        let mut seen = 0usize;
        let mut target = None;
        for (i, field) in self.message.fields.iter().enumerate() {
            if field.number != field_number {
                continue;
            }
            if seen == occurrence {
                target = Some(i);
                break;
            }
            seen += 1;
        }
        match target {
            Some(i) => {
                self.message.fields.remove(i);
                Ok(())
            }
            None => Err(crate::error::Error::Proto(format!(
                "no {} entry {occurrence}",
                container.label()
            ))),
        }
    }

    /// Move one entry from `from` to the end of `to`. Returns the new
    /// occurrence in `to`.
    pub fn move_entry(
        &mut self,
        from: Container,
        occurrence: usize,
        to: Container,
    ) -> crate::error::Result<usize> {
        if from == to {
            return Ok(occurrence);
        }
        let serial = self
            .entries_in(from)
            .get(occurrence)
            .map(|e| e.serial.clone())
            .ok_or_else(|| {
                crate::error::Error::Proto(format!("no {} entry {occurrence}", from.label()))
            })?;
        self.remove_entry(from, occurrence)?;
        self.append_entry(to, &serial)
    }

    /// Set `PackedItemData.Quantity` (field 2) on a backpack item.
    pub fn set_entry_quantity(
        &mut self,
        container: Container,
        occurrence: usize,
        quantity: i32,
    ) -> crate::error::Result<()> {
        self.mutate_entry(container.field_number(), occurrence, |sub| {
            sub.set(Field::varint(2, quantity as u32 as u64));
        })
    }

    /// Set `PackedItemData.Equipped` (field 3) on a backpack item.
    pub fn set_entry_equipped(
        &mut self,
        container: Container,
        occurrence: usize,
        equipped: bool,
    ) -> crate::error::Result<()> {
        self.mutate_entry(container.field_number(), occurrence, |sub| {
            sub.set(Field::varint(3, u64::from(equipped)));
        })
    }

    /// Set `PackedWeaponData.QuickSlot` (field 2) on a backpack weapon.
    pub fn set_entry_quick_slot(
        &mut self,
        container: Container,
        occurrence: usize,
        slot: i32,
    ) -> crate::error::Result<()> {
        self.mutate_entry(container.field_number(), occurrence, |sub| {
            sub.set(Field::varint(2, slot as u32 as u64));
        })
    }

    /// Set the `PlayerMark` of an entry (item field 4 / weapon field 3).
    pub fn set_entry_mark(
        &mut self,
        container: Container,
        occurrence: usize,
        mark: i32,
    ) -> crate::error::Result<()> {
        let field = if container == Container::BackpackWeapon {
            3
        } else {
            4
        };
        self.mutate_entry(container.field_number(), occurrence, |sub| {
            sub.set(Field::varint(field, mark as u32 as u64));
        })
    }

    /// All `SkillData` entries (field 8), in order.
    pub fn skills(&self) -> Vec<Skill> {
        self.message
            .get_all(8)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| Skill {
                skill: sub
                    .get(1)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                grade: sub.get(2).and_then(|f| f.as_i32()).unwrap_or(0),
                grade_points: sub.get(3).and_then(|f| f.as_i32()).unwrap_or(0),
                equipped_slot_index: sub.get(4).and_then(|f| f.as_i32()).unwrap_or(0),
            })
            .collect()
    }

    /// Set `SkillData.Grade` (field 2) on skill `index`.
    pub fn set_skill_grade(&mut self, index: usize, grade: i32) -> crate::error::Result<()> {
        self.mutate_entry(8, index, |sub| {
            sub.set(Field::varint(2, grade as u32 as u64));
        })
    }

    /// Set `SkillData.GradePoints` (field 3) on skill `index`.
    pub fn set_skill_grade_points(
        &mut self,
        index: usize,
        points: i32,
    ) -> crate::error::Result<()> {
        self.mutate_entry(8, index, |sub| {
            sub.set(Field::varint(3, points as u32 as u64));
        })
    }

    // ---- vehicles ----

    /// All `ChosenVehicleCustomization` entries (field 57), in order.
    pub fn vehicles(&self) -> Vec<VehicleCustomization> {
        self.message
            .get_all(57)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| VehicleCustomization {
                family: sub
                    .get(1)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                customizations: sub
                    .get_all(2)
                    .filter_map(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .collect(),
            })
            .collect()
    }

    /// `VehicleSteeringMode` (field 58), if present.
    pub fn vehicle_steering_mode(&self) -> Option<i32> {
        self.get_i32(58)
    }

    pub fn set_vehicle_steering_mode(&mut self, mode: i32) {
        self.set_i32(58, mode);
    }

    /// Replace the whole `ChosenVehicleCustomizations` list (field 57).
    pub fn set_vehicles(&mut self, vehicles: &[VehicleCustomization]) {
        self.message.remove_all(57);
        for vehicle in vehicles {
            let mut sub = Message::default();
            sub.push(Field::string(1, &vehicle.family));
            for customization in &vehicle.customizations {
                sub.push(Field::string(2, customization));
            }
            self.message.push(Field::bytes(57, &sub.to_bytes()));
        }
    }

    // ---- section import, fresh characters and blank saves ----

    /// Replace all occurrences of the given top-level fields with `other`'s.
    /// `other`'s fields are inserted at the position of the first existing
    /// occurrence (appended when the save has none); everything else stays
    /// byte-identical.
    pub fn import_fields_from(&mut self, other: &SaveGame, numbers: &[u32]) {
        for &number in numbers {
            let incoming: Vec<Field> = other.message.get_all(number).cloned().collect();
            let at = self
                .message
                .fields
                .iter()
                .position(|f| f.number == number)
                .unwrap_or(self.message.fields.len());
            self.message.remove_all(number);
            for (offset, field) in incoming.into_iter().enumerate() {
                self.message.fields.insert(at + offset, field);
            }
        }
    }

    /// Rotate the save GUID — a fresh identity for a cloned or blank save.
    pub fn refresh_guid(&mut self) {
        self.message
            .set(Field::bytes(34, &random_guid()));
    }

    /// Reset character-specific progress so a cloned save behaves like a new
    /// game: chosen class, level 1, exp 0, 3 skill points, zeroed currencies,
    /// no items/weapons/bank. Missions, travel and challenges are left alone
    /// so the save's structure stays recognisably valid to the game.
    pub fn make_fresh_character(&mut self, player_class: &str) {
        self.set_str(1, player_class);
        self.set_i32(2, 1);
        self.set_i32(3, 0);
        self.set_i32(4, 3);
        self.set_i32(5, 0);
        self.set_int_list_packed(6, &[0, 0, 0, 0]);
        self.set_i32(7, 0);
        self.set_i32(48, 0);
        self.set_i32(49, 0);
        self.message.remove_all(12); // ItemData
        self.message.remove_all(14); // WeaponData
        self.message.remove_all(41); // BankSlots
        self.message.remove_all(53); // PackedItemData
        self.message.remove_all(54); // PackedWeaponData
        self.refresh_guid();
    }

    /// A minimal, spec-driven blank save for `player_class` (level 1, fresh
    /// character defaults). The game is stricter about a from-scratch save
    /// than about a cloned [`SaveGame::make_fresh_character`] save, so treat
    /// blank saves as experimental and verify in game.
    pub fn blank(player_class: &str) -> SaveGame {
        let mut save = SaveGame {
            message: Message::default(),
        };
        save.set_str(1, player_class);
        save.set_i32(2, 1); // ExpLevel
        save.set_i32(3, 0); // ExpPoints
        save.set_i32(4, 3); // GeneralSkillPoints
        save.set_i32(5, 0); // SpecialistSkillPoints
        save.set_int_list_packed(6, &[0, 0, 0, 0]); // money/eridium/seraph/torgue
        save.set_i32(7, 0); // PlaythroughsCompleted
        save.set_i32(20, 0); // SaveGameId
        save.set_str(26, ""); // LastSavedDate
        save.refresh_guid();
        save.set_int_list_packed(36, &[0, 0, 0, 0]); // BlackMarketUpgrades
        save.set_i32(37, 0); // ActiveMissionNumber
        save.set_bool(50, false); // ShowNewPlaythroughNotification
        save.set_bool(51, false); // ReceivedDefaultWeapon
        save.set_i32(48, 0); // NumGoldenKeysNotified
        save.set_i32(49, 0); // LastPlaythroughNumber
        save.set_bool(55, false); // AwesomeSkillDisabled
        save.set_i32(58, 0); // VehicleSteeringMode
        let mut slots = Message::default(); // InventorySlotData (field 13)
        slots.push(Field::varint(1, 12)); // backpack max
        slots.push(Field::varint(2, 2)); // weapon ready max
        slots.push(Field::varint(3, 2)); // quick slots flourished
        save.message.set(Field::bytes(13, &slots.to_bytes()));
        save
    }

    // ---- challenges & missions ----

    /// All `ChallengeData` entries (field 38), in order.
    pub fn challenges(&self) -> Vec<Challenge> {
        self.message
            .get_all(38)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| Challenge {
                challenge: sub
                    .get(1)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                is_from_dlc: sub.get(2).and_then(|f| f.as_bool()).unwrap_or(false),
                dlc_package_id: sub.get(3).and_then(|f| f.as_i32()).unwrap_or(0),
            })
            .collect()
    }

    pub fn num_challenge_prestiges(&self) -> Option<i32> {
        self.get_i32(42)
    }

    pub fn plot_mission_number(&self) -> Option<i32> {
        self.get_i32(21)
    }

    pub fn set_plot_mission_number(&mut self, value: i32) {
        self.set_i32(21, value);
    }

    pub fn active_mission_number(&self) -> Option<i32> {
        self.get_i32(37)
    }

    pub fn set_active_mission_number(&mut self, value: i32) {
        self.set_i32(37, value);
    }

    /// All `MissionPlaythroughData` entries (field 18), with their missions.
    pub fn mission_playthroughs(&self) -> Vec<MissionPlaythrough> {
        self.message
            .get_all(18)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|pt| MissionPlaythrough {
                playthrough_number: pt.get(1).and_then(|f| f.as_i32()),
                active_mission: pt
                    .get(2)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                missions: pt
                    .get_all(3)
                    .filter_map(|f| f.as_message().ok().flatten())
                    .map(|md| MissionEntry {
                        mission: md
                            .get(1)
                            .and_then(|f| f.as_str())
                            .map(|s| s.into_owned())
                            .unwrap_or_default(),
                        status: md.get(2).and_then(|f| f.as_i32()).unwrap_or(0),
                        is_from_dlc: md.get(3).and_then(|f| f.as_bool()).unwrap_or(false),
                        dlc_package_id: md.get(4).and_then(|f| f.as_i32()).unwrap_or(0),
                        active_objective_set_index: md.get(6).and_then(|f| f.as_i32()).unwrap_or(0),
                        needs_rewards: md.get(8).and_then(|f| f.as_bool()).unwrap_or(false),
                        heard_kickoff: md.get(10).and_then(|f| f.as_bool()).unwrap_or(false),
                        game_stage: md.get(11).and_then(|f| f.as_i32()).unwrap_or(0),
                    })
                    .collect(),
                pending_rewards: pt.count(4),
                filtered_missions: pt
                    .get_all(5)
                    .filter_map(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .collect(),
            })
            .collect()
    }

    /// Set `MissionData.Status` (field 2) on mission `mission` of playthrough
    /// `playthrough`.
    pub fn set_mission_status(
        &mut self,
        playthrough: usize,
        mission: usize,
        status: i32,
    ) -> Result<()> {
        let mut pt_seen = 0usize;
        for field in &mut self.message.fields {
            if field.number != 18 {
                continue;
            }
            if pt_seen == playthrough {
                let pt_bytes = field
                    .as_bytes()
                    .ok_or_else(|| Error::Proto("playthrough is not length-delimited".into()))?
                    .to_vec();
                let mut pt = Message::parse(&pt_bytes)?;
                let mut md_seen = 0usize;
                let mut found = false;
                for md_field in &mut pt.fields {
                    if md_field.number != 3 {
                        continue;
                    }
                    if md_seen == mission {
                        let md_bytes = md_field
                            .as_bytes()
                            .ok_or_else(|| {
                                Error::Proto("mission entry is not length-delimited".into())
                            })?
                            .to_vec();
                        let mut md = Message::parse(&md_bytes)?;
                        md.set(Field::varint(2, status as u32 as u64));
                        *md_field = Field::bytes(3, &md.to_bytes());
                        found = true;
                        break;
                    }
                    md_seen += 1;
                }
                if !found {
                    return Err(Error::Proto(format!(
                        "no mission {mission} in playthrough {playthrough}"
                    )));
                }
                *field = Field::bytes(18, &pt.to_bytes());
                return Ok(());
            }
            pt_seen += 1;
        }
        Err(Error::Proto(format!("no playthrough {playthrough}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_fields_replaces_at_first_position() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.set_i32(2, 72);
        save.message.push(Field::string(16, "keep-a"));

        let mut other = SaveGame::blank("GD_Siren.Character.CharClass_Siren");
        other.set_i32(2, 5);
        other.message.push(Field::string(16, "from-a"));
        other.message.push(Field::string(16, "from-b"));

        save.import_fields_from(&other, &[2, 16]);

        // Playerclass untouched by field-2 import.
        assert_eq!(save.player_class(), Some("GD_Soldier.Character.CharClass_Soldier".into()));
        assert_eq!(save.exp_level(), Some(5));
        let teleporters: Vec<String> = save
            .message
            .get_all(16)
            .filter_map(|f| f.as_str().map(|s| s.into_owned()))
            .collect();
        assert_eq!(teleporters, ["from-a", "from-b"]);
        // Field order: the imported teleporters sit where the first one was.
        let order: Vec<u32> = save.message.fields.iter().map(|f| f.number).collect();
        let first = order.iter().position(|n| *n == 16).unwrap();
        assert_eq!(order[first], 16);
    }

    #[test]
    fn import_missing_field_clears_it() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.message.push(Field::string(16, "gone"));
        let other = SaveGame::blank("GD_Siren.Character.CharClass_Siren");
        save.import_fields_from(&other, &[16]);
        assert_eq!(save.message.count(16), 0);
    }

    #[test]
    fn blank_save_round_trips_through_container() {
        let save = SaveGame::blank("GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass");
        assert_eq!(save.player_class(), Some("GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass".into()));
        assert_eq!(save.exp_level(), Some(1));
        assert_eq!(save.general_skill_points(), Some(3));
        assert_eq!(save.currency_on_hand(), [0, 0, 0, 0]);
        assert!(save.get_bytes(34).is_some_and(|g| g.len() == 16));
    }

    #[test]
    fn fresh_character_resets_progress() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.set_i32(2, 72);
        save.set_i32(3, 999_999);
        save.set_i32(4, 100);
        save.message.push(Field::bytes(53, &[1, 2, 3, 4]));
        save.make_fresh_character("GD_Siren.Character.CharClass_Siren");
        assert_eq!(save.player_class(), Some("GD_Siren.Character.CharClass_Siren".into()));
        assert_eq!(save.exp_level(), Some(1));
        assert_eq!(save.general_skill_points(), Some(3));
        assert_eq!(save.message.count(53), 0);
    }

    #[test]
    fn refresh_guid_changes_identity() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        let before = save.get_bytes(34).map(|b| b.to_vec()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(2));
        save.refresh_guid();
        let after = save.get_bytes(34).map(|b| b.to_vec()).unwrap();
        assert_eq!(after.len(), 16);
        assert_ne!(before, after);
    }
}

#[cfg(test)]
mod raw_list_tests {
    use super::*;
    use crate::proto::Field;

    #[test]
    fn set_int_list_keeps_packed_shape() {
        // blank() writes CurrencyOnHand (6) packed as one LEN field.
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.set_int_list(6, &[1000, 50, 10]);
        assert_eq!(save.message.count(6), 1, "stays packed");
        assert_eq!(save.currency_on_hand(), [1000, 50, 10]);
    }

    #[test]
    fn set_int_list_keeps_unpacked_shape() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.message.remove_all(9);
        save.message.push(Field::varint(9, 1));
        save.message.push(Field::varint(9, 2));
        save.message.push(Field::varint(9, 3));
        save.set_int_list(9, &[7, 8]);
        assert_eq!(save.message.count(9), 2, "stays unpacked");
        let values: Vec<i32> = save
            .message
            .get_all(9)
            .filter_map(|f| f.as_i32())
            .collect();
        assert_eq!(values, [7, 8]);
    }
}

/// One ammo/resource pool entry (field 11 `ResourceData`): the resource path,
/// its pool, the current amount, and the SDU upgrade level.
#[derive(Clone, Debug, PartialEq)]
pub struct Resource {
    pub resource: String,
    pub pool: String,
    pub amount: f32,
    pub upgrade_level: i32,
}

impl Resource {
    /// Player-facing name (`D_Resourcepools.AmmoPools.Ammo_Combat_Rifle_Pool`
    /// → "Combat Rifle").
    pub fn label(&self) -> String {
        let last = self.pool.rsplit('.').next().unwrap_or(&self.pool);
        let name = last.strip_suffix("_Pool").unwrap_or(last);
        name.strip_prefix("Ammo_").unwrap_or(name)
            .replace('_', " ")
    }
}

impl SaveGame {
    /// All `ResourceData` entries (field 11), in order.
    pub fn resources(&self) -> Vec<Resource> {
        self.message
            .get_all(11)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| Resource {
                resource: sub
                    .get(1)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                pool: sub
                    .get(2)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                amount: sub
                    .get(3)
                    .and_then(|f| f.as_fixed32())
                    .map(f32::from_bits)
                    .unwrap_or(0.0),
                upgrade_level: sub
                    .get(4)
                    .and_then(|f| f.as_i32())
                    .unwrap_or(0),
            })
            .collect()
    }

    /// Mutate the `ResourceData` entry at `index`, re-encoding it in place.
    pub fn mutate_resource(
        &mut self,
        index: usize,
        change: impl FnOnce(&mut Resource),
    ) -> Result<()> {
        for (seen, field) in self
            .message
            .fields
            .iter_mut()
            .filter(|f| f.number == 11)
            .enumerate()
        {
            if seen != index {
                continue;
            }
            let Some(mut sub) = field.as_message().ok().flatten() else {
                return Err(Error::Proto("resource entry is not a message".into()));
            };
            let mut resource = Resource {
                resource: sub
                    .get(1)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                pool: sub
                    .get(2)
                    .and_then(|f| f.as_str())
                    .map(|s| s.into_owned())
                    .unwrap_or_default(),
                amount: sub
                    .get(3)
                    .and_then(|f| f.as_fixed32())
                    .map(f32::from_bits)
                    .unwrap_or(0.0),
                upgrade_level: sub.get(4).and_then(|f| f.as_i32()).unwrap_or(0),
            };
            change(&mut resource);
            sub.set(Field::string(1, &resource.resource));
            sub.set(Field::string(2, &resource.pool));
            sub.set(Field::fixed32(3, resource.amount.to_bits()));
            sub.set(Field::varint(4, resource.upgrade_level as u32 as u64));
            *field = Field::bytes(11, &sub.to_bytes());
            return Ok(());
        }
        Err(Error::Proto(format!("no resource entry #{index}")))
    }
}

#[cfg(test)]
mod resource_tests {
    use super::*;

    #[test]
    fn resources_round_trip_and_edit_in_place() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        for (pool, amount, level) in [
            ("D_Resourcepools.AmmoPools.Ammo_Combat_Rifle_Pool", 273.0f32, 2),
            ("D_Resourcepools.AmmoPools.Ammo_Grenade_Protean_Pool", 4.0, 0),
        ] {
            let mut sub = Message::default();
            sub.push(Field::string(1, "D_Resources.AmmoResources.Ammo_Test"));
            sub.push(Field::string(2, pool));
            sub.push(Field::fixed32(3, amount.to_bits()));
            sub.push(Field::varint(4, level as u32 as u64));
            save.message.push(Field::bytes(11, &sub.to_bytes()));
        }
        let loaded = save.resources();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded[0].label(), "Combat Rifle");
        assert_eq!(loaded[0].upgrade_level, 2);

        save.mutate_resource(1, |r| {
            r.amount = 7.5;
            r.upgrade_level = 4;
        })
        .expect("edit");

        let reload = save.resources();
        assert_eq!(reload[1].amount, 7.5);
        assert_eq!(reload[1].upgrade_level, 4);
        // The other entry is untouched.
        assert_eq!(reload[0].amount, 273.0);
        assert_eq!(reload[0].upgrade_level, 2);
    }
}

/// The nine `BlackMarketUpgrades` (field 36) entries, in save order (0-based):
/// seven ammo pools, then the backpack SDU (7) and the bank SDU (8).
pub const BLACK_MARKET_KEYS: [&str; 9] = [
    "Assault rifle",
    "Pistol",
    "Rocket launcher",
    "Shotgun",
    "SMG",
    "Sniper rifle",
    "Grenade",
    "Backpack",
    "Bank",
];

/// Vanilla SDU caps: ammo tops out at 7, backpack/bank at 9 (per the Black
/// Market's own price ladder). Community edits go higher; the game clamps
/// what it dislikes, but stay within these unless you know better.
pub const SDU_LEVEL_MAX: [i32; 9] = [7, 7, 7, 7, 7, 7, 7, 9, 9];

pub const BACKPACK_SLOT_BASE: i32 = 12;
pub const BACKPACK_SLOTS_PER_SDU: i32 = 3;
pub const BANK_SLOT_BASE: i32 = 6;
pub const BANK_SLOTS_PER_SDU: i32 = 2;

impl SaveGame {
    /// `BlackMarketUpgrades` (field 36) as a list of SDU levels (0–…), padded
    /// to nine entries.
    pub fn black_market_upgrades(&self) -> [i32; 9] {
        let values = self.get_int_list(36);
        let mut out = [0i32; 9];
        for (i, v) in out.iter_mut().enumerate() {
            *v = values.get(i).copied().unwrap_or(0);
        }
        out
    }

    /// Replace the whole `BlackMarketUpgrades` list.
    pub fn set_black_market_upgrades(&mut self, levels: &[i32; 9]) {
        self.set_int_list(36, levels);
    }

    /// `InventorySlotData` (field 13): `(backpack max, weapon-ready max, quick
    /// slots used)`; `None` when the field is missing.
    pub fn inventory_slots(&self) -> Option<(i32, i32, i32)> {
        let sub = self
            .message
            .get(13)
            .and_then(|f| f.as_message().ok().flatten())?;
        Some((
            sub.get(1).and_then(|f| f.as_i32()).unwrap_or(0),
            sub.get(2).and_then(|f| f.as_i32()).unwrap_or(0),
            sub.get(3).and_then(|f| f.as_i32()).unwrap_or(0),
        ))
    }

    /// Set the backpack's slot max (field 13's first int, keeping the rest);
    /// creates the field when the save has none.
    pub fn set_backpack_slot_max(&mut self, slots: i32) -> Result<()> {
        let mut sub = self
            .message
            .get(13)
            .and_then(|f| f.as_message().ok().flatten())
            .unwrap_or_default();
        sub.set(Field::varint(1, slots.max(0) as u64));
        self.message.set(Field::bytes(13, &sub.to_bytes()));
        Ok(())
    }

    /// Set one Black Market SDU level, with consistency writes: the backpack
    /// SDU (index 7) rewrites `InventorySlotData`'s slot max (12 + 3·level),
    /// and the bank SDU (index 8) rewrites `MaxBankSlots` (6 + 2·level).
    pub fn set_sdu(&mut self, index: usize, level: i32) -> Result<()> {
        if index >= 9 {
            return Err(Error::Proto(format!("no SDU #{index}")));
        }
        let mut levels = self.black_market_upgrades();
        levels[index] = level.max(0);
        self.set_black_market_upgrades(&levels);
        match index {
            7 => self.set_backpack_slot_max(
                BACKPACK_SLOT_BASE + BACKPACK_SLOTS_PER_SDU * levels[7].max(0),
            )?,
            8 => self.set_i32(56, BANK_SLOT_BASE + BANK_SLOTS_PER_SDU * levels[8].max(0)),
            _ => {}
        }
        Ok(())
    }

    /// Set every SDU to its vanilla cap and apply the consistent slot counts
    /// (backpack 39, bank 24).
    pub fn max_all_sdus(&mut self) -> Result<()> {
        let mut levels = [0i32; 9];
        for (i, level) in levels.iter_mut().enumerate() {
            *level = SDU_LEVEL_MAX[i];
        }
        self.set_black_market_upgrades(&levels);
        self.set_backpack_slot_max(
            BACKPACK_SLOT_BASE + BACKPACK_SLOTS_PER_SDU * levels[7],
        )?;
        self.set_i32(56, BANK_SLOT_BASE + BANK_SLOTS_PER_SDU * levels[8]);
        Ok(())
    }
}

#[cfg(test)]
mod sdu_tests {
    use super::*;

    #[test]
    fn sdu_edits_keep_sizes_consistent() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.set_i32(56, 6);

        save.set_sdu(7, 5).expect("backpack sdu");
        assert_eq!(save.black_market_upgrades()[7], 5);
        assert_eq!(save.inventory_slots().map(|i| i.0), Some(27), "12 + 3·5");

        save.set_sdu(8, 7).expect("bank sdu");
        assert_eq!(save.black_market_upgrades()[8], 7);
        assert_eq!(save.max_bank_slots(), Some(20), "6 + 2·7");

        // Untouched entries stay zero; untouched sizes unchanged.
        assert_eq!(save.black_market_upgrades()[0], 0);
        assert_eq!(save.inventory_slots().map(|i| i.1), Some(2), "weapon-ready untouched");
    }

    #[test]
    fn max_all_sdus_hits_vanilla_caps() {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.max_all_sdus().expect("max");
        let levels = save.black_market_upgrades();
        assert_eq!(levels[..7], [7; 7]);
        assert_eq!(levels[7], 9);
        assert_eq!(levels[8], 9);
        assert_eq!(save.inventory_slots().map(|i| i.0), Some(39));
        assert_eq!(save.max_bank_slots(), Some(24));
    }

    #[test]
    fn round_trip_preserves_the_list_shape() {
        // A real-shaped save must survive a max-all write + reparse.
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        save.set_i32(7, 72);
        save.max_all_sdus().expect("max");
        let bytes = save.to_bytes();
        let parsed = SaveGame::parse(&bytes).expect("reparse");
        assert_eq!(parsed.black_market_upgrades(), save.black_market_upgrades());
    }
}
