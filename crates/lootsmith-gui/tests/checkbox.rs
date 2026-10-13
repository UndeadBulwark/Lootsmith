//! The shared notched checkbox must show its state by shape in both themes.

mod common;

use egui::Vec2;
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::widgets::checkbox;

struct C {
    theme: Theme,
    checked: bool,
}

fn build(ui: &mut egui::Ui, s: &mut C) {
    theme::apply(ui.ctx(), s.theme);
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, theme::active(ui.ctx()).pan);
    let mut checked = s.checked;
    checkbox(ui, &mut checked, "Badass mode");
}

fn hash(theme: Theme, checked: bool) -> u64 {
    let mut h = Harness::builder()
        .with_size(Vec2::new(160.0, 26.0))
        .build_ui_state(build, C { theme, checked });
    h.run();
    let img = h.render().expect("render checkbox");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in img.as_raw() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[test]
fn checked_and_unchecked_differ_in_both_themes() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("checkbox: skipped (no GPU adapter in this environment)");
        return;
    }
    for theme in [Theme::Dark, Theme::Light] {
        assert_ne!(
            hash(theme, false),
            hash(theme, true),
            "{theme:?}: checked and unchecked render identically"
        );
    }
}
