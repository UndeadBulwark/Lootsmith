//! CI smoke tests using the synthetic save — no personal fixture required.

use egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_core::fixture;
use lootsmith_gui::app::LootsmithApp;

fn harness_with_synthetic() -> Harness<'static, LootsmithApp> {
    let path = std::env::temp_dir().join("lootsmith-synthetic.sav");
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

#[allow(clippy::unwrap_used)] #[test]
fn synthetic_save_loads_and_switches_tabs() {
    let mut h = harness_with_synthetic();
    h.run();
    assert!(h.state().is_loaded());
    assert_eq!(h.state().form.level, 40);
    assert_eq!(h.state().backpack.len(), 3, "1 item + 2 weapons");
    assert_eq!(h.state().bank.len(), 1);
    assert_eq!(h.state().travel.len(), 3);

    for label in ["Character", "Currency", "Skills", "Fast Travel", "Vehicles",
                  "Missions", "Backpack (3)", "Bank (1)", "Raw", "About", "General"] {
        h.get_by_label(label).click();
        h.run();
    }
}

#[allow(clippy::unwrap_used)] #[test]
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
