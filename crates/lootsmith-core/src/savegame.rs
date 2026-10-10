//! Typed view over `WillowTwoPlayerSaveGame`.
//!
//! Field numbers and kinds are transcribed from
//! `ProtoBufFormats/WillowTwoSave/WillowTwoPlayerSaveGame.cs`. The underlying
//! [`Message`] keeps raw bytes, so untouched fields re-emit exactly; setters
//! replace a single field with a canonical encoding.

use crate::error::Result;
use crate::proto::{Field, Message};

#[derive(Clone, Copy, PartialEq, Eq)]
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
