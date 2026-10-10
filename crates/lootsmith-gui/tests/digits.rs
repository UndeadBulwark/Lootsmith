//! Digits must render in VT323 (`ui::widgets::num`), never in the label font
//! (Pixelify Sans' `5` reads as `S`).

use egui::Vec2;
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::widgets::num;

struct One(char);

fn build(ui: &mut egui::Ui, state: &mut One) {
    theme::apply(ui.ctx(), Theme::Dark);
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, egui::Color32::BLACK);
    // Fixed rect + left alignment: only the glyph shape can differ between runs.
    ui.put(
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(28.0, 36.0)),
        egui::Label::new(num(state.0.to_string()).color(egui::Color32::WHITE)),
    );
}

/// FNV-1a over the rendered pixels of one digit.
fn hash_digit(c: char) -> u64 {
    let mut h = Harness::builder()
        .with_size(Vec2::new(28.0, 36.0))
        .build_ui_state(build, One(c));
    h.run();
    let img = h.render().expect("render digit");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in img.as_raw() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

/// `"0123456789"` in `num()`: every glyph is distinct.
#[test]
fn every_digit_is_a_distinct_glyph() {
    let mut seen = std::collections::HashMap::new();
    for d in 0..10 {
        let c = char::from_digit(d, 10).unwrap();
        let hash = hash_digit(c);
        if let Some(prev) = seen.insert(hash, c) {
            panic!("digits {prev:?} and {c:?} render identically in num()");
        }
    }
    assert_eq!(seen.len(), 10, "ten distinct digit glyphs");
}
