//! Primitive gallery snapshots (content lives in `common::gallery`).

mod common;

use common::gallery;
use egui::Vec2;
use lootsmith_gui::theme::Theme;

#[test]
fn gallery_dark_1x() {
    let mut h = gallery::harness(Theme::Dark, 1.0, Vec2::new(760.0, 700.0));
    h.run();
    h.snapshot("gallery_dark_1x");
}

#[test]
fn gallery_light_1x() {
    let mut h = gallery::harness(Theme::Light, 1.0, Vec2::new(760.0, 700.0));
    h.run();
    h.snapshot("gallery_light_1x");
}

#[test]
fn gallery_dark_2x() {
    let mut h = gallery::harness(Theme::Dark, 2.0, Vec2::new(1520.0, 1400.0));
    h.run();
    h.snapshot("gallery_dark_2x");
}

#[test]
fn gallery_light_2x() {
    let mut h = gallery::harness(Theme::Light, 2.0, Vec2::new(1520.0, 1400.0));
    h.run();
    h.snapshot("gallery_light_2x");
}
