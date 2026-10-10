//! Status glyphs must be distinguishable by **shape**, not only colour.

mod common;

use egui::Vec2;
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::widgets::{paint_glyph, Glyph};

struct One(Glyph);

fn build(ui: &mut egui::Ui, state: &mut One) {
    theme::apply(ui.ctx(), Theme::Dark);
    ui.painter()
        .rect_filled(ui.max_rect(), 0.0, egui::Color32::BLACK);
    paint_glyph(
        ui.painter(),
        ui.max_rect().center(),
        state.0,
        egui::Color32::WHITE,
        1.0,
    );
}

fn hash_glyph(glyph: Glyph) -> u64 {
    let mut h = Harness::builder()
        .with_size(Vec2::new(16.0, 16.0))
        .build_ui_state(build, One(glyph));
    h.run();
    let img = h.render().expect("render glyph");
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in img.as_raw() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[test]
fn each_status_glyph_is_distinct() {
    let mut seen: std::collections::HashMap<u64, Glyph> = std::collections::HashMap::new();
    for glyph in [Glyph::Check, Glyph::Cross, Glyph::Dot, Glyph::Ring] {
        let hash = hash_glyph(glyph);
        if let Some(prev) = seen.insert(hash, glyph) {
            panic!("status glyphs {prev:?} and {glyph:?} render identically");
        }
    }
    assert_eq!(seen.len(), 4, "four distinct status glyphs");
}
