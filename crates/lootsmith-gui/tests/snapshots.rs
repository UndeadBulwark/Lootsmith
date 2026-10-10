//! Snapshot (visual-regression) harness.
//!
//! One smoke snapshot only for now — the real per-screen baselines land after
//! the "Homestead" shell exists, so we never baseline a look that the theme
//! rewrite replaces. See `LOOTSMITH_UI_WORKFLOW.md` §4.
//!
//! Regenerate deliberately with:
//!     UPDATE_SNAPSHOTS=1 cargo test -p lootsmith-gui --test snapshots
//! and review `tests/snapshots/*.png` before committing.

mod common;

use common::{harness_isolated, harness_with};

/// A 1280x800 render of the Backpack screen on the synthetic save.
#[test]
fn smoke_backpack() {
    // Renders must not overlap: the active palette is a process-global.
    let _serial = common::snapshot_lock();
    // A *fixed* path: the save-file field is part of the pixels, so the name
    // must not change between runs.
    let path = std::env::temp_dir().join("lootsmith-snapshot.sav");
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    let mut h = harness_with(&path.display().to_string(), common::dumps_path().as_deref());
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack");
}

/// The same screen in the light palette, for side-by-side review.
#[test]
fn smoke_backpack_light() {
    let _serial = common::snapshot_lock();
    let path = std::env::temp_dir().join("lootsmith-snapshot.sav");
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    let mut h = harness_with(&path.display().to_string(), common::dumps_path().as_deref());
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Light);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack_light");
}

/// The item inspector with **no** game-data folder: part names must resolve
/// from the bundled tables (review PNG). The harness is isolated, so no
/// `LOOTSMITH_*` variable set in the shell can change it.
#[test]
fn smoke_inspector_bundled() {
    let _serial = common::snapshot_lock();
    let path = std::env::temp_dir().join("lootsmith-snapshot.sav");
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    let mut h = harness_isolated(&path.display().to_string());
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_inspector_bundled");
}

/// A **real weapon** (the personal fixture's Dahl pistol, "Smooth Repeater") in
/// the inspector with **no game-data folder**: names must come from the bundled
/// tables, including the composed display name. Skipped when the personal save
/// is absent (hard failure under LOOTSMITH_REQUIRE_FIXTURES=1).
#[test]
fn smoke_inspector_real_item_bundled() {
    let _serial = common::snapshot_lock();
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
