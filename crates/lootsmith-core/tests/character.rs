//! Character name (UIPreferences, field 19) and applied customizations (field 35).

mod common;

use common::synthetic;
use lootsmith_core::{container, Platform};

#[test]
fn character_name_and_customizations_round_trip() {
    let mut sg = synthetic();
    assert!(
        sg.character_name().is_some(),
        "the synthetic save has a UI preferences block"
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
    assert_eq!(
        customs[0],
        "GD_DefaultCustoms_MainGame.Soldier.Head_Default"
    );
    assert_eq!(
        customs[4],
        "GD_DefaultCustoms_MainGame.Soldier.Skin_Default"
    );
}

#[test]
fn overpower_level_round_trips_and_only_touches_field_40() {
    let mut sg = synthetic();
    assert_eq!(sg.overpower_level(), 0, "synthetic save starts with no OP");

    // Grow past the empty list: the zeroed `OneOffLevelChallengeData` skeleton.
    assert!(sg.set_overpower_level(4));
    assert_eq!(sg.overpower_level(), 4);

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());
    let sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.overpower_level(), 4);
    assert_eq!(sg2.playthroughs_completed(), sg.playthroughs_completed());
    assert_eq!(sg2.character_name(), sg.character_name());
    assert_eq!(sg2.general_skill_points(), sg.general_skill_points());

    // Shrink is exact, and 0 clears every completion.
    assert!(sg.set_overpower_level(2));
    assert_eq!(sg.overpower_level(), 2);
    assert!(sg.set_overpower_level(0));
    assert_eq!(sg.overpower_level(), 0);

    // Out of range is refused and leaves the list untouched.
    assert!(!sg.set_overpower_level(11));
    assert!(!sg.set_overpower_level(-1));
    assert_eq!(sg.overpower_level(), 0);
}
