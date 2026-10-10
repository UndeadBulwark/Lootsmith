//! Vehicle customizations (field 57) and steering mode (field 58).

use lootsmith_core::{container, Customizations, Platform, VehicleCustomization, VEHICLE_FAMILIES};

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
fn fixture_has_no_vehicles_but_a_steering_mode() {
    let Some(sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    assert!(sg.vehicles().is_empty(), "fixture has no chosen vehicles");
    assert_eq!(sg.vehicle_steering_mode(), Some(2));
}

#[test]
fn sets_vehicles_and_steering_and_reloads() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let runner_family = VEHICLE_FAMILIES[0].1;
    sg.set_vehicles(&[VehicleCustomization {
        family: runner_family.to_string(),
        customizations: vec![
            "GD_AllCustoms_Runner.Skin_Fire".to_string(),
            "None".to_string(),
        ],
    }]);
    sg.set_vehicle_steering_mode(1);

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());

    let sg2 = reopened.save_game().expect("reparse");
    let vehicles = sg2.vehicles();
    assert_eq!(vehicles.len(), 1);
    assert_eq!(vehicles[0].family, runner_family);
    assert_eq!(
        vehicles[0].customizations,
        vec![
            "GD_AllCustoms_Runner.Skin_Fire".to_string(),
            "None".to_string()
        ]
    );
    assert_eq!(sg2.vehicle_steering_mode(), Some(1));

    // Clearing the list removes field 57 again.
    let mut sg3 = sg2;
    sg3.set_vehicles(&[]);
    assert!(sg3.vehicles().is_empty());
}

#[test]
fn resolves_skin_options_for_each_family() {
    let path = dumps_dir().join("Customizations.json");
    if !path.exists() {
        eprintln!("no Customizations.json at {}; skipping", path.display());
        return;
    }
    let customizations = Customizations::from_path(&path).expect("parse Customizations.json");
    assert!(customizations.len() > 1000);
    for (usage, _family) in VEHICLE_FAMILIES {
        let options = customizations.skin_options(usage);
        assert!(
            options.len() >= 60,
            "{usage} should offer a large skin list, got {}",
            options.len()
        );
        // Every option is a real key whose name resolves.
        for (key, name) in &options {
            assert_eq!(&customizations.label_for(key), name);
        }
    }
}
