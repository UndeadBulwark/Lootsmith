//! Stage 1: the Homestead widget set has accessible labels and responds.

mod common;

use egui::{pos2, vec2, Rect};
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::widgets::{
    button, checkbox, chip, dropdown, gem_rarity, nav_row, number_field, panel, part_row,
    plank_bar, ribbon, search_field, stat_block, status_chip, text_field, trail_divider,
    ButtonKind,
};

struct G {
    theme: Theme,
    clicks: Vec<String>,
    badass: bool,
    filter: String,
    level: i32,
    choice: usize,
    options: Vec<String>,
}

fn build(ui: &mut egui::Ui, g: &mut G) {
    theme::apply(ui.ctx(), g.theme);
    let p = *theme::active(ui.ctx());
    ui.painter().rect_filled(ui.max_rect(), 0.0, p.bg);

    panel(
        ui,
        Rect::from_min_size(pos2(8.0, 8.0), vec2(320.0, 90.0)),
        |ui| {
            ui.label("panel content");
        },
    );
    ribbon(
        ui,
        Rect::from_min_size(pos2(8.0, 108.0), vec2(200.0, 28.0)),
        "Backpack",
    );
    plank_bar(
        ui,
        Rect::from_min_size(pos2(8.0, 144.0), vec2(320.0, 28.0)),
        true,
    );
    trail_divider(ui, pos2(8.0, 184.0), pos2(328.0, 184.0));
    nav_row(
        ui,
        Rect::from_min_size(pos2(8.0, 194.0), vec2(160.0, 28.0)),
        "General",
        true,
        Some(3),
    );
    nav_row(
        ui,
        Rect::from_min_size(pos2(8.0, 226.0), vec2(160.0, 28.0)),
        "Skills",
        false,
        None,
    );
    if button(
        ui,
        Rect::from_min_size(pos2(8.0, 262.0), vec2(90.0, 28.0)),
        "Save",
        ButtonKind::Primary,
    )
    .clicked()
    {
        g.clicks.push("Save".into());
    }
    button(
        ui,
        Rect::from_min_size(pos2(106.0, 262.0), vec2(90.0, 28.0)),
        "Remove",
        ButtonKind::Danger,
    );
    chip(
        ui,
        Rect::from_min_size(pos2(204.0, 264.0), vec2(90.0, 24.0)),
        "raw ids",
        p.dng,
    );
    gem_rarity(ui, pos2(30.0, 320.0), p.gold);
    stat_block(
        ui,
        Rect::from_min_size(pos2(60.0, 306.0), vec2(90.0, 36.0)),
        "Level",
        "72",
    );
    part_row(
        ui,
        Rect::from_min_size(pos2(8.0, 350.0), vec2(320.0, 26.0)),
        "Body",
        "Pistol_Body_Dahl",
    );
    checkbox(ui, &mut g.badass, "Badass mode");
    text_field(ui, "tf", &mut g.filter, "filter items…", 200.0);
    search_field(ui, "sf", &mut g.filter, 200.0);
    number_field(ui, "level", &mut g.level, 80.0);
    let options = g.options.clone();
    if let Some(i) = dropdown(ui, "dd", "Standard", &options) {
        g.choice = i;
    }
    status_chip(
        ui,
        Rect::from_min_size(pos2(8.0, 470.0), vec2(200.0, 24.0)),
        "Game data",
        true,
    );
}

fn harness(theme: Theme) -> Harness<'static, G> {
    Harness::builder()
        .with_size(vec2(360.0, 520.0))
        .build_ui_state(
            build,
            G {
                theme,
                clicks: Vec::new(),
                badass: false,
                filter: String::new(),
                level: 40,
                choice: 0,
                options: vec!["Standard".into(), "Trash".into()],
            },
        )
}

#[test]
fn widgets_are_labelled_and_respond() {
    for theme in [Theme::Dark, Theme::Light] {
        let mut h = harness(theme);
        h.run();
        // Accessibility labels.
        for label in [
            "Backpack header",
            "General",
            "Skills",
            "Save",
            "Remove",
            "Badass mode",
            "Game data",
            "Body",
        ] {
            assert!(
                h.query_by_label(label).is_some(),
                "{theme:?}: no accessible label {label:?}"
            );
        }
        // Behaviour: primary button click.
        h.get_by_label("Save").click();
        h.run();
        assert_eq!(h.state().clicks, ["Save"], "{theme:?}: button click");
        // Checkbox toggles by shape.
        assert!(!h.state().badass);
        h.get_by_label("Badass mode").click();
        h.run();
        assert!(h.state().badass, "{theme:?}: checkbox toggles");
    }
}
