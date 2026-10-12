//! Render every screen (both themes, 100% and 200%) plus the mark-column evidence
//! for `scripts/review-bundle.sh`. Offscreen, no window.
//!
//! Usage: bundle_shots <out_dir>

use eframe::egui::{self, Vec2};
use egui_kittest::Harness;
use lootsmith_core::Container;
use lootsmith_gui::app::{LootsmithApp, Tab};
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::item_table::{item_table, ItemRow};

const SCREENS: [Tab; 13] = [
    Tab::General,
    Tab::Character,
    Tab::Currency,
    Tab::Ammo,
    Tab::World,
    Tab::Skills,
    Tab::Travel,
    Tab::Vehicles,
    Tab::Missions,
    Tab::Backpack,
    Tab::Bank,
    Tab::Raw,
    Tab::About,
];

fn main() {
    let out = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "/tmp/opencode/bundle-shots".to_string());
    std::fs::create_dir_all(&out).expect("create out dir");
    let save = std::path::Path::new(&out).join("synthetic-save.sav");
    lootsmith_core::fixture::write_synthetic_save(&save).expect("write synthetic save");
    let save = save.display().to_string();

    for tab in SCREENS {
        for (theme, tname) in [(Theme::Dark, "dark"), (Theme::Light, "light")] {
            for (scale, sname) in [(1.0_f32, "1x"), (2.0_f32, "2x")] {
                let mut h = Harness::builder()
                    .with_size(Vec2::new(1280.0 * scale, 880.0 * scale))
                    .build_ui_state(
                        |ui, app: &mut LootsmithApp| app.build(ui),
                        LootsmithApp::new(Some(save.clone()), None),
                    );
                h.state_mut().set_theme(theme);
                h.state_mut().set_ui_scale(scale);
                h.state_mut().set_tab(tab);
                match tab {
                    Tab::Backpack => {
                        h.state_mut().select_entry(Container::BackpackWeapon, 0);
                    }
                    Tab::Bank => {
                        h.state_mut().select_entry(Container::Bank, 0);
                    }
                    _ => {}
                }
                h.run();
                let name = format!(
                    "{}_{}_{}.png",
                    format!("{tab:?}").to_lowercase(),
                    tname,
                    sname
                );
                render(&mut h, &out, &name);
            }
        }
    }

    // Mark-column evidence: Standard, Trash and Favorite rows in one list.
    for (theme, tname) in [(Theme::Dark, "dark"), (Theme::Light, "light")] {
        let mut h = Harness::builder()
            .with_size(Vec2::new(420.0, 220.0))
            .build_ui_state(build_marks, theme);
        h.run();
        h.run();
        render(&mut h, &out, &format!("marks_{tname}_1x.png"));
    }
}

fn build_marks(ui: &mut egui::Ui, theme: &mut Theme) {
    theme::apply(ui.ctx(), *theme);
    let p = *theme::active(ui.ctx());
    let screen = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(420.0, 220.0));
    ui.painter().rect_filled(screen, 0.0, p.bg);
    let row = |name: &str, id: i32, mark: i32| ItemRow {
        name: name.into(),
        kind: "Pistol".into(),
        id,
        level: 10,
        mark,
        rarity: lootsmith_core::Rarity::Common,
    };
    // PLAYER_MARKS = ["Trash", "Standard", "Favorite"]
    let rows = vec![
        row("Standard row", 1, 1),
        row("Trash row", 2, 0),
        row("Favorite row", 3, 2),
    ];
    item_table(ui, &p, &rows, None);
}

fn render<S>(harness: &mut Harness<'_, S>, out_dir: &str, name: &str) {
    match harness.render() {
        Ok(image) => {
            let path = format!("{out_dir}/{name}");
            image.save(&path).expect("save png");
            println!("wrote {path}");
        }
        Err(e) => eprintln!("render {name} failed: {e}"),
    }
}
