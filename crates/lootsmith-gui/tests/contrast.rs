//! Rendered-contrast gate.
//!
//! The token-pair gate in `theme.rs` only checks the palette's own pairs; it
//! cannot see what a widget actually renders (e.g. a *selected* button drawing
//! `fg` on `acc`). This renders the real app in both themes, finds every text
//! pixel and the palette fill behind it, and fails below WCAG 4.5:1.
//!
//! Text and fill colours are taken from the Homestead palettes only, so the
//! not-yet-migrated `Tokens::` colours (step f) are ignored rather than
//! reported as false positives.

mod common;

use egui::Color32;
use image::RgbaImage;
use lootsmith_core::Container;
use lootsmith_gui::app::Theme;
use lootsmith_gui::theme::{contrast, palette_fill_colors, palette_text_colors};

fn rgb(c: Color32) -> [u8; 3] {
    [c.r(), c.g(), c.b()]
}

/// Every text pixel whose dominant palette fill is too close to it.
fn low_contrast(
    img: &RgbaImage,
    text: &[Color32],
    fill: &[Color32],
    threshold: f32,
) -> Vec<String> {
    let (w, h) = img.dimensions();
    let get = |x: i64, y: i64| -> Option<[u8; 3]> {
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            return None;
        }
        let p = img.get_pixel(x as u32, y as u32);
        Some([p[0], p[1], p[2]])
    };
    let text_rgb: Vec<[u8; 3]> = text.iter().copied().map(rgb).collect();
    let fill_rgb: Vec<[u8; 3]> = fill.iter().copied().map(rgb).collect();

    let mut seen = std::collections::HashSet::new();
    let mut out = Vec::new();
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let Some(color) = get(x, y) else { continue };
            if !text_rgb.contains(&color) {
                continue;
            }
            // Dominant palette fill in a 5x5 window. Require it to be the
            // majority of the fill pixels, so a glyph on a fill boundary (a
            // different fill a few pixels away) is skipped rather than
            // mis-attributed.
            let mut counts: std::collections::HashMap<[u8; 3], u32> =
                std::collections::HashMap::new();
            let mut fill_pixels = 0u32;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    if let Some(n) = get(x + dx, y + dy) {
                        if fill_rgb.contains(&n) {
                            *counts.entry(n).or_insert(0) += 1;
                            fill_pixels += 1;
                        }
                    }
                }
            }
            let Some((&bg, &n)) = counts.iter().max_by_key(|(_, &v)| v) else {
                continue;
            };
            if n < 8 || n * 2 < fill_pixels {
                continue; // no clear, dominant fill behind this pixel
            }
            let ratio = contrast(
                Color32::from_rgb(color[0], color[1], color[2]),
                Color32::from_rgb(bg[0], bg[1], bg[2]),
            );
            if ratio < threshold && seen.insert((color, bg)) {
                out.push(format!(
                    "text #{:02X}{:02X}{:02X} on fill #{:02X}{:02X}{:02X} = {ratio:.2}:1 (at {x},{y})",
                    color[0], color[1], color[2], bg[0], bg[1], bg[2],
                ));
            }
        }
    }
    out
}

fn render(save: &str, theme: Theme) -> RgbaImage {
    let mut h = common::harness_isolated(save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    h.render().expect("render the app")
}

#[test]
fn rendered_text_meets_wcag_in_both_themes() {
    let save = common::synthetic_save_path();
    let fill = palette_fill_colors();
    // A colour used both as text and as a fill (dark `on` == dark `bg`) cannot be
    // told apart by pixel inspection, so drop those from the text set.
    let text: Vec<Color32> = palette_text_colors()
        .into_iter()
        .filter(|c| !fill.contains(c))
        .collect();
    for theme in [Theme::Dark, Theme::Light] {
        let img = render(&save, theme);
        let failures = low_contrast(&img, &text, &fill, 4.5);
        assert!(
            failures.is_empty(),
            "{theme:?}: rendered text below 4.5:1:\n{}",
            failures.join("\n")
        );
    }
}

/// The analyser itself must flag the exact bug that motivated it: `fg` text on
/// an `acc` fill (a pale label on the orange selected-button colour).
#[test]
fn analyser_flags_fg_text_on_acc_fill() {
    let palette = lootsmith_gui::theme::DARK;
    let mut img = RgbaImage::from_pixel(
        64,
        64,
        image::Rgba([palette.acc.r(), palette.acc.g(), palette.acc.b(), 255]),
    );
    for x in 8..56 {
        for y in 30..34 {
            img.put_pixel(
                x,
                y,
                image::Rgba([palette.fg.r(), palette.fg.g(), palette.fg.b(), 255]),
            );
        }
    }
    let text = palette_text_colors();
    let fill = palette_fill_colors();
    let failures = low_contrast(&img, &text, &fill, 4.5);
    assert!(
        failures.iter().any(|f| f.contains("fill")
            && f.contains(&format!(
                "#{:02X}{:02X}{:02X}",
                palette.acc.r(),
                palette.acc.g(),
                palette.acc.b()
            ))),
        "expected fg-on-acc to be flagged, got {failures:?}"
    );
}
