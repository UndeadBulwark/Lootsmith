//! Container (backpack/bank) operations: enumerate, move, append, remove, and
//! round-trip the whole save through the container codec.
//!
//! The bank (field 41) stores the same `InventorySerialNumber` blob shape as
//! the backpack (fields 53/54), so moving a blob between lists must preserve its
//! bytes and survive a serialize/reload with crc32, protobuf and huffman intact.

use lootsmith_core::gameinfo::AssetLibraryConfigs;
use lootsmith_core::{container, items, Container, Platform};
use std::path::Path;

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn load_configs() -> AssetLibraryConfigs {
    for candidate in [
        std::env::var("LOOTSMITH_ASSET_LIBRARY").ok(),
        Some(format!(
            "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
            env!("CARGO_MANIFEST_DIR")
        )),
    ]
    .into_iter()
    .flatten()
    {
        let path = Path::new(&candidate);
        if path.exists() {
            if let Ok(configs) = AssetLibraryConfigs::from_path(path) {
                return configs;
            }
        }
    }
    AssetLibraryConfigs::pc_default()
}

fn fixture_save() -> Option<lootsmith_core::SaveGame> {
    let data = std::fs::read(fixture_path()).ok()?;
    let save = container::unpack(&data, Platform::PC).ok()?;
    save.save_game().ok()
}

#[test]
fn fixture_containers_enumerate() {
    let Some(sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    assert_eq!(
        sg.entries_in(Container::BackpackItem).len(),
        5,
        "expected 5 backpack items"
    );
    assert_eq!(
        sg.entries_in(Container::BackpackWeapon).len(),
        9,
        "expected 9 backpack weapons"
    );
    assert_eq!(sg.bank_entries().len(), 0, "fixture bank should be empty");
    assert_eq!(sg.entries().len(), 14);

    // Weapon quick-slot metadata comes from field 2 of `PackedWeaponData`.
    let weapons = sg.entries_in(Container::BackpackWeapon);
    assert!(weapons.iter().all(|e| e.quick_slot >= 0));
}

#[test]
fn bank_move_round_trips_and_survives_reload() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let configs = load_configs();
    let weapons_before = sg.entries_in(Container::BackpackWeapon).len();
    let moved = sg.entries_in(Container::BackpackWeapon)[0].serial.clone();

    let occurrence = sg
        .move_entry(Container::BackpackWeapon, 0, Container::Bank)
        .expect("move to bank");
    assert_eq!(occurrence, 0);
    assert_eq!(sg.bank_entries().len(), 1);
    assert_eq!(
        sg.entries_in(Container::BackpackWeapon).len(),
        weapons_before - 1
    );
    assert_eq!(
        sg.bank_entries()[0].serial,
        moved,
        "blob bytes must be kept"
    );

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid(), "crc32 mismatch after move");
    assert!(reopened.proto_stable(), "protobuf unstable after move");
    assert!(reopened.huffman_stable(), "huffman unstable after move");

    let mut sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.bank_entries().len(), 1);
    assert_eq!(sg2.bank_entries()[0].serial, moved);
    let decoded = items::unpack(&sg2.bank_entries()[0].serial, &configs).expect("decode bank blob");
    assert!(decoded.is_weapon, "the moved blob should still be a weapon");

    // And it can come back.
    sg2.move_entry(Container::Bank, 0, Container::BackpackWeapon)
        .expect("move back");
    assert_eq!(sg2.bank_entries().len(), 0);
    assert_eq!(
        sg2.entries_in(Container::BackpackWeapon).len(),
        weapons_before
    );
}

#[test]
fn append_and_remove_entries() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let items_before = sg.entries_in(Container::BackpackItem).len();
    let serial = sg.entries_in(Container::BackpackItem)[0].serial.clone();

    let occurrence = sg.append_entry(Container::Bank, &serial).expect("append");
    assert_eq!(occurrence, 0);
    assert_eq!(sg.bank_entries().len(), 1);
    assert_eq!(sg.bank_entries()[0].serial, serial);

    // A second copy lands at occurrence 1.
    let occurrence = sg.append_entry(Container::Bank, &serial).expect("append");
    assert_eq!(occurrence, 1);
    assert_eq!(sg.bank_entries().len(), 2);

    // Removing the first leaves the second.
    sg.remove_entry(Container::Bank, 0).expect("remove");
    assert_eq!(sg.bank_entries().len(), 1);

    // Metadata edits on a backpack item write quantity/equipped and reload.
    sg.set_entry_quantity(Container::BackpackItem, 0, 7)
        .expect("set quantity");
    sg.set_entry_equipped(Container::BackpackItem, 0, true)
        .expect("set equipped");
    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg2 = reopened.save_game().expect("reparse");
    let entry = &sg2.entries_in(Container::BackpackItem)[0];
    assert_eq!(entry.quantity, 7);
    assert!(entry.equipped);
    assert_eq!(sg2.entries_in(Container::BackpackItem).len(), items_before);
    assert_eq!(sg2.bank_entries().len(), 1);
}

#[test]
fn edits_a_bank_entry_serial() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let configs = load_configs();
    let serial = sg.entries_in(Container::BackpackWeapon)[0].serial.clone();
    let occurrence = sg.append_entry(Container::Bank, &serial).expect("append");

    let mut decoded = items::unpack(&serial, &configs).expect("decode");
    match &mut decoded.slot {
        items::PackedSlot::Weapon(w) => w.elemental_part = items::PackedAssetReference::NONE,
        items::PackedSlot::Item(_) => panic!("expected a weapon"),
    }
    let edited = items::pack(&decoded, &configs).expect("re-pack");
    assert_ne!(edited, serial);
    sg.replace_serial(Container::Bank.field_number(), occurrence, &edited)
        .expect("replace bank serial");

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());

    let sg2 = reopened.save_game().expect("reparse");
    let bank = sg2.bank_entries();
    assert_eq!(bank.len(), 1);
    let round_tripped = items::unpack(&bank[0].serial, &configs).expect("re-decode");
    match round_tripped.slot {
        items::PackedSlot::Weapon(w) => {
            assert!(
                w.elemental_part.is_none(),
                "elemental part should be cleared"
            )
        }
        items::PackedSlot::Item(_) => panic!("expected a weapon"),
    }
}

#[test]
fn moving_does_not_touch_other_blobs() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let before: Vec<Vec<u8>> = sg
        .packed_entries()
        .iter()
        .map(|e| e.serial.clone())
        .collect();
    sg.move_entry(Container::BackpackWeapon, 3, Container::Bank)
        .expect("move");
    let after: Vec<Vec<u8>> = sg
        .packed_entries()
        .iter()
        .map(|e| e.serial.clone())
        .collect();
    // One weapon was removed; the rest keep their exact bytes and order.
    let mut expected = before.clone();
    expected.remove(5 + 3); // 5 backpack items precede the weapons
    assert_eq!(after, expected);
}
