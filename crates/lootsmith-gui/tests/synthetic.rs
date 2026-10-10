//! CI smoke tests using the synthetic save — no personal fixture required.

use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_core::fixture;
use lootsmith_gui::app::LootsmithApp;

fn harness_with_synthetic() -> Harness<'static, LootsmithApp> {
    static COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let unique = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("lootsmith-synthetic-{unique}.sav"));
    let file = fixture::write_synthetic_save(&path).expect("synthetic save");
    let save = file.save_game().expect("parse");
    assert_eq!(save.exp_level(), Some(40));
    Harness::builder()
        .with_size(Vec2::new(1100.0, 1000.0))
        .build_ui_state(
            |ui, app: &mut LootsmithApp| app.build(ui),
            LootsmithApp::new(Some(path.to_str().unwrap().to_string()), None),
        )
}

#[allow(clippy::unwrap_used)]
#[test]
fn synthetic_save_loads_and_switches_tabs() {
    let mut h = harness_with_synthetic();
    h.run();
    assert!(h.state().is_loaded());
    assert_eq!(h.state().form.level, 40);
    assert_eq!(h.state().backpack.len(), 3, "1 item + 2 weapons");
    assert_eq!(h.state().bank.len(), 1);
    assert_eq!(h.state().travel.len(), 3);

    for label in [
        "Character",
        "Currency",
        "Skills",
        "Fast Travel",
        "Vehicles",
        "Missions",
        "Backpack (3)",
        "Bank (1)",
        "Raw",
        "About",
        "General",
    ] {
        h.get_by_label(label).click();
        h.run();
    }
}

#[allow(clippy::unwrap_used)]
#[test]
fn synthetic_save_skills_click_and_save_round_trip() {
    let mut h = harness_with_synthetic();
    h.run();
    h.get_by_label("Skills").click();
    h.run();
    // Sentry is in the first tree's first row (grade 3 in the fixture).
    h.get_by_label("Sentry").click();
    h.run();
    assert!(h.state().dirty, "adding a rank marks the save dirty");
    let out = std::env::temp_dir().join("lootsmith-synthetic-edited.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = lootsmith_core::container::unpack(
        &std::fs::read(&out).expect("read edited save"),
        lootsmith_core::Platform::PC,
    )
    .expect("reparse");
    let save = reopened.save_game().expect("save game");
    assert_eq!(save.exp_level(), Some(40));
    let sentry = save
        .skills()
        .into_iter()
        .find(|s| s.skill.contains("Sentry"))
        .expect("sentry skill");
    assert_eq!(sentry.grade, 4, "clicked through to grade 4 of 5");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn ammo_tab_edits_pool_amounts() {
    let mut h = harness_with_synthetic();
    h.run();
    h.get_by_label("Ammo").click();
    h.run();
    assert_eq!(h.state().ammo.len(), 3, "the synthetic save has 3 pools");

    // Edit the first pool's amount and SDU (the tab's drag values call the
    // same mutator).
    h.state_mut().set_ammo(1, Some(999.0), Some(4));
    h.run();
    assert!(h.state().dirty, "the ammo edit flows through to the model");
    let out = std::env::temp_dir().join("lootsmith-synthetic-ammo.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = lootsmith_core::container::unpack(
        &std::fs::read(&out).expect("read edited save"),
        lootsmith_core::Platform::PC,
    )
    .expect("reparse");
    let save = reopened.save_game().expect("save game");
    let pools = save.resources();
    assert_eq!(pools.len(), 3);
    assert_eq!(pools[0].amount, 999.0, "amount edited");
    assert_eq!(pools[0].upgrade_level, 4, "sdu edited");
    let _ = std::fs::remove_file(&out);
}

#[test]
fn sdu_edits_and_max_all_stay_consistent() {
    let mut h = harness_with_synthetic();
    h.run();
    h.get_by_label("Ammo").click();
    h.run();
    assert_eq!(
        h.state().sdus,
        [0; 9],
        "the synthetic save starts un-upgraded"
    );

    // Consistent per-SDU edit: the backpack grows to 12 + 3·2 = 18.
    h.state_mut().set_sdu(8, 2);
    h.run();
    assert_eq!(h.state().sdus[7], 2);
    assert_eq!(h.state().backpack_slots, 18);
    assert_eq!(h.state().bank_size, 6, "bank untouched");

    // Slot-size edits derive the SDU level the other way.
    h.state_mut().set_bank_size(24);
    h.run();
    assert_eq!(h.state().sdus[8], 9);
    assert_eq!(h.state().bank_size, 24);

    h.state_mut().max_all_sdus();
    h.run();
    let out = std::env::temp_dir().join("lootsmith-synthetic-sdus.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = lootsmith_core::container::unpack(
        &std::fs::read(&out).expect("read edited save"),
        lootsmith_core::Platform::PC,
    )
    .expect("reparse");
    let save = reopened.save_game().expect("save game");
    let levels = save.black_market_upgrades();
    assert_eq!(levels[..7], [7; 7]);
    assert_eq!(levels[7], 9);
    assert_eq!(levels[8], 9);
    assert_eq!(save.inventory_slots().map(|i| i.0), Some(39));
    assert_eq!(save.max_bank_slots(), Some(24));
    let _ = std::fs::remove_file(&out);
}

#[test]
fn challenge_stats_edit_through_the_missions_tab() {
    let mut h = harness_with_synthetic();
    h.run();
    h.get_by_label("Missions").click();
    h.run();
    assert_eq!(h.state().challenge_stats.len(), 2);
    assert_eq!(h.state().prestiges, 0);

    // Edit challenge 1621's total value (same mutator the drag values use).
    h.state_mut().set_challenge_value(1621, 500);
    h.run();
    let out = std::env::temp_dir().join("lootsmith-synthetic-chal.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let reopened = lootsmith_core::container::unpack(
        &std::fs::read(&out).expect("read edited save"),
        lootsmith_core::Platform::PC,
    )
    .expect("reparse");
    let save = reopened.save_game().expect("save game");
    let stats = save.challenge_stats().expect("blob");
    let edited = stats
        .entries
        .iter()
        .find(|entry| entry.id == 1621)
        .expect("edited entry");
    assert_eq!(edited.total_value, 500);
    assert_eq!(
        stats
            .entries
            .iter()
            .find(|entry| entry.id == 1622)
            .expect("untouched entry")
            .total_value,
        9_767,
        "siblings keep their values"
    );
    let _ = std::fs::remove_file(&out);
}
