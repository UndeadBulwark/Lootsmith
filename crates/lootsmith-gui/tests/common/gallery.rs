//! The primitive gallery, shared by the snapshot test and the digit gate.

use egui::{pos2, vec2, Align2, FontId, Rect, Vec2};
use egui_kittest::Harness;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::{pixel, widgets};

pub struct Gallery {
    theme: Theme,
    scale: f32,
    applied: bool,
    dither: Option<egui::TextureHandle>,
}

pub fn build(ui: &mut egui::Ui, g: &mut Gallery) {
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
        let job = widgets::job_with_nums(text, p.fg, 15.0);
        let galley = painter.layout_job(job);
        painter.galley(pos2(x, y), galley, p.fg);
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
        let job = widgets::job_with_nums(name, p.dim, 10.0);
        let galley = painter.layout_job(job);
        let rect = Align2::CENTER_TOP.anchor_size(pos2(cx, y + 70.0), galley.size());
        painter.galley(rect.min, galley, p.dim);
    }
    y += 100.0;

    caption(
        &painter,
        y,
        "Dither tile (8x8 checker, 2x) and hatch (45deg, 2 px / 7 px)",
    );
    // The dither is deliberately subtle; show it at 2x so the checker reads.
    let dither_rect = Rect::from_min_size(pos2(x, y + 22.0), vec2(160.0, 48.0));
    let hatch_rect = Rect::from_min_size(pos2(x + 180.0, y + 22.0), vec2(160.0, 48.0));
    painter.rect_filled(dither_rect, 0.0, p.bg);
    paint_dither_2x(&painter, dither_rect, p);
    painter.rect_filled(hatch_rect, 0.0, p.bg);
    pixel::hatch(&painter, hatch_rect, ppp, p);
    let edge = egui::Stroke::new(pixel::snap_len(1.0, ppp), p.brd);
    painter.rect_stroke(dither_rect, 0.0, edge, egui::StrokeKind::Inside);
    painter.rect_stroke(hatch_rect, 0.0, edge, egui::StrokeKind::Inside);
    y += 90.0;

    caption(
        &painter,
        y,
        "Fonts: Pixelify Sans (labels) / VT323 (numbers)",
    );
    let label = widgets::job_with_nums("Pixelify Sans 15 px", p.fg, 15.0);
    let galley = painter.layout_job(label);
    painter.galley(pos2(x, y + 28.0), galley, p.fg);
    painter.text(
        pos2(x + 220.0, y + 24.0),
        Align2::LEFT_TOP,
        "id=1234567",
        FontId::monospace(15.0),
        p.gold,
    );
}

/// The dither tile at 2x (16x16 px tiles) so its checker is visible in review.
pub fn paint_dither_2x(painter: &egui::Painter, rect: Rect, p: &lootsmith_gui::theme::Palette) {
    let tile = 16.0;
    let mut y = rect.top();
    let mut row = 0;
    while y < rect.bottom() {
        let mut x = rect.left();
        let mut col = 0;
        while x < rect.right() {
            let color = if (col + row) % 2 == 0 { p.tex } else { p.bg };
            let end = ((x + tile).min(rect.right()), (y + tile).min(rect.bottom()));
            painter.rect_filled(
                Rect::from_min_max(pos2(x, y), pos2(end.0, end.1)),
                0.0,
                color,
            );
            x += tile;
            col += 1;
        }
        y += tile;
        row += 1;
    }
}

pub fn harness(theme: Theme, scale: f32, size: Vec2) -> Harness<'static, Gallery> {
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
