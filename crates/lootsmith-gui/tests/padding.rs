//! Rendered padding gate: a text run that lies **fully inside** a shell panel may
//! not sit within 10 px of that panel's border (spec: panel content is inset
//! 14-16 px; header 20 px).
//!
//! The **status bar** is only 34 px tall, which cannot hold 10 px of vertical
//! padding around 12 px text, so its padding is checked horizontally (left/right)
//! only. Runs that are not fully inside the panel (a clipped scroll row, or screen
//! content overflowing the body) are ignored here — those are containment bugs for
//! the screen migrations, not padding.

mod common;

use common::frame::extract;
use lootsmith_gui::app::Tab;
use lootsmith_gui::theme::Theme;

fn synthetic(tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("lootsmith-padding-{tag}.sav"));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    path.display().to_string()
}

#[test]
fn text_is_padded_inside_each_shell_panel() {
    let mut bad: Vec<String> = Vec::new();
    for theme in [Theme::Dark, Theme::Light] {
        for scale in [1.0_f32, 2.0] {
            let save = synthetic(&format!("{theme:?}-{scale}"));
            let mut h = common::harness_isolated(&save);
            h.state_mut().set_theme(theme);
            h.state_mut().set_ui_scale(scale);
            h.state_mut().set_tab(Tab::Backpack);
            h.run();
            let shell = h.state().shell;
            let runs = {
                let out = h.output();
                extract(out).0
            };
            // (name, rect, check_vertical)
            // The sidebar and status bar are scrollable / short, so they are
            // checked horizontally; the top bar on all four borders.
            let panels = [
                ("top", shell.top, true),
                ("sidebar", shell.sidebar, false),
                ("status", shell.status, false),
            ];
            for run in &runs {
                if run.text.trim().is_empty() {
                    continue;
                }
                for (name, panel, vertical) in panels {
                    if !panel.contains_rect(run.rect) {
                        continue; // clipped / overflowing, not a padding question
                    }
                    let horiz =
                        (run.rect.left() - panel.left()).min(panel.right() - run.rect.right());
                    let vert =
                        (run.rect.top() - panel.top()).min(panel.bottom() - run.rect.bottom());
                    let d = if vertical { horiz.min(vert) } else { horiz };
                    if d < 10.0 {
                        bad.push(format!(
                            "{theme:?} {scale}x {name}: {:?} is only {d:.1} px from the panel border",
                            run.text
                        ));
                    }
                }
            }
        }
    }
    assert!(
        bad.is_empty(),
        "text too close to a panel border:\n{}",
        bad.join("\n")
    );
}
