//! Design rule gate (spec §2.1): no gradient meshes (shadows, the scroll-area
//! fade) and no non-integer physical border widths, on every screen.

mod common;

use common::frame::{screen_rule_violations_at, SCREENS};
use lootsmith_gui::theme::Theme;

#[test]
fn no_gradients_or_odd_borders_on_any_screen() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut bad = Vec::new();
    for tab in SCREENS {
        for theme in [Theme::Dark, Theme::Light] {
            for scale in [1.0_f32, 2.0, 3.0] {
                for v in screen_rule_violations_at(tab, theme, scale) {
                    bad.push(format!("{tab:?} {theme:?} {scale}x: {v}"));
                }
            }
        }
    }
    assert!(bad.is_empty(), "rule violations:\n{}", bad.join("\n"));
}
