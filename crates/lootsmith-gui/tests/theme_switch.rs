//! Switching the theme (and the UI scale) through the real toolbar buttons must
//! produce exactly the same frame as building the app fresh in the target
//! theme/scale — zero pixel diff.

mod common;

use egui_kittest::kittest::Queryable;
use image::RgbaImage;
use lootsmith_gui::app::Theme;

fn fixed_save() -> String {
    let path = std::env::temp_dir().join("lootsmith-themeswitch.sav");
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    path.display().to_string()
}

fn fresh(save: &str, theme: Theme, scale: f32) -> RgbaImage {
    let mut h = common::harness_isolated(save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_ui_scale(scale);
    h.run();
    h.render().expect("render")
}

fn switched(save: &str, theme: Theme, scale: f32, click: &str) -> RgbaImage {
    let mut h = common::harness_isolated(save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_ui_scale(scale);
    h.run();
    // The theme and scale controls live in the settings dialog.
    h.get_by_label("Menu").click();
    h.run();
    h.get_by_label(click).click();
    h.run();
    // The dialog that hosted the control must not appear in the compared frame.
    h.state_mut().show_settings = false;
    h.run();
    // Move the pointer off the button so it is not drawn hovered.
    h.remove_cursor();
    h.run();
    h.render().expect("render")
}

fn diff(a: &RgbaImage, b: &RgbaImage) -> usize {
    a.as_raw()
        .iter()
        .zip(b.as_raw())
        .filter(|(x, y)| x != y)
        .count()
}

#[test]
fn theme_and_scale_switches_match_fresh_renders() {
    let save = fixed_save();
    let cases = [
        (
            "dark->light",
            Theme::Dark,
            1.0,
            "Toggle light/dark",
            Theme::Light,
            1.0,
        ),
        (
            "light->dark",
            Theme::Light,
            1.0,
            "Toggle light/dark",
            Theme::Dark,
            1.0,
        ),
        ("100->200", Theme::Dark, 1.0, "200%", Theme::Dark, 2.0),
    ];
    let mut bad = Vec::new();
    for (name, start_theme, start_scale, click, end_theme, end_scale) in cases {
        let sw = switched(&save, start_theme, start_scale, click);
        let fresh = fresh(&save, end_theme, end_scale);
        if sw.dimensions() != fresh.dimensions() {
            bad.push(format!(
                "{name}: size {:?} != {:?}",
                sw.dimensions(),
                fresh.dimensions()
            ));
            continue;
        }
        let d = diff(&sw, &fresh);
        if d != 0 {
            bad.push(format!("{name}: {d} pixels differ from a fresh render"));
        }
    }
    assert!(bad.is_empty(), "switch mismatches:\n{}", bad.join("\n"));
}
