//! Shared test harness: always the synthetic save, plus the game-data dumps
//! when the environment provides them.
//!
//! Per `LOOTSMITH_UI_WORKFLOW.md`:
//! - tests never depend on a personal save — `synthetic_save_path()` builds one
//!   from `SaveGame::blank` on first use;
//! - dumps-dependent tests call [`dumps_path`]; under
//!   `LOOTSMITH_REQUIRE_FIXTURES=1` a missing dumps folder is a **hard panic**
//!   instead of a silent skip, so a green run always proved something.

#![allow(dead_code)]

pub mod frame;
pub mod gallery;

use egui::Vec2;
use egui_kittest::Harness;
use lootsmith_core::fixture;
use lootsmith_gui::app::LootsmithApp;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Whether missing fixtures must fail the test rather than skip it.
pub fn require_fixtures() -> bool {
    matches!(
        std::env::var("LOOTSMITH_REQUIRE_FIXTURES").as_deref(),
        Ok("1") | Ok("true")
    )
}

/// A private copy of the synthetic save, unique per call so tests can run in
/// parallel.
pub fn synthetic_save_path() -> String {
    static COUNTER: AtomicUsize = AtomicUsize::new(0);
    let unique = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!(
        "lootsmith-test-{}-{unique}.sav",
        std::process::id()
    ));
    fixture::write_synthetic_save(&path).expect("write synthetic save");
    freeze_mtime(&path);
    path.display().to_string()
}

/// Freeze a synthetic save's mtime so time-derived UI (the status bar
/// "saved HH:MM") is deterministic across runs.
pub fn freeze_mtime(path: &std::path::Path) {
    use std::time::{Duration, UNIX_EPOCH};
    let fixed = UNIX_EPOCH + Duration::from_secs(65_000); // 18:03:20 UTC
    if let Ok(f) = std::fs::File::options().write(true).open(path) {
        let _ = f.set_times(std::fs::FileTimes::new().set_modified(fixed));
    }
}

/// The game-data dumps folder (`LOOTSMITH_ASSET_LIBRARY` or the well-known
/// local checkout), or `None`. Panics under `LOOTSMITH_REQUIRE_FIXTURES=1`.
pub fn dumps_path() -> Option<String> {
    let from_env = std::env::var("LOOTSMITH_ASSET_LIBRARY").ok();
    let local = format!(
        "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let candidate = from_env
        .filter(|p| !p.trim().is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            PathBuf::from(&local)
                .exists()
                .then(|| PathBuf::from(&local))
        });
    match candidate {
        Some(path) if path.exists() => Some(path.display().to_string()),
        _ if require_fixtures() => panic!(
            "LOOTSMITH_REQUIRE_FIXTURES=1 but no game-data dumps found; \
             set LOOTSMITH_ASSET_LIBRARY to an Asset Library Manager.json"
        ),
        _ => None,
    }
}

/// The synthetic save with the **bundled** game-data tables (no external
/// folder, no environment): the common case for every GUI test.
pub fn harness() -> Harness<'static, LootsmithApp> {
    harness_isolated(&synthetic_save_path())
}

/// A harness over an explicit save (and optional dumps).
pub fn harness_with(save: &str, dumps: Option<&str>) -> Harness<'static, LootsmithApp> {
    Harness::builder()
        .with_size(Vec2::new(1280.0, 860.0))
        .with_max_steps(240)
        .build_ui_state(
            |ui, app: &mut LootsmithApp| app.build(ui),
            LootsmithApp::new(Some(save.to_string()), dumps.map(str::to_string)),
        )
}

/// A harness at an explicit size (physical pixels), for the 200% render: the
/// logical point-space is `size / pixels_per_point`, so 2× needs a 2560×1600
/// harness to show the full 1280×800 layout.
pub fn harness_sized(
    save: &str,
    dumps: Option<&str>,
    size: Vec2,
) -> Harness<'static, LootsmithApp> {
    Harness::builder()
        .with_size(size)
        .with_max_steps(240)
        .build_ui_state(
            |ui, app: &mut LootsmithApp| app.build(ui),
            LootsmithApp::new(Some(save.to_string()), dumps.map(str::to_string)),
        )
}

/// A harness whose app is **isolated** from the host (`LootsmithApp::new_isolated`):
/// it ignores every `LOOTSMITH_*` variable, never runs first-run discovery, and
/// always uses the bundled tables. Use this for the "no folder" tests so a
/// variable set in the developer's shell cannot change the result.
pub fn harness_isolated(save: &str) -> Harness<'static, LootsmithApp> {
    Harness::builder()
        .with_size(Vec2::new(1280.0, 860.0))
        .with_max_steps(240)
        .build_ui_state(
            |ui, app: &mut LootsmithApp| app.build(ui),
            LootsmithApp::new_isolated(Some(save.to_string())),
        )
}

/// Re-open a written save from disk.
pub fn reopen(path: &std::path::Path) -> lootsmith_core::SaveFile {
    let bytes = std::fs::read(path).expect("read saved file");
    lootsmith_core::container::unpack(&bytes, lootsmith_core::Platform::PC).expect("reopen")
}

/// A dumps-path-or-skip guard for name-dependent tests: panics under
/// `LOOTSMITH_REQUIRE_FIXTURES=1`, prints a skip note otherwise.
pub fn dumps_or_none(tag: &str) -> Option<String> {
    let dumps = dumps_path();
    if dumps.is_none() {
        eprintln!("{tag}: skipped (no game-data dumps)");
    }
    dumps
}
