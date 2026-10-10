//! Widget-level, focus-free UI tests via `egui_kittest` (AccessKit).
//!
//! These manipulate the UI by widget label — no window, no coordinates, no
//! focus — and assert on the application state.

use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_core::Container;
use lootsmith_gui::app::{LootsmithApp, Tab};

/// Unique id of the first backpack weapon (Dahl pistol) in the fixture.
const DAHL_PISTOL_ID: &str = "id=1572495195";

fn fixtures() -> Option<(String, String)> {
    let save = format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    );
    let dumps = std::env::var("LOOTSMITH_ASSET_LIBRARY").ok().or_else(|| {
        Some(format!(
            "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
            env!("CARGO_MANIFEST_DIR")
        ))
    })?;
    if std::path::Path::new(&save).exists() && std::path::Path::new(&dumps).exists() {
        Some((save, dumps))
    } else {
        None
    }
}

fn harness(save: String, dumps: String) -> Harness<'static, LootsmithApp> {
    // A tall window so the item editor's buttons stay on-screen for pointer clicks.
    Harness::builder()
        .with_size(Vec2::new(1100.0, 1000.0))
        .build_ui_state(
            |ui, app: &mut LootsmithApp| app.build(ui),
            LootsmithApp::new(Some(save), Some(dumps)),
        )
}

fn reopen(path: &std::path::Path) -> lootsmith_core::SaveFile {
    let bytes = std::fs::read(path).expect("read saved file");
    lootsmith_core::container::unpack(&bytes, lootsmith_core::Platform::PC).expect("reopen")
}

#[test]
fn loads_and_switches_tabs_by_label() {
    let Some((save, dumps)) = fixtures() else {
        eprintln!("fixtures missing; skipping");
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    assert!(h.state().is_loaded(), "save should load on the first frame");
    assert_eq!(h.state().backpack.len(), 14, "5 items + 9 weapons");
    assert!(h.state().bank.is_empty(), "fixture bank is empty");

    for (label, tab) in [
        ("Character", Tab::Character),
        ("Currency", Tab::Currency),
        ("Skills", Tab::Skills),
        ("Fast Travel", Tab::Travel),
        ("Vehicles", Tab::Vehicles),
        ("Backpack", Tab::Backpack),
        ("Bank", Tab::Bank),
        ("General", Tab::General),
    ] {
        h.get_by_label(label).click();
        h.run();
        assert_eq!(h.state().tab, tab, "clicking {label} should switch tab");
    }
}

#[test]
fn selects_a_backpack_row_by_label() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack").click();
    h.run();

    h.get_by_label_contains(DAHL_PISTOL_ID).click();
    h.run();
    assert_eq!(
        h.state().selected,
        Some((Container::BackpackWeapon, 0)),
        "clicking a row should select that entry"
    );
}

#[test]
fn edits_a_weapon_part_and_saves() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let before = h
        .state()
        .item_serial(Container::BackpackWeapon, 0)
        .expect("serial");

    h.state_mut()
        .set_selected_part(
            "elemental",
            "GD_Weap_Pistol.elemental.Pistol_Elemental_Fire",
        )
        .expect("set part");
    h.state_mut().apply_item();
    h.run();

    let after = h
        .state()
        .item_serial(Container::BackpackWeapon, 0)
        .expect("serial");
    assert_ne!(before, after, "the edited serial should differ");

    let out = std::env::temp_dir().join("lootsmith-ui-test.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid());
    assert!(reopened.proto_stable());
    assert!(reopened.huffman_stable());
}

#[test]
fn moves_a_weapon_to_the_bank_and_back() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack").click();
    h.run();
    h.get_by_label_contains(DAHL_PISTOL_ID).click();
    h.run();
    assert_eq!(h.state().selected, Some((Container::BackpackWeapon, 0)));

    h.get_by_label("Move to bank").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Bank, "moving switches to the Bank tab");
    assert_eq!(h.state().bank.len(), 1);
    assert_eq!(h.state().backpack.len(), 13);
    assert_eq!(h.state().selected, Some((Container::Bank, 0)));

    let out = std::env::temp_dir().join("lootsmith-ui-bank.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    assert_eq!(sg.bank_entries().len(), 1, "bank item should persist");

    h.get_by_label("Move to backpack").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Backpack);
    assert!(h.state().bank.is_empty());
    assert_eq!(h.state().backpack.len(), 14);
}

#[test]
fn edits_a_skill_and_saves() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.get_by_label("Skills").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Skills);
    assert_eq!(h.state().skills.len(), 33);

    let index = h
        .state()
        .skills
        .iter()
        .position(|s| s.grade == 0)
        .expect("a rank-0 skill");
    h.state_mut().skills[index].grade = 5;
    h.run();
    assert!(h.state().skills_dirty());
    h.get_by_label("Apply skill changes").click();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-skills.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    assert_eq!(sg.skills()[index].grade, 5, "the skill grade should persist");
}

#[test]
fn edits_fast_travel_and_saves() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.get_by_label("Fast Travel").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Travel);
    assert_eq!(h.state().travel.len(), 23);
    assert_eq!(h.state().travel_last.as_deref(), Some("SouthernShelfTown"));
    assert!(
        h.state().stations.is_some(),
        "Travel Stations.json should load from the dumps directory"
    );

    // Remove the first visited station programmatically.
    let first = h.state().travel[0].clone();
    h.state_mut().remove_teleporter(0);
    h.run();
    assert_eq!(h.state().travel.len(), 22);
    assert!(!h.state().travel.contains(&first));

    // Add one through the dropdown + button.
    h.state_mut().new_station = Some("Xmas".to_string());
    h.run();
    h.get_by_label("Add station").click();
    h.run();
    assert!(h.state().travel.iter().any(|s| s == "Xmas"));

    h.state_mut().set_last_teleporter("Xmas");
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-travel.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    assert_eq!(sg.visited_teleporters().len(), 23, "23 - 1 removed + 1 added");
    assert!(sg.visited_teleporters().iter().any(|s| s == "Xmas"));
    assert!(!sg.visited_teleporters().contains(&first));
    assert_eq!(sg.last_visited_teleporter().as_deref(), Some("Xmas"));
}

#[test]
fn edits_vehicles_and_saves() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.get_by_label("Vehicles").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Vehicles);
    assert!(
        h.state().vehicles.iter().all(|r| r.skin1 == "None"),
        "the fixture has no chosen vehicle skins"
    );
    assert_eq!(h.state().steering_mode, 2);
    assert!(
        h.state().customizations.is_some(),
        "Customizations.json should load from the dumps directory"
    );

    // Pick the first Runner skin, as the combo box would.
    let key = h
        .state()
        .customizations
        .clone()
        .unwrap()
        .skin_options("Runner")
        .first()
        .expect("a Runner skin")
        .0
        .clone();
    h.state_mut().vehicles[0].skin1 = key.clone();
    h.state_mut().steering_mode = 0;
    h.run();
    assert!(h.state().vehicles_dirty());
    h.get_by_label("Apply vehicle changes").click();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-vehicles.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    let vehicles = sg.vehicles();
    assert_eq!(vehicles.len(), 1);
    assert_eq!(
        vehicles[0].family,
        "GD_Globals.VehicleSpawnStation.VehicleFamily_Runner"
    );
    assert_eq!(vehicles[0].customizations[0], key);
    assert_eq!(sg.vehicle_steering_mode(), Some(0));
}

#[test]
fn removes_an_entry() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let before = h.state().backpack.len();

    h.get_by_label("Remove").click();
    h.run();
    assert_eq!(h.state().backpack.len(), before - 1);
    assert_eq!(h.state().selected, None);
}
