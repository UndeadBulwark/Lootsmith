//! Digit gate: **no digit may be drawn in the label font** (Pixelify Sans) on
//! any screen the contrast gate renders, plus the gallery — every digit run must
//! go through `ui::widgets::num` / `text_with_nums` (VT323).

mod common;

use common::frame::{extract, render_tab_at, Section, SCREENS};
use egui::Vec2;
use lootsmith_gui::theme::Theme;

fn violations(sections: &[Section]) -> Vec<String> {
    sections
        .iter()
        .filter(|s| s.proportional && s.text.chars().any(|c| c.is_ascii_digit()))
        .map(|s| s.text.clone())
        .collect()
}

#[test]
fn digits_never_use_the_label_font() {
    let mut bad: Vec<String> = Vec::new();
    for theme in [Theme::Dark, Theme::Light] {
        for tab in SCREENS {
            for scale in [1.0_f32, 2.0, 3.0] {
                let (_runs, _fills, sections, _img) = render_tab_at(tab, theme, scale);
                for s in violations(&sections) {
                    bad.push(format!("{tab:?} {theme:?} {scale}x: {s:?}"));
                }
            }
        }
        let mut h = common::gallery::harness(theme, 1.0, Vec2::new(760.0, 700.0));
        h.run();
        let sections = {
            let out = h.output();
            extract(out).2
        };
        for s in violations(&sections) {
            bad.push(format!("Gallery {theme:?}: {s:?}"));
        }
    }
    assert!(
        bad.is_empty(),
        "digits drawn in the label font ({}):\n{}",
        bad.len(),
        bad.join("\n")
    );
}
