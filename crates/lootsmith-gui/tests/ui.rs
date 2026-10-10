//! Widget-level, focus-free UI tests via `egui_kittest` (AccessKit).
//!
//! These manipulate the UI by widget label — no window, no coordinates, no
//! focus — and assert on the application state.

use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_gui::app::{LootsmithApp, Tab};

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
    let app = LootsmithApp::new(Some(save), Some(dumps));
    Harness::new_ui_state(|ui, app: &mut LootsmithApp| app.build(ui), app)
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
    assert!(!h.state().items.is_empty(), "items should be listed");

    h.get_by_label("Character").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Character);

    h.get_by_label("Items").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Items);

    h.get_by_label("Currency").click();
    h.run();
    assert_eq!(h.state().tab, Tab::Currency);

    h.get_by_label("General").click();
    h.run();
    assert_eq!(h.state().tab, Tab::General);
}

#[test]
fn selects_an_item_row_by_label() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();
    h.get_by_label("Items").click();
    h.run();

    // The first weapon in the fixture is row #6.
    h.get_by_label_contains("#6 weapon").click();
    h.run();
    assert!(
        h.state().selected_item.is_some(),
        "clicking an item row should select it"
    );
}

#[test]
fn edits_a_weapon_part_and_saves() {
    let Some((save, dumps)) = fixtures() else {
        return;
    };
    let mut h = harness(save, dumps);
    h.run();

    h.state_mut().select_item(5); // index 5 == row #6 (first weapon)
    h.run();
    let before = h.state().item_serial(5).expect("serial");

    h.state_mut()
        .set_selected_part(
            "elemental",
            "GD_Weap_Pistol.elemental.Pistol_Elemental_Fire",
        )
        .expect("set part");
    h.state_mut().apply_item();
    h.run();

    let after = h.state().item_serial(5).expect("serial");
    assert_ne!(before, after, "the edited serial should differ");

    // Write and verify with the core library.
    let out = std::env::temp_dir().join("lootsmith-ui-test.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let bytes = std::fs::read(&out).expect("read saved file");
    let reopened = lootsmith_core::container::unpack(&bytes, lootsmith_core::Platform::PC)
        .expect("reopen");
    assert!(reopened.crc32_valid());
    assert!(reopened.proto_stable());
    assert!(reopened.huffman_stable());
}
