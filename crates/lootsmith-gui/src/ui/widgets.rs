//! Shared UI components (spec §2.7) and the digit rule (§2.4).

use eframe::egui;

/// VT323 is rendered at `size * NUM_SCALE` so its cap height matches the
/// surrounding Pixelify Sans at `size`. Measured by ink height: VT323 19 px ==
/// Pixelify 15 px, so this is exactly the spec's "19 px" body-size equivalence.
/// Without it, mixed text shows digits as small, low subscripts.
pub const NUM_SCALE: f32 = 19.0 / 15.0;

pub(crate) fn num_font(size: f32) -> egui::FontId {
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
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, hover));
    crate::ui::pixel::icon(ui.painter(), response.rect.center(), kind, color, cut, 1.0);
    response
}

/// A notched checkbox: a square with a 2 px `wd` border and a drawn tick when
/// checked, plus an accessible label. Replaces the egui checkbox.
pub fn checkbox(ui: &mut egui::Ui, checked: &mut bool, label: &str) -> egui::Response {
    let p = *crate::theme::active(ui.ctx());
    let box_size = 16.0;
    let galley =
        (!label.is_empty()).then(|| ui.painter().layout_job(job_with_nums(label, p.fg, 15.0)));
    let text_w = galley.as_ref().map_or(0.0, |g| g.size().x + 6.0);
    let text_h = galley.as_ref().map_or(0.0, |g| g.size().y);
    let size = egui::vec2(box_size + text_w, box_size.max(text_h));
    let (rect, mut response) = ui.allocate_exact_size(size, egui::Sense::click());
    if response.clicked() {
        *checked = !*checked;
        response.mark_changed();
    }
    let painter = ui.painter();
    let bx = egui::Rect::from_min_size(rect.min, egui::vec2(box_size, box_size));
    painter.add(egui::Shape::convex_polygon(
        crate::ui::pixel::notched_rect(bx, 2.0, 1.0),
        p.bg,
        egui::Stroke::new(2.0, p.wd),
    ));
    if *checked {
        paint_glyph(painter, bx.center(), Glyph::Check, p.fg, 0.9);
    }
    if let Some(galley) = galley {
        painter.galley(
            egui::pos2(bx.right() + 6.0, rect.center().y - galley.size().y / 2.0),
            galley,
            p.fg,
        );
    }
    response.widget_info(|| {
        egui::WidgetInfo::selected(egui::WidgetType::Checkbox, true, *checked, label)
    });
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

// ---------------------------------------------------------------------------
// Homestead widget set (spec §2.7). The shell lays out rects; each widget paints
// itself and returns an accessible `Response`.
// ---------------------------------------------------------------------------

use crate::theme::Palette;
use egui::{pos2, Color32, FontId, Rect, Response, Sense, Stroke, WidgetInfo, WidgetType};

fn palette(ui: &egui::Ui) -> Palette {
    *crate::theme::active(ui.ctx())
}

fn ppp(ui: &egui::Ui) -> f32 {
    ui.ctx().pixels_per_point()
}

fn respond(ui: &egui::Ui, rect: Rect, tag: &str, sense: Sense) -> Response {
    ui.interact(rect, ui.id().with(tag), sense)
}

/// A notched panel: `pan` fill, `wd` border, dashed `brd` stitch, content inset.
pub fn panel<R>(ui: &mut egui::Ui, rect: Rect, add: impl FnOnce(&mut egui::Ui) -> R) -> R {
    let p = palette(ui);
    crate::ui::pixel::panel(ui.painter(), rect, ppp(ui), &p);
    let inner = rect.shrink(14.0);
    ui.scope_builder(egui::UiBuilder::new().max_rect(inner), add)
        .inner
}

/// A plank bar (top/status bars): `wd` fill, seams, gold rule.
pub fn plank_bar(ui: &egui::Ui, rect: Rect, horizontal: bool) {
    let p = palette(ui);
    crate::ui::pixel::plank_bar(ui.painter(), rect, ppp(ui), &p, horizontal);
}

/// A 3 px dotted trail divider.
pub fn trail_divider(ui: &egui::Ui, from: egui::Pos2, to: egui::Pos2) {
    let p = palette(ui);
    crate::ui::pixel::trail_divider(ui.painter(), from, to, ppp(ui), &p);
}

/// A banner ribbon with `onr` text.
pub fn ribbon(ui: &egui::Ui, rect: Rect, text: &str) -> Response {
    let p = palette(ui);
    crate::ui::pixel::ribbon(
        ui.painter(),
        rect,
        ppp(ui),
        &p,
        "",
        FontId::proportional(15.0),
    );
    let galley = ui.painter().layout_job(job_with_nums(text, p.onr, 15.0));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, p.onr);
    let response = respond(ui, rect, &format!("ribbon-{text}"), Sense::hover());
    // The ribbon repeats the screen title; keep its accessible label distinct from
    // the sidebar row so a query for the tab label matches only the sidebar.
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, format!("{text} header")));
    response
}

/// Button style.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ButtonKind {
    Normal,
    Primary,
    Danger,
}

/// A notched button (`Normal` pan, `Primary` acc, `Danger` destructive text).
pub fn button(ui: &egui::Ui, rect: Rect, label: &str, kind: ButtonKind) -> Response {
    let p = palette(ui);
    let (fill, border, text) = match kind {
        ButtonKind::Normal => (p.pan, p.wd, p.fg),
        ButtonKind::Primary => (p.acc, p.gold, p.on),
        ButtonKind::Danger => (p.pan, p.wd, p.dng),
    };
    let points = crate::ui::pixel::notched_rect(rect, 4.0, ppp(ui));
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        fill,
        Stroke::new(2.0, border),
    ));
    let galley = ui.painter().layout_job(job_with_nums(label, text, 15.0));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, text);
    let response = respond(ui, rect, &format!("button-{label}"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, label));
    response
}

/// A small status/label chip.
pub fn chip(ui: &egui::Ui, rect: Rect, text: &str, color: Color32) -> Response {
    let p = palette(ui);
    let points = crate::ui::pixel::notched_rect(rect, 3.0, ppp(ui));
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        p.pan,
        Stroke::new(1.0, color),
    ));
    let galley = ui.painter().layout_job(job_with_nums(text, color, 13.0));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);
    let response = respond(ui, rect, &format!("chip-{text}"), Sense::hover());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    response
}

/// Middle-elide `text` until its **real** rendered width fits `max_w`, measured
/// from the actual font galley — never a px-per-char estimate (the char width
/// varies with the glyphs and the scale).
pub fn fit_text(ui: &egui::Ui, text: &str, max_w: f32, size: f32) -> String {
    let width = |s: &str| {
        ui.painter()
            .layout_job(job_with_nums(s, Color32::WHITE, size))
            .size()
            .x
    };
    if width(text) <= max_w {
        return text.to_string();
    }
    let mut n = text.chars().count();
    while n > 5 {
        n -= 1;
        let cand = crate::ui::item_table::elide_middle(text, n);
        if width(&cand) <= max_w {
            return cand;
        }
    }
    crate::ui::item_table::elide_middle(text, 5)
}

/// A rarity diamond (14 px) with the mandatory outline.
pub fn gem_rarity(ui: &egui::Ui, center: egui::Pos2, fill: Color32) {
    let p = palette(ui);
    crate::ui::pixel::diamond(ui.painter(), center, 14.0, fill, ppp(ui), &p);
}

/// A stat block: a small caption above a large VT323 value.
pub fn stat_block(ui: &egui::Ui, rect: Rect, label: &str, value: &str) {
    let p = palette(ui);
    ui.painter().text(
        rect.left_top() + egui::vec2(0.0, 2.0),
        egui::Align2::LEFT_TOP,
        label,
        FontId::proportional(13.0),
        p.gold,
    );
    ui.painter().text(
        rect.left_bottom() - egui::vec2(0.0, 2.0),
        egui::Align2::LEFT_BOTTOM,
        value,
        num_font(20.0),
        p.fg,
    );
}

/// A part row: a label on the left and (in the shell) a dropdown on the right.
pub fn part_row(ui: &egui::Ui, rect: Rect, label: &str, value: &str) -> Response {
    let p = palette(ui);
    ui.painter().text(
        rect.left_center(),
        egui::Align2::LEFT_CENTER,
        label,
        FontId::proportional(14.0),
        p.dim,
    );
    ui.painter().text(
        rect.right_center() - egui::vec2(20.0, 0.0),
        egui::Align2::RIGHT_CENTER,
        value,
        FontId::proportional(14.0),
        p.fg,
    );
    crate::ui::pixel::trail_divider(
        ui.painter(),
        pos2(rect.left(), rect.bottom()),
        pos2(rect.right(), rect.bottom()),
        ppp(ui),
        &p,
    );
    let response = respond(ui, rect, &format!("part-{label}"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, label));
    response
}

/// A single-line text field with a hint (identifier fields use VT323).
pub fn text_field(
    ui: &mut egui::Ui,
    id: &str,
    text: &mut String,
    hint: &str,
    width: f32,
) -> Response {
    let response = ui.add_sized(
        [width, 24.0],
        egui::TextEdit::singleline(text).id_salt(id).hint_text(hint),
    );
    response
}

/// A numeric field (VT323) with an accessible label.
pub fn number_field(ui: &mut egui::Ui, id: &str, value: &mut i32, width: f32) -> Response {
    let response = num_scope(ui, 15.0, |ui| {
        ui.add_sized([width, 24.0], egui::DragValue::new(value).speed(0.2))
    });
    response.widget_info(|| WidgetInfo::labeled(WidgetType::DragValue, true, id));
    response
}

/// A search field (label font, per spec §2.4).
pub fn search_field(ui: &mut egui::Ui, id: &str, text: &mut String, width: f32) -> Response {
    ui.add_sized(
        [width, 24.0],
        egui::TextEdit::singleline(text)
            .id_salt(id)
            .hint_text("filter items…"),
    )
}

/// A notched dropdown. Returns the newly chosen index, if any.
pub fn dropdown(ui: &mut egui::Ui, id: &str, current: &str, options: &[String]) -> Option<usize> {
    let p = palette(ui);
    let mut chosen = None;
    egui::ComboBox::from_id_salt(id)
        .selected_text(text_with_nums(current, p.fg, 15.0))
        .width(270.0)
        .show_ui(ui, |ui| {
            for (i, option) in options.iter().enumerate() {
                if ui.selectable_label(option == current, option).clicked() {
                    chosen = Some(i);
                }
            }
        });
    chosen
}

/// A sidebar navigation row with an active block and an optional count badge.
pub fn nav_row(
    ui: &egui::Ui,
    rect: Rect,
    label: &str,
    active: bool,
    count: Option<usize>,
) -> Response {
    let p = palette(ui);
    let response = respond(ui, rect, &format!("nav-{label}"), Sense::click());
    let hovered = response.hovered();
    if active {
        ui.painter().rect_filled(rect, 0.0, p.acc);
    }
    let color = if active { p.on } else { p.fg };
    let galley = ui.painter().layout_job(job_with_nums(label, color, 15.0));
    let lh = galley.size().y;
    ui.painter().galley(
        pos2(rect.left() + 10.0, rect.center().y - lh / 2.0),
        galley,
        color,
    );
    if let Some(n) = count {
        ui.painter().text(
            rect.right_center() - egui::vec2(10.0, 0.0),
            egui::Align2::RIGHT_CENTER,
            n.to_string(),
            num_font(15.0),
            color,
        );
    }
    let _ = hovered;
    let a11y = match count {
        Some(n) => format!("{label} ({n})"),
        None => label.to_string(),
    };
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, active, a11y.as_str()));
    response
}

/// A status plank chip: a glyph plus text, ok or missing.
pub fn status_chip(ui: &egui::Ui, rect: Rect, text: &str, ok: bool) -> Response {
    let p = palette(ui);
    let points = crate::ui::pixel::notched_rect(rect, 3.0, ppp(ui));
    ui.painter().add(egui::Shape::convex_polygon(
        points,
        p.pan,
        Stroke::new(1.0, p.wd),
    ));
    let glyph = if ok { Glyph::Check } else { Glyph::Cross };
    let color = if ok { p.gold } else { p.dng };
    paint_glyph(
        ui.painter(),
        pos2(rect.left() + 12.0, rect.center().y),
        glyph,
        color,
        0.9,
    );
    ui.painter().text(
        pos2(rect.left() + 24.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        text,
        FontId::proportional(12.0),
        p.fg,
    );
    let response = respond(ui, rect, &format!("status-{text}"), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, text));
    response
}

/// Style the controls drawn on a wood plank (the top and status bars): cream
/// `onw` text with flat, transparent control fills so text sits on the wood.
pub fn wood_style(ui: &mut egui::Ui, p: &Palette) {
    let mut style = (**ui.style()).clone();
    let stroke = Stroke::new(1.0, p.onw);
    style.visuals.widgets.noninteractive.fg_stroke = stroke;
    style.visuals.widgets.inactive.fg_stroke = stroke;
    style.visuals.widgets.hovered.fg_stroke = stroke;
    style.visuals.widgets.active.fg_stroke = stroke;
    style.visuals.widgets.open.fg_stroke = stroke;
    style.visuals.widgets.noninteractive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.inactive.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.hovered.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.active.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.widgets.open.weak_bg_fill = Color32::TRANSPARENT;
    style.visuals.override_text_color = Some(p.onw);
    // Selected controls (the scale chips) use the accent fill with `on` text.
    style.visuals.selection.bg_fill = p.acc;
    style.visuals.selection.stroke = Stroke::new(1.0, p.on);
    ui.set_style(style);
}

/// A small toggle chip (the UI-scale options): `acc` fill with `on` text when
/// selected, otherwise flat `onw` text on whatever is behind it. Keeps digits
/// in the VT323 number font.
pub fn toggle_chip(ui: &egui::Ui, rect: Rect, label: &str, selected: bool) -> Response {
    let p = palette(ui);
    let response = respond(ui, rect, &format!("toggle-{label}"), Sense::click());
    if selected {
        ui.painter().rect_filled(rect, 0.0, p.acc);
    }
    let color = if selected { p.on } else { p.onw };
    let galley = ui.painter().layout_job(job_with_nums(label, color, 15.0));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, color);
    response.widget_info(|| WidgetInfo::selected(WidgetType::Button, true, selected, label));
    response
}
