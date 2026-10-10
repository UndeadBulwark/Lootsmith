//! Character name (UIPreferences, field 19) and applied customizations (field 35).

use lootsmith_core::{container, Platform};

fn fixture_save() -> Option<lootsmith_core::SaveGame> {
    let path = format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    );
    let data = std::fs::read(path).ok()?;
    let save = container::unpack(&data, Platform::PC).ok()?;
    save.save_game().ok()
}

#[test]
fn character_name_and_customizations_round_trip() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    assert!(
        sg.character_name().is_some(),
        "the fixture has a UI preferences block"
    );
    assert!(
        sg.applied_customizations().len() >= 5,
        "applied customizations list is present"
    );

    sg.set_character_name("Lootsmith Tester").unwrap();
    sg.set_applied_head_skin(
        "GD_DefaultCustoms_MainGame.Soldier.Head_Default",
        "GD_DefaultCustoms_MainGame.Soldier.Skin_Default",
    );

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());

    let sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.character_name().as_deref(), Some("Lootsmith Tester"));
    let customs = sg2.applied_customizations();
    assert_eq!(customs[0], "GD_DefaultCustoms_MainGame.Soldier.Head_Default");
    assert_eq!(customs[4], "GD_DefaultCustoms_MainGame.Soldier.Skin_Default");
}
