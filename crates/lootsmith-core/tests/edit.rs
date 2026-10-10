//! Phase 5 gate: edit in memory, serialize the whole file, and re-open it.

use lootsmith_core::{container, items, Platform};

fn fixture() -> Option<Vec<u8>> {
    let path = format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read(path).ok()
}

#[test]
fn edit_level_serializes_and_reopens() {
    let Some(data) = fixture() else {
        eprintln!("fixture not present; skipping");
        return;
    };

    let save = container::unpack(&data, Platform::PC).expect("unpack");
    let mut save_game = save.save_game().expect("parse");

    let original_class = save_game.player_class();
    let original_currency = save_game.currency_on_hand();
    let original_level = save_game.exp_level();

    save_game.set_i32(2, 61);
    save_game.set_i32(48, 42); // golden keys

    let proto = save_game.to_bytes();
    let file = container::serialize(&proto, Platform::PC).expect("serialize");

    let reopened = container::unpack(&file, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid(), "crc32 invalid after edit");
    assert!(reopened.proto_stable(), "protobuf not stable after edit");
    assert!(reopened.huffman_stable(), "huffman unstable after edit");

    let edited = reopened.save_game().expect("reparse");
    assert_eq!(edited.exp_level(), Some(61));
    assert_eq!(edited.num_golden_keys_notified(), Some(42));

    // Untouched fields are preserved.
    assert_eq!(edited.player_class(), original_class);
    assert_eq!(edited.currency_on_hand(), original_currency);
    assert_ne!(original_level, Some(61));
}

#[test]
fn edit_currency_replaces_first_slot_only() {
    let Some(data) = fixture() else {
        return;
    };
    let save = container::unpack(&data, Platform::PC).expect("unpack");
    let mut save_game = save.save_game().expect("parse");

    let before = save_game.currency_on_hand();
    assert!(!before.is_empty());
    let mut list = before.clone();
    list[0] = 123_456_789;
    save_game.set_int_list_packed(6, &list);

    let file = container::serialize(&save_game.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&file, Platform::PC).expect("reopen");
    let edited = reopened.save_game().expect("reparse");

    let after = edited.currency_on_hand();
    assert_eq!(after[0], 123_456_789);
    assert_eq!(&after[1..], &before[1..]);
}

fn dumps_path() -> Option<std::path::PathBuf> {
    [
        std::env::var("LOOTSMITH_ASSET_LIBRARY").ok(),
        Some(format!(
            "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
            env!("CARGO_MANIFEST_DIR")
        )),
    ]
    .into_iter()
    .flatten()
    .map(std::path::PathBuf::from)
    .find(|p| p.exists())
}

#[test]
fn edit_weapon_part_by_name() {
    let Some(data) = fixture() else {
        return;
    };
    let Some(dump) = dumps_path() else {
        eprintln!("no dumps file; skipping");
        return;
    };
    let manager = lootsmith_core::AssetLibraryManager::from_path(&dump).expect("manager");

    let save = container::unpack(&data, Platform::PC).expect("unpack");
    let mut save_game = save.save_game().expect("parse");
    let entries = save_game.packed_entries();

    let entry = entries
        .iter()
        .find(|e| e.field_number == 54)
        .expect("a weapon entry")
        .clone();
    let mut decoded = items::unpack(&entry.serial, &manager.configs).expect("decode");

    // Point the title at a real, different part.
    decoded
        .set_part(
            &manager,
            "title",
            "GD_Weap_Pistol.Name.Title_Jakobs.Title_Barrel_Jakobs_SlowDamage",
        )
        .expect("set title");
    let new_serial = items::pack(&decoded, &manager.configs).expect("pack");

    save_game
        .replace_serial(entry.field_number, entry.occurrence, &new_serial)
        .expect("replace serial");

    let file = container::serialize(&save_game.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&file, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid());
    assert!(reopened.proto_stable());

    let edited = reopened.save_game().expect("reparse");
    let entries2 = edited.packed_entries();
    assert_eq!(entries2.len(), entries.len(), "entry count changed");

    let entry2 = entries2
        .iter()
        .find(|e| e.field_number == 54)
        .expect("weapon entry");
    let decoded2 = items::unpack(&entry2.serial, &manager.configs).expect("decode");
    let title = decoded2.part("title").expect("title slot").0;
    let name = manager
        .lookup_asset(
            lootsmith_core::AssetGroup::WeaponParts,
            decoded2.asset_library_set_id,
            title.asset_index,
            title.sublibrary_index,
            title.use_set_id,
        )
        .expect("lookup title");
    assert_eq!(
        name,
        "GD_Weap_Pistol.Name.Title_Jakobs.Title_Barrel_Jakobs_SlowDamage"
    );

    // A different weapon's serial is untouched.
    let other = entries2
        .iter()
        .filter(|e| e.field_number == 54)
        .nth(1)
        .expect("second weapon");
    let other_original = entries
        .iter()
        .filter(|e| e.field_number == 54)
        .nth(1)
        .expect("second weapon original");
    assert_eq!(other.serial, other_original.serial);
}
