//! Render Lootsmith GUI screens offscreen to PNGs — no window, no focus.
//!
//! Usage: snapshot <out_dir> [save] [dumps]

use egui::Vec2;
use egui_kittest::Harness;
use lootsmith_gui::app::{LootsmithApp, Tab};

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
        std::env::var("LOOTSMITH_ASSET_LIBRARY").ok().or_else(|| {
            Some(format!(
                "{}/../../../bl2-rogue/data/gibbed/Asset Library Manager.json",
                env!("CARGO_MANIFEST_DIR")
            ))
        })
    });
    std::fs::create_dir_all(&out_dir).expect("create out dir");

    let shots: [(&str, Tab, Option<usize>); 5] = [
        ("general", Tab::General, None),
        ("character", Tab::Character, None),
        ("currency", Tab::Currency, None),
        ("items", Tab::Items, None),
        ("item-selected", Tab::Items, Some(5)),
    ];

    for (name, tab, select) in shots {
        let app = LootsmithApp::new(Some(save.clone()), dumps.clone());
        let mut harness = Harness::builder()
            .with_size(Vec2::new(1020.0, 720.0))
            .build_ui_state(|ui, app: &mut LootsmithApp| app.build(ui), app);
        harness.state_mut().set_tab(tab);
        harness.run();
        if let Some(index) = select {
            harness.state_mut().select_item(index);
            harness.run();
        }
        match harness.render() {
            Ok(image) => {
                let path = format!("{out_dir}/{name}.png");
                image.save(&path).expect("save png");
                println!("wrote {path}");
            }
            Err(e) => eprintln!("render {name} failed: {e}"),
        }
    }
}
