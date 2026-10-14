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

/// A fixed fake game-data path: baselines must not contain a `/home/...` path.
const FAKE_DUMPS: &str = "/home/user/bl2-dumps";

/// A per-test **fixed** save path. The path text is part of the pixels, so it
/// must be stable between runs; and each test needs its own file because the
/// snapshot tests run in parallel.
fn synthetic_at(tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("lootsmith-snapshot-{tag}.sav"));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    common::freeze_mtime(&path);
    path.display().to_string()
}

/// A 1280x800 render of the Backpack screen on the synthetic save (100%, dark).
#[test]
fn smoke_backpack() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("backpack");
    let mut h = harness_with(&save, Some(FAKE_DUMPS));
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Dark);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack");
}

/// The same screen in the light palette (100%).
#[test]
fn smoke_backpack_light() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("backpack-light");
    let mut h = harness_with(&save, Some(FAKE_DUMPS));
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("backpack-2x");
    let mut h = common::harness_sized(&save, Some(FAKE_DUMPS), Vec2::new(2560.0, 1720.0));
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Dark);
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("backpack-light-2x");
    let mut h = common::harness_sized(&save, Some(FAKE_DUMPS), Vec2::new(2560.0, 1720.0));
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("inspector-bundled");
    let mut h = harness_isolated(&save);
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Dark);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_inspector_bundled");
}

/// The Backpack screen at 300% scale (dark) — the pixel grid must hold at 3x.
#[test]
fn smoke_backpack_dark_3x() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("backpack-3x");
    let mut h = common::harness_sized(&save, Some(FAKE_DUMPS), Vec2::new(3840.0, 2580.0));
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Dark);
    h.state_mut().set_ui_scale(3.0);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack_dark_3x");
}

/// The Backpack screen at 300% scale (light).
#[test]
fn smoke_backpack_light_3x() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("backpack-light-3x");
    let mut h = common::harness_sized(&save, Some(FAKE_DUMPS), Vec2::new(3840.0, 2580.0));
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Light);
    h.state_mut().set_ui_scale(3.0);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    h.snapshot("smoke_backpack_light_3x");
}

/// The collapsed sidebar rail (100%, dark): crest, icon column and the centred
/// between-group dividers at narrow width.
#[test]
fn smoke_sidebar_rail() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("rail");
    let mut h = harness_with(&save, Some(FAKE_DUMPS));
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Dark);
    h.state_mut().set_sidebar_collapsed(true);
    h.run();
    h.snapshot("smoke_sidebar_rail");
}

/// A wide Skills render: the right column is visible and shows the build summary
/// while nothing is hovered.
#[test]
fn smoke_skills_wide() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let save = synthetic_at("skills-wide");
    let mut h = common::harness_sized(&save, Some(FAKE_DUMPS), Vec2::new(1720.0, 940.0));
    h.state_mut().set_theme(lootsmith_gui::app::Theme::Dark);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Skills);
    h.run();
    h.snapshot("smoke_skills_wide");
}
