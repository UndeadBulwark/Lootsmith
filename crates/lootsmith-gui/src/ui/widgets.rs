//! Shared UI components (spec §2.7). First entry: [`num`], the numeric font.

use eframe::egui;

/// Numeric text: **VT323** (the monospace family), 19 px. Use this for **every
/// digit run** — ids, counts, levels, ranks, quick-slot and slot counts — so
/// digits never render in the label font (Pixelify Sans' `5` reads as `S`).
/// Spec §2.4.
pub fn num(text: impl Into<String>) -> egui::RichText {
    egui::RichText::new(text.into()).monospace().size(19.0)
}

/// A label whose trailing count is set in the numeric font, e.g.
/// `Backpack (5)` — the word in Pixelify Sans, the count in VT323.
pub fn counted_label(
    name: &str,
    count: usize,
    color: egui::Color32,
    size: f32,
) -> egui::WidgetText {
    let mut job = egui::text::LayoutJob::default();
    job.append(
        name,
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::proportional(size),
            color,
            ..Default::default()
        },
    );
    job.append(
        &format!(" ({count})"),
        0.0,
        egui::TextFormat {
            font_id: egui::FontId::monospace(size),
            color,
            ..Default::default()
        },
    );
    egui::WidgetText::LayoutJob(std::sync::Arc::new(job))
}
