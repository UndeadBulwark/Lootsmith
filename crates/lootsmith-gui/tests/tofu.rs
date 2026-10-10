//! Tofu gate: no rendered Text shape may contain a codepoint missing from its font.

mod common;

use common::frame::{screen_tofu, SCREENS};
use lootsmith_gui::theme::Theme;

#[test]
fn no_text_renders_a_missing_glyph() {
    let mut bad = Vec::new();
    for tab in SCREENS {
        for theme in [Theme::Dark, Theme::Light] {
            for v in screen_tofu(tab, theme) {
                bad.push(format!("{tab:?} {theme:?}: {v}"));
            }
        }
    }
    assert!(
        bad.is_empty(),
        "text with glyphs missing from its font:\n{}",
        bad.join("\n")
    );
}
