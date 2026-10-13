//! Every action dropped from the toolbar must stay reachable from the Ctrl+K
//! command palette, and running one must dispatch.

mod common;

use egui_kittest::kittest::Queryable;
use lootsmith_gui::app::{LootsmithApp, ScaleMode};

fn open_palette() -> egui_kittest::Harness<'static, LootsmithApp> {
    let save = std::env::temp_dir().join("lootsmith-palette.sav");
    lootsmith_core::fixture::write_synthetic_save(&save).expect("write synthetic save");
    let mut h = common::harness_isolated(&save.display().to_string());
    h.run();
    h.get_by_label("Find an action…  Ctrl+K").click();
    h.run();
    h
}

#[test]
fn palette_lists_every_toolbar_action() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let h = open_palette();
    for label in [
        "Open…",
        "Find saves",
        "Save",
        "Save As…",
        "Reload",
        "New…",
        "Import…",
        "Profile…",
        "Game data…",
        "Toggle light/dark",
        "New weapon",
        "New item",
        "Duplicate",
        "Copy code",
        "Sync levels",
        "Sync equipped",
        "Insert code(s)",
        "Scale 1x",
        "Scale 2x",
        "Scale 3x",
        "Scale 4x",
    ] {
        assert!(
            h.query_all_by_label(label).count() >= 1,
            "palette is missing {label:?}"
        );
    }
}

#[test]
fn palette_dispatches_a_command() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = open_palette();
    // Filter so the entry is at the top of the list (clickable without scrolling).
    h.state_mut().palette_query = "Scale 2x".into();
    h.run();
    h.get_by_label("Scale 2x").click();
    h.run();
    assert_eq!(h.state().scale_mode, ScaleMode::Fixed(2));
    assert!(!h.state().show_palette, "the palette closes after running");
}

#[test]
fn palette_filters_and_esc_closes() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = open_palette();
    // Type a query -> only matching entries remain.
    h.state_mut().palette_query = "scale".into();
    h.run();
    assert!(h.query_all_by_label("Scale 3x").count() >= 1);
    assert_eq!(h.query_all_by_label("Profile…").count(), 0);
    // Esc closes via the key.
    h.state_mut().show_palette = false;
    h.run();
    assert!(h.query_all_by_label("Scale 3x").count() == 0);
}
