//! Pixel-grid helpers and the Homestead drawing primitives
//! (`LOOTSMITH_UI_SPEC.md` §2.1 rule 7 and §2.5).
//!
//! 150% is not an integer multiple, so a value authored in points can land on a
//! half physical pixel. Every painted rect, line and stroke is therefore
//! *snapped* to whole physical pixels before it is drawn. A logical coordinate
//! `v` maps to `v * ppp` physical pixels; snapping rounds that product and
//! divides back:
//!
//! ```
//! # use lootsmith_gui::ui::pixel::snap;
//! assert!((snap(3.0, 1.5) * 1.5 - 5.0).abs() < 1e-3); // 4.5 px -> 5 px at 150%
//! assert_eq!(snap(3.0, 2.0), 3.0);                    // 6 px stays 6 px at 200%
//! ```

use crate::theme::Palette;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Pos2, Rect, Stroke, TextureOptions};

// ---------------------------------------------------------------------------
// Snapping (spec rule 7)
// ---------------------------------------------------------------------------

/// Snap a logical length to a whole number of physical pixels.
pub fn snap(value: f32, pixels_per_point: f32) -> f32 {
    if pixels_per_point <= 0.0 {
        return value;
    }
    (value * pixels_per_point).round() / pixels_per_point
}

/// Snap a point to the physical-pixel grid.
pub fn snap_pos(pos: Pos2, pixels_per_point: f32) -> Pos2 {
    pos2(snap(pos.x, pixels_per_point), snap(pos.y, pixels_per_point))
}

/// Snap both corners of a rectangle, so every edge lands on a physical pixel.
pub fn snap_rect(rect: Rect, pixels_per_point: f32) -> Rect {
    Rect::from_min_max(
        snap_pos(rect.min, pixels_per_point),
        snap_pos(rect.max, pixels_per_point),
    )
}

/// Snap a stroke width (or any small length) to at least one whole physical
/// pixel — a "3 px" dotted line is 4.5 px at 150%, so it becomes 5 px.
pub fn snap_len(len: f32, pixels_per_point: f32) -> f32 {
    if pixels_per_point <= 0.0 {
        return len;
    }
    (len * pixels_per_point).round().max(1.0) / pixels_per_point
}

/// Whether a logical coordinate already sits on a whole physical pixel.
pub fn is_whole_physical(value: f32, pixels_per_point: f32) -> bool {
    ((value * pixels_per_point) - (value * pixels_per_point).round()).abs() < 1e-3
}

// ---------------------------------------------------------------------------
// Primitives (spec §2.5)
// ---------------------------------------------------------------------------

/// A rectangle with 4 px **stepped** corners: a 12-point polygon (two steps per
/// corner). `step` is the diagonal cut per corner; corners are cut twice, so
/// the notch spans `2 * step`.
pub fn notched_rect(rect: Rect, step: f32, ppp: f32) -> Vec<Pos2> {
    let r = snap_rect(rect, ppp);
    let s = snap_len(step, ppp);
    let (l, t, rr, b) = (r.left(), r.top(), r.right(), r.bottom());
    vec![
        pos2(l + s * 2.0, t),
        pos2(rr - s * 2.0, t),
        pos2(rr - s, t + s),
        pos2(rr, t + s * 2.0),
        pos2(rr, b - s * 2.0),
        pos2(rr - s, b - s),
        pos2(rr - s * 2.0, b),
        pos2(l + s * 2.0, b),
        pos2(l + s, b - s),
        pos2(l, b - s * 2.0),
        pos2(l, t + s * 2.0),
        pos2(l + s, t + s),
    ]
}

/// A stitched panel: notched `pan` fill, a `wd` border, and a dashed `brd`
/// stitch inset by `stitch_inset`.
pub fn panel(painter: &egui::Painter, rect: Rect, ppp: f32, p: &Palette) {
    let points = notched_rect(rect, 4.0, ppp);
    painter.add(egui::Shape::convex_polygon(
        points,
        p.pan,
        Stroke::new(snap_len(4.0, ppp), p.wd),
    ));
    let inset = snap_rect(rect.shrink(10.0), ppp);
    let mut stitch = notched_rect(inset, 4.0, ppp);
    stitch.push(stitch[0]);
    painter.extend(egui::Shape::dashed_line(
        &stitch,
        Stroke::new(snap_len(2.0, ppp), p.brd),
        snap_len(6.0, ppp),
        snap_len(4.0, ppp),
    ));
}

/// A banner ribbon with swallow-tail ends. Drawn as two convex halves (no
/// backing rect), so the notches are genuinely transparent.
pub fn ribbon(
    painter: &egui::Painter,
    rect: Rect,
    ppp: f32,
    p: &Palette,
    text: &str,
    font: FontId,
) {
    let r = snap_rect(rect, ppp);
    let notch = snap_len(10.0, ppp);
    let cy = snap((r.top() + r.bottom()) / 2.0, ppp);
    let upper = vec![
        pos2(r.left(), r.top()),
        pos2(r.right(), r.top()),
        pos2(r.right() - notch, cy),
        pos2(r.left() + notch, cy),
    ];
    let lower = vec![
        pos2(r.left() + notch, cy),
        pos2(r.right() - notch, cy),
        pos2(r.right(), r.bottom()),
        pos2(r.left(), r.bottom()),
    ];
    painter.add(egui::Shape::convex_polygon(upper, p.rib, Stroke::NONE));
    painter.add(egui::Shape::convex_polygon(lower, p.rib, Stroke::NONE));
    painter.text(r.center(), Align2::CENTER_CENTER, text, font, p.onr);
}

/// A plank bar: `wd` fill, **horizontal** plank seams (`wd2`, 2 px every 26 px —
/// the mockup's `repeating-linear-gradient(180deg, ...)`), and a 4 px `gold`
/// rule on the inner edge. A vertical bar uses vertical seams.
pub fn plank_bar(painter: &egui::Painter, rect: Rect, ppp: f32, p: &Palette, horizontal: bool) {
    let r = snap_rect(rect, ppp);
    painter.rect_filled(r, 0.0, p.wd);
    let line = Stroke::new(snap_len(2.0, ppp), p.wd2);
    let spacing = 26.0;
    if horizontal {
        let mut y = r.top() + spacing;
        while y < r.bottom() {
            let sy = snap(y, ppp);
            painter.line_segment([pos2(r.left(), sy), pos2(r.right(), sy)], line);
            y += spacing;
        }
        painter.rect_filled(
            Rect::from_min_max(pos2(r.left(), r.bottom() - 4.0), r.right_bottom()),
            0.0,
            p.gold,
        );
    } else {
        let mut x = r.left() + spacing;
        while x < r.right() {
            let sx = snap(x, ppp);
            painter.line_segment([pos2(sx, r.top()), pos2(sx, r.bottom())], line);
            x += spacing;
        }
        painter.rect_filled(
            Rect::from_min_max(pos2(r.right() - 4.0, r.top()), r.right_bottom()),
            0.0,
            p.gold,
        );
    }
}

/// A 45° hatch: `tex` lines 2 px wide on a 7 px period (the mockup's
/// `repeating-linear-gradient(45deg, tex 0 2px, transparent 2px 7px)`).
pub fn hatch(painter: &egui::Painter, rect: Rect, ppp: f32, p: &Palette) {
    let r = snap_rect(rect, ppp);
    let painter = painter.with_clip_rect(r);
    let stroke = Stroke::new(snap_len(2.0, ppp), p.tex);
    let period = snap_len(7.0, ppp) * std::f32::consts::SQRT_2;
    let h = r.height();
    let mut x = r.left() - h;
    while x < r.right() {
        let sx = snap(x, ppp);
        painter.line_segment([pos2(sx, r.top()), pos2(sx + h, r.bottom())], stroke);
        x += period;
    }
}

/// An 8x8 px `gold` rivet centred on `center`.
pub fn rivet(painter: &egui::Painter, center: Pos2, ppp: f32, p: &Palette) {
    let c = snap_pos(center, ppp);
    painter.rect_filled(
        snap_rect(Rect::from_center_size(c, vec2(8.0, 8.0)), ppp),
        0.0,
        p.gold,
    );
}

/// A 3 px dotted `brd` trail divider (spec: dash 3, gap 5).
pub fn trail_divider(painter: &egui::Painter, from: Pos2, to: Pos2, ppp: f32, p: &Palette) {
    let from = snap_pos(from, ppp);
    let to = snap_pos(to, ppp);
    painter.extend(egui::Shape::dashed_line(
        &[from, to],
        Stroke::new(snap_len(3.0, ppp), p.brd),
        snap_len(3.0, ppp),
        snap_len(5.0, ppp),
    ));
}

/// A flat 4-point rarity diamond with the mandatory `fg` outline (spec §2.3).
/// Returns the polygon for tests.
pub fn diamond(
    painter: &egui::Painter,
    center: Pos2,
    size: f32,
    fill: Color32,
    ppp: f32,
    p: &Palette,
) -> Vec<Pos2> {
    let c = snap_pos(center, ppp);
    // Keep the diamond even in physical pixels so its centre is exact.
    let mut s = snap(size, ppp);
    if ((s * ppp).round() as i32) % 2 != 0 {
        s = snap(size + 1.0, ppp);
    }
    let h = s / 2.0;
    let points = vec![
        pos2(c.x, c.y - h),
        pos2(c.x + h, c.y),
        pos2(c.x, c.y + h),
        pos2(c.x - h, c.y),
    ];
    painter.add(egui::Shape::convex_polygon(
        points.clone(),
        fill,
        Stroke::new(snap_len(crate::theme::DIAMOND_OUTLINE, ppp), p.fg),
    ));
    points
}

/// An 8x8 px two-colour checker tile for the background dither.
pub fn dither_tile(p: &Palette) -> egui::ColorImage {
    let mut pixels = Vec::with_capacity(64);
    for y in 0..8 {
        for x in 0..8 {
            pixels.push(if (x + y) % 2 == 0 { p.tex } else { p.bg });
        }
    }
    egui::ColorImage::new([8, 8], pixels)
}

/// Upload the dither tile with nearest filtering and repeat wrapping.
pub fn dither_texture(ctx: &egui::Context, p: &Palette) -> egui::TextureHandle {
    ctx.load_texture(
        "lootsmith.dither",
        dither_tile(p),
        TextureOptions {
            magnification: egui::TextureFilter::Nearest,
            minification: egui::TextureFilter::Nearest,
            wrap_mode: egui::TextureWrapMode::Repeat,
            ..Default::default()
        },
    )
}

/// Tile the dither texture across `rect` (one tile per 8x8 px).
pub fn dither(painter: &egui::Painter, rect: Rect, texture: &egui::TextureHandle) {
    let r = snap_rect(rect, painter.ctx().pixels_per_point());
    let tiles = vec2(r.width() / 8.0, r.height() / 8.0);
    painter.image(
        texture.id(),
        r,
        Rect::from_min_size(Pos2::ZERO, tiles),
        Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The dimensions called out in the spec that are **not** whole at 150%:
    /// the 3 px dotted line, 13/15 px text, 14 px diamonds, and the bar heights.
    /// After snapping, each lands on a whole physical pixel at every scale.
    #[test]
    fn odd_authored_sizes_snap_onto_the_physical_grid() {
        for ppp in [1.0_f32, 1.5, 2.0] {
            for authored in [3.0_f32, 13.0, 14.0, 15.0, 26.0, 34.0, 56.0] {
                let snapped = snap(authored, ppp);
                assert!(
                    is_whole_physical(snapped, ppp),
                    "ppp={ppp} authored={authored} -> {snapped}"
                );
            }
            let dotted = snap_len(3.0, ppp) * ppp;
            assert!(dotted.fract().abs() < 1e-3, "ppp={ppp} dotted={dotted}");
            assert!(dotted >= 3.0, "never thinner than authored");
        }
        assert!((snap_len(3.0, 1.5) * 1.5 - 5.0).abs() < 1e-3);
        assert!((snap(13.0, 1.5) * 1.5 - 20.0).abs() < 1e-3); // text origin
        assert!((snap(14.0, 1.5) * 1.5 - 21.0).abs() < 1e-3); // diamond
    }

    /// `notched_rect` returns 12 points with corners stepped at the given size.
    #[test]
    fn notched_rect_is_a_twelve_point_stepped_polygon() {
        let r = Rect::from_min_size(pos2(0.0, 0.0), vec2(100.0, 50.0));
        let pts = notched_rect(r, 4.0, 1.0);
        assert_eq!(pts.len(), 12);
        assert_eq!(pts[0], pos2(8.0, 0.0));
        assert_eq!(pts[1], pos2(92.0, 0.0));
        assert_eq!(pts[2], pos2(96.0, 4.0));
        assert_eq!(pts[3], pos2(100.0, 8.0));
        assert_eq!(pts[10], pos2(0.0, 8.0));
        assert_eq!(pts[11], pos2(4.0, 4.0));
    }

    /// Render the two shell shapes that carry borders (a panel rect and a dash)
    /// at 100/150/200% and assert **every** tessellated vertex lands on a whole
    /// physical pixel (feathering off, so edges are exact).
    #[test]
    fn painted_edges_land_on_whole_physical_pixels() {
        for ppp in [1.0_f32, 1.5, 2.0] {
            let ctx = egui::Context::default();
            ctx.set_pixels_per_point(ppp);
            ctx.tessellation_options_mut(|o| o.feathering = false);
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, egui::vec2(400.0, 300.0))),
                ..Default::default()
            };
            let egui::FullOutput {
                shapes,
                mut textures_delta,
                ..
            } = ctx.run_ui(input, |ui| {
                let ppp = ui.ctx().pixels_per_point();
                let painter = ui.painter();
                let panel = snap_rect(
                    Rect::from_min_size(pos2(13.0, 15.0), egui::vec2(101.0, 57.0)),
                    ppp,
                );
                painter.rect_filled(panel, 0.0, egui::Color32::WHITE);
                let dash = snap_rect(
                    Rect::from_min_size(pos2(7.0, 31.0), egui::vec2(3.0, 3.0)),
                    ppp,
                );
                painter.rect_filled(dash, 0.0, egui::Color32::WHITE);
            });
            textures_delta.clear();

            let primitives = ctx.tessellate(shapes, ppp);
            let mut count = 0;
            for primitive in primitives {
                if let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive {
                    for vertex in mesh.vertices {
                        count += 1;
                        let (x, y) = (vertex.pos.x * ppp, vertex.pos.y * ppp);
                        assert!(
                            (x - x.round()).abs() < 1e-3 && (y - y.round()).abs() < 1e-3,
                            "ppp={ppp}: vertex {:?} is not on a whole physical pixel",
                            vertex.pos
                        );
                    }
                }
            }
            assert!(count >= 8, "ppp={ppp}: expected the shapes to render");
        }
    }
}
