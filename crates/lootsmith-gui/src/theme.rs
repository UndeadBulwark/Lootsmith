//! The visual design system: colour tokens, spacing/typography, and small
//! reusable widgets. Keeps the look consistent and centralised.

use eframe::egui;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Dark,
    Light,
}

// ---------------------------------------------------------------------------
// Fonts (spec §2.4). Both are OFL; their licence texts ship next to the files
// (`assets/fonts/OFL-PixelifySans.txt`, `OFL-VT323.txt`).
// ---------------------------------------------------------------------------

/// Pixelify Sans (OFL) — UI labels, headings, buttons.
const FONT_PIXELIFY: &[u8] = include_bytes!("../assets/fonts/PixelifySans.ttf");
/// VT323 (OFL) — numbers and item codes.
const FONT_VT323: &[u8] = include_bytes!("../assets/fonts/VT323.ttf");

/// Family name of the UI font (Pixelify Sans).
pub const FONT_UI: &str = "pixelify";
/// Family name of the numeric font (VT323).
pub const FONT_NUM: &str = "vt323";

/// Build the font set: Pixelify Sans first in the proportional family, VT323
/// first in the monospace family (numbers and codes).
pub fn font_definitions() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        FONT_UI.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(FONT_PIXELIFY)),
    );
    fonts.font_data.insert(
        FONT_NUM.to_owned(),
        std::sync::Arc::new(egui::FontData::from_static(FONT_VT323)),
    );
    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, FONT_UI.to_owned());
    fonts
        .families
        .entry(egui::FontFamily::Monospace)
        .or_default()
        .insert(0, FONT_NUM.to_owned());
    fonts
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
    /// Wood used for borders on `pan` (3:1); `wd` stays the wood fill.
    pub wdb: egui::Color32,
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
    wdb: rgb(0x96714A),
    wd2: rgb(0x4A2F1A),
    onw: rgb(0xFFD9A8),
    rib: rgb(0xD9A25A),
    onr: rgb(0x14100C),
    brd: rgb(0x8C6A40),
    tex: rgb(0x1A150F),
    dng: rgb(0xFF8A70),
};

// Item rarity diamonds (spec §2.3). Drawn flat, each with a 2 px `fg` outline so
// the white tier stays visible on parchment.
pub const RARITY_WHITE: egui::Color32 = rgb(0xD8DAE0);
pub const RARITY_GREEN: egui::Color32 = rgb(0x5ED08A);
pub const RARITY_BLUE: egui::Color32 = rgb(0x4C9BFF);
pub const RARITY_PURPLE: egui::Color32 = rgb(0xB06CFF);
pub const RARITY_ETECH: egui::Color32 = rgb(0xFF4FA3);
pub const RARITY_LEGENDARY: egui::Color32 = rgb(0xFF8A1F);
pub const RARITY_PEARLESCENT: egui::Color32 = rgb(0x3DE0D0);
/// The mandatory outline width for rarity diamonds (spec §2.3).
pub const DIAMOND_OUTLINE: f32 = 2.0;

pub const LIGHT: Palette = Palette {
    name: "Light",
    bg: rgb(0xCDBB92),
    pan: rgb(0xE8DDBF),
    fg: rgb(0x3A2616),
    dim: rgb(0x523D25),
    acc: rgb(0xA8470F),
    on: rgb(0xF6EBCF),
    // Deliberately darker than the mockup's #8C5F0E, which measures ~4.1:1 on
    // `pan` (below the 4.5:1 target for small label text).
    gold: rgb(0x5E3E06),
    wd: rgb(0x8A5A32),
    wdb: rgb(0x8A5A32),
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

    /// Text pairs: WCAG 4.5:1.
    pub fn text_pairs(&self) -> [(&'static str, egui::Color32, egui::Color32); 10] {
        [
            ("fg/pan", self.fg, self.pan),
            ("fg/bg", self.fg, self.bg),
            ("dim/pan", self.dim, self.pan),
            ("dim/bg", self.dim, self.bg),
            ("gold/pan", self.gold, self.pan),
            ("gold/bg", self.gold, self.bg),
            ("on/acc", self.on, self.acc),
            ("onw/wd", self.onw, self.wd),
            ("onr/rib", self.onr, self.rib),
            ("dng/pan", self.dng, self.pan),
        ]
    }

    /// Non-text pairs: WCAG 3:1 (borders, focus outlines, stitching).
    pub fn non_text_pairs(&self) -> [(&'static str, egui::Color32, egui::Color32); 5] {
        [
            ("brd/pan", self.brd, self.pan),
            ("gold/pan", self.gold, self.pan),
            ("wdb/pan", self.wdb, self.pan),
            ("acc/pan", self.acc, self.pan),
            ("acc/bg", self.acc, self.bg),
        ]
    }

    /// Rarity diamonds against the panel. The light theme darkens six of them
    /// so each clears 3:1 on parchment; the white tier relies on the mandatory
    /// 2 px `fg` outline (spec §2.3).
    pub fn diamond_pairs(&self) -> [(&'static str, egui::Color32); 7] {
        if self.name == "Light" {
            [
                ("white", RARITY_WHITE),
                ("green", rgb(0x2F7D4C)),
                ("blue", rgb(0x1E5FA8)),
                ("purple", rgb(0x6B3FB0)),
                ("e-tech", rgb(0xB02060)),
                ("legendary", rgb(0xA8500A)),
                ("pearlescent", rgb(0x17776D)),
            ]
        } else {
            [
                ("white", RARITY_WHITE),
                ("green", RARITY_GREEN),
                ("blue", RARITY_BLUE),
                ("purple", RARITY_PURPLE),
                ("e-tech", RARITY_ETECH),
                ("legendary", RARITY_LEGENDARY),
                ("pearlescent", RARITY_PEARLESCENT),
            ]
        }
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

/// Colours used for **text** in either palette (for the rendered-contrast gate).
pub fn palette_text_colors() -> Vec<egui::Color32> {
    let mut v = Vec::new();
    for p in [&DARK, &LIGHT] {
        v.extend([p.fg, p.dim, p.gold, p.on, p.onw, p.onr, p.dng]);
    }
    v
}

/// Colours used as **fills** in either palette (for the rendered-contrast gate).
pub fn palette_fill_colors() -> Vec<egui::Color32> {
    let mut v = Vec::new();
    for p in [&DARK, &LIGHT] {
        v.extend([p.bg, p.pan, p.acc, p.wd, p.wd2, p.rib, p.tex]);
    }
    v
}

/// The palette lives in the `Context` (spec: helpers paint with the installed
/// palette). Storing it in `ctx.data` keeps two contexts — e.g. two test
/// harnesses running in parallel — from ever sharing or racing on a palette.
fn palette_id() -> egui::Id {
    egui::Id::new("lootsmith.palette")
}

/// The palette installed in this `Context` (defaults to dark until [`apply`]).
pub fn active(ctx: &egui::Context) -> &'static Palette {
    match ctx.data(|d| d.get_temp::<Theme>(palette_id())) {
        Some(Theme::Light) => &LIGHT,
        _ => &DARK,
    }
}

/// Install visuals, spacing and typography for the chosen theme.
pub fn apply(ctx: &egui::Context, theme: Theme) {
    ctx.data_mut(|d| d.insert_temp(palette_id(), theme));
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
    // NB: no `override_text_color` — it would force every plain-string atom to
    // `fg`, including selected widgets' text, which must be `on` on `acc`.
    // Labels get `fg` from `widgets.*.fg_stroke` instead.

    visuals.panel_fill = p.bg;
    visuals.window_fill = p.pan;
    visuals.window_stroke = egui::Stroke::new(1.0, p.wd);
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.faint_bg_color = p.pan;
    visuals.extreme_bg_color = p.bg;
    visuals.hyperlink_color = p.acc;
    visuals.selection.bg_fill = p.acc;
    // Selected text sits on `acc`, so it must use `on` (spec §2.2), never `fg`.
    visuals.selection.stroke = egui::Stroke::new(1.0, p.on);

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
    w.active.fg_stroke = egui::Stroke::new(1.0, p.on);
    w.active.corner_radius = egui::CornerRadius::same(6);

    w.open.bg_fill = p.pan;
    w.open.weak_bg_fill = p.pan;
    w.open.bg_stroke = egui::Stroke::new(1.0, p.wd);
    w.open.fg_stroke = egui::Stroke::new(1.0, p.fg);
    w.open.corner_radius = egui::CornerRadius::same(6);

    ctx.set_visuals(visuals);
    ctx.set_fonts(font_definitions());

    ctx.all_styles_mut(|style| {
        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.button_padding = egui::vec2(10.0, 4.0);
        style.spacing.interact_size.y = 26.0;
        // Spec §2.4: 15 px body, 13 px small labels, 18–24 px headings; VT323 at
        // 19 px for numbers and item codes.
        style.text_styles.insert(
            egui::TextStyle::Heading,
            egui::FontId::new(20.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::new(15.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Small,
            egui::FontId::new(13.0, egui::FontFamily::Proportional),
        );
        style.text_styles.insert(
            egui::TextStyle::Monospace,
            egui::FontId::new(19.0, egui::FontFamily::Monospace),
        );
    });
}

/// A section heading.
pub fn section(ui: &mut egui::Ui, title: &str) {
    let gold = active(ui.ctx()).gold;
    ui.add_space(2.0);
    ui.label(egui::RichText::new(title).strong().size(15.0).color(gold));
    ui.add_space(2.0);
}

/// A dimmed secondary label.
pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    let dim = active(ui.ctx()).dim;
    ui.label(egui::RichText::new(text.into()).color(dim).size(12.0));
}

/// A faint label (paths, ids).
pub fn faint(ui: &mut egui::Ui, text: impl Into<String>) {
    let dim = active(ui.ctx()).dim;
    ui.label(egui::RichText::new(text.into()).color(dim).size(11.5));
}

/// Monospaced text for ids.
pub fn mono(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into()).monospace().size(12.0)
}

/// A modern tab. Returns true when clicked.
pub fn tab(ui: &mut egui::Ui, selected: bool, label: &str) -> bool {
    let p = active(ui.ctx());
    let text = egui::RichText::new(label).size(13.0);
    let text = if selected {
        text.strong().color(p.fg)
    } else {
        text.color(p.dim)
    };
    let response = ui.add(
        egui::Button::new(text)
            .frame(false)
            .min_size(egui::vec2(0.0, 28.0)),
    );
    let rect = response.rect;
    let underline = if selected {
        Some(p.acc)
    } else if response.hovered() {
        Some(p.wd)
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

    /// The spec's hard gate: text pairs 4.5:1, non-text 3:1, diamonds 3:1 (the
    /// white tier is expected to rely on the mandated `fg` outline). Prints the
    /// full table for the report.
    #[test]
    fn contrast_meets_wcag_in_both_themes() {
        let mut failures = Vec::new();
        for palette in [&DARK, &LIGHT] {
            println!("== {}", palette.name);
            for (pair, fg, bg) in palette.text_pairs() {
                let ratio = contrast(fg, bg);
                println!(
                    "  text     {pair:10} #{:02X}{:02X}{:02X} on #{:02X}{:02X}{:02X}  {ratio:5.2}:1  {}",
                    fg.r(), fg.g(), fg.b(), bg.r(), bg.g(), bg.b(),
                    if ratio >= 4.5 { "PASS" } else { "FAIL" }
                );
                if ratio < 4.5 {
                    failures.push(format!("{} text {pair} {ratio:.2}:1", palette.name));
                }
            }
            for (pair, fg, bg) in palette.non_text_pairs() {
                let ratio = contrast(fg, bg);
                println!(
                    "  non-text {pair:10} #{:02X}{:02X}{:02X} on #{:02X}{:02X}{:02X}  {ratio:5.2}:1  {}",
                    fg.r(), fg.g(), fg.b(), bg.r(), bg.g(), bg.b(),
                    if ratio >= 3.0 { "PASS" } else { "FAIL" }
                );
                if ratio < 3.0 {
                    failures.push(format!("{} non-text {pair} {ratio:.2}:1", palette.name));
                }
            }
            for (name, fill) in palette.diamond_pairs() {
                let ratio = contrast(fill, palette.pan);
                let ok = ratio >= 3.0 || name == "white";
                println!(
                    "  diamond  {name:10} #{:02X}{:02X}{:02X} on pan            {ratio:5.2}:1  {}",
                    fill.r(),
                    fill.g(),
                    fill.b(),
                    if ratio >= 3.0 {
                        "PASS"
                    } else if ok {
                        "OUTLINED (spec 2.3)"
                    } else {
                        "FAIL"
                    }
                );
                if !ok {
                    failures.push(format!("{} diamond {name} {ratio:.2}:1", palette.name));
                }
            }
        }
        assert!(failures.is_empty(), "contrast failures: {failures:?}");
    }

    #[test]
    fn diamond_outline_is_mandated() {
        assert_eq!(DIAMOND_OUTLINE, 2.0, "spec 2.3: 2 px fg outline");
    }

    #[test]
    fn fonts_are_registered_first_in_their_families() {
        let fonts = font_definitions();
        assert_eq!(
            fonts.families.get(&egui::FontFamily::Proportional).unwrap()[0],
            FONT_UI,
            "Pixelify Sans must lead the proportional family"
        );
        assert_eq!(
            fonts.families.get(&egui::FontFamily::Monospace).unwrap()[0],
            FONT_NUM,
            "VT323 must lead the monospace family"
        );
        assert!(fonts.font_data.contains_key(FONT_UI));
        assert!(fonts.font_data.contains_key(FONT_NUM));
        for (name, bytes) in [("pixelify", FONT_PIXELIFY), ("vt323", FONT_VT323)] {
            assert!(bytes.len() > 10_000, "{name} looks empty");
            let tag = &bytes[..4];
            assert!(
                tag == [0x00, 0x01, 0x00, 0x00] || tag == *b"true" || tag == *b"OTTO",
                "{name} is not a TrueType/OpenType font: {tag:?}"
            );
        }
    }

    #[test]
    fn active_palette_follows_the_installed_theme() {
        let ctx = egui::Context::default();
        apply(&ctx, Theme::Light);
        assert_eq!(active(&ctx).name, "Light");
        apply(&ctx, Theme::Dark);
        assert_eq!(active(&ctx).name, "Dark");

        // The palette is per-Context: two contexts never share one.
        let other = egui::Context::default();
        assert_eq!(active(&other).name, "Dark");
        apply(&other, Theme::Light);
        assert_eq!(active(&other).name, "Light");
        assert_eq!(active(&ctx).name, "Dark", "the first context is untouched");
    }
}
