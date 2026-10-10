//! Challenges (field 38) and missions (fields 18/21/37) on the synthetic save.

mod common;

use common::synthetic;
use lootsmith_core::{container, mission_status_name, Platform};

#[test]
fn synthetic_challenges_and_missions_enumerate() {
    let sg = synthetic();
    assert_eq!(sg.plot_mission_number(), Some(18));
    assert_eq!(sg.active_mission_number(), Some(18));

    let playthroughs = sg.mission_playthroughs();
    assert_eq!(playthroughs.len(), 1);
    assert_eq!(playthroughs[0].missions.len(), 2);
    assert_eq!(playthroughs[0].missions[0].status, 4, "first is Complete");
    assert_eq!(
        playthroughs[0].missions[1].status, 0,
        "second is NotStarted"
    );

    let challenges = sg.challenges();
    assert_eq!(challenges.len(), 2);
    assert!(challenges.iter().all(|c| c.challenge.starts_with("GD_")));
    assert!(
        challenges.iter().any(|c| c.is_from_dlc),
        "one synthetic challenge is from a DLC"
    );
    assert_eq!(mission_status_name(4), "Complete");
}

#[test]
fn edits_mission_status_and_numbers_and_reloads() {
    let mut sg = synthetic();
    sg.set_plot_mission_number(19);
    sg.set_active_mission_number(19);
    sg.set_mission_status(0, 1, 4).expect("set status");

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());

    let sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.plot_mission_number(), Some(19));
    assert_eq!(sg2.active_mission_number(), Some(19));
    let playthroughs = sg2.mission_playthroughs();
    assert_eq!(
        playthroughs[0].missions.len(),
        2,
        "no missions added/removed"
    );
    assert_eq!(playthroughs[0].missions[1].status, 4, "now Complete");

    // Out-of-range targets error rather than corrupt.
    let mut sg3 = sg2;
    assert!(sg3.set_mission_status(9, 0, 4).is_err());
    assert!(sg3.set_mission_status(0, 999, 4).is_err());
}

#[test]
fn resolves_mission_names() {
    let missions = lootsmith_core::bundled::missions().expect("bundled Missions.json");
    assert!(missions.len() > 100);

    let sg = synthetic();
    // The synthetic plot mission number resolves to a named mission.
    let number = sg.plot_mission_number().expect("plot number");
    let label = missions
        .label_for_number(number)
        .expect("resolved plot mission");
    assert!(
        label.contains('/'),
        "label includes the key and number: {label}"
    );
    // Every mission key in the save resolves too.
    for pt in sg.mission_playthroughs() {
        for mission in pt.missions {
            assert!(
                missions.find(&mission.mission).is_some(),
                "mission {} not found in Missions.json",
                mission.mission
            );
        }
    }
}
