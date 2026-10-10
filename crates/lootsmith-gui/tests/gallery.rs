//! Primitive gallery, for human review before the shell uses the primitives.
//!
//! Renders every Homestead primitive (notched panel, ribbon, plank bar, rivets,
//! trail divider, rarity diamonds, dither tile, fonts) in both themes at 100%
//! and 200% scale. Review the PNGs in `tests/snapshots/gallery_*.png`.

use egui::{pos2, vec2, Align2, FontId, Rect, Vec2};
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::pixel;

struct Gallery {
    theme: Theme,
    scale: f32,
    applied: bool,
    dither: Option<egui::TextureHandle>,
}

fn build(ui: &mut egui::Ui, g: &mut Gallery) {
    let ctx = ui.ctx().clone();
    ctx.set_pixels_per_point(g.scale);
    if !g.applied {
        theme::apply(&ctx, g.theme);
        g.applied = true;
    }
    let p = theme::active(&ctx);
    let ppp = ctx.pixels_per_point();
    let painter = ui.painter().clone();
    if g.dither.is_none() {
        g.dither = Some(pixel::dither_texture(&ctx, p));
    }

    let bg = ui.max_rect();
    painter.rect_filled(bg, 0.0, p.bg);
    if let Some(texture) = &g.dither {
        pixel::dither(&painter, bg, texture);
    }

    let x = 24.0;
    let w = 560.0;
    let mut y = 16.0;
    let caption = |painter: &egui::Painter, y: f32, text: &str| {
        painter.text(
            pos2(x, y),
            Align2::LEFT_TOP,
            text,
            FontId::proportional(15.0),
            p.fg,
        );
    };

    caption(&painter, y, "Panel: notched, 4 px border, stitched");
    pixel::panel(
        &painter,
        Rect::from_min_size(pos2(x, y + 22.0), vec2(w, 64.0)),
        ppp,
        p,
    );
    y += 104.0;

    caption(
        &painter,
        y,
        "Ribbon: swallow-tail ends (transparent), onr text",
    );
    pixel::ribbon(
        &painter,
        Rect::from_min_size(pos2(x + 20.0, y + 24.0), vec2(w - 40.0, 30.0)),
        ppp,
        p,
        "Weapon parts",
        FontId::proportional(15.0),
    );
    y += 78.0;

    caption(
        &painter,
        y,
        "Plank bar: wd fill, horizontal wd2 seams, gold rule",
    );
    pixel::plank_bar(
        &painter,
        Rect::from_min_size(pos2(x, y + 22.0), vec2(w, 74.0)),
        ppp,
        p,
        true,
    );
    y += 118.0;

    caption(&painter, y, "Rivets: 8x8 gold");
    for i in 0..6 {
        pixel::rivet(&painter, pos2(x + 24.0 + i as f32 * 30.0, y + 38.0), ppp, p);
    }
    y += 72.0;

    caption(&painter, y, "Trail divider: 3 px dotted brd");
    pixel::trail_divider(&painter, pos2(x, y + 32.0), pos2(x + w, y + 32.0), ppp, p);
    y += 66.0;

    caption(&painter, y, "Rarity diamonds: 14 px, 2 px fg outline");
    for (i, (name, fill)) in p.diamond_pairs().iter().enumerate() {
        let cx = x + 28.0 + i as f32 * 64.0;
        pixel::diamond(&painter, pos2(cx, y + 54.0), 14.0, *fill, ppp, p);
        painter.text(
            pos2(cx, y + 70.0),
            Align2::CENTER_TOP,
            *name,
            FontId::proportional(10.0),
            p.dim,
        );
    }
    y += 100.0;

    caption(
        &painter,
        y,
        "Dither tile (8x8 checker) and hatch (45deg, 2 px / 7 px)",
    );
    let dither_rect = Rect::from_min_size(pos2(x, y + 22.0), vec2(160.0, 48.0));
    let hatch_rect = Rect::from_min_size(pos2(x + 180.0, y + 22.0), vec2(160.0, 48.0));
    painter.rect_filled(dither_rect, 0.0, p.bg);
    if let Some(texture) = &g.dither {
        pixel::dither(&painter, dither_rect, texture);
    }
    painter.rect_filled(hatch_rect, 0.0, p.bg);
    pixel::hatch(&painter, hatch_rect, ppp, p);
    painter.rect_stroke(
        dither_rect,
        0.0,
        egui::Stroke::new(lootsmith_gui::ui::pixel::snap_len(1.0, ppp), p.brd),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        hatch_rect,
        0.0,
        egui::Stroke::new(lootsmith_gui::ui::pixel::snap_len(1.0, ppp), p.brd),
        egui::StrokeKind::Inside,
    );
    y += 90.0;

    caption(
        &painter,
        y,
        "Fonts: Pixelify Sans (labels) / VT323 (numbers)",
    );
    painter.text(
        pos2(x, y + 28.0),
        Align2::LEFT_TOP,
        "Pixelify Sans 15 px",
        FontId::proportional(15.0),
        p.fg,
    );
    painter.text(
        pos2(x + 220.0, y + 24.0),
        Align2::LEFT_TOP,
        "id=1234567",
        FontId::monospace(19.0),
        p.gold,
    );
}

fn harness(theme: Theme, scale: f32, size: Vec2) -> Harness<'static, Gallery> {
    Harness::builder().with_size(size).build_ui_state(
        |ui, g: &mut Gallery| build(ui, g),
        Gallery {
            theme,
            scale,
            applied: false,
            dither: None,
        },
    )
}

#[test]
fn gallery_dark_1x() {
    let mut h = harness(Theme::Dark, 1.0, Vec2::new(760.0, 700.0));
    h.run();
    h.snapshot("gallery_dark_1x");
}

#[test]
fn gallery_light_1x() {
    let mut h = harness(Theme::Light, 1.0, Vec2::new(760.0, 700.0));
    h.run();
    h.snapshot("gallery_light_1x");
}

#[test]
fn gallery_dark_2x() {
    let mut h = harness(Theme::Dark, 2.0, Vec2::new(1520.0, 1400.0));
    h.run();
    h.snapshot("gallery_dark_2x");
}

#[test]
fn gallery_light_2x() {
    let mut h = harness(Theme::Light, 2.0, Vec2::new(1520.0, 1400.0));
    h.run();
    h.snapshot("gallery_light_2x");
}
