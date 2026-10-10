//! World progress: region game stages (29), world discoveries (30),
//! lockouts (43), fully-explored areas (46), DLC expansion state (27) and
//! the item/weapon mementos (32/33).
//!
//! Schemas from upstream `WillowTwoSave/*.cs`:
//! - `RegionGameStageData { string Region=1; int32 GameStage=2; bool IsFromDLC=3;
//!   int32 DLCPackageId=4; int32 PlaythroughIndex=5 }`
//! - `WorldDiscoveryData { string DiscoveryName=1; bool HasBeenUncovered=2 }`
//! - `LockoutData { string Lockout=1; int32 Time=2; bool IsFromDLC=3; int32 DLCPackageId=4 }`
//! - `DLCExpansionData { int32 Tag=1; bytes Data=2 }`

use crate::error::{Error, Result};
use crate::proto::{Field, Message};
use crate::savegame::SaveGame;

#[derive(Clone, Debug, PartialEq, Default)]
pub struct RegionGameStage {
    pub region: String,
    pub game_stage: i32,
    pub is_from_dlc: bool,
    pub dlc_package_id: i32,
    pub playthrough_index: i32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct WorldDiscovery {
    pub name: String,
    pub uncovered: bool,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Lockout {
    pub lockout: String,
    pub time: i32,
    pub is_from_dlc: bool,
    pub dlc_package_id: i32,
}

impl Lockout {
    /// Player-ish display for the lockout key (raid-boss lockouts are long
    /// comma-joined strings; show the region-ish head + the timer).
    pub fn label(&self) -> String {
        let head = self.lockout.split(',').next().unwrap_or(&self.lockout);
        format!("{head} ({})", self.time)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DlcExpansion {
    pub tag: i32,
    pub data: Vec<u8>,
}

fn string_at(sub: &Message, number: u32) -> String {
    sub.get(number)
        .and_then(|f| f.as_str())
        .map(|s| s.into_owned())
        .unwrap_or_default()
}

fn int_at(sub: &Message, number: u32) -> i32 {
    sub.get(number).and_then(|f| f.as_i32()).unwrap_or(0)
}

fn bool_at(sub: &Message, number: u32) -> bool {
    sub.get(number).and_then(|f| f.as_bool()).unwrap_or(false)
}

impl SaveGame {
    // ---- reads ----

    /// `RegionGameStages` (field 29), in order.
    pub fn region_game_stages(&self) -> Vec<RegionGameStage> {
        self.message
            .get_all(29)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| RegionGameStage {
                region: string_at(&sub, 1),
                game_stage: int_at(&sub, 2),
                is_from_dlc: bool_at(&sub, 3),
                dlc_package_id: int_at(&sub, 4),
                playthrough_index: int_at(&sub, 5),
            })
            .collect()
    }

    /// `WorldDiscoveryList` (field 30), in order.
    pub fn world_discoveries(&self) -> Vec<WorldDiscovery> {
        self.message
            .get_all(30)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| WorldDiscovery {
                name: string_at(&sub, 1),
                uncovered: bool_at(&sub, 2),
            })
            .collect()
    }

    /// Number of discoveries already uncovered.
    pub fn discoveries_uncovered(&self) -> usize {
        self.world_discoveries()
            .iter()
            .filter(|d| d.uncovered)
            .count()
    }

    /// `LockoutList` (field 43), in order.
    pub fn lockouts(&self) -> Vec<Lockout> {
        self.message
            .get_all(43)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| Lockout {
                lockout: string_at(&sub, 1),
                time: int_at(&sub, 2),
                is_from_dlc: bool_at(&sub, 3),
                dlc_package_id: int_at(&sub, 4),
            })
            .collect()
    }

    /// `DLCExpansionData` (field 27), in order.
    pub fn dlc_expansions(&self) -> Vec<DlcExpansion> {
        self.message
            .get_all(27)
            .filter_map(|field| field.as_message().ok().flatten())
            .map(|sub| DlcExpansion {
                tag: int_at(&sub, 1),
                data: sub
                    .get(2)
                    .and_then(|f| f.as_bytes())
                    .map(<[u8]>::to_vec)
                    .unwrap_or_default(),
            })
            .collect()
    }

    // ---- writes (each re-encodes only the entry it touches) ----

    /// Set one region's game stage by index (0-based in field order).
    pub fn set_region_game_stage(&mut self, index: usize, stage: i32) -> Result<()> {
        let mut seen = 0usize;
        for field in self.message.fields.iter_mut().filter(|f| f.number == 29) {
            if seen != index {
                seen += 1;
                continue;
            }
            let Some(mut sub) = field.as_message().ok().flatten() else {
                return Err(Error::Proto("region entry is not a message".into()));
            };
            sub.set(Field::varint(2, stage.max(0) as u64));
            *field = Field::bytes(29, &sub.to_bytes());
            return Ok(());
        }
        Err(Error::Proto(format!("no region game stage #{index}")))
    }

    /// Set one discovery's `HasBeenUncovered`, rewriting just that entry.
    pub fn set_discovery(&mut self, index: usize, uncovered: bool) -> Result<()> {
        let mut seen = 0usize;
        for field in self.message.fields.iter_mut().filter(|f| f.number == 30) {
            if seen != index {
                seen += 1;
                continue;
            }
            let Some(mut sub) = field.as_message().ok().flatten() else {
                return Err(Error::Proto("discovery entry is not a message".into()));
            };
            sub.set(Field::varint(2, u64::from(uncovered)));
            *field = Field::bytes(30, &sub.to_bytes());
            return Ok(());
        }
        Err(Error::Proto(format!("no discovery #{index}")))
    }

    /// Mark every discovery uncovered.
    pub fn mark_all_discovered(&mut self) -> Result<usize> {
        let total = self.message.count(30);
        let mut changed = 0usize;
        for index in 0..total {
            if self.set_discovery(index, true).is_ok() {
                changed += 1;
            }
        }
        Ok(changed)
    }

    /// Zero every lockout timer (raid bosses farmable again).
    pub fn clear_all_lockouts(&mut self) -> usize {
        let mut changed = 0usize;
        for field in self.message.fields.iter_mut().filter(|f| f.number == 43) {
            let Some(mut sub) = field.as_message().ok().flatten() else {
                continue;
            };
            sub.set(Field::varint(2, 0));
            *field = Field::bytes(43, &sub.to_bytes());
            changed += 1;
        }
        changed
    }

    /// Replace the `FullyExploredAreas` string list (field 46).
    pub fn set_fully_explored_areas(&mut self, areas: &[String]) {
        self.message.remove_all(46);
        for area in areas {
            self.message.push(Field::string(46, area));
        }
    }

    // ---- mementos (32/33) ----

    /// `(weapon mementos, item mementos)` counts.
    pub fn memento_counts(&self) -> (usize, usize) {
        (self.message.count(32), self.message.count(33))
    }

    /// Drop every weapon/item memento (they re-add removed items to lists).
    pub fn clear_mementos(&mut self) -> (usize, usize) {
        let counts = self.memento_counts();
        self.message.remove_all(32);
        self.message.remove_all(33);
        counts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world_save() -> SaveGame {
        let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
        // Field 29: one region at stage 30.
        let mut region = Message::default();
        region.push(Field::string(1, "GD_GameStages.Zone3.FinalBoss"));
        region.push(Field::varint(2, 30));
        region.push(Field::varint(3, 0));
        region.push(Field::varint(4, 0));
        region.push(Field::varint(5, 0));
        save.message.push(Field::bytes(29, &region.to_bytes()));
        // Field 30: two discoveries, one uncovered.
        for (name, uncovered) in [("SOUTHERNSHELF_PWDA_0", false), ("CLIFFS_VISTA", true)] {
            let mut sub = Message::default();
            sub.push(Field::string(1, name));
            sub.push(Field::varint(2, u64::from(uncovered)));
            save.message.push(Field::bytes(30, &sub.to_bytes()));
        }
        // Field 43: one lockout with a non-zero time.
        let mut lockout = Message::default();
        lockout.push(Field::string(1, "D_X.A.B"));
        lockout.push(Field::varint(2, 123_456));
        lockout.push(Field::varint(3, 1));
        lockout.push(Field::varint(4, 99));
        save.message.push(Field::bytes(43, &lockout.to_bytes()));
        save
    }

    #[test]
    fn reads_and_edits_world_data() {
        let mut save = world_save();
        assert_eq!(save.region_game_stages().len(), 1);
        assert_eq!(
            save.region_game_stages()[0].region,
            "GD_GameStages.Zone3.FinalBoss"
        );

        save.set_region_game_stage(0, 45).expect("stage");
        assert_eq!(save.region_game_stages()[0].game_stage, 45);
        assert!(
            !save.region_game_stages()[0].is_from_dlc,
            "region untouched"
        );

        let discoveries = save.world_discoveries();
        assert_eq!(discoveries.len(), 2);
        assert_eq!(save.discoveries_uncovered(), 1);

        save.set_discovery(0, true).expect("discover");
        assert_eq!(save.discoveries_uncovered(), 2);
        assert_eq!(
            save.world_discoveries()[0].name,
            "SOUTHERNSHELF_PWDA_0",
            "name kept"
        );

        save.set_discovery(5, true).unwrap_err();
    }

    #[test]
    fn mark_all_and_lockout_clear() {
        let mut save = world_save();
        assert_eq!(save.mark_all_discovered().expect("mark"), 2);
        assert_eq!(save.discoveries_uncovered(), 2);

        let mut save = world_save();
        assert_eq!(save.lockouts().len(), 1);
        assert_eq!(save.lockouts()[0].time, 123_456);
        assert_eq!(save.clear_all_lockouts(), 1);
        assert_eq!(save.lockouts()[0].time, 0);
        assert_eq!(save.lockouts()[0].dlc_package_id, 99, "untouched field");
    }

    #[test]
    fn fully_explored_and_dlc_expansions() {
        let mut save = world_save();
        save.set_fully_explored_areas(&["A".into(), "B".into()]);
        assert_eq!(save.message.get_all(46).count(), 2);
        assert_eq!(
            save.message
                .get(46)
                .and_then(|f| f.as_str())
                .map(|s| s.into_owned()),
            Some("A".into())
        );

        save.set_fully_explored_areas(&[]);
        assert_eq!(save.message.get_all(46).count(), 0);

        let mut expansion = Message::default();
        expansion.push(Field::varint(1, 5));
        expansion.push(Field::bytes(2, &[1, 2, 3]));
        save.message.push(Field::bytes(27, &expansion.to_bytes()));
        let loaded = save.dlc_expansions();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].tag, 5);
        assert_eq!(loaded[0].data, [1, 2, 3]);
    }

    #[test]
    fn mementos_list_and_clear() {
        let mut save = world_save();
        assert_eq!(save.memento_counts(), (0, 0));
        save.message.push(Field::bytes(32, &[1, 2, 3]));
        save.message.push(Field::bytes(33, &[4]));
        assert_eq!(save.memento_counts(), (1, 1));
        save.clear_mementos();
        assert_eq!(save.memento_counts(), (0, 0));
    }
}
