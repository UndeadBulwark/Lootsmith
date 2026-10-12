//! The mockup's item list (spec §2.2 item 4): rarity diamond, name, type, level,
//! mark; 40 px rows, dotted row dividers, a solid `acc` selected row and a gold
//! hover border. Replaces the `egui_extras` table in `tabs/backpack.rs`.
use crate::theme::Palette;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, FontId, Sense, Stroke, StrokeKind, Ui, WidgetInfo,
    WidgetType,
};
use lootsmith_core::Rarity;

pub const ROW_H: f32 = 40.0;
pub const HEADER_H: f32 = 28.0;

/// Spec §2.3 colours. `Seraph`/`Unknown` are not in the spec table: pink and a
/// neutral grey are used (Unknown is never a guess).
pub fn rarity_color(r: Rarity) -> Color32 {
    let c = |h: u32| Color32::from_rgb((h >> 16) as u8, (h >> 8) as u8, h as u8);
    match r {
        Rarity::Common => c(0xD8DAE0),
        Rarity::Uncommon => c(0x5ED08A),
        Rarity::Rare => c(0x4C9BFF),
        Rarity::VeryRare => c(0xB06CFF),
        Rarity::Etech => c(0xFF4FA3),
        Rarity::Legendary => c(0xFF8A1F),
        Rarity::Seraph => c(0xFF6FD8),
        Rarity::Pearlescent => c(0x3DE0D0),
        Rarity::Unknown => c(0x8A8F99),
    }
}

/// One row of the list.
#[derive(Clone)]
pub struct ItemRow {
    pub name: String,
    pub kind: String,
    pub id: i32,
    pub level: i32,
    /// `PLAYER_MARKS` index: 0 = Trash, 1 = Standard, 2 = Favorite.
    pub mark: i32,
    pub rarity: Rarity,
}

/// Draws the list; returns the index clicked this frame.
pub fn item_table(
    ui: &mut Ui,
    p: &Palette,
    rows: &[ItemRow],
    selected: Option<usize>,
) -> Option<usize> {
    let ppp = ui.ctx().pixels_per_point();
    let w = ui.available_width();
    let mut clicked = None;

    let (x_name, x_type, x_lvl, x_mark) = (44.0, w * 0.52, w * 0.74, w * 0.88);

    // Header
    let (hr, _) = ui.allocate_exact_size(vec2(w, HEADER_H), Sense::hover());
    for (x, t) in [
        (x_name, "NAME"),
        (x_type, "TYPE"),
        (x_lvl, "LVL"),
        (x_mark, "MARK"),
    ] {
        ui.painter().text(
            pos2(hr.left() + x, hr.center().y),
            Align2::LEFT_CENTER,
            t,
            FontId::proportional(13.0),
            p.gold,
        );
    }
    crate::ui::pixel::trail_divider(ui.painter(), hr.left_bottom(), hr.right_bottom(), ppp, p);

    egui::ScrollArea::vertical()
        .id_salt("item_table")
        .auto_shrink([false, false])
        .show_rows(ui, ROW_H, rows.len(), |ui, range| {
            for i in range {
                let r = &rows[i];
                let (rect, resp) =
                    ui.allocate_exact_size(vec2(ui.available_width(), ROW_H), Sense::click());
                let sel = selected == Some(i);
                let (text, sub) = if sel { (p.on, p.on) } else { (p.fg, p.dim) };
                if sel {
                    ui.painter().rect_filled(rect, 0.0, p.acc);
                }
                // Rarity diamond (14 px centre 22 px in; mandatory 2 px fg outline).
                crate::ui::pixel::diamond(
                    ui.painter(),
                    pos2(rect.left() + 22.0, rect.center().y),
                    14.0,
                    rarity_color(r.rarity),
                    ppp,
                    p,
                );
                let cy = rect.center().y;
                let max_name = ((x_type - x_name) / 8.0) as usize;
                let name = elide_middle(&r.name, max_name);
                let galley = ui
                    .painter()
                    .layout_job(crate::ui::widgets::job_with_nums(&name, text, 15.0));
                ui.painter().galley(
                    pos2(rect.left() + x_name, cy - galley.size().y / 2.0),
                    galley,
                    text,
                );
                ui.painter().text(
                    pos2(rect.left() + x_type, cy),
                    Align2::LEFT_CENTER,
                    &r.kind,
                    FontId::proportional(15.0),
                    sub,
                );
                ui.painter().text(
                    pos2(rect.left() + x_lvl, cy),
                    Align2::LEFT_CENTER,
                    r.level.to_string(),
                    crate::ui::widgets::num_font(15.0),
                    text,
                );
                // Mark: the mockup shows a diamond for a real favourite and a dim
                // dot for Standard/Trash (never blank, never a diamond on every row).
                let mark_c = pos2(rect.left() + x_mark + 6.0, cy);
                match r.mark {
                    2 => {
                        crate::ui::pixel::diamond(ui.painter(), mark_c, 10.0, text, ppp, p);
                    }
                    0 | 1 => {
                        ui.painter().circle_filled(
                            crate::ui::pixel::snap_pos(mark_c, ppp),
                            3.0,
                            p.dim,
                        );
                    }
                    _ => {}
                }
                if !sel {
                    crate::ui::pixel::trail_divider(
                        ui.painter(),
                        rect.left_bottom(),
                        rect.right_bottom(),
                        ppp,
                        p,
                    );
                }
                if resp.hovered() && !sel {
                    ui.painter().rect_stroke(
                        rect.shrink(1.0),
                        0.0,
                        Stroke::new(2.0, p.gold),
                        StrokeKind::Inside,
                    );
                }
                if resp.has_focus() {
                    ui.painter().rect_stroke(
                        rect.shrink(1.0),
                        0.0,
                        Stroke::new(2.0, p.acc),
                        StrokeKind::Inside,
                    );
                }
                // Colour is never the only signal: the label carries the rarity text.
                resp.widget_info(|| {
                    WidgetInfo::labeled(
                        WidgetType::Button,
                        true,
                        format!(
                            "{} {} id={} level {} {}",
                            r.name,
                            r.kind,
                            r.id,
                            r.level,
                            r.rarity.label()
                        ),
                    )
                });
                if resp.clicked() {
                    clicked = Some(i);
                }
            }
        });
    clicked
}

/// Middle-elide `s` to at most `max_chars` characters, keeping both ends.
pub fn elide_middle(s: &str, max_chars: usize) -> String {
    let n = s.chars().count();
    if n <= max_chars || max_chars < 5 {
        return s.to_string();
    }
    let keep = max_chars - 3; // "..."
    let left = keep.div_ceil(2);
    let right = keep - left;
    let head: String = s.chars().take(left).collect();
    let tail: String = s.chars().skip(n - right).collect();
    format!("{head}...{tail}")
}

/// A readable fallback name from a balance path: the last dotted segment with
/// underscores turned into spaces.
pub fn balance_to_name(balance: &str) -> String {
    let last = balance.rsplit('.').next().unwrap_or(balance);
    let mut parts: Vec<&str> = last.split('_').collect();
    // Drop a trailing rarity word so the distinguishing part survives elision.
    if let Some(t) = parts.last() {
        if matches!(
            t.to_ascii_lowercase().as_str(),
            "common"
                | "uncommon"
                | "rare"
                | "veryrare"
                | "legendary"
                | "pearlescent"
                | "seraph"
                | "etech"
        ) {
            parts.pop();
        }
    }
    parts.join(" ")
}
