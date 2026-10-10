//! Pixel-grid helpers (spec `LOOTSMITH_UI_SPEC.md` §2.1 rule 7).
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
//!
//! The full primitive set (notched rect, ribbon, plank bar, stitched panel,
//! rivets, trail divider, diamond, dither tile) is built on these helpers in the
//! next shell step.

use eframe::egui::{pos2, Pos2, Rect};

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
            // A 3 px dotted line becomes 5 px at 150% (4.5 rounds up), 6 px at 200%.
            let dotted = snap_len(3.0, ppp) * ppp;
            assert!(dotted.fract().abs() < 1e-3, "ppp={ppp} dotted={dotted}");
            assert!(dotted >= 3.0, "never thinner than authored");
        }
        // The specific 150% cases.
        assert!((snap_len(3.0, 1.5) * 1.5 - 5.0).abs() < 1e-3);
        assert!((snap(13.0, 1.5) * 1.5 - 20.0).abs() < 1e-3); // text origin
        assert!((snap(14.0, 1.5) * 1.5 - 21.0).abs() < 1e-3); // diamond
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
                // A panel (bordered rect) with odd authored min/size.
                let panel = snap_rect(
                    Rect::from_min_size(pos2(13.0, 15.0), egui::vec2(101.0, 57.0)),
                    ppp,
                );
                painter.rect_filled(panel, 0.0, egui::Color32::WHITE);
                // A trail-divider dash, 3 px square (odd at 150%).
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
