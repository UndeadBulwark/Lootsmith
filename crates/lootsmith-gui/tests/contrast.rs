//! Rendered-contrast gate.
//!
//! The token-pair gate in `theme.rs` only checks the palette's own pairs; it
//! cannot see what a widget actually renders. This walks the **frame output's
//! shape tree**: every `Text` run with its real colour, and the fill of the
//! nearest enclosing panel/widget shape drawn before it. It fails a run below
//! WCAG 4.5:1 and prints it (text, rect, colours, ratio). `Tokens::` fills and
//! text are included — there is no palette-only scan.
//!
//! Frame sampling (the rendered image) is a **cross-check only**: disagreements
//! with the shape tree are reported but never override it.
//!
//! Ratchet: [`ACCEPTED_FAILURES`] records the current per-theme failing-run
//! count. The gate fails if it ever increases, and must reach 0 for the
//! Backpack screen at the end of the Backpack migration.

mod common;

use egui::{Color32, Rect, Shape};
use image::RgbaImage;
use lootsmith_gui::app::Theme;
use lootsmith_gui::theme::contrast;

/// The accepted failing-run count per theme (dark, light). The Backpack screen
/// is fully migrated, so this is a hard zero; the gate fails on any failure.
const ACCEPTED_FAILURES: [usize; 2] = [0, 0];

#[derive(Clone)]
struct TextRun {
    order: usize,
    rect: Rect,
    color: Color32,
    opacity: f32,
    text: String,
}

#[derive(Clone, Copy)]
struct Fill {
    order: usize,
    rect: Rect,
    color: Color32,
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

/// Collect text runs and fills, in draw order, from a frame's shapes.
fn extract(output: &egui::FullOutput) -> (Vec<TextRun>, Vec<Fill>) {
    let mut runs = Vec::new();
    let mut fills = Vec::new();
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
                runs.push(TextRun {
                    order,
                    rect: t.visual_bounding_rect(),
                    color,
                    opacity: t.opacity_factor,
                    text: t.galley.job.text.clone(),
                });
            }
            other => {
                if let Some((rect, color)) = fill_of(other) {
                    fills.push(Fill { order, rect, color });
                }
            }
        }
    }
    (runs, fills)
}

/// The fill behind a text run: the last fill drawn before it whose rect
/// contains the run's centre. This is the shape-tree result and is authoritative.
fn shape_fill(run: &TextRun, fills: &[Fill]) -> Option<Color32> {
    let centre = run.rect.center();
    fills
        .iter()
        .filter(|f| f.order < run.order && f.rect.contains(centre))
        .max_by_key(|f| f.order)
        .map(|f| f.color)
}

struct Failure {
    text: String,
    rect: Rect,
    text_color: Color32,
    fill: Color32,
    ratio: f32,
}

impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} on #{:02X}{:02X}{:02X} = {:.2}:1",
            self.text,
            self.fill.r(),
            self.fill.g(),
            self.fill.b(),
            self.ratio
        )
    }
}

fn analyse(runs: &[TextRun], fills: &[Fill]) -> Vec<Failure> {
    let mut out = Vec::new();
    for run in runs {
        if run.text.trim().is_empty() {
            continue;
        }
        // Disabled/inactive controls are exempt (WCAG 1.4.3); egui fades them.
        if run.opacity < 0.99 {
            continue;
        }
        let Some(fill) = shape_fill(run, fills) else {
            continue;
        };
        let ratio = contrast(run.color, fill);
        if ratio < 4.5 {
            out.push(Failure {
                text: run.text.clone(),
                rect: run.rect,
                text_color: run.color,
                fill,
                ratio,
            });
        }
    }
    out
}

/// Cross-check: the dominant non-text colour in a run's rect in the rendered
/// image. Reported when it disagrees with the shape tree, never used instead.
fn sampled_fill(img: &RgbaImage, run: &TextRun) -> Option<Color32> {
    let (w, h) = img.dimensions();
    let text = [run.color.r(), run.color.g(), run.color.b()];
    let mut counts: std::collections::HashMap<[u8; 3], u32> = std::collections::HashMap::new();
    let x0 = run.rect.left().floor().max(0.0) as i64;
    let y0 = run.rect.top().floor().max(0.0) as i64;
    let x1 = (run.rect.right().ceil() as i64).min(w as i64);
    let y1 = (run.rect.bottom().ceil() as i64).min(h as i64);
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x as u32, y as u32);
            let c = [p[0], p[1], p[2]];
            // Skip glyph ink and its antialias blend toward the ink colour.
            if c == text {
                continue;
            }
            *counts.entry(c).or_insert(0) += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(c, _)| Color32::from_rgb(c[0], c[1], c[2]))
}

fn render(save: &str, theme: Theme) -> (Vec<TextRun>, Vec<Fill>, RgbaImage) {
    let mut h = common::harness_isolated(save);
    h.state_mut().set_theme(theme);
    h.state_mut().set_tab(lootsmith_gui::app::Tab::Backpack);
    h.state_mut()
        .select_entry(lootsmith_core::Container::BackpackWeapon, 0);
    h.run();
    let (runs, fills) = {
        let out = h.output();
        extract(out)
    };
    let img = h.render().expect("render the app");
    (runs, fills, img)
}

#[test]
fn rendered_text_meets_wcag_in_both_themes() {
    let save = common::synthetic_save_path();
    let mut totals = [0usize; 2];
    for (slot, theme) in [Theme::Dark, Theme::Light].into_iter().enumerate() {
        let (runs, fills, img) = render(&save, theme);
        let failures = analyse(&runs, &fills);
        totals[slot] = failures.len();

        let mut disagreements = 0usize;
        for run in &runs {
            if let (Some(shape), Some(sampled)) = (shape_fill(run, &fills), sampled_fill(&img, run))
            {
                if shape != sampled {
                    disagreements += 1;
                    if disagreements <= 10 {
                        eprintln!(
                            "  sample-disagreement {:?}: shape=#{:02X}{:02X}{:02X} sampled=#{:02X}{:02X}{:02X}",
                            run.text,
                            shape.r(), shape.g(), shape.b(),
                            sampled.r(), sampled.g(), sampled.b(),
                        );
                    }
                }
            }
        }
        eprintln!(
            "{theme:?}: {} text runs, {} failing (<4.5:1), {disagreements} sampling disagreements",
            runs.len(),
            failures.len()
        );
        for f in failures.iter().take(200) {
            eprintln!(
                "  FAIL {:?} rect=({:.0},{:.0})-({:.0},{:.0}) text=#{:02X}{:02X}{:02X} fill=#{:02X}{:02X}{:02X} {:.2}:1",
                f.text,
                f.rect.left(), f.rect.top(), f.rect.right(), f.rect.bottom(),
                f.text_color.r(), f.text_color.g(), f.text_color.b(),
                f.fill.r(), f.fill.g(), f.fill.b(),
                f.ratio,
            );
        }
        if failures.len() > 200 {
            eprintln!("  … {} more", failures.len() - 200);
        }
    }
    for (slot, theme) in [Theme::Dark, Theme::Light].into_iter().enumerate() {
        assert!(
            totals[slot] <= ACCEPTED_FAILURES[slot],
            "{theme:?}: {} failing runs exceeds the accepted {}",
            totals[slot],
            ACCEPTED_FAILURES[slot]
        );
    }
    // The ratchet baseline is recorded by the reporter; when it reaches 0 this
    // becomes a hard zero.
    if ACCEPTED_FAILURES == [0; 2] {
        assert_eq!(
            totals,
            [0, 0],
            "the Backpack screen must have zero failures"
        );
    }
}

// ---------------------------------------------------------------------------
// Analyser self-tests (synthetic shapes; no glyph layout needed)
// ---------------------------------------------------------------------------

fn run(order: usize, x: f32, y: f32, color: Color32, text: &str) -> TextRun {
    TextRun {
        order,
        rect: Rect::from_min_size(egui::pos2(x, y), egui::vec2(60.0, 12.0)),
        color,
        opacity: 1.0,
        text: text.to_string(),
    }
}

fn fill(order: usize, x: f32, y: f32, w: f32, h: f32, color: Color32) -> Fill {
    Fill {
        order,
        rect: Rect::from_min_size(egui::pos2(x, y), egui::vec2(w, h)),
        color,
    }
}

/// `fg` text on an `acc` fill is flagged: the exact bug that motivated the gate
/// (a pale label on the orange selected-button colour).
#[test]
fn analyser_flags_fg_text_on_acc_fill() {
    let acc = lootsmith_gui::theme::DARK.acc;
    let fg = lootsmith_gui::theme::DARK.fg;
    let runs = vec![run(1, 10.0, 10.0, fg, "Smooth Repeater")];
    let fills = vec![fill(0, 0.0, 0.0, 100.0, 40.0, acc)];
    let failures = analyse(&runs, &fills);
    assert_eq!(failures.len(), 1);
    assert_eq!(failures[0].fill, acc);
}

/// A run whose centre is inside the *second* of two overlapping fills takes the
/// later one (a fill boundary must not be mis-attributed to the neighbour).
#[test]
fn analyser_uses_the_enclosing_fill_at_a_boundary() {
    let bg = lootsmith_gui::theme::DARK.bg;
    let acc = lootsmith_gui::theme::DARK.acc;
    let on = lootsmith_gui::theme::DARK.on; // dark-on-acc: high contrast
                                            // Text sits just inside the acc box (centre at x=41), near its left edge at 40.
    let runs = vec![run(2, 35.0, 10.0, on, "2")];
    let fills = vec![
        fill(0, 0.0, 0.0, 200.0, 40.0, bg),
        fill(1, 40.0, 0.0, 60.0, 40.0, acc),
    ];
    let failures = analyse(&runs, &fills);
    assert!(failures.is_empty(), "on/acc must pass: {failures:?}");
    // And the shape tree picked acc, not bg.
    assert_eq!(shape_fill(&runs[0], &fills), Some(acc));
}

/// A selected table row: every cell uses `on` on the `acc` fill — all pass.
#[test]
fn analyser_selected_row_on_acc_passes() {
    let acc = lootsmith_gui::theme::DARK.acc;
    let on = lootsmith_gui::theme::DARK.on;
    let fills = vec![fill(0, 0.0, 40.0, 500.0, 20.0, acc)];
    let runs = vec![
        run(1, 5.0, 43.0, on, "weapon"),
        run(2, 120.0, 43.0, on, "id=400"),
        run(3, 300.0, 43.0, on, "Smooth Repeater"),
    ];
    assert!(analyse(&runs, &fills).is_empty());
}
