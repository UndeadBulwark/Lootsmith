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
    let save = args.get(2).cloned().unwrap_or_else(|| {
        format!(
            "{}/../../tests/fixtures/pc_Save0001.sav",
            env!("CARGO_MANIFEST_DIR")
        )
    });
    let dumps = args.get(3).cloned().or_else(|| {
        std::env::var("LOOTSMITH_ASSET_LIBRARY")
            .ok()
            .or_else(|| {
                Some(format!(
                    "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
                    env!("CARGO_MANIFEST_DIR")
                ))
            })
    });
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let app = |save: &str, dumps: &Option<String>| {
        let mut harness = Harness::builder()
            .with_size(Vec2::new(1020.0, 760.0))
            .build_ui_state(
                |ui, app: &mut LootsmithApp| app.build(ui),
                LootsmithApp::new(Some(save.to_string()), dumps.clone()),
            );
        harness.run();
        harness
    };

    let shots: [Shot; 8] = [
        ("general", Tab::General, None),
        ("character", Tab::Character, None),
        ("currency", Tab::Currency, None),
        ("skills", Tab::Skills, None),
        ("travel", Tab::Travel, None),
        ("vehicles", Tab::Vehicles, None),
        ("backpack", Tab::Backpack, None),
        ("backpack-selected", Tab::Backpack, Some((Container::BackpackWeapon, 0))),
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
