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

/// Dahl pistol parts (real asset names, so the inspector resolves them).
const DAHL_PISTOL: &[(&str, &str)] = &[
    ("type", "GD_Weap_Pistol.A_Weapons.WeaponType_Dahl_Pistol"),
    ("balance", "GD_Weap_Pistol.A_Weapons.Pistol_Dahl_4_VeryRare"),
    ("manufacturer", "GD_Manufacturers.Manufacturers.Dahl"),
    ("body", "GD_Weap_Pistol.Body.Pistol_Body_Dahl_4"),
    ("grip", "GD_Weap_Pistol.Grip.Pistol_Grip_Dahl"),
    ("barrel", "GD_Weap_Pistol.Barrel.Pistol_Barrel_Dahl"),
    (
        "material",
        "GD_Weap_Pistol.ManufacturerMaterials.Mat_Dahl_4",
    ),
    (
        "prefix",
        "GD_Weap_Shared_Names.Name.Prefix_Dahl.Prefix_Grip_Dahl",
    ),
    ("title", "GD_Weap_Pistol.Name.Title_Dahl.Title__Repeater"),
];

/// Jakobs pistol parts.
const JAKOBS_PISTOL: &[(&str, &str)] = &[
    ("type", "GD_Weap_Pistol.A_Weapons.WeaponType_Jakobs_Pistol"),
    (
        "balance",
        "GD_Weap_Pistol.A_Weapons.Pistol_Jakobs_4_VeryRare",
    ),
    ("manufacturer", "GD_Manufacturers.Manufacturers.Jakobs"),
    ("body", "GD_Weap_Pistol.Body.Pistol_Body_Jakobs_4"),
    ("grip", "GD_Weap_Pistol.Grip.Pistol_Grip_Jakobs"),
    ("barrel", "GD_Weap_Pistol.Barrel.Pistol_Barrel_Jakobs"),
    (
        "material",
        "GD_Weap_Pistol.ManufacturerMaterials.Mat_Jakobs_4",
    ),
    (
        "prefix",
        "GD_Weap_Shared_Names.Name.Prefix_Jakobs.Prefix_Hard",
    ),
    ("title", "GD_Weap_Pistol.Name.Title_Jakobs.Title__Iron"),
];

/// A weapon assembled by **name** from the bundled asset library, so the item
/// inspector resolves real parts and a composed display name without any
/// personal save. Falls back to the index-based test weapon if the bundled
/// library is unavailable.
fn named_weapon(unique_id: i32, stage: i32, parts: &[(&str, &str)]) -> PackedData {
    let Ok(manager) = crate::bundled::asset_library() else {
        return test_weapon(unique_id, stage);
    };
    let weapon = PackedWeapon {
        manufacturer_grade_index: 2,
        game_stage: stage,
        ..Default::default()
    };
    let mut decoded = weapon.into_packed(unique_id);
    for (slot, name) in parts {
        let _ = decoded.set_part(&manager, slot, name);
    }
    decoded
}

/// A named artifact item (all references resolve in the bundled library).
fn named_item(unique_id: i32, stage: i32) -> PackedData {
    let Ok(manager) = crate::bundled::asset_library() else {
        return test_item(unique_id);
    };
    let set = 0;
    let r = |group, name| {
        manager
            .find_asset(group, set, name)
            .map(
                |(asset_index, sublibrary_index, use_set_id)| PackedAssetReference {
                    asset_index,
                    sublibrary_index,
                    use_set_id,
                },
            )
            .unwrap_or(PackedAssetReference::NONE)
    };
    let item = PackedItem {
        item: r(
            crate::AssetGroup::ItemTypes,
            "GD_Artifacts.A_Item.Artifact_AggressionA",
        ),
        balance: r(
            crate::AssetGroup::BalanceDefs,
            "GD_Artifacts.A_Item.A_AggressionA",
        ),
        manufacturer: r(
            crate::AssetGroup::Manufacturers,
            "GD_Manufacturers.Artifacts.Artifact_TypeA",
        ),
        manufacturer_grade_index: 1,
        game_stage: stage,
        alpha_part: r(
            crate::AssetGroup::ItemParts,
            "GD_Artifacts.Body.Body_Aggression",
        ),
        material_part: r(
            crate::AssetGroup::ItemParts,
            "GD_Artifacts.Body.Body_Afterburner",
        ),
        ..Default::default()
    };
    item.into_packed(unique_id)
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
                         // A *fixed* GUID: synthetic saves must be byte-identical between runs so
                         // snapshots and byte-level tests stay deterministic.
    save.message.set(Field::bytes(
        34,
        &[
            0x10, 0x32, 0x54, 0x76, 0x98, 0xBA, 0xDC, 0xFE, 0x0F, 0x1E, 0x2D, 0x3C, 0x4B, 0x5A,
            0x69, 0x78,
        ],
    ));
    save.set_str(26, "20260101000001");

    // UI preferences (field 19) with a character name, and applied
    // customizations (field 35: index 0 is the head, index 4 the skin), so the
    // character tests run on the synthetic save with no personal fixture.
    {
        let mut prefs = Message::default();
        prefs.push(Field::bytes(1, b"Lootsmith Test"));
        save.message.push(Field::bytes(19, &prefs.to_bytes()));
        save.set_str_list(
            35,
            &[
                "GD_DefaultCustoms_MainGame.Soldier.Head_Default".to_string(),
                "GD_DefaultCustoms_MainGame.Soldier.Head_A".to_string(),
                "GD_DefaultCustoms_MainGame.Soldier.Head_B".to_string(),
                "GD_DefaultCustoms_MainGame.Soldier.Skin_A".to_string(),
                "GD_DefaultCustoms_MainGame.Soldier.Skin_Default".to_string(),
            ],
        );
    }

    let configs = crate::gameinfo::AssetLibraryConfigs::pc_default();
    let blobs: Vec<(Container, PackedData)> = vec![
        (Container::BackpackItem, named_item(100, 30)),
        // All blobs are built from real asset names, so name/index round trips
        // and the inspector apply to every entry (no personal save needed).
        (
            Container::BackpackWeapon,
            named_weapon(400, 40, DAHL_PISTOL),
        ),
        (
            Container::BackpackWeapon,
            named_weapon(401, 40, JAKOBS_PISTOL),
        ),
        (
            Container::BackpackWeapon,
            named_weapon(402, 12, DAHL_PISTOL),
        ),
        (
            Container::BackpackWeapon,
            named_weapon(403, 17, JAKOBS_PISTOL),
        ),
        (Container::Bank, named_weapon(300, 5, JAKOBS_PISTOL)),
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

    // The action skill (the action point is always invested in a real save) and
    // a few tree skills, shaped like the real save (`<tree>.<SkillName>` leaves
    // that the skill board matches by object path).
    for (skill, grade) in [
        ("GD_Soldier_Skills.Scorpio.Skill_Scorpio", 1),
        ("GD_Soldier_Skills.Guerrilla.Sentry", 3),
        ("GD_Soldier_Skills.Guerrilla.Ready", 5),
        ("GD_Soldier_Skills.Survival.Willing", 2),
        ("GD_Soldier_Skills.Survival.Hardened", 0),
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

    // Challenge list (field 38): two entries (one DLC) so the challenge
    // accessor has data without a personal save.
    for (challenge, dlc) in [
        ("GD_Challenges.Player.Player_Combat_Assault", false),
        ("GD_ChallengeDLC1.Challenges.Challenge_RaidBoss_Kill", true),
    ] {
        let mut sub = Message::default();
        sub.push(Field::string(1, challenge));
        sub.push(Field::varint(2, u64::from(dlc)));
        sub.push(Field::varint(3, 0));
        save.message.push(Field::bytes(38, &sub.to_bytes()));
    }

    // One mission playthrough with two missions (field 18).
    {
        let mut playthrough = Message::default();
        playthrough.push(Field::varint(1, 1)); // PlayThroughNumber
        playthrough.push(Field::string(2, "GD_Episode01.M_Ep1_Champion"));
        for (mission, status) in [
            ("GD_Episode01.M_Ep1_Champion", 4), // Complete
            ("GD_Episode02.M_Ep2_Henchman", 0), // NotStarted
        ] {
            let mut entry = Message::default();
            entry.push(Field::string(1, mission));
            entry.push(Field::varint(2, status));
            entry.push(Field::varint(3, 0));
            entry.push(Field::varint(4, 0));
            entry.push(Field::varint(6, 0));
            entry.push(Field::varint(8, 0));
            entry.push(Field::varint(10, 0));
            entry.push(Field::varint(11, 40));
            playthrough.push(Field::bytes(3, &entry.to_bytes()));
        }
        save.message.push(Field::bytes(18, &playthrough.to_bytes()));
        save.set_i32(21, 18); // PlotMissionNumber
        save.set_i32(37, 18); // ActiveMissionNumber
    }

    // One vehicle family with two skins (field 57) + steering mode (58).
    {
        let mut vehicle = Message::default();
        vehicle.push(Field::string(
            1,
            "GD_Globals.VehicleSpawnStation.VehicleFamily_Runner",
        ));
        vehicle.push(Field::string(2, "GD_VehicleCustomizations.Runner.Skin_A"));
        vehicle.push(Field::string(2, "GD_VehicleCustomizations.Runner.Skin_B"));
        save.message.push(Field::bytes(57, &vehicle.to_bytes()));
        save.set_i32(58, 0);
    }

    // Travel: a few visited stations + the last one. Names must exist in the
    // bundled `Travel Stations.json` so the lookups resolve.
    save.set_visited_teleporters(&[
        "SouthernShelfTown".to_string(),
        "Sanctuary".to_string(),
        "Helios".to_string(),
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
        assert_eq!(save.packed_entries().len(), 5);
        assert_eq!(save.bank_entries().len(), 1);
        assert_eq!(save.skills().len(), 5);
        assert_eq!(save.challenges().len(), 2);
        assert_eq!(
            save.character_name().as_deref(),
            Some("Lootsmith Test"),
            "synthetic UIPreferences name"
        );
        assert_eq!(save.applied_customizations().len(), 5);
        assert_eq!(save.resources().len(), 3);
        assert_eq!(save.mission_playthroughs().len(), 1);
        assert_eq!(save.mission_playthroughs()[0].missions.len(), 2);
        assert_eq!(save.vehicles().len(), 1);
        assert_eq!(save.world_discoveries().len(), 2);
        assert_eq!(save.lockouts().len(), 1);
        assert_eq!(save.challenge_stats().expect("blob").entries.len(), 2);
        assert_eq!(save.level_challenge_unlocks(), [1, 2, 5]);
        assert_eq!(
            save.visited_teleporters(),
            ["SouthernShelfTown", "Sanctuary", "Helios"]
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
