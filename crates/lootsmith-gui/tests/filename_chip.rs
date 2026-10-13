//! The top-bar file-name chip must never let its text overflow — at long names,
//! in both themes and at 1x/2x. The fit is measured from the real font galley
//! (`ui::widgets::fit_text`), not a px-per-char estimate.

mod common;

use common::frame::extract;
use eframe::egui::{Rect, Sense, Ui, Vec2};
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, contrast, Theme};
use lootsmith_gui::ui::widgets::{chip, fit_text};

const LONG: &str = "GD_Weapon_Manufacturer_Something_A_Really_Long_Balance_Name_Xyz.sav";
const CHIP_W: f32 = 190.0;

struct S {
    theme: Theme,
    scale: f32,
    chip: Option<Rect>,
    fitted: String,
}

fn build(ui: &mut Ui, s: &mut S) {
    ui.ctx().set_pixels_per_point(s.scale);
    theme::apply(ui.ctx(), s.theme);
    let p = *theme::active(ui.ctx());
    ui.painter().rect_filled(ui.max_rect(), 0.0, p.bg);
    let name = fit_text(ui, LONG, CHIP_W - 16.0, 13.0);
    s.fitted = name.clone();
    let (r, _) = ui.allocate_exact_size(Vec2::new(CHIP_W, 24.0), Sense::hover());
    s.chip = Some(r);
    chip(ui, r, &name, p.fg);
}

#[test]
fn filename_chip_never_overflows_and_keeps_contrast() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    assert!(LONG.chars().count() >= 60, "the test name must be long");
    for theme in [Theme::Dark, Theme::Light] {
        let pal = if theme == Theme::Dark {
            theme::DARK
        } else {
            theme::LIGHT
        };
        for scale in [1.0_f32, 2.0] {
            let mut h = Harness::builder()
                .with_size(Vec2::new(220.0, 40.0))
                .build_ui_state(
                    build,
                    S {
                        theme,
                        scale,
                        chip: None,
                        fitted: String::new(),
                    },
                );
            h.run();
            h.run();
            let chip_rect = h.state().chip.expect("the chip is drawn");
            let fitted = h.state().fitted.clone();
            let runs = {
                let out = h.output();
                extract(out).0
            };
            let texts: Vec<&str> = runs
                .iter()
                .map(|r| r.text.as_str())
                .filter(|t| !t.trim().is_empty())
                .collect();
            assert_eq!(
                texts.len(),
                1,
                "{theme:?} {scale}x: only the chip text: {texts:?}"
            );
            let run = runs.iter().find(|r| !r.text.trim().is_empty()).unwrap();

            // 1. It was actually elided (long name -> shorter, with "...").
            assert!(
                fitted.chars().count() < LONG.chars().count() && fitted.contains("..."),
                "{theme:?} {scale}x: not elided: {fitted:?}"
            );
            // 2. The rendered text stays inside the chip.
            assert!(
                chip_rect.expand(1.0).contains_rect(run.rect),
                "{theme:?} {scale}x: text {:?} overflows the chip {:?} -> {:?}",
                run.text,
                chip_rect,
                run.rect
            );
            // 3. And keeps WCAG text contrast on the chip surface.
            let ratio = contrast(run.color, pal.pan);
            assert!(
                ratio >= 4.5,
                "{theme:?} {scale}x: chip text {ratio:.2}:1 < 4.5:1"
            );
        }
    }
}
