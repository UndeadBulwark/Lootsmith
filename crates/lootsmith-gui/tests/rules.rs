//! Design rule gate (spec §2.1): no gradient meshes (shadows, the scroll-area
//! fade) and no non-integer physical border widths, on every screen.

mod common;

use common::frame::{screen_rule_violations, SCREENS};
use lootsmith_gui::theme::Theme;

#[test]
fn no_gradients_or_odd_borders_on_any_screen() {
    let mut bad = Vec::new();
    for tab in SCREENS {
        for theme in [Theme::Dark, Theme::Light] {
            for v in screen_rule_violations(tab, theme) {
                bad.push(format!("{tab:?} {theme:?}: {v}"));
            }
        }
    }
    assert!(bad.is_empty(), "rule violations:\n{}", bad.join("\n"));
}
