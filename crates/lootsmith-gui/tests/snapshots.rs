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

use common::harness_with;

/// A 1280x800 render of the Backpack screen on the synthetic save.
#[test]
fn smoke_backpack() {
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
