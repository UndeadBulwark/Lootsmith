//! Rendered-contrast gate.
//!
//! Walks the frame's **shape tree** (`common::frame`): every `Text` run's real
//! colour and the fill of the nearest enclosing shape drawn before it. `Tokens::`
//! fills and text are included. Frame sampling is a cross-check only.
//!
//! The Backpack screen is fully migrated, so this is a hard zero; Fix 3 extends
//! the gate to every screen with per-screen ratchets.

mod common;

use common::frame::{render_tab, shape_fill, Fill, Run, SCREENS};
use egui::{Color32, Rect};
use image::RgbaImage;
use lootsmith_gui::theme::{contrast, Theme};

/// Accepted failing-run count per screen, per theme `[dark, light]`, indexed by
/// [`SCREENS`]. Recorded from today's render; the gate fails on any increase.
/// Backpack, Bank, Character and Currency are fully migrated, so they are hard
/// zero. Step (f) must drive the rest to zero.
const ACCEPTED_FAILURES: [[usize; 2]; SCREENS.len()] = [
    [0, 11],  // General
    [0, 0],   // Character
    [0, 0],   // Currency
    [6, 6],   // Ammo
    [0, 2],   // World
    [25, 29], // Skills
    [1, 2],   // Travel
    [5, 7],   // Vehicles
    [4, 4],   // Missions
    [0, 0],   // Backpack
    [0, 0],   // Bank
    [5, 5],   // Raw
    [0, 3],   // About
];

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
            "{:?} rect=({:.0},{:.0})-({:.0},{:.0}) text=#{:02X}{:02X}{:02X} fill=#{:02X}{:02X}{:02X} {:.2}:1",
            self.text,
            self.rect.left(), self.rect.top(), self.rect.right(), self.rect.bottom(),
            self.text_color.r(), self.text_color.g(), self.text_color.b(),
            self.fill.r(), self.fill.g(), self.fill.b(),
            self.ratio,
        )
    }
}

fn analyse(runs: &[Run], fills: &[Fill]) -> Vec<Failure> {
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
fn sampled_fill(img: &RgbaImage, run: &Run) -> Option<Color32> {
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

#[test]
fn rendered_text_meets_wcag_in_both_themes() {
    let mut totals = [[0usize; 2]; SCREENS.len()];
    for (s, tab) in SCREENS.iter().enumerate() {
        for (slot, theme) in [Theme::Dark, Theme::Light].into_iter().enumerate() {
            let (runs, fills, _sections, img) = render_tab(*tab, theme);
            let failures = analyse(&runs, &fills);
            totals[s][slot] = failures.len();

            let mut disagreements = 0usize;
            for run in &runs {
                if let (Some(shape), Some(sampled)) =
                    (shape_fill(run, &fills), sampled_fill(&img, run))
                {
                    if shape != sampled {
                        disagreements += 1;
                    }
                }
            }
            eprintln!(
                "{tab:?} {theme:?}: {} runs, {} failing, {disagreements} sampling disagreements",
                runs.len(),
                failures.len(),
            );
            for f in failures.iter().take(60) {
                eprintln!("  FAIL {f:?}");
            }
            if failures.len() > 60 {
                eprintln!("  … {} more", failures.len() - 60);
            }
        }
    }
    let mut exceeded = Vec::new();
    for (s, tab) in SCREENS.iter().enumerate() {
        for (slot, theme) in [Theme::Dark, Theme::Light].into_iter().enumerate() {
            if totals[s][slot] > ACCEPTED_FAILURES[s][slot] {
                exceeded.push(format!(
                    "{tab:?} {theme:?}: {} > accepted {}",
                    totals[s][slot], ACCEPTED_FAILURES[s][slot]
                ));
            }
        }
    }
    assert!(
        exceeded.is_empty(),
        "contrast ratchet exceeded:\n{}",
        exceeded.join("\n")
    );
    eprintln!("ratchet table (dark,light):");
    for (s, tab) in SCREENS.iter().enumerate() {
        eprintln!("  {tab:?}: {:?}", totals[s]);
    }
}

// ---------------------------------------------------------------------------
// Analyser self-tests (synthetic shapes; no glyph layout needed)
// ---------------------------------------------------------------------------

fn run(order: usize, x: f32, y: f32, color: Color32, text: &str) -> Run {
    Run {
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

#[test]
fn analyser_uses_the_enclosing_fill_at_a_boundary() {
    let bg = lootsmith_gui::theme::DARK.bg;
    let acc = lootsmith_gui::theme::DARK.acc;
    let on = lootsmith_gui::theme::DARK.on;
    let runs = vec![run(2, 35.0, 10.0, on, "2")];
    let fills = vec![
        fill(0, 0.0, 0.0, 200.0, 40.0, bg),
        fill(1, 40.0, 0.0, 60.0, 40.0, acc),
    ];
    let failures = analyse(&runs, &fills);
    assert!(failures.is_empty(), "on/acc must pass: {failures:?}");
    assert_eq!(shape_fill(&runs[0], &fills), Some(acc));
}

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
