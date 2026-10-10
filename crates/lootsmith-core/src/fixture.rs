//! A synthetic, fixture-free test save, so the whole test-suite (and CI) can
//! run without a personal `.sav`.
//!
//! Built entirely from spec'd types: a blank soldier save ("Lootsmith Test"),
//! level 40, some currency and golden keys, three packed blobs (two weapons,
//! one backpack item, one bank item), a few skills, some visited travel
//! stations and DLC challenges. Never load this save into a real game.

use crate::container;
use crate::container::SaveFile;
use crate::error::Result;
use crate::items::{self, PackedAssetReference, PackedData, PackedItem, PackedWeapon};
use crate::platform::Platform;
use crate::proto::{Field, Message};
use crate::savegame::{Container, SaveGame};

/// A packed asset reference that just points at index `i`/`s` of set 0.
fn asset(index: i32, sublibrary: i32) -> PackedAssetReference {
    PackedAssetReference {
        asset_index: index,
        sublibrary_index: sublibrary,
        use_set_id: false,
    }
}

/// A weapon blob: Dahl pistol shaped, indices meaningless outside the test.
fn test_weapon(unique_id: i32, stage: i32) -> PackedData {
    PackedWeapon {
        weapon_type: asset(1, 2),
        balance: asset(2, 3),
        manufacturer: asset(3, 4),
        manufacturer_grade_index: 2,
        game_stage: stage,
        prefix_part: asset(4, 5),
        title_part: asset(5, 6),
        ..Default::default()
    }
    .into_packed(unique_id)
}

/// An item blob: grenade-mod shaped.
fn test_item(unique_id: i32) -> PackedData {
    PackedItem {
        item: asset(1, 1),
        balance: asset(2, 2),
        manufacturer: asset(3, 3),
        manufacturer_grade_index: 1,
        game_stage: 30,
        alpha_part: asset(4, 4),
        material_part: asset(5, 5),
        prefix_part: asset(6, 6),
        title_part: asset(7, 7),
        ..Default::default()
    }
    .into_packed(unique_id)
}

/// The synthetic save.
pub fn synthetic_save() -> SaveGame {
    let mut save = SaveGame::blank("GD_Soldier.Character.CharClass_Soldier");
    save.set_str(1, "GD_Soldier.Character.CharClass_Soldier");
    save.set_i32(2, 40);
    save.set_i32(3, 321_987);
    save.set_i32(4, 5);
    save.set_i32(25, 1_000);
    save.set_i32(48, 3); // golden keys notified
    save.set_int_list_packed(6, &[12_345, 30, 4, 9]);
    save.set_i32(20, 7); // SaveGameId
    save.refresh_guid();
    save.set_str(26, "20260101000001");

    let configs = crate::gameinfo::AssetLibraryConfigs::pc_default();
    let blobs: &[(Container, PackedData)] = &[
        (Container::BackpackItem, test_item(100)),
        (Container::BackpackWeapon, test_weapon(200, 12)),
        (Container::BackpackWeapon, test_weapon(201, 17)),
        (Container::Bank, test_weapon(300, 5)),
    ];
    let quantity = 3;
    let packed_entries: Vec<(Container, Vec<u8>)> = blobs
        .iter()
        .map(|(container, data)| {
            (
                *container,
                items::pack(data, &configs).expect("pack test blob"),
            )
        })
        .collect();

    for (container, serial) in &packed_entries {
        let mut sub = Message::default();
        sub.push(Field::bytes(1, serial));
        match container {
            Container::BackpackItem => {
                sub.push(Field::varint(2, quantity as u64));
                sub.push(Field::varint(3, 0)); // not equipped
                sub.push(Field::varint(4, 5)); // mark: standard
                save.message.push(Field::bytes(53, &sub.to_bytes()));
            }
            Container::BackpackWeapon => {
                sub.push(Field::varint(2, 0)); // no quick slot
                sub.push(Field::varint(3, 1)); // mark
                save.message.push(Field::bytes(54, &sub.to_bytes()));
            }
            Container::Bank => {
                save.message.push(Field::bytes(41, &sub.to_bytes()));
            }
        }
    }

    // Three skills, shaped like the real save (`<tree>.<SkillName>` leaves
    // that the skill calculator matches by normalized leaf name).
    for (skill, grade) in [
        ("GD_Soldier_Skills.Guerrilla.Sentry", 3),
        ("GD_Soldier_Skills.Guerrilla.Ready", 5),
        ("GD_Soldier_Skills.Survival.Willing", 2),
    ] {
        let mut sub = Message::default();
        sub.push(Field::string(1, skill));
        sub.push(Field::varint(2, grade as u64));
        sub.push(Field::varint(3, grade.max(0) as u64));
        sub.push(Field::varint(4, 0));
        save.message.push(Field::bytes(8, &sub.to_bytes()));
    }

    // Ammo pools (field 11).
    for (pool, amount, level) in [
        (
            "D_Resourcepools.AmmoPools.Ammo_Combat_Rifle_Pool",
            273.0f32,
            2,
        ),
        ("D_Resourcepools.AmmoPools.Ammo_Pistol_Pool", 150.0, 1),
        (
            "D_Resourcepools.AmmoPools.Ammo_Grenade_Protean_Pool",
            4.0,
            0,
        ),
    ] {
        let mut sub = Message::default();
        sub.push(Field::string(1, "D_Resources.AmmoResources.Ammo_Test"));
        sub.push(Field::string(2, pool));
        sub.push(Field::fixed32(3, amount.to_bits()));
        sub.push(Field::varint(4, level as u64));
        save.message.push(Field::bytes(11, &sub.to_bytes()));
    }

    // World progress: region 29, discoveries 30, lockouts 43.
    {
        let mut region = Message::default();
        region.push(Field::string(1, "GD_GameStages.Zone1.SouthernShelf"));
        region.push(Field::varint(2, 30));
        region.push(Field::varint(3, 0));
        region.push(Field::varint(4, 0));
        region.push(Field::varint(5, 0));
        save.message.push(Field::bytes(29, &region.to_bytes()));

        for (name, uncovered) in [("SOUTHERNSHELF_PWDA_0", false), ("CLIFFS_VISTA", true)] {
            let mut sub = Message::default();
            sub.push(Field::string(1, name));
            sub.push(Field::varint(2, u64::from(uncovered)));
            save.message.push(Field::bytes(30, &sub.to_bytes()));
        }

        let mut lockout = Message::default();
        lockout.push(Field::string(1, "D_RaidBoss.Lockout.Rawr"));
        lockout.push(Field::varint(2, 123_456));
        lockout.push(Field::varint(3, 0));
        lockout.push(Field::varint(4, 0));
        save.message.push(Field::bytes(43, &lockout.to_bytes()));
    }

    // Challenge-progress blob (field 15) with two entries.
    let mut blob = Vec::new();
    blob.extend_from_slice(&4u32.to_le_bytes());
    blob.extend_from_slice(&((2 * 12 + 2) as u32).to_le_bytes());
    blob.extend_from_slice(&2u16.to_le_bytes());
    for (id, total) in [(1621u16, 9_845i32), (1622, 9_767)] {
        blob.extend_from_slice(&id.to_le_bytes());
        blob.push(6); // flag
        blob.extend_from_slice(&total.to_le_bytes());
        blob.push(1); // flag
        blob.extend_from_slice(&0i32.to_le_bytes());
    }
    save.set_bytes(15, &blob);
    save.set_int_list_packed(39, &[1, 2, 5]);

    // Travel: a few visited stations + the last one.
    save.set_visited_teleporters(&[
        "SouthernShelfBay".to_string(),
        "Sanctuary".to_string(),
        "ThreeHornsValley".to_string(),
    ]);
    save.set_last_visited_teleporter("Sanctuary");

    save
}

/// The synthetic save, serialized as a PC `.sav`.
pub fn synthetic_save_bytes() -> Result<Vec<u8>> {
    container::serialize(&synthetic_save().to_bytes(), Platform::PC)
}

/// Write the synthetic save to `path` and return the parsed copy for tests.
pub fn write_synthetic_save(path: &std::path::Path) -> Result<SaveFile> {
    std::fs::write(path, synthetic_save_bytes()?)?;
    container::unpack(&std::fs::read(path)?, Platform::PC)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_save_round_trips() {
        let path = std::env::temp_dir().join("lootsmith-synthetic-test.sav");
        let file = write_synthetic_save(&path).expect("write");
        assert!(file.crc32_valid());
        let save = file.save_game().expect("parse");
        assert_eq!(save.exp_level(), Some(40));
        assert_eq!(save.currency_on_hand(), [12_345, 30, 4, 9]);
        assert_eq!(save.num_golden_keys_notified(), Some(3));
        assert_eq!(save.packed_entries().len(), 3);
        assert_eq!(save.bank_entries().len(), 1);
        assert_eq!(save.skills().len(), 3);
        assert_eq!(save.resources().len(), 3);
        assert_eq!(save.world_discoveries().len(), 2);
        assert_eq!(save.lockouts().len(), 1);
        assert_eq!(save.challenge_stats().expect("blob").entries.len(), 2);
        assert_eq!(save.level_challenge_unlocks(), [1, 2, 5]);
        assert_eq!(
            save.visited_teleporters(),
            ["SouthernShelfBay", "Sanctuary", "ThreeHornsValley"]
        );
        assert_eq!(
            save.last_visited_teleporter(),
            Some("Sanctuary".to_string())
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn weapon_blobs_decode_back() {
        let configs = crate::gameinfo::AssetLibraryConfigs::pc_default();
        let decoded = items::unpack(
            &items::pack(&test_weapon(55, 9), &configs).unwrap(),
            &configs,
        )
        .expect("round trip");
        assert_eq!(decoded.unique_id, 55);
        assert_eq!(decoded.game_stage(), 9);
        assert_eq!(decoded.manufacturer_grade(), 2);
        assert!(decoded.is_weapon);
    }
}
