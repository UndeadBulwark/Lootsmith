//! Widget-level, focus-free UI tests via `egui_kittest` (AccessKit).
//!
//! These manipulate the UI by widget label — no window, no coordinates, no
//! focus — and assert on the application state.

use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_core::Container;
use lootsmith_gui::app::{LootsmithApp, RawEdit, Tab, Theme};

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
        ("Missions", Tab::Missions),
        ("Backpack (14)", Tab::Backpack),
        ("Bank (0)", Tab::Bank),
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
    h.get_by_label("Backpack (14)").click();
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
    h.get_by_label("Backpack (14)").click();
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
fn skill_calculator_adds_a_rank() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.get_by_label("Skills").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Skills);
    assert_eq!(h.state().skills.len(), 33);

    let grade = |h: &Harness<'static, LootsmithApp>| {
        h.state()
            .skills
            .iter()
            .find(|r| r.name.ends_with(".Sentry"))
            .expect("Sentry entry")
            .grade
    };
    let before = grade(&h);

    // The tier-0 Guerilla skill "Sentry" is unlocked; clicking the node adds a rank.
    h.get_by_label_contains("Sentry").click();
    h.run();
    assert_eq!(grade(&h), before + 1, "clicking the node adds a rank");

    let out = std::env::temp_dir().join("lootsmith-ui-skills.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    let persisted = sg
        .skills()
        .into_iter()
        .find(|s| s.skill.ends_with(".Sentry"))
        .expect("Sentry in save");
    assert_eq!(persisted.grade, before + 1, "the rank persists");
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

    // Add one programmatically (the tab now uses per-station checkboxes).
    h.state_mut().add_teleporter("Xmas");
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
fn edits_missions_and_saves() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.get_by_label("Missions").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Missions);
    assert_eq!(h.state().plot_mission, 18);
    assert_eq!(h.state().active_mission, 18);
    assert_eq!(h.state().mission_rows.len(), 19);
    assert_eq!(h.state().challenges.len(), 132);
    assert!(
        h.state().missions_data.is_some(),
        "Missions.json should load from the dumps directory"
    );

    // Mark the active mission Complete and bump the plot mission, as the UI would.
    h.state_mut().mission_rows[18].status = 4;
    h.state_mut().plot_mission = 19;
    h.run();
    assert!(h.state().missions_dirty());
    h.get_by_label("Apply mission changes").click();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-missions.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    assert_eq!(sg.plot_mission_number(), Some(19));
    assert_eq!(sg.mission_playthroughs()[0].missions[18].status, 4);
}

#[test]
fn settings_round_trip() {
    let mut app = LootsmithApp::new(None, None);
    app.path = "/tmp/some/Save0007.sav".to_string();
    app.dumps = "/tmp/some/dumps".to_string();
    app.theme = Theme::Light;
    app.tab = Tab::Missions;

    let json = app.settings_json();
    let mut restored = LootsmithApp::new(None, None);
    restored.load_settings_json(&json);
    assert_eq!(restored.path, "/tmp/some/Save0007.sav");
    assert_eq!(restored.dumps, "/tmp/some/dumps");
    assert_eq!(restored.theme, Theme::Light);
    assert_eq!(restored.tab, Tab::Missions);
}

#[test]
fn dirty_tracking() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    assert!(!h.state().dirty, "freshly loaded save is not dirty");

    h.state_mut().add_teleporter("Xmas");
    assert!(h.state().dirty, "an edit marks the document dirty");

    let out = std::env::temp_dir().join("lootsmith-ui-dirty.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    assert!(!h.state().dirty, "saving clears the dirty flag");
}

#[test]
fn creates_duplicates_and_inserts_codes() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack (14)").click();
    h.run();
    let before = h.state().backpack.len();

    // Create a blank weapon.
    h.get_by_label("New weapon").click();
    h.run();
    assert_eq!(h.state().backpack.len(), before + 1);
    assert_eq!(h.state().tab, Tab::Backpack);
    assert!(h.state().dirty);

    // The new weapon is selected; round-trip it through a BL2 code.
    let code = {
        let (container, occurrence) = h.state().selected.unwrap();
        lootsmith_core::items::encode_code(
            &h.state().item_serial(container, occurrence).unwrap(),
        )
    };
    h.state_mut().item_code = code;
    h.run();
    h.get_by_label("Insert code(s)").click();
    h.run();
    assert_eq!(h.state().backpack.len(), before + 2);
}

#[test]
fn duplicates_a_selected_item() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack (14)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let before = h.state().backpack.len();
    h.get_by_label("Duplicate").click();
    h.run();
    assert_eq!(h.state().backpack.len(), before + 1);
}

#[test]
fn syncs_item_levels() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack (14)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.state_mut().form.level = 60;
    h.run();
    h.get_by_label("Sync equipped").click();
    h.run();

    let serial = h
        .state()
        .item_serial(Container::BackpackWeapon, 0)
        .unwrap();
    let configs = lootsmith_core::AssetLibraryConfigs::pc_default();
    let decoded = lootsmith_core::items::unpack(&serial, &configs).unwrap();
    match decoded.slot {
        lootsmith_core::items::PackedSlot::Weapon(weapon) => {
            assert_eq!(weapon.manufacturer_grade_index, 60);
            assert_eq!(weapon.game_stage, 60);
        }
        _ => panic!("expected a weapon"),
    }
}

#[test]
fn raw_view_lists_and_targets_fields() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Raw").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Raw);
    let level = h
        .state()
        .raw
        .iter()
        .find(|f| f.number == 2)
        .expect("ExpLevel is listed");
    assert_eq!(level.name, "ExpLevel");
    assert!(matches!(level.edit, RawEdit::Int(72)));
    assert!(h.state().raw.iter().any(|f| f.number == 1 && f.name == "PlayerClass"));
}

#[test]
fn removes_an_entry() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Backpack (14)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let before = h.state().backpack.len();

    h.get_by_label("Remove").click();
    h.run();
    assert_eq!(h.state().backpack.len(), before - 1);
    assert_eq!(h.state().selected, None);
}

#[test]
fn new_blank_save_is_fresh_and_writable() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();
    app.new_class = 0; // sorted ids: GD_Assassin first → Zer0
    app.new_blank_save();

    let state = &app;
    assert!(state.save_game.is_some());
    assert_eq!(state.form.level, 1);
    assert_eq!(state.form.general_skill_points, 3);
    assert!(state.backpack.is_empty() && state.bank.is_empty());
    assert!(state.dirty, "a new save is unsaved until Save As…");
    assert!(state.status.contains("Zer0") || state.status.contains("Assassin"));

    let out = std::env::temp_dir().join("lootsmith-test-blank.sav");
    app.save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    let save = reopened.save_game().expect("parse blank save");
    assert_eq!(
        save.player_class().as_deref(),
        Some("GD_Assassin.Character.CharClass_Assassin")
    );
    assert_eq!(save.exp_level(), Some(1));
    assert_eq!(save.currency_on_hand(), [0, 0, 0, 0]);
    let _ = std::fs::remove_file(&out);
}

#[test]
fn new_clone_save_resets_progress() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();
    assert_eq!(app.form.level, 72);
    app.new_clone_save();

    assert_eq!(app.form.level, 1);
    assert_eq!(app.form.general_skill_points, 3);
    assert!(app.backpack.is_empty() && app.bank.is_empty());
    assert!(app.dirty);
    let guid = app
        .save_game
        .as_ref()
        .expect("cloned save")
        .get_bytes(34)
        .map(|b| b.to_vec());
    assert!(guid.is_some());
}

#[test]
fn import_sections_copies_travel_from_another_save() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    // Work on a copy so the fixture itself is never touched.
    let copy = std::env::temp_dir().join("lootsmith-test-import.sav");
    std::fs::copy(&save, &copy).expect("copy fixture");
    let mut app = LootsmithApp::new(Some(copy.to_str().unwrap().to_string()), Some(dumps));
    app.load();

    let expected = app.travel.clone();
    assert!(!expected.is_empty(), "fixture has visited teleporters");
    let last = expected.last().cloned().unwrap();
    app.save_game
        .as_mut()
        .expect("open save")
        .remove_visited_teleporter(&last);
    app.travel.retain(|t| *t != last);
    assert_eq!(app.travel.len(), expected.len() - 1);

    app.import_sel = lootsmith_gui::app::ImportSel {
        travel: true,
        ..lootsmith_gui::app::ImportSel::default()
    };
    app.import_path = copy.to_str().unwrap().to_string();
    app.do_import();

    assert_eq!(app.travel.len(), expected.len());
    assert!(app.travel.contains(&last));
    assert!(app.status.contains("fast travel"));
    assert!(app.dirty);
    let _ = std::fs::remove_file(&copy);
}

#[test]
fn profile_editor_round_trips_values() {
    use lootsmith_core::profile::Profile;
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    // A small profile with 52 golden keys.
    let probe = std::env::temp_dir().join("lootsmith-test-profile.bin");
    let mut profile = Profile::default();
    profile.set_golden_keys(52);
    std::fs::write(&probe, profile.to_bytes()).expect("write probe profile");

    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();
    app.profile_path = probe.to_str().unwrap().to_string();
    app.load_profile();

    assert!(app.profile_form.loaded);
    assert_eq!(app.profile_form.golden_keys, 52);

    app.profile_form.golden_keys = 200;
    app.save_profile();

    println!("STATUS: {}", app.status);
    let reopened = Profile::from_path(&probe).expect("re-read profile");
    assert_eq!(reopened.golden_keys(), Some(200));
    let _ = std::fs::remove_file(&probe);
}

#[test]
fn balance_constrained_choices_use_the_part_lists() {
    use lootsmith_core::AssetGroup;
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();

    let Some(lists) = app.part_lists.clone() else {
        eprintln!("no part lists in game data; skipping");
        return;
    };
    assert!(lists.len() > 100, "part lists loaded: {}", lists.len());

    // Find an item whose balance has a part list record, then check that the
    // picker offers the balance's parts.
    let mut found = false;
    for row in app.backpack.clone() {
        if row.balance.starts_with('#') {
            continue; // balance not resolved by the manager
        }
        let Some(record) = lists.for_balance(&row.balance) else {
            continue;
        };
        let slot = ["barrel", "body", "alpha", "beta", "gamma"]
            .into_iter()
            .find(|slot| record.slots.contains_key(*slot));
        let Some(slot) = slot else { continue };
        app.select_entry(row.container, row.occurrence);
        let choices = app.balanced_choices(&0, &row.balance, AssetGroup::WeaponParts, slot);
        let expected = record
            .slots
            .get(slot)
            .and_then(|names| names.first())
            .expect("record has parts");
        assert_eq!(
            choices.first().map(String::as_str),
            Some(expected.as_str()),
            "balance parts come first for {slot} on {}",
            row.balance
        );
        found = true;
        break;
    }
    assert!(found, "no backpack item matched a balance part list");
}

#[test]
fn items_get_display_names_from_name_parts() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();
    let Some(parts) = app.name_parts.clone() else {
        eprintln!("name parts missing; skipping");
        return;
    };
    assert!(parts.len() > 500, "name parts loaded: {}", parts.len());
    let named = app
        .backpack
        .iter()
        .filter(|row| !row.empty && !row.name.is_empty())
        .count();
    assert!(named > 0, "at least one item resolves to a display name");
}

#[test]
fn change_class_updates_flags_and_calculator() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();
    assert_eq!(app.read_info.player_class, "GD_Soldier.Character.CharClass_Soldier");

    app.change_class("GD_Siren.Character.CharClass_Siren");
    let save = app.save_game.as_ref().expect("open save");
    assert_eq!(save.player_class(), Some("GD_Siren.Character.CharClass_Siren".into()));
    assert_eq!(save.get_bool(44), Some(false));
    assert_eq!(save.get_i32(45), Some(0));
    assert_eq!(app.read_info.player_class, "GD_Siren.Character.CharClass_Siren");

    app.change_class("GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass");
    let save = app.save_game.as_ref().expect("open save");
    assert_eq!(save.get_bool(44), Some(true));
    assert_eq!(save.get_i32(45), Some(6), "Psycho Pack is package 6");

    app.change_class("GD_Tulip_Mechromancer.Character.CharClass_Mechromancer");
    let save = app.save_game.as_ref().expect("open save");
    assert_eq!(save.get_i32(45), Some(2), "Mechromancer is package 2");
    assert!(app.dirty);
}

#[test]
fn item_game_stage_edits_through_the_inspector_model() {
    use lootsmith_core::{items as core_items, Platform};
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut app = LootsmithApp::new(Some(save), Some(dumps));
    app.load();
    app.select_entry(Container::BackpackWeapon, 0);
    let mut before = 0;
    app.selected_item_editor(|edit| {
        before = edit.decoded.game_stage();
        edit.decoded.set_game_stage(50);
        edit.decoded.set_manufacturer_grade(2);
        edit.dirty = true;
    });
    assert_ne!(before, 0, "weapon has a game stage");
    app.apply_item();
    let serial = app
        .save_game
        .as_ref()
        .expect("open save")
        .entries_in(Container::BackpackWeapon)
        .into_iter()
        .next()
        .expect("weapon entry")
        .serial
        .clone();
    let configs = app.configs();
    let decoded = core_items::unpack(&serial, &configs).expect("re-decode");
    assert_eq!(decoded.game_stage(), 50);
    assert_eq!(decoded.manufacturer_grade(), 2);
    assert!(app.dirty);
    let _ = Platform::PC;
}
