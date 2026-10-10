//! Shared UI components (spec §2.7) and the digit rule (§2.4).

use eframe::egui;

/// VT323 is rendered at `size * NUM_SCALE` so its cap height matches the
/// surrounding Pixelify Sans at `size`. Measured by ink height: VT323 19 px ==
/// Pixelify 15 px, so this is exactly the spec's "19 px" body-size equivalence.
/// Without it, mixed text shows digits as small, low subscripts.
pub const NUM_SCALE: f32 = 19.0 / 15.0;

fn num_font(size: f32) -> egui::FontId {
    egui::FontId::monospace(size * NUM_SCALE)
}

/// Numeric text: **VT323** at the cap height of `size` in the label font.
pub fn num(text: impl Into<String>, size: f32) -> egui::RichText {
    egui::RichText::new(text.into()).font(num_font(size))
}

/// A layout with every **digit run** in VT323 and everything else in the label
/// font, so mixed strings (paths, `id=400`, status lines) stay compliant.
pub fn job_with_nums(text: &str, color: egui::Color32, size: f32) -> egui::text::LayoutJob {
    let mut job = egui::text::LayoutJob::default();
    let mut run = String::new();
    let mut run_digit = false;
    let flush = |job: &mut egui::text::LayoutJob, run: &mut String, digit: bool| {
        if run.is_empty() {
            return;
        }
        let font_id = if digit {
            num_font(size)
        } else {
            egui::FontId::proportional(size)
        };
        job.append(
            run,
            0.0,
            egui::TextFormat {
                font_id,
                color,
                ..Default::default()
            },
        );
        run.clear();
    };
    for ch in text.chars() {
        let digit = ch.is_ascii_digit();
        if !run.is_empty() && digit != run_digit {
            flush(&mut job, &mut run, run_digit);
        }
        run.push(ch);
        run_digit = digit;
    }
    flush(&mut job, &mut run, run_digit);
    job
}

/// [`job_with_nums`] as a widget text.
pub fn text_with_nums(text: &str, color: egui::Color32, size: f32) -> egui::WidgetText {
    egui::WidgetText::LayoutJob(std::sync::Arc::new(job_with_nums(text, color, size)))
}

/// A label whose trailing count uses VT323, e.g. `Backpack (5)`.
pub fn counted_label(
    name: &str,
    count: usize,
    color: egui::Color32,
    size: f32,
) -> egui::WidgetText {
    text_with_nums(&format!("{name} ({count})"), color, size)
}

/// A label with every digit run in VT323 ([`text_with_nums`]).
pub fn label_nums(
    ui: &mut egui::Ui,
    text: impl AsRef<str>,
    color: egui::Color32,
    size: f32,
) -> egui::Response {
    ui.add(egui::Label::new(text_with_nums(text.as_ref(), color, size)))
}

/// A pure-numeric label in VT323 ([`num`]).
pub fn label_num(
    ui: &mut egui::Ui,
    text: impl Into<String>,
    color: egui::Color32,
    size: f32,
) -> egui::Response {
    ui.label(num(text, size).color(color))
}

/// A distinct status glyph, so state is readable by shape as well as colour.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Glyph {
    /// OK / loaded.
    Check,
    /// Missing / error.
    Cross,
    /// Unsaved / dirty.
    Dot,
    /// Inactive / neutral.
    Ring,
}

/// Paint a [`Glyph`] centred on `center` (about 12x12 px at `scale` 1.0).
pub fn paint_glyph(
    painter: &egui::Painter,
    center: egui::Pos2,
    glyph: Glyph,
    color: egui::Color32,
    scale: f32,
) {
    let width = (2.0 * scale).round().max(1.0);
    let stroke = egui::Stroke::new(width, color);
    match glyph {
        Glyph::Dot => {
            painter.circle_filled(center, 3.2 * scale, color);
        }
        Glyph::Ring => {
            painter.circle_stroke(center, 3.2 * scale, stroke);
        }
        Glyph::Check => {
            painter.add(egui::Shape::line(
                vec![
                    center + egui::vec2(-4.0 * scale, 0.5 * scale),
                    center + egui::vec2(-1.2 * scale, 3.4 * scale),
                    center + egui::vec2(4.0 * scale, -3.6 * scale),
                ],
                stroke,
            ));
        }
        Glyph::Cross => {
            let s = 3.4 * scale;
            painter.line_segment(
                [center + egui::vec2(-s, -s), center + egui::vec2(s, s)],
                stroke,
            );
            painter.line_segment(
                [center + egui::vec2(-s, s), center + egui::vec2(s, -s)],
                stroke,
            );
        }
    }
}

/// A square button that draws an [`Icon`](crate::ui::pixel::Icon) instead of a
/// font symbol (no tofu).
pub fn icon_button(
    ui: &mut egui::Ui,
    kind: crate::ui::pixel::Icon,
    color: egui::Color32,
    cut: egui::Color32,
    size: f32,
    hover: &str,
) -> egui::Response {
    let response = ui
        .add_sized([size, size], egui::Button::new(""))
        .on_hover_text(hover);
    crate::ui::pixel::icon(ui.painter(), response.rect.center(), kind, color, cut, 1.0);
    response
}

/// Run `add` with the body/button font set to VT323 at `size`, for pure-number
/// input widgets (`DragValue`, numeric `TextEdit`) that pick their font from the
/// style.
pub fn num_scope<R>(ui: &mut egui::Ui, size: f32, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let mut style: egui::Style = (**ui.style()).clone();
    style
        .text_styles
        .insert(egui::TextStyle::Body, num_font(size));
    style
        .text_styles
        .insert(egui::TextStyle::Button, num_font(size));
    ui.scope(|ui| {
        ui.set_style(style);
        add(ui)
    })
    .inner
}
