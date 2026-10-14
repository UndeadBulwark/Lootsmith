//! Shape-tree frame analysis shared by the contrast gate (`tests/contrast.rs`)
//! and the digit gate (`tests/digits.rs`).
//!
//! Everything is derived from the frame's shapes: text runs (colour, rect,
//! opacity), the fills behind them, and every text *section* with its font
//! family (so a digit rendered in the label font can be caught).

use egui::{Color32, Rect, Shape};
use image::RgbaImage;
use lootsmith_core::Container;
use lootsmith_gui::app::{LootsmithApp, Tab};
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
    render_tab_at(tab, theme, 1.0)
}

/// [`render_tab`] at an explicit UI scale.
pub fn render_tab_at(
    tab: Tab,
    theme: Theme,
    scale: f32,
) -> (Vec<Run>, Vec<Fill>, Vec<Section>, RgbaImage) {
    let save = super::synthetic_save_path();
    let mut h = super::harness_isolated(&save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_ui_scale(scale);
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
pub const SCREENS: [Tab; 14] = [
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
    Tab::Profile,
    Tab::Import,
];

/// The design's hard rules (spec §2.1), scanned from a frame's shapes: no
/// gradient meshes (shadows and the scroll-area fade are meshes whose vertices
/// differ in colour) and no non-integer physical border widths. Rounded
/// rectangles are **not** checked yet — `card`/`tree_card`/`skill_tile` still use
/// `CornerRadius`, and step (d) replaces them with notched primitives.
pub fn rule_violations(output: &egui::FullOutput, ppp: f32) -> Vec<String> {
    use egui::Shape;
    let mut out = Vec::new();
    let mut gradient = 0usize;
    let mut borders = 0usize;
    for cs in &output.shapes {
        match &cs.shape {
            Shape::Mesh(mesh) => {
                let colours: std::collections::HashSet<(u8, u8, u8)> = mesh
                    .vertices
                    .iter()
                    .map(|v| (v.color.r(), v.color.g(), v.color.b()))
                    .collect();
                if colours.len() > 1 {
                    gradient += 1;
                }
            }
            Shape::Rect(r) => {
                let w = r.stroke.width * ppp;
                if (w - w.round()).abs() > 1e-3 {
                    borders += 1;
                }
            }
            Shape::Path(p) => {
                let w = p.stroke.width * ppp;
                if (w - w.round()).abs() > 1e-3 {
                    borders += 1;
                }
            }
            _ => {}
        }
    }
    if gradient > 0 {
        out.push(format!("{gradient} gradient mesh(es)"));
    }
    if borders > 0 {
        out.push(format!("{borders} non-integer border width(s)"));
    }
    out
}

/// Rule violations for one rendered screen.
pub fn screen_rule_violations(tab: Tab, theme: Theme) -> Vec<String> {
    screen_rule_violations_at(tab, theme, 1.0)
}

/// [`screen_rule_violations`] at an explicit UI scale.
pub fn screen_rule_violations_at(tab: Tab, theme: Theme, scale: f32) -> Vec<String> {
    let save = super::synthetic_save_path();
    let mut h = super::harness_isolated(&save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_ui_scale(scale);
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
    rule_violations(h.output(), scale)
}

/// Any text section on `tab` in `theme` whose characters are not all present in
/// its font would render as a "tofu" box. Run on a plain `Context` (no renderer).
pub fn screen_tofu(tab: Tab, theme: Theme) -> Vec<String> {
    let save = super::synthetic_save_path();
    let mut app = LootsmithApp::new_isolated(Some(save));
    app.set_theme(theme);
    app.set_tab(tab);
    match tab {
        Tab::Backpack => {
            app.select_entry(Container::BackpackWeapon, 0);
        }
        Tab::Bank => {
            app.select_entry(Container::Bank, 0);
        }
        _ => {}
    }
    let ctx = egui::Context::default();
    let input = || egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(1280.0, 800.0),
        )),
        ..Default::default()
    };
    let mut out = ctx.run_ui(input(), |ui| app.build(ui));
    out.textures_delta.clear();
    out = ctx.run_ui(input(), |ui| app.build(ui));
    out.textures_delta.clear();

    let mut bad = Vec::new();
    for cs in &out.shapes {
        if let egui::Shape::Text(t) = &cs.shape {
            for s in &t.galley.job.sections {
                let range = s.byte_range.start.0..s.byte_range.end.0;
                let text = t.galley.job.text.get(range).unwrap_or_default();
                if !text.chars().any(|c| !c.is_whitespace()) {
                    continue;
                }
                if !ctx.fonts_mut(|f| f.has_glyphs(&s.format.font_id, text)) {
                    bad.push(format!("{text:?} in {:?}", s.format.font_id));
                }
            }
        }
    }
    bad
}
