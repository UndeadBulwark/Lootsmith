//! Primitive gallery snapshots (content lives in `common::gallery`).

mod common;

use common::gallery;
use egui::Vec2;
use lootsmith_gui::theme::Theme;

#[test]
fn gallery_dark_1x() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = gallery::harness(Theme::Dark, 1.0, Vec2::new(760.0, 700.0));
    h.run();
    h.snapshot("gallery_dark_1x");
}

#[test]
fn gallery_light_1x() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = gallery::harness(Theme::Light, 1.0, Vec2::new(760.0, 700.0));
    h.run();
    h.snapshot("gallery_light_1x");
}

#[test]
fn gallery_dark_2x() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = gallery::harness(Theme::Dark, 2.0, Vec2::new(1520.0, 1400.0));
    h.run();
    h.snapshot("gallery_dark_2x");
}

#[test]
fn gallery_light_2x() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = gallery::harness(Theme::Light, 2.0, Vec2::new(1520.0, 1400.0));
    h.run();
    h.snapshot("gallery_light_2x");
}

/// The dither swatch shows **both** tile colours in a checker. The tile is 8 px;
/// the swatch magnifies it 2x (16 px cells) so it is visible in review.
#[test]
fn dither_swatch_contains_both_tile_colors_in_a_checker() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    use lootsmith_gui::theme::Palette;
    for theme in [Theme::Dark, Theme::Light] {
        let p = Palette::for_theme(theme);
        let mut h = gallery::harness(theme, 1.0, Vec2::new(760.0, 700.0));
        h.run();
        let swatch = h.state().dither_swatch;
        assert!(swatch.is_positive(), "the swatch rect must be recorded");
        let img = h.render().expect("render gallery");

        // Sample the centre of each 16 px (2x of 8 px) cell.
        let tex = [p.tex.r(), p.tex.g(), p.tex.b()];
        let bg = [p.bg.r(), p.bg.g(), p.bg.b()];
        let mut tex_cells = 0usize;
        let mut bg_cells = 0usize;
        let mut y = swatch.top() + 8.0;
        let mut row = 0;
        while y < swatch.bottom() {
            let mut x = swatch.left() + 8.0;
            let mut col = 0;
            while x < swatch.right() {
                let px = img.get_pixel(x as u32, y as u32);
                let c = [px[0], px[1], px[2]];
                let expect = if (col + row) % 2 == 0 { tex } else { bg };
                assert_eq!(
                    c, expect,
                    "{theme:?}: swatch cell ({col},{row}) is not the checker colour"
                );
                if c == tex {
                    tex_cells += 1;
                } else {
                    bg_cells += 1;
                }
                x += 16.0;
                col += 1;
            }
            y += 16.0;
            row += 1;
        }
        assert!(
            tex_cells > 0 && bg_cells > 0,
            "{theme:?}: both tile colours present"
        );

        // Report whether the *real* 1x background dither is visible.
        let ratio = lootsmith_gui::theme::contrast(p.tex, p.bg);
        eprintln!("{theme:?}: 1x background dither tex/bg contrast = {ratio:.3}:1");
    }
}
