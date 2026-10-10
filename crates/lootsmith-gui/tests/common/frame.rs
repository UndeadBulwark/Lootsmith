//! Shape-tree frame analysis shared by the contrast gate (`tests/contrast.rs`)
//! and the digit gate (`tests/digits.rs`).
//!
//! Everything is derived from the frame's shapes: text runs (colour, rect,
//! opacity), the fills behind them, and every text *section* with its font
//! family (so a digit rendered in the label font can be caught).

use egui::{Color32, Rect, Shape};
use image::RgbaImage;
use lootsmith_core::Container;
use lootsmith_gui::app::Tab;
use lootsmith_gui::theme::Theme;

pub struct Run {
    pub order: usize,
    pub rect: Rect,
    pub color: Color32,
    pub opacity: f32,
    pub text: String,
}

#[derive(Clone, Copy)]
pub struct Fill {
    pub order: usize,
    pub rect: Rect,
    pub color: Color32,
}

pub struct Section {
    pub text: String,
    pub proportional: bool,
    pub color: Color32,
}

fn fill_of(shape: &Shape) -> Option<(Rect, Color32)> {
    match shape {
        Shape::Rect(r) if r.fill.a() > 0 => Some((r.rect, r.fill)),
        Shape::Circle(c) if c.fill.a() > 0 => Some((
            Rect::from_center_size(c.center, egui::vec2(c.radius * 2.0, c.radius * 2.0)),
            c.fill,
        )),
        Shape::Path(p) if p.fill.a() > 0 => {
            let mut rect = Rect::NOTHING;
            for pt in &p.points {
                rect.extend_with(*pt);
            }
            rect.is_positive().then_some((rect, p.fill))
        }
        _ => None,
    }
}

pub fn extract(output: &egui::FullOutput) -> (Vec<Run>, Vec<Fill>, Vec<Section>) {
    let mut runs = Vec::new();
    let mut fills = Vec::new();
    let mut sections = Vec::new();
    for (order, cs) in output.shapes.iter().enumerate() {
        match &cs.shape {
            Shape::Text(t) => {
                let color = t
                    .override_text_color
                    .or_else(|| {
                        t.galley
                            .job
                            .sections
                            .first()
                            .map(|s| s.format.color)
                            .filter(|c| *c != Color32::PLACEHOLDER)
                    })
                    .unwrap_or(t.fallback_color);
                runs.push(Run {
                    order,
                    rect: t.visual_bounding_rect(),
                    color,
                    opacity: t.opacity_factor,
                    text: t.galley.job.text.clone(),
                });
                for section in &t.galley.job.sections {
                    let range = section.byte_range.start.0..section.byte_range.end.0;
                    let text = t.galley.job.text.get(range).unwrap_or_default().to_string();
                    let family = section.format.font_id.family.clone();
                    let color = if section.format.color == Color32::PLACEHOLDER {
                        t.fallback_color
                    } else {
                        section.format.color
                    };
                    sections.push(Section {
                        text,
                        proportional: family == egui::FontFamily::Proportional,
                        color,
                    });
                }
            }
            other => {
                if let Some((rect, color)) = fill_of(other) {
                    fills.push(Fill { order, rect, color });
                }
            }
        }
    }
    (runs, fills, sections)
}

/// The fill behind a run: the last fill drawn before it that contains its centre.
pub fn shape_fill(run: &Run, fills: &[Fill]) -> Option<Color32> {
    let centre = run.rect.center();
    fills
        .iter()
        .filter(|f| f.order < run.order && f.rect.contains(centre))
        .max_by_key(|f| f.order)
        .map(|f| f.color)
}

/// Render one tab in one theme; returns the frame analysis and the image.
pub fn render_tab(tab: Tab, theme: Theme) -> (Vec<Run>, Vec<Fill>, Vec<Section>, RgbaImage) {
    let save = super::synthetic_save_path();
    let mut h = super::harness_isolated(&save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_tab(tab);
    match tab {
        Tab::Backpack => {
            h.state_mut().select_entry(Container::BackpackWeapon, 0);
        }
        Tab::Bank => {
            h.state_mut().select_entry(Container::Bank, 0);
        }
        _ => {}
    }
    h.run();
    let (runs, fills, sections) = {
        let out = h.output();
        extract(out)
    };
    let img = h.render().expect("render the screen");
    (runs, fills, sections, img)
}

/// Every tab, in the order the shell will show them.
pub const SCREENS: [Tab; 13] = [
    Tab::General,
    Tab::Character,
    Tab::Currency,
    Tab::Ammo,
    Tab::World,
    Tab::Skills,
    Tab::Travel,
    Tab::Vehicles,
    Tab::Missions,
    Tab::Backpack,
    Tab::Bank,
    Tab::Raw,
    Tab::About,
];
