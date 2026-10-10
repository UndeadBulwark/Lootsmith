//! The visual design system: colour tokens, spacing/typography, and small
//! reusable widgets. Keeps the look consistent and centralised.

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Dark,
    Light,
}

/// Colour tokens (dark theme).
pub struct Tokens;

impl Tokens {
    pub const BG: egui::Color32 = egui::Color32::from_rgb(0x0F, 0x11, 0x15);
    pub const SURFACE: egui::Color32 = egui::Color32::from_rgb(0x17, 0x1A, 0x20);
    pub const SURFACE_ALT: egui::Color32 = egui::Color32::from_rgb(0x1C, 0x20, 0x27);
    pub const BORDER: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x2F, 0x38);
    pub const TEXT: egui::Color32 = egui::Color32::from_rgb(0xE7, 0xE9, 0xEC);
    pub const TEXT_DIM: egui::Color32 = egui::Color32::from_rgb(0xA2, 0xA8, 0xB3);
    pub const TEXT_FAINT: egui::Color32 = egui::Color32::from_rgb(0x6C, 0x74, 0x80);
    pub const ACCENT: egui::Color32 = egui::Color32::from_rgb(0xF0, 0xA2, 0x3C);
    pub const SEL_FILL: egui::Color32 = egui::Color32::from_rgb(0x2A, 0x20, 0x13);
    pub const OK: egui::Color32 = egui::Color32::from_rgb(0x5B, 0xBF, 0x7A);
    pub const WARN: egui::Color32 = egui::Color32::from_rgb(0xE0, 0xA1, 0x5A);
}

// ---------------------------------------------------------------------------
// Palette (Homestead spec, LOOTSMITH_UI_SPEC.md §2.2). Two complete token sets;
// never mixed. Every text/surface pair must reach WCAG 4.5:1 (see the test).
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Palette {
    pub name: &'static str,
    pub bg: egui::Color32,
    pub pan: egui::Color32,
    pub fg: egui::Color32,
    pub dim: egui::Color32,
    pub acc: egui::Color32,
    pub on: egui::Color32,
    pub gold: egui::Color32,
    pub wd: egui::Color32,
    pub wd2: egui::Color32,
    pub onw: egui::Color32,
    pub rib: egui::Color32,
    pub onr: egui::Color32,
    pub brd: egui::Color32,
    pub tex: egui::Color32,
    pub dng: egui::Color32,
}

const fn rgb(hex: u32) -> egui::Color32 {
    egui::Color32::from_rgb(
        ((hex >> 16) & 0xFF) as u8,
        ((hex >> 8) & 0xFF) as u8,
        (hex & 0xFF) as u8,
    )
}

pub const DARK: Palette = Palette {
    name: "Dark",
    bg: rgb(0x14100C),
    pan: rgb(0x1E1812),
    fg: rgb(0xFFC98A),
    dim: rgb(0xC29862),
    acc: rgb(0xFFB866),
    on: rgb(0x14100C),
    gold: rgb(0xE3B04B),
    wd: rgb(0x5A3A20),
    wd2: rgb(0x4A2F1A),
    onw: rgb(0xFFD9A8),
    rib: rgb(0xD9A25A),
    onr: rgb(0x14100C),
    brd: rgb(0x7A5A38),
    tex: rgb(0x1A150F),
    dng: rgb(0xFF8A70),
};

pub const LIGHT: Palette = Palette {
    name: "Light",
    bg: rgb(0xCDBB92),
    pan: rgb(0xE8DDBF),
    fg: rgb(0x3A2616),
    dim: rgb(0x6B5236),
    acc: rgb(0xA8470F),
    on: rgb(0xF6EBCF),
    // Deliberately darker than the mockup's #8C5F0E, which measures ~4.1:1 on
    // `pan` (below the 4.5:1 target for small label text).
    gold: rgb(0x7A520A),
    wd: rgb(0x8A5A32),
    wd2: rgb(0x7A4E2B),
    onw: rgb(0xF6EBCF),
    rib: rgb(0xA8470F),
    onr: rgb(0xF6EBCF),
    brd: rgb(0x8A6A44),
    tex: rgb(0xC4B186),
    dng: rgb(0x9E2B14),
};

impl Palette {
    pub fn for_theme(theme: Theme) -> &'static Palette {
        match theme {
            Theme::Dark => &DARK,
            Theme::Light => &LIGHT,
        }
    }

    /// The text/surface pairs the spec requires to clear 4.5:1.
    pub fn contrast_pairs(&self) -> [(&'static str, egui::Color32, egui::Color32); 7] {
        [
            ("fg/pan", self.fg, self.pan),
            ("dim/pan", self.dim, self.pan),
            ("gold/pan", self.gold, self.pan),
            ("on/acc", self.on, self.acc),
            ("onw/wd", self.onw, self.wd),
            ("onr/rib", self.onr, self.rib),
            ("dng/pan", self.dng, self.pan),
        ]
    }
}

/// WCAG 2.x relative-luminance contrast ratio for two colours.
pub fn contrast(a: egui::Color32, b: egui::Color32) -> f32 {
    fn channel(value: u8) -> f32 {
        let c = f32::from(value) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    }
    fn luminance(color: egui::Color32) -> f32 {
        0.2126 * channel(color.r()) + 0.7152 * channel(color.g()) + 0.0722 * channel(color.b())
    }
    let (a, b) = (luminance(a), luminance(b));
    let (hi, lo) = if a > b { (a, b) } else { (b, a) };
    (hi + 0.05) / (lo + 0.05)
}

/// The palette currently installed by [`apply`] (helpers paint with it).
static ACTIVE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

pub fn active() -> &'static Palette {
    match ACTIVE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => &LIGHT,
        _ => &DARK,
    }
}

fn set_active(theme: Theme) {
    ACTIVE.store(
        match theme {
            Theme::Dark => 0,
            Theme::Light => 1,
        },
        std::sync::atomic::Ordering::Relaxed,
    );
}

/// Install visuals, spacing and typography for the chosen theme.
pub fn apply(ctx: &egui::Context, theme: Theme) {
    set_active(theme);
    let p = Palette::for_theme(theme);
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };
    // Homestead hard rules: flat, no shadows.
    visuals.window_shadow = egui::Shadow::NONE;
    visuals.popup_shadow = egui::Shadow::NONE;
    visuals.panel_fill = p.bg;
    visuals.window_fill = p.pan;
    visuals.window_stroke = egui::Stroke::new(1.0, p.wd);
    visuals.faint_bg_color = p.pan;
    visuals.extreme_bg_color = p.bg;
    visuals.hyperlink_color = p.gold;
    visuals.selection.bg_fill = p.acc;
    visuals.selection.stroke = egui::Stroke::new(1.0, p.on);
    visuals.override_text_color = Some(p.fg);

    visuals.panel_fill = p.bg;
    visuals.window_fill = p.pan;
    visuals.window_stroke = egui::Stroke::new(1.0, p.wd);
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.faint_bg_color = p.pan;
    visuals.extreme_bg_color = p.bg;
    visuals.hyperlink_color = p.acc;
    visuals.selection.bg_fill = p.acc;
    visuals.selection.stroke = egui::Stroke::new(1.0, p.acc);

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = p.pan;
    w.noninteractive.weak_bg_fill = p.pan;
    w.noninteractive.bg_stroke = egui::Stroke::new(1.0, p.wd);
    w.noninteractive.fg_stroke = egui::Stroke::new(1.0, p.fg);
    w.noninteractive.corner_radius = egui::CornerRadius::same(6);

    w.inactive.bg_fill = p.pan;
    w.inactive.weak_bg_fill = p.pan;
    w.inactive.bg_stroke = egui::Stroke::new(1.0, p.wd);
    w.inactive.fg_stroke = egui::Stroke::new(1.0, p.fg);
    w.inactive.corner_radius = egui::CornerRadius::same(6);

    w.hovered.bg_fill = p.pan;
    w.hovered.weak_bg_fill = p.pan;
    w.hovered.bg_stroke = egui::Stroke::new(1.0, p.acc);
    w.hovered.fg_stroke = egui::Stroke::new(1.0, p.fg);
    w.hovered.corner_radius = egui::CornerRadius::same(6);

    w.active.bg_fill = p.acc;
    w.active.weak_bg_fill = p.acc;
    w.active.bg_stroke = egui::Stroke::new(1.0, p.acc);
    w.active.fg_stroke = egui::Stroke::new(1.0, p.fg);
    w.active.corner_radius = egui::CornerRadius::same(6);

    w.open.bg_fill = p.pan;
    w.open.weak_bg_fill = p.pan;
    w.open.bg_stroke = egui::Stroke::new(1.0, p.wd);
    w.open.fg_stroke = egui::Stroke::new(1.0, p.fg);
    w.open.corner_radius = egui::CornerRadius::same(6);

    ctx.set_visuals(visuals);

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style.spacing.interact_size.y = 26.0;
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(11.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(12.0, egui::FontFamily::Monospace),
        );
    });
}

/// A section heading.
pub fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(title)
            .strong()
            .size(15.0)
            .color(active().gold),
    );
    ui.add_space(2.0);
}

/// A dimmed secondary label.
pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        egui::RichText::new(text.into())
            .color(active().dim)
            .size(12.0),
    );
}

/// A faint label (paths, ids).
pub fn faint(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        egui::RichText::new(text.into())
            .color(active().dim)
            .size(11.5),
    );
}

/// Monospaced text for ids.
pub fn mono(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into()).monospace().size(12.0)
}

/// A modern tab. Returns true when clicked.
pub fn tab(ui: &mut egui::Ui, selected: bool, label: &str) -> bool {
    let text = egui::RichText::new(label).size(13.0);
    let text = if selected {
        text.strong().color(active().fg)
    } else {
        text.color(active().dim)
    };
    let response = ui.add(
        egui::Button::new(text)
            .frame(false)
            .min_size(egui::vec2(0.0, 28.0)),
    );
    let rect = response.rect;
    let underline = if selected {
        Some(active().acc)
    } else if response.hovered() {
        Some(active().wd)
    } else {
        None
    };
    if let Some(color) = underline {
        ui.painter().hline(
            rect.x_range(),
            rect.bottom() - 2.0,
            egui::Stroke::new(2.0, color),
        );
    }
    response.clicked()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The spec's hard gate: every text/surface pair clears WCAG 4.5:1 in both
    /// palettes. The test prints the full table for the report.
    #[test]
    fn contrast_meets_wcag_in_both_themes() {
        let mut failures = Vec::new();
        for palette in [&DARK, &LIGHT] {
            println!("== {}", palette.name);
            for (pair, fg, bg) in palette.contrast_pairs() {
                let ratio = contrast(fg, bg);
                println!(
                    "  {pair:9} #{:02X}{:02X}{:02X} on #{:02X}{:02X}{:02X}  {ratio:5.2}:1  {}",
                    fg.r(),
                    fg.g(),
                    fg.b(),
                    bg.r(),
                    bg.g(),
                    bg.b(),
                    if ratio >= 4.5 { "PASS" } else { "FAIL" }
                );
                if ratio < 4.5 {
                    failures.push(format!("{}: {pair} {ratio:.2}:1", palette.name));
                }
            }
        }
        assert!(failures.is_empty(), "contrast failures: {failures:?}");
    }

    #[test]
    fn active_palette_follows_the_installed_theme() {
        set_active(Theme::Light);
        assert_eq!(active().name, "Light");
        set_active(Theme::Dark);
        assert_eq!(active().name, "Dark");
    }
}
