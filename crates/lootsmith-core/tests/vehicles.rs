//! Vehicle customizations (field 57) and steering mode (field 58), on the
//! synthetic save, plus the bundled `Customizations.json` lookup.

mod common;

use common::synthetic;
use lootsmith_core::{container, Platform, VehicleCustomization, VEHICLE_FAMILIES};

#[test]
fn synthetic_has_one_vehicle_and_a_steering_mode() {
    let sg = synthetic();
    let vehicles = sg.vehicles();
    assert_eq!(
        vehicles.len(),
        1,
        "the synthetic save has one vehicle family"
    );
    assert!(vehicles[0].family.contains("Runner"));
    assert_eq!(sg.vehicle_steering_mode(), Some(0));
}

#[test]
fn sets_vehicles_and_steering_and_reloads() {
    let mut sg = synthetic();
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
    let customizations =
        lootsmith_core::bundled::customizations().expect("bundled Customizations.json");
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
