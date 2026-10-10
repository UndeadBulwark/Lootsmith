//! Challenges (field 38) and missions (fields 18/21/37).

use lootsmith_core::{container, mission_status_name, Missions, Platform};

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn dumps_dir() -> std::path::PathBuf {
    std::env::var("LOOTSMITH_DUMPS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(format!(
                "{}/../../../bl2-rogue/data/gibbed",
                env!("CARGO_MANIFEST_DIR")
            ))
        })
}

fn fixture_save() -> Option<lootsmith_core::SaveGame> {
    let data = std::fs::read(fixture_path()).ok()?;
    let save = container::unpack(&data, Platform::PC).ok()?;
    save.save_game().ok()
}

#[test]
fn fixture_challenges_and_missions_enumerate() {
    let Some(sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    assert_eq!(sg.plot_mission_number(), Some(18));
    assert_eq!(sg.active_mission_number(), Some(18));

    let playthroughs = sg.mission_playthroughs();
    assert_eq!(playthroughs.len(), 3);
    assert_eq!(playthroughs[0].missions.len(), 19);
    assert!(
        playthroughs[0]
            .missions
            .iter()
            .all(|m| m.status == 4 || m.status == 1),
        "playthrough 1 missions are Complete or Active"
    );
    assert_eq!(playthroughs[0].missions[18].status, 1, "the last is Active");

    let challenges = sg.challenges();
    assert_eq!(challenges.len(), 132);
    assert!(challenges.iter().all(|c| c.challenge.starts_with("GD_")));
    assert_eq!(mission_status_name(4), "Complete");
}

#[test]
fn edits_mission_status_and_numbers_and_reloads() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    sg.set_plot_mission_number(19);
    sg.set_active_mission_number(19);
    sg.set_mission_status(0, 18, 4).expect("set status");

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());

    let sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.plot_mission_number(), Some(19));
    assert_eq!(sg2.active_mission_number(), Some(19));
    let playthroughs = sg2.mission_playthroughs();
    assert_eq!(
        playthroughs[0].missions.len(),
        19,
        "no missions added/removed"
    );
    assert_eq!(playthroughs[0].missions[18].status, 4, "now Complete");

    // Out-of-range targets error rather than corrupt.
    let mut sg3 = sg2;
    assert!(sg3.set_mission_status(9, 0, 4).is_err());
    assert!(sg3.set_mission_status(0, 999, 4).is_err());
}

#[test]
fn resolves_mission_names() {
    let path = dumps_dir().join("Missions.json");
    if !path.exists() {
        eprintln!("no Missions.json at {}; skipping", path.display());
        return;
    }
    let missions = Missions::from_path(&path).expect("parse Missions.json");
    assert!(missions.len() > 100);
    if let Some(sg) = fixture_save() {
        // The fixture's plot mission number resolves to a named mission.
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
}
