//! Render Lootsmith GUI screens offscreen to PNGs — no window, no focus.
//!
//! Usage: snapshot <out_dir> [save] [dumps]

use egui::Vec2;
use egui_kittest::Harness;
use lootsmith_core::Container;
use lootsmith_gui::app::{LootsmithApp, Tab};

/// (name, tab, optional selection to make first)
type Shot = (&'static str, Tab, Option<(Container, usize)>);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_dir = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/tmp/opencode/shots".to_string());
    // Optional 4th arg `WxH` (e.g. 1900x1010), default 1280x880.
    let size = args
        .get(4)
        .and_then(|s| s.split_once(['x', 'X']))
        .and_then(|(w, h)| Some(Vec2::new(w.parse().ok()?, h.parse().ok()?)))
        .unwrap_or(Vec2::new(1280.0, 880.0));
    let save = args.get(2).cloned().unwrap_or_else(|| {
        let personal = format!(
            "{}/../../tests/fixtures/pc_Save0001.sav",
            env!("CARGO_MANIFEST_DIR")
        );
        if std::path::Path::new(&personal).exists() {
            personal
        } else {
            // No user save available: render the synthetic fixture instead.
            let path = std::path::Path::new(&out_dir).join("synthetic-save.sav");
            match lootsmith_core::fixture::write_synthetic_save(&path) {
                Ok(_) => path.display().to_string(),
                Err(_) => personal,
            }
        }
    });
    let dumps = args.get(3).cloned().or_else(|| {
        std::env::var("LOOTSMITH_ASSET_LIBRARY").ok().or_else(|| {
            Some(format!(
                "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
                env!("CARGO_MANIFEST_DIR")
            ))
        })
    });
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let theme = match args.get(5).map(|s| s.as_str()) {
        Some("light") => lootsmith_gui::theme::Theme::Light,
        _ => lootsmith_gui::theme::Theme::Dark,
    };
    let app = |save: &str, dumps: &Option<String>| {
        let mut harness = Harness::builder().with_size(size).build_ui_state(
            |ui, app: &mut LootsmithApp| app.build(ui),
            LootsmithApp::new(Some(save.to_string()), dumps.clone()),
        );
        harness.state_mut().set_theme(theme);
        harness.run();
        harness
    };

    let shots: [Shot; 12] = [
        ("general", Tab::General, None),
        ("character", Tab::Character, None),
        ("currency", Tab::Currency, None),
        ("ammo", Tab::Ammo, None),
        ("world", Tab::World, None),
        ("skills", Tab::Skills, None),
        ("travel", Tab::Travel, None),
        ("vehicles", Tab::Vehicles, None),
        ("missions", Tab::Missions, None),
        ("backpack", Tab::Backpack, None),
        (
            "backpack-selected",
            Tab::Backpack,
            Some((Container::BackpackWeapon, 0)),
        ),
        ("raw", Tab::Raw, None),
    ];

    for (name, tab, select) in shots {
        let mut harness = app(&save, &dumps);
        harness.state_mut().set_tab(tab);
        harness.run();
        if let Some((container, occurrence)) = select {
            harness.state_mut().select_entry(container, occurrence);
            harness.run();
        }
        render(&mut harness, &out_dir, name);
    }

    // The bank is empty in the fixture, so move a weapon in before shooting it.
    let mut harness = app(&save, &dumps);
    harness.state_mut().set_tab(Tab::Backpack);
    harness.run();
    harness.state_mut().select_entry(Container::BackpackItem, 0);
    harness.state_mut().move_selected_to(Container::Bank);
    harness.run();
    render(&mut harness, &out_dir, "bank");

    // The "Find saves" picker (scans the real machine).
    let mut harness = app(&save, &dumps);
    harness.state_mut().scan_saves();
    harness.state_mut().show_saves = true;
    harness.run();
    render(&mut harness, &out_dir, "find-saves");

    // The collapsed sidebar rail (per-tab icons + centred toggle).
    let mut harness = app(&save, &dumps);
    harness.state_mut().set_sidebar_collapsed(true);
    harness.state_mut().set_tab(Tab::Skills);
    harness.run();
    render(&mut harness, &out_dir, "sidebar-collapsed");
}

fn render(harness: &mut Harness<'_, LootsmithApp>, out_dir: &str, name: &str) {
    match harness.render() {
        Ok(image) => {
            let path = format!("{out_dir}/{name}.png");
            image.save(&path).expect("save png");
            println!("wrote {path}");
        }
        Err(e) => eprintln!("render {name} failed: {e}"),
    }
}
