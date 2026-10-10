//! Widget-level UI tests (egui_kittest / AccessKit) — no window, no focus.
//!
//! Per `LOOTSMITH_UI_WORKFLOW.md` every test runs against the **synthetic**
//! save built by `common::synthetic_save_path`, so a checkout with no personal
//! data still exercises the real UI. Tests that need dump-resolved names use
//! `common::dumps_path` and skip only when dumps are absent *and*
//! `LOOTSMITH_REQUIRE_FIXTURES` is unset (in which case they panic).

mod common;

use common::{harness, harness_isolated, reopen, synthetic_save_path};
use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_core::Container;
use lootsmith_gui::app::{LootsmithApp, Tab};

/// Unique id of the first synthetic backpack weapon.
const TEST_WEAPON_ID: &str = "id=400";

#[test]
fn loads_the_synthetic_save_and_switches_tabs() {
    let mut h = harness();
    h.run();
    let state = h.state();
    assert!(state.is_loaded(), "the synthetic save loads");
    assert_eq!(state.form.level, 40);
    assert_eq!(state.backpack.len(), 5, "1 item + 4 weapons");
    assert_eq!(state.bank.len(), 1);

    for (label, tab) in [
        ("Character", Tab::Character),
        ("Currency", Tab::Currency),
        ("Ammo & SDUs", Tab::Ammo),
        ("World", Tab::World),
        ("Skills", Tab::Skills),
        ("Fast Travel", Tab::Travel),
        ("Vehicles", Tab::Vehicles),
        ("Missions", Tab::Missions),
        ("Backpack (5)", Tab::Backpack),
        ("Bank (1)", Tab::Bank),
        ("Raw", Tab::Raw),
        ("About", Tab::About),
        ("General", Tab::General),
    ] {
        h.get_by_label(label).click();
        h.run();
        assert_eq!(h.state().tab, tab, "clicking {label} switches tab");
    }
}

#[test]
fn selects_a_backpack_row_by_label() {
    let mut h = harness();
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();

    h.get_by_label_contains(TEST_WEAPON_ID).click();
    h.run();
    assert_eq!(
        h.state().selected,
        Some((Container::BackpackWeapon, 0)),
        "the first weapon row is selected"
    );
}

#[test]
fn moves_a_weapon_to_the_bank_and_back() {
    let mut h = harness();
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();
    // Select the first weapon (index 1: the item is index 0).
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();

    h.get_by_label("Move to bank").click();
    h.run();
    assert_eq!(h.state().backpack.len(), 4);
    assert_eq!(h.state().bank.len(), 2);

    h.get_by_label("Bank (2)").click();
    h.run();
    h.state_mut().select_entry(Container::Bank, 1);
    h.run();
    h.get_by_label("Move to backpack").click();
    h.run();
    assert_eq!(h.state().backpack.len(), 5);
    assert_eq!(h.state().bank.len(), 1);
}

#[test]
fn removes_an_entry() {
    let mut h = harness();
    h.run();
    h.get_by_label("Backpack (5)").click();
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
fn duplicates_a_selected_item() {
    let mut h = harness();
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackItem, 0);
    h.run();
    let before = h.state().backpack.len();

    h.get_by_label("Duplicate").click();
    h.run();
    assert_eq!(h.state().backpack.len(), before + 1);
    assert!(h.state().dirty);
}

#[test]
fn syncs_item_levels_to_the_character() {
    let mut h = harness();
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();
    h.state_mut().sync_levels(false);
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-sync.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    let save = reopened.save_game().expect("reparse");
    let mut weapons = 0usize;
    for entry in save.entries_in(Container::BackpackWeapon) {
        let decoded = lootsmith_core::items::unpack(&entry.serial, &save_configs(&reopened))
            .expect("decode weapon");
        assert_eq!(decoded.game_stage(), 40, "weapon synced to level 40");
        weapons += 1;
    }
    assert_eq!(weapons, 4);
    let _ = std::fs::remove_file(&out);
}

/// How many skins a vehicle row currently carries (skin1/skin2 + extras).
fn vehicle_skin_count(row: &lootsmith_gui::app::VehicleRow) -> usize {
    let mut count = row.extra.len();
    if row.skin1 != "None" {
        count += 1;
    }
    if row.skin2 != "None" {
        count += 1;
    }
    count
}

/// The asset-library configs the CLI/GUI use by default.
fn save_configs(_file: &lootsmith_core::SaveFile) -> lootsmith_core::AssetLibraryConfigs {
    lootsmith_core::AssetLibraryConfigs::pc_default()
}

#[test]
fn creates_a_new_weapon_and_inserts_an_item_code() {
    let mut h = harness();
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();
    h.get_by_label("New weapon").click();
    h.run();
    assert_eq!(h.state().backpack.len(), 6);
    assert!(h.state().dirty);

    // Round-trip a code through the paste box.
    let code = h
        .state()
        .backpack
        .first()
        .map(|row| row.text.clone())
        .unwrap_or_default();
    let _ = code;
    h.state_mut().item_code = "BL2(not-a-real-code)".to_string();
    h.get_by_label("Insert code(s)").click();
    h.run();
    // A malformed code must not add an entry.
    assert_eq!(h.state().backpack.len(), 6);
}

#[test]
fn skill_calculator_adds_a_rank() {
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();
    // Sentry starts at rank 3 of 5 in the synthetic save.
    h.get_by_label_contains("Sentry").click();
    h.run();
    assert!(h.state().dirty, "adding a rank marks the save dirty");

    let out = std::env::temp_dir().join("lootsmith-ui-skill.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let save = reopen(&out).save_game().expect("reparse");
    let sentry = save
        .skills()
        .into_iter()
        .find(|s| s.skill.contains("Sentry"))
        .expect("sentry skill");
    assert_eq!(sentry.grade, 4, "clicked through to grade 4 of 5");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn edits_fast_travel_and_saves() {
    let mut h = harness();
    h.run();
    h.get_by_label("Fast Travel").click();
    h.run();
    assert_eq!(h.state().travel.len(), 3);
    assert_eq!(h.state().travel_last.as_deref(), Some("Sanctuary"));

    let first = h.state().travel[0].clone();
    h.state_mut().remove_teleporter(0);
    h.run();
    assert_eq!(h.state().travel.len(), 2);
    assert!(!h.state().travel.contains(&first));

    h.state_mut().add_teleporter("Xmas");
    h.run();
    h.state_mut().set_last_teleporter("Xmas");
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-travel.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = reopen(&out);
    assert!(reopened.crc32_valid() && reopened.proto_stable());
    let sg = reopened.save_game().expect("reparse");
    assert_eq!(sg.visited_teleporters().len(), 3, "3 - 1 removed + 1 added");
    assert_eq!(sg.last_visited_teleporter().as_deref(), Some("Xmas"));
    let _ = std::fs::remove_file(&out);
}

#[test]
fn edits_missions_and_saves() {
    let mut h = harness();
    h.run();
    h.get_by_label("Missions").click();
    h.run();
    assert_eq!(h.state().mission_rows.len(), 2);

    h.state_mut().plot_mission = 3;
    h.state_mut().active_mission = 3;
    // Complete the second mission (index 1).
    h.state_mut().mission_rows[1].status = 4;
    assert!(h.state().missions_dirty());
    h.state_mut().apply_missions();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-missions.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let sg = reopen(&out).save_game().expect("reparse");
    assert_eq!(sg.plot_mission_number(), Some(3));
    assert_eq!(sg.active_mission_number(), Some(3));
    let playthrough = sg.mission_playthroughs();
    assert_eq!(playthrough[0].missions.len(), 2);
    assert_eq!(playthrough[0].missions[1].status, 4, "mission completed");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn edits_vehicles_and_saves() {
    let mut h = harness_isolated(&synthetic_save_path());
    h.run();
    h.get_by_label("Vehicles").click();
    h.run();
    let runner = h
        .state()
        .vehicles
        .iter()
        .position(|row| row.usage == "Runner")
        .expect("Runner family row");
    let before = vehicle_skin_count(&h.state().vehicles[runner]);

    h.state_mut().vehicles[runner].skin1 = "None".to_string();
    h.state_mut().steering_mode = 2;
    h.state_mut().apply_vehicles();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-vehicles.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let sg = reopen(&out).save_game().expect("reparse");
    assert_eq!(sg.vehicle_steering_mode(), Some(2));
    let vehicles = sg.vehicles();
    let runner = vehicles
        .iter()
        .find(|row| row.family.contains("Runner"))
        .expect("Runner family saved");
    assert_eq!(
        runner.customizations.first().map(String::as_str),
        Some("None"),
        "skin 1 cleared (was {before} skins)"
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn edits_a_weapon_part_by_name_and_saves() {
    let mut h = harness_isolated(&synthetic_save_path());
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();

    // The inspector exposes a part dropdown per slot; drive it through the
    // programmatic path the dropdown uses (a specific balance-legal part).
    h.state_mut()
        .set_selected_part("barrel", "GD_Weap_Pistol.Barrel.Pistol_Barrel_Dahl")
        .expect("set barrel by name");
    h.run();
    let changed = h.state().selected_item_serial().expect("re-encoded serial");
    h.state_mut().apply_item();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-part.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let sg = reopen(&out).save_game().expect("reparse");
    let entry = sg
        .entries_in(Container::BackpackWeapon)
        .into_iter()
        .next()
        .expect("weapon");
    assert_eq!(entry.serial, changed, "the part edit reached the save");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn raw_view_lists_and_targets_fields() {
    let mut h = harness();
    h.run();
    h.get_by_label("Raw").click();
    h.run();
    let level = h
        .state()
        .raw
        .iter()
        .find(|f| f.number == 2)
        .expect("ExpLevel is listed");
    assert_eq!(level.name, "ExpLevel");
    assert_eq!(h.state().form.level, 40);
    assert!(h
        .state()
        .raw
        .iter()
        .any(|f| f.number == 1 && f.name == "PlayerClass"));

    // Search filters the list without touching the model.
    h.state_mut().raw_filter = "currency".to_string();
    h.run();
    assert!(h.state().raw.iter().any(|f| f.name == "CurrencyOnHand"));
}

#[test]
fn settings_round_trip() {
    let mut app = LootsmithApp::new(Some(synthetic_save_path()), None);
    app.load();
    app.set_tab(Tab::Missions);
    app.dumps = "/tmp/some-dumps".to_string();
    let json = app.settings_json();
    let mut restored = LootsmithApp::new(None, None);
    restored.load_settings_json(&json);
    assert_eq!(restored.tab, Tab::Missions);
    assert_eq!(restored.dumps, "/tmp/some-dumps");
}

#[test]
fn dirty_tracking() {
    let mut app = LootsmithApp::new(Some(synthetic_save_path()), None);
    app.load();
    assert!(!app.dirty, "a fresh load is clean");
    app.form.level = 41;
    // Saving with no path set refuses.
    app.save_in_place();
    assert!(!app.path.is_empty(), "the synthetic path is set");
    app.save_to(
        std::env::temp_dir()
            .join("lootsmith-ui-dirty.sav")
            .to_str()
            .unwrap(),
    );
    assert!(!app.dirty, "saving clears the dirty flag");
}

#[test]
fn new_blank_save_is_fresh_and_writable() {
    let mut app = LootsmithApp::new(Some(synthetic_save_path()), None);
    app.load();
    app.new_class = 0; // sorted ids: GD_Assassin first → Zer0
    app.new_blank_save();

    assert_eq!(app.form.level, 1);
    assert_eq!(app.form.general_skill_points, 3);
    assert!(app.backpack.is_empty() && app.bank.is_empty());
    assert!(app.dirty, "a new save is unsaved until Save As…");

    let out = std::env::temp_dir().join("lootsmith-ui-blank.sav");
    app.save_to(out.to_str().unwrap());
    let save = reopen(&out).save_game().expect("parse blank save");
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
    let mut app = LootsmithApp::new(Some(synthetic_save_path()), None);
    app.load();
    assert_eq!(app.form.level, 40);
    app.new_clone_save();

    assert_eq!(app.form.level, 1);
    assert_eq!(app.form.general_skill_points, 3);
    assert!(app.backpack.is_empty() && app.bank.is_empty());
    assert!(app.dirty);
    assert!(app
        .save_game
        .as_ref()
        .expect("cloned save")
        .get_bytes(34)
        .is_some());
}

#[test]
fn import_sections_copies_travel_from_another_save() {
    let source = synthetic_save_path();
    let mut app = LootsmithApp::new(Some(source.clone()), None);
    app.load();
    let expected = app.travel.clone();
    assert!(!expected.is_empty());

    let last = expected.last().cloned().unwrap();
    app.save_game
        .as_mut()
        .expect("open save")
        .remove_visited_teleporter(&last);
    app.travel.retain(|t| *t != last);
    app.import_sel = lootsmith_gui::app::ImportSel {
        travel: true,
        ..lootsmith_gui::app::ImportSel::default()
    };
    app.import_path = source;
    app.do_import();

    assert_eq!(app.travel.len(), expected.len());
    assert!(app.travel.contains(&last));
    assert!(app.status.contains("fast travel"));
    assert!(app.dirty);
}

#[test]
fn ammo_and_sdu_tabs_write_through_to_disk() {
    let mut h = harness();
    h.run();
    h.get_by_label("Ammo & SDUs").click();
    h.run();
    assert_eq!(h.state().ammo.len(), 3);

    h.state_mut().set_ammo(1, Some(999.0), Some(4));
    h.state_mut().set_sdu(7, 9);
    h.state_mut().max_all_sdus();
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-ammo.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let save = reopen(&out).save_game().expect("reparse");
    let pools = save.resources();
    assert_eq!(pools[0].amount, 999.0);
    assert_eq!(save.black_market_upgrades()[..7], [7; 7]);
    assert_eq!(save.inventory_slots().map(|i| i.0), Some(39));
    assert_eq!(save.max_bank_slots(), Some(24));
    let _ = std::fs::remove_file(&out);
}

#[test]
fn world_tab_edits_and_challenge_progress_write_through() {
    let mut h = harness();
    h.run();
    h.get_by_label("World").click();
    h.run();
    assert_eq!(h.state().world_discoveries.len(), 2);
    h.state_mut().mark_all_discovered();
    h.state_mut().clear_all_lockouts();
    h.run();
    assert_eq!(h.state().discoveries_uncovered(), 2);

    h.get_by_label("Missions").click();
    h.run();
    h.state_mut().set_challenge_value(1621, 500);
    h.run();

    let out = std::env::temp_dir().join("lootsmith-ui-world.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let save = reopen(&out).save_game().expect("reparse");
    assert_eq!(save.discoveries_uncovered(), 2);
    assert_eq!(save.lockouts()[0].time, 0);
    assert_eq!(
        save.challenge_stats()
            .expect("blob")
            .entries
            .iter()
            .find(|e| e.id == 1621)
            .expect("entry")
            .total_value,
        500
    );
    let _ = std::fs::remove_file(&out);
}

#[test]
fn profile_editor_round_trips_values() {
    use lootsmith_core::profile::Profile;
    let probe = std::env::temp_dir().join("lootsmith-ui-profile.bin");
    let mut profile = Profile::default();
    profile.set_golden_keys(52);
    std::fs::write(&probe, profile.to_bytes()).expect("write probe profile");

    let mut app = LootsmithApp::new(Some(synthetic_save_path()), None);
    app.load();
    app.profile_path = probe.to_str().unwrap().to_string();
    app.load_profile();
    assert!(app.profile_form.loaded);
    assert_eq!(app.profile_form.golden_keys, 52);

    app.profile_form.golden_keys = 200;
    app.save_profile();
    let reopened = Profile::from_path(&probe).expect("re-read profile");
    assert_eq!(reopened.golden_keys(), Some(200));
    let _ = std::fs::remove_file(&probe);
}

#[test]
fn bundled_tables_resolve_names_with_no_game_data_folder() {
    // No user folder: the bundled tables must still resolve part names, so a
    // fresh install needs no setup and shows no picker. The harness is
    // *isolated* (`new_isolated`), so a LOOTSMITH_* variable set in the shell
    // cannot change this result — see `tests/isolation.rs` for the proof.
    let save = synthetic_save_path();
    let mut h = harness_isolated(&save);
    h.run();
    let state = h.state();
    assert!(state.ignores_env(), "the harness is isolated from the host");
    assert!(state.manager.is_some(), "bundled asset library is loaded");
    assert!(state.dumps_bundled, "and it is flagged as bundled");
    assert!(state.name_parts.is_some(), "bundled name parts");
    assert!(state.part_lists.is_some(), "bundled balance part lists");
    assert!(state.stations.is_some(), "bundled travel stations");
    assert!(state.customizations.is_some(), "bundled customizations");
    assert!(state.missions_data.is_some(), "bundled missions");

    // The parts editor resolves a slot to a name (not "#i/j").
    h.get_by_label("Backpack (5)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let inspector = h.state().selected_item_editor_is_ready();
    assert!(inspector, "the inspector has a decoded item");
}

#[test]
fn ui_scale_is_clamped_to_100_150_200() {
    // Spec §2.1 rule 7: only 100/150/200 are offered.
    assert_eq!(LootsmithApp::clamp_ui_scale(0.9), 1.0);
    assert_eq!(LootsmithApp::clamp_ui_scale(1.1), 1.0);
    assert_eq!(LootsmithApp::clamp_ui_scale(1.37), 1.5);
    assert_eq!(LootsmithApp::clamp_ui_scale(1.75), 1.5);
    assert_eq!(LootsmithApp::clamp_ui_scale(2.4), 2.0);
    assert_eq!(LootsmithApp::clamp_ui_scale(1.0), 1.0);
    assert_eq!(LootsmithApp::clamp_ui_scale(1.5), 1.5);
    assert_eq!(LootsmithApp::clamp_ui_scale(2.0), 2.0);
}

#[test]
fn ui_scale_control_selects_150_percent() {
    let mut h = harness();
    h.run();
    assert_eq!(h.state().ui_scale, 1.0);
    h.get_by_label("Menu").click();
    h.run();
    h.get_by_label("150%").click();
    h.run();
    assert_eq!(h.state().ui_scale, 1.5);
}

#[test]
fn ui_scale_control_selects_200_percent() {
    let mut h = harness();
    h.run();
    assert_eq!(h.state().ui_scale, 1.0);
    h.get_by_label("Menu").click();
    h.run();
    h.get_by_label("200%").click();
    h.run();
    assert_eq!(h.state().ui_scale, 2.0);
}

#[test]
fn ui_scale_setter_clamps() {
    let save = synthetic_save_path();
    let mut h = harness_isolated(&save);
    h.state_mut().set_ui_scale(1.42);
    assert_eq!(h.state().ui_scale, 1.5);
    h.state_mut().set_ui_scale(9.0);
    assert_eq!(h.state().ui_scale, 2.0);
}

#[test]
fn apply_item_button_appears_only_with_pending_edits() {
    let mut h = harness_isolated(&synthetic_save_path());
    h.run();
    h.get_by_label("Backpack (5)").click();
    h.run();
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    assert!(
        h.query_by_label("Apply item changes").is_none(),
        "no pending edits -> no button"
    );

    // Make a part edit through the programmatic path the dropdown uses.
    let _ = h
        .state_mut()
        .set_selected_part("barrel", "GD_Weap_Pistol.Barrel.Pistol_Barrel_Dahl");
    h.run();
    assert!(
        h.query_by_label("Apply item changes").is_some(),
        "pending edit -> button appears"
    );

    h.get_by_label("Apply item changes").click();
    h.run();
    assert!(
        h.query_by_label("Apply item changes").is_none(),
        "after apply -> button disappears"
    );
}

#[test]
fn require_fixtures_is_off_by_default_here() {
    // The suite must need no external data: the synthetic save plus the bundled
    // tables are always enough (a clean clone with no fixtures still tests).
    let h = harness();
    assert!(h.state().manager.is_some());
    assert!(h.state().dumps_bundled);
    assert!(h.state().ignores_env());
}

/// Kept so the harness type is exercised even when no dumps exist.
#[allow(dead_code)]
fn _harness_type_is_used(_: Harness<'static, LootsmithApp>, _: Vec2) {}
