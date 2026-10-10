//! Snapshot (visual-regression) harness.
//!
//! Covers the current screens in both themes at 100% and 200% scale. The full
//! per-screen set for the "Homestead" shell lands as the shell steps progress.
//! See `LOOTSMITH_UI_WORKFLOW.md` §4.
//!
//! Regenerate deliberately with:
//!     UPDATE_SNAPSHOTS=1 cargo test -p lootsmith-gui --test snapshots
//! and review `tests/snapshots/*.png` before committing.

mod common;

use common::{harness_isolated, harness_with};
use egui::Vec2;

/// A per-test **fixed** save path. The path text is part of the pixels, so it
/// must be stable between runs; and each test needs its own file because the
/// snapshot tests run in parallel.
fn synthetic_at(tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("lootsmith-snapshot-{tag}.sav"));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    path.display().to_string()
}

/// A 1280x800 render of the Backpack screen on the synthetic save (100%, dark).
#[test]
fn smoke_backpack() {
    let save = synthetic_at("backpack");
    let mut h = harness_with(&save, common::dumps_path().as_deref());
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack");
}

/// The same screen in the light palette (100%).
#[test]
fn smoke_backpack_light() {
    let save = synthetic_at("backpack-light");
    let mut h = harness_with(&save, common::dumps_path().as_deref());
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Light);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack_light");
}

/// The Backpack screen at 200% scale (dark), to check the pixel grid.
#[test]
fn smoke_backpack_dark_2x() {
    let save = synthetic_at("backpack-2x");
    let mut h = common::harness_sized(
        &save,
        common::dumps_path().as_deref(),
        Vec2::new(2560.0, 1600.0),
    );
    h.state_mut().set_ui_scale(2.0);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack_dark_2x");
}

/// The Backpack screen at 200% scale (light).
#[test]
fn smoke_backpack_light_2x() {
    let save = synthetic_at("backpack-light-2x");
    let mut h = common::harness_sized(
        &save,
        common::dumps_path().as_deref(),
        Vec2::new(2560.0, 1600.0),
    );
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Light);
    h.state_mut().set_ui_scale(2.0);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack_light_2x");
}

/// The item inspector with **no** game-data folder: part names must resolve
/// from the bundled tables (review PNG). The harness is isolated, so no
/// `LOOTSMITH_*` variable set in the shell can change it.
#[test]
fn smoke_inspector_bundled() {
    let save = synthetic_at("inspector-bundled");
    let mut h = harness_isolated(&save);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_inspector_bundled");
}

/// A **real weapon** (the personal fixture's Dahl pistol, "Smooth Repeater") in
/// the inspector with **no** game-data folder: names must come from the bundled
/// tables, including the composed display name. Skipped when the personal save
/// is absent (hard failure under `LOOTSMITH_REQUIRE_FIXTURES=1`).
#[test]
fn smoke_inspector_real_item_bundled() {
    let save = format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    );
    if !std::path::Path::new(&save).exists() {
        if common::require_fixtures() {
            panic!("LOOTSMITH_REQUIRE_FIXTURES=1 but the personal save is missing");
        }
        eprintln!("smoke_inspector_real_item_bundled: skipped (no personal save)");
        return;
    }
    let mut h = harness_isolated(&save);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_inspector_real_item_bundled");
}
