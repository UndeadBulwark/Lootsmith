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

/// Install visuals, spacing and typography for the chosen theme.
pub fn apply(ctx: &egui::Context, theme: Theme) {
    let mut visuals = match theme {
        Theme::Dark => egui::Visuals::dark(),
        Theme::Light => egui::Visuals::light(),
    };

    visuals.panel_fill = Tokens::BG;
    visuals.window_fill = Tokens::SURFACE;
    visuals.window_stroke = egui::Stroke::new(1.0, Tokens::BORDER);
    visuals.window_corner_radius = egui::CornerRadius::same(8);
    visuals.menu_corner_radius = egui::CornerRadius::same(8);
    visuals.faint_bg_color = Tokens::SURFACE;
    visuals.extreme_bg_color = Tokens::BG;
    visuals.hyperlink_color = Tokens::ACCENT;
    visuals.selection.bg_fill = Tokens::SEL_FILL;
    visuals.selection.stroke = egui::Stroke::new(1.0, Tokens::ACCENT);

    let w = &mut visuals.widgets;
    w.noninteractive.bg_fill = Tokens::SURFACE;
    w.noninteractive.weak_bg_fill = Tokens::SURFACE;
    w.noninteractive.bg_stroke = egui::Stroke::new(1.0, Tokens::BORDER);
    w.noninteractive.fg_stroke = egui::Stroke::new(1.0, Tokens::TEXT);
    w.noninteractive.corner_radius = egui::CornerRadius::same(6);

    w.inactive.bg_fill = Tokens::SURFACE_ALT;
    w.inactive.weak_bg_fill = Tokens::SURFACE_ALT;
    w.inactive.bg_stroke = egui::Stroke::new(1.0, Tokens::BORDER);
    w.inactive.fg_stroke = egui::Stroke::new(1.0, Tokens::TEXT);
    w.inactive.corner_radius = egui::CornerRadius::same(6);

    w.hovered.bg_fill = Tokens::SURFACE;
    w.hovered.weak_bg_fill = Tokens::SURFACE;
    w.hovered.bg_stroke = egui::Stroke::new(1.0, Tokens::ACCENT);
    w.hovered.fg_stroke = egui::Stroke::new(1.0, Tokens::TEXT);
    w.hovered.corner_radius = egui::CornerRadius::same(6);

    w.active.bg_fill = Tokens::SEL_FILL;
    w.active.weak_bg_fill = Tokens::SEL_FILL;
    w.active.bg_stroke = egui::Stroke::new(1.0, Tokens::ACCENT);
    w.active.fg_stroke = egui::Stroke::new(1.0, Tokens::TEXT);
    w.active.corner_radius = egui::CornerRadius::same(6);

    w.open.bg_fill = Tokens::SURFACE;
    w.open.weak_bg_fill = Tokens::SURFACE;
    w.open.bg_stroke = egui::Stroke::new(1.0, Tokens::BORDER);
    w.open.fg_stroke = egui::Stroke::new(1.0, Tokens::TEXT);
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
            .color(Tokens::ACCENT),
    );
    ui.add_space(2.0);
}

/// A dimmed secondary label.
pub fn muted(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        egui::RichText::new(text.into())
            .color(Tokens::TEXT_DIM)
            .size(12.0),
    );
}

/// A faint label (paths, ids).
pub fn faint(ui: &mut egui::Ui, text: impl Into<String>) {
    ui.label(
        egui::RichText::new(text.into())
            .color(Tokens::TEXT_FAINT)
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
        text.strong().color(Tokens::TEXT)
    } else {
        text.color(Tokens::TEXT_DIM)
    };
    let response = ui.add(
        egui::Button::new(text)
            .frame(false)
            .min_size(egui::vec2(0.0, 28.0)),
    );
    let rect = response.rect;
    let underline = if selected {
        Some(Tokens::ACCENT)
    } else if response.hovered() {
        Some(Tokens::BORDER)
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
