//! Proves that the "no folder" tests are isolated from the host.
//!
//! This lives in its **own test binary** (its own process) and holds a single
//! test, so mutating the process environment here cannot race the other test
//! files. It demonstrates three things:
//!
//! 1. a *normal* app honours `LOOTSMITH_ASSET_LIBRARY` (the probe is real);
//! 2. `LootsmithApp::new_isolated` ignores it — and every other `LOOTSMITH_*`
//!    variable — and uses the bundled tables;
//! 3. the constructor never reads persisted settings (those live in `main.rs`
//!    via eframe `Storage`; the app has no settings input at all).
//!
//! See `LootsmithApp::env_var`, the single gate every environment read passes.

mod common;

use lootsmith_gui::app::LootsmithApp;
use std::path::PathBuf;

#[test]
fn isolated_app_ignores_env_and_uses_bundled_tables() {
    // A *real* game-data folder in a temp dir. If an env variable to it were
    // honoured, the app would load from disk (`dumps_bundled == false`).
    let core_data = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../lootsmith-core/assets/data");
    let dir = std::env::temp_dir().join(format!("lootsmith-iso-{}", std::process::id()));
    std::fs::create_dir_all(&dir).expect("create temp dumps dir");
    std::fs::copy(
        core_data.join("asset-library-manager.json"),
        dir.join("Asset Library Manager.json"),
    )
    .expect("copy the bundled asset library");

    let save = common::synthetic_save_path();
    let dir_str = dir.display().to_string();

    // Point *every* game-data variable at that folder. Any read would change
    // behaviour or the status line.
    for name in [
        "LOOTSMITH_ASSET_LIBRARY",
        "LOOTSMITH_NAME_PARTS",
        "LOOTSMITH_PART_LISTS",
        "LOOTSMITH_TRAVEL_STATIONS",
        "LOOTSMITH_CUSTOMIZATIONS",
        "LOOTSMITH_MISSIONS",
        "LOOTSMITH_DUMP_DIRS",
        "LOOTSMITH_SAVE_ROOTS",
        "LOOTSMITH_SAVES",
    ] {
        std::env::set_var(name, &dir_str);
    }
    eprintln!("isolation test: LOOTSMITH_* set to {dir_str}");

    // Control: a normal app *does* honour the variable — the probe is real.
    let mut control = LootsmithApp::new(Some(save.clone()), None);
    assert!(!control.ignores_env(), "a normal app is not isolated");
    control.load_dumps();
    assert!(
        control.manager.is_some(),
        "the control loads an asset library"
    );
    assert!(
        !control.dumps_bundled,
        "the control honours LOOTSMITH_ASSET_LIBRARY (proof the probe works)"
    );

    // Isolated: the same environment is ignored; bundled tables are used.
    let iso = LootsmithApp::new_isolated(Some(save.clone()));
    assert!(iso.ignores_env(), "new_isolated sets the flag");
    assert!(
        iso.dumps_bundled,
        "isolated app uses the bundled tables, not the env folder"
    );
    assert!(iso.manager.is_some(), "bundled asset library");
    assert!(iso.name_parts.is_some(), "bundled name parts");
    assert!(iso.part_lists.is_some(), "bundled part lists");
    assert!(iso.stations.is_some(), "bundled travel stations");
    assert!(iso.customizations.is_some(), "bundled customizations");
    assert!(iso.missions_data.is_some(), "bundled missions");
    assert!(iso.dlc_titles.is_some(), "bundled DLC titles");

    // Persisted settings cannot leak in: `LootsmithApp` has no settings input;
    // `main.rs` is the only place that reads eframe `Storage`.
}
