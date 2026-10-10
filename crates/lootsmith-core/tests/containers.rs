//! Container (backpack/bank) operations on the synthetic save: enumerate, move,
//! append, remove, and a full serialize/reload with crc32/protobuf/huffman intact.

mod common;

use common::synthetic;
use lootsmith_core::gameinfo::AssetLibraryConfigs;
use lootsmith_core::{container, items, Container, Platform};

fn configs() -> AssetLibraryConfigs {
    AssetLibraryConfigs::pc_default()
}

#[test]
fn synthetic_containers_enumerate() {
    let sg = synthetic();
    assert_eq!(
        sg.entries_in(Container::BackpackItem).len(),
        1,
        "expected 1 backpack item"
    );
    assert_eq!(
        sg.entries_in(Container::BackpackWeapon).len(),
        4,
        "expected 4 backpack weapons"
    );
    assert_eq!(
        sg.bank_entries().len(),
        1,
        "the synthetic save banks one weapon"
    );
    assert_eq!(sg.entries().len(), 6);

    let weapons = sg.entries_in(Container::BackpackWeapon);
    assert!(weapons.iter().all(|e| e.quick_slot >= 0));
}

#[test]
fn bank_move_round_trips_and_survives_reload() {
    let mut sg = synthetic();
    let configs = configs();
    let weapons_before = sg.entries_in(Container::BackpackWeapon).len();
    let bank_before = sg.bank_entries().len();
    let moved = sg.entries_in(Container::BackpackWeapon)[0].serial.clone();

    let occurrence = sg
        .move_entry(Container::BackpackWeapon, 0, Container::Bank)
        .expect("move to bank");
    assert_eq!(occurrence, bank_before);
    assert_eq!(sg.bank_entries().len(), bank_before + 1);
    assert_eq!(
        sg.entries_in(Container::BackpackWeapon).len(),
        weapons_before - 1
    );
    assert_eq!(
        sg.bank_entries()[bank_before].serial,
        moved,
        "blob bytes must be kept"
    );

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid(), "crc32 mismatch after move");
    assert!(reopened.proto_stable(), "protobuf unstable after move");
    assert!(reopened.huffman_stable(), "huffman unstable after move");

    let mut sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.bank_entries().len(), bank_before + 1);
    assert_eq!(sg2.bank_entries()[bank_before].serial, moved);
    let decoded =
        items::unpack(&sg2.bank_entries()[bank_before].serial, &configs).expect("decode bank blob");
    assert!(decoded.is_weapon, "the moved blob should still be a weapon");

    // And it can come back.
    sg2.move_entry(Container::Bank, bank_before, Container::BackpackWeapon)
        .expect("move back");
    assert_eq!(sg2.bank_entries().len(), bank_before);
    assert_eq!(
        sg2.entries_in(Container::BackpackWeapon).len(),
        weapons_before
    );
}

#[test]
fn append_and_remove_entries() {
    let mut sg = synthetic();
    let items_before = sg.entries_in(Container::BackpackItem).len();
    let bank_before = sg.bank_entries().len();
    let serial = sg.entries_in(Container::BackpackItem)[0].serial.clone();

    let occurrence = sg.append_entry(Container::Bank, &serial).expect("append");
    assert_eq!(occurrence, bank_before);
    assert_eq!(sg.bank_entries().len(), bank_before + 1);
    assert_eq!(sg.bank_entries()[bank_before].serial, serial);

    // A second copy lands at the next occurrence.
    let occurrence = sg.append_entry(Container::Bank, &serial).expect("append");
    assert_eq!(occurrence, bank_before + 1);
    assert_eq!(sg.bank_entries().len(), bank_before + 2);

    // Removing the first appended copy leaves the second.
    sg.remove_entry(Container::Bank, bank_before)
        .expect("remove");
    assert_eq!(sg.bank_entries().len(), bank_before + 1);

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
    assert_eq!(sg2.bank_entries().len(), bank_before + 1);
}

#[test]
fn edits_a_bank_entry_serial() {
    let mut sg = synthetic();
    let configs = configs();
    let bank_before = sg.bank_entries().len();
    let serial = sg.entries_in(Container::BackpackWeapon)[0].serial.clone();
    let occurrence = sg.append_entry(Container::Bank, &serial).expect("append");

    let mut decoded = items::unpack(&serial, &configs).expect("decode");
    match &mut decoded.slot {
        items::PackedSlot::Weapon(w) => w.grip_part = items::PackedAssetReference::NONE,
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
    assert_eq!(bank.len(), bank_before + 1);
    let round_tripped = items::unpack(&bank[bank_before].serial, &configs).expect("re-decode");
    match round_tripped.slot {
        items::PackedSlot::Weapon(w) => {
            assert!(w.grip_part.is_none(), "grip part should be cleared")
        }
        items::PackedSlot::Item(_) => panic!("expected a weapon"),
    }
}

#[test]
fn moving_does_not_touch_other_blobs() {
    let mut sg = synthetic();
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
    // One weapon was removed; the rest keep their exact bytes and order. There
    // is 1 backpack item before the 4 weapons.
    let mut expected = before.clone();
    expected.remove(1 + 3);
    assert_eq!(after, expected);
}
