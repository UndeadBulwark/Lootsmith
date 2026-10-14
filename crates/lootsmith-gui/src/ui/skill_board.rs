//! Homestead skill board. Draws only; every decision comes from
//! [`crate::skill_rules`]. Tile columns come from the generated `col` data
//! (`crate::skills`), so tiers match the in-game trees instead of a count rule.

use crate::skill_rules::{self as rules, Deny, Tree};
use crate::theme::Palette;
use eframe::egui::{
    self, pos2, vec2, Align2, Color32, Rect, Sense, Stroke, StrokeKind, TextureHandle, Ui,
};

pub const TILE: f32 = 64.0;
pub const GAP: f32 = 8.0;

/// UV covering a whole texture (no atlas).
fn full_uv() -> Rect {
    Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0))
}

/// What the board asks the app to do. The app applies it (grade change + apply_skills()).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Act {
    /// skill index into app.skills (None when the skill is missing from the save), new grade
    Set(usize, i32),
    Respec,
    /// Show a refusal reason (e.g. clearing the action point while tier 1 holds ranks).
    Status(String),
}

pub struct NodeView<'a> {
    pub name: &'a str,
    pub desc: &'a str,
    pub icon: Option<&'a TextureHandle>,
    pub col: Option<u32>,
    pub save_idx: Option<usize>, // index in app.skills, None => not in this save
}

pub struct TreeView<'a> {
    pub name: &'a str,
    pub accent: Color32, // marker diamond only; text always uses palette colours
    pub model: Tree,     // current grades
    pub nodes: Vec<Vec<NodeView<'a>>>, // same shape as model.tiers
}

/// Panel width at scale `k` (1.0 = 3 tiles + 2 gaps + 14 px padding each side).
pub fn panel_w(k: f32) -> f32 {
    k * (3.0 * TILE + 2.0 * GAP + 28.0)
}

/// Panel height for a tree with `tiers` tiers at scale `k`.
pub fn panel_h(tiers: usize, k: f32) -> f32 {
    k * (44.0 /*ribbon*/ + tiers as f32 * (TILE + GAP + 18.0) + 20.0)
}

/// Draw one tree panel into an explicit `rect` at scale `k` (1.0 = native).
/// The caller owns placement so a fixed-width, three-column panel can never be
/// squeezed by the surrounding layout. Returns an action and the (tier,node)
/// hovered/focused for the details pane.
#[allow(clippy::too_many_arguments)]
pub fn draw_tree(
    ui: &mut Ui,
    p: &Palette,
    rect: Rect,
    t: &TreeView,
    available: i32,
    enforce: bool,
    action_ready: bool,
    k: f32,
) -> (Option<Act>, Option<(usize, usize)>) {
    let ppp = ui.ctx().pixels_per_point();
    let w = rect.width();
    let d = |v: f32| v * k;
    let tile_sz = k * TILE;
    let gap = k * GAP;
    let mut act = None;
    let mut focus = None;
    let total = rules::total(&t.model);
    crate::ui::pixel::panel(ui.painter(), rect, ppp, p);

    // Ribbon title: tree name + marker diamond + points (VT323).
    let rib = Rect::from_min_size(
        rect.min + vec2(d(14.0), d(14.0)),
        vec2(w - d(28.0), d(28.0)),
    );
    crate::ui::widgets::ribbon(ui, rib, &format!("{}  {}", t.name, total));
    crate::ui::pixel::diamond(
        ui.painter(),
        pos2(rib.left() + d(22.0), rib.center().y),
        d(12.0),
        t.accent,
        ppp,
        p,
    );

    let mut y = rib.bottom() + d(12.0);
    for (ti, tier) in t.model.tiers.iter().enumerate() {
        let open = rules::tier_open(total, tier.level, ti, action_ready);
        // Tier label on a trail divider: "TIER 2 . needs 5" (text, not only colour)
        let label = if open {
            format!("TIER {}", ti + 1)
        } else {
            format!(
                "TIER {}  LOCKED  NEEDS {}",
                ti + 1,
                rules::needed(tier.level)
            )
        };
        // Digits in the label must use the num font (the digit gate).
        let tier_color = if open { p.gold } else { p.dim };
        let galley = ui.painter().layout_job(crate::ui::widgets::job_with_nums(
            &label,
            tier_color,
            d(11.0),
        ));
        ui.painter()
            .galley(pos2(rect.left() + d(16.0), y), galley, tier_color);
        y += d(14.0);
        let cols: Vec<Option<u32>> = t.nodes[ti].iter().map(|n| n.col).collect();
        let slots = rules::slots(&cols);
        for (ni, node) in tier.nodes.iter().enumerate() {
            let col = slots.get(ni).copied().unwrap_or(ni) as f32;
            let r = Rect::from_min_size(
                pos2(rect.left() + d(14.0) + col * (tile_sz + gap), y),
                vec2(tile_sz, tile_sz),
            );
            let view = &t.nodes[ti][ni];
            let id = ui.make_persistent_id(("skill_tile", t.name, ti, ni));
            let resp = tile(ui, p, id, r, view, node.grade, node.max, open, ppp, k);
            if resp.hovered() || resp.has_focus() {
                focus = Some((ti, ni));
            }
            if let Some(idx) = view.save_idx {
                let shift = ui.input(|i| i.modifiers.shift);
                let add = resp.clicked()
                    || (resp.has_focus()
                        && ui.input(|i| {
                            i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::Space)
                        }));
                let rem = resp.secondary_clicked()
                    || (resp.has_focus()
                        && ui.input(|i| {
                            i.key_pressed(egui::Key::Backspace) || i.key_pressed(egui::Key::Delete)
                        }));
                if add {
                    if shift {
                        // fill as far as the rules allow
                        let mut m = t.model.clone();
                        let mut avail = available;
                        while rules::can_add(&m, ti, ni, avail, enforce, action_ready).is_ok() {
                            m.tiers[ti].nodes[ni].grade += 1;
                            avail -= 1;
                        }
                        act = Some(Act::Set(idx, m.tiers[ti].nodes[ni].grade));
                    } else if rules::can_add(&t.model, ti, ni, available, enforce, action_ready)
                        .is_ok()
                    {
                        act = Some(Act::Set(idx, node.grade + 1));
                    }
                } else if rem {
                    if shift {
                        let mut m = t.model.clone();
                        while rules::can_remove(&m, ti, ni).is_ok() {
                            m.tiers[ti].nodes[ni].grade -= 1;
                        }
                        act = Some(Act::Set(idx, m.tiers[ti].nodes[ni].grade));
                    } else if rules::can_remove(&t.model, ti, ni).is_ok() {
                        act = Some(Act::Set(idx, node.grade - 1));
                    }
                }
            }
        }
        y += tile_sz + gap;
    }
    (act, focus)
}

#[allow(clippy::too_many_arguments)]
fn tile(
    ui: &mut Ui,
    p: &Palette,
    id: egui::Id,
    r: Rect,
    v: &NodeView,
    grade: i32,
    max: i32,
    open: bool,
    ppp: f32,
    k: f32,
) -> egui::Response {
    let d = |val: f32| val * k;
    let stroke = |width: f32| crate::ui::pixel::snap_len(d(width), ppp).max(1.0);
    // `interact` (not `allocate_rect`): the panel's layout is owned by the
    // caller, so the tile must not advance the layout cursor.
    let resp = ui.interact(r, id, Sense::click());
    let maxed = max > 1 && grade >= max;
    let border = if maxed {
        p.gold
    } else if grade > 0 {
        p.acc
    } else {
        p.wdb
    };
    ui.painter().add(egui::Shape::convex_polygon(
        crate::ui::pixel::notched_rect(r, d(4.0), ppp),
        p.pan,
        Stroke::new(stroke(2.0), border),
    ));
    if let Some(tex) = v.icon {
        let tint = if open {
            Color32::WHITE
        } else {
            Color32::from_gray(120)
        };
        ui.painter()
            .image(tex.id(), r.shrink(d(8.0)), full_uv(), tint);
    }
    if !open {
        // flat 45deg lines = locked
        crate::ui::pixel::hatch(
            &ui.painter().with_clip_rect(r.shrink(d(2.0))),
            r.shrink(d(2.0)),
            ppp,
            p,
        );
        lock_glyph(ui, p, r.left_top() + vec2(d(12.0), d(12.0)), k, ppp);
    }
    // Rank badge: boxed VT323 "3/5". Colour never the only signal: text always shown.
    let b = Rect::from_min_size(
        r.right_bottom() - vec2(d(36.0), d(20.0)),
        vec2(d(34.0), d(18.0)),
    );
    ui.painter().rect_filled(b, 0.0, p.pan);
    ui.painter()
        .rect_stroke(b, 0.0, Stroke::new(stroke(1.0), border), StrokeKind::Inside);
    ui.painter().text(
        b.center(),
        Align2::CENTER_CENTER,
        format!("{grade}/{max}"),
        crate::ui::widgets::num_font(d(17.0)),
        if maxed { p.gold } else { p.fg },
    );
    if resp.hovered() {
        ui.painter().rect_stroke(
            r,
            0.0,
            Stroke::new(stroke(2.0), p.gold),
            StrokeKind::Outside,
        );
    }
    if resp.has_focus() {
        ui.painter().rect_stroke(
            r.expand(d(2.0)),
            0.0,
            Stroke::new(stroke(2.0), p.acc),
            StrokeKind::Outside,
        );
    }
    let label = format!(
        "{}, rank {} of {}{}",
        v.name,
        grade,
        max,
        if open { "" } else { ", locked" }
    );
    resp.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Button, true, label.clone()));
    resp
}

/// The action-skill banner: a wide 0/1 skill drawn into `rect` (the caller
/// centres it over the trees). Left-click sets it, right-click clears — unless
/// tier 1 still holds points, in which case clearing is refused.
#[allow(clippy::too_many_arguments)]
pub fn action_banner(
    ui: &mut Ui,
    p: &Palette,
    rect: Rect,
    name: &str,
    icon: Option<&TextureHandle>,
    save_idx: Option<usize>,
    grade: i32,
    max: i32,
    clear_ok: bool,
) -> Option<Act> {
    let ppp = ui.ctx().pixels_per_point();
    crate::ui::pixel::panel(ui.painter(), rect, ppp, p);
    let resp = ui.interact(rect, ui.id().with("action_banner"), Sense::click());

    // The wide icon keeps its 3:1 aspect; cap it so the name and badge fit.
    let ih = 48.0_f32.min(rect.height() - 16.0).max(24.0);
    let iw = ih * 3.0;
    let icon_rect = Rect::from_min_size(
        pos2(rect.left() + 14.0, rect.center().y - ih / 2.0),
        vec2(iw, ih),
    );
    if let Some(tex) = icon {
        ui.painter()
            .image(tex.id(), icon_rect, full_uv(), Color32::WHITE);
    } else {
        ui.painter().rect_filled(icon_rect, 0.0, p.pan);
        ui.painter()
            .rect_stroke(icon_rect, 0.0, Stroke::new(2.0, p.wdb), StrokeKind::Inside);
    }
    let tx = icon_rect.right() + 14.0;
    // Leave room for the rank badge on the right.
    let name_max = (rect.right() - 58.0 - tx).max(40.0);
    let mut nj = crate::ui::widgets::job_with_nums(name, p.fg, 18.0);
    nj.wrap.max_width = name_max;
    let ng = ui.painter().layout_job(nj);
    ui.painter().galley(
        pos2(tx, rect.center().y - 11.0 - ng.size().y / 2.0),
        ng,
        p.fg,
    );
    let mut sj = crate::ui::widgets::job_with_nums("ACTION SKILL", p.gold, 11.0);
    sj.wrap.max_width = name_max;
    let sg = ui.painter().layout_job(sj);
    ui.painter().galley(
        pos2(tx, rect.center().y + 11.0 - sg.size().y / 2.0),
        sg,
        p.gold,
    );

    // Rank badge "0/1".
    let b = Rect::from_min_size(
        pos2(rect.right() - 56.0, rect.center().y - 11.0),
        vec2(42.0, 22.0),
    );
    ui.painter().rect_filled(b, 0.0, p.pan);
    ui.painter().rect_stroke(
        b,
        0.0,
        Stroke::new(1.0, if grade > 0 { p.gold } else { p.wdb }),
        StrokeKind::Inside,
    );
    ui.painter().text(
        b.center(),
        Align2::CENTER_CENTER,
        format!("{grade}/{max}"),
        crate::ui::widgets::num_font(19.0),
        if grade > 0 { p.gold } else { p.fg },
    );

    if resp.hovered() {
        ui.painter()
            .rect_stroke(rect, 0.0, Stroke::new(2.0, p.gold), StrokeKind::Outside);
    }
    resp.widget_info(|| {
        egui::WidgetInfo::labeled(
            egui::WidgetType::Button,
            save_idx.is_some(),
            format!("{name} action skill, rank {grade} of {max}"),
        )
    });

    let idx = save_idx?;
    if resp.clicked() && grade < max {
        Some(Act::Set(idx, grade + 1))
    } else if resp.secondary_clicked() && grade > 0 {
        if clear_ok {
            Some(Act::Set(idx, grade - 1))
        } else {
            Some(Act::Status(
                "Can't remove: tier 1 still has points in this class".into(),
            ))
        }
    } else {
        None
    }
}

/// Drawn padlock (no font symbol; U+1F512 is absent from both fonts).
fn lock_glyph(ui: &Ui, p: &Palette, c: egui::Pos2, k: f32, ppp: f32) {
    let pt = ui.painter();
    pt.rect_filled(
        Rect::from_center_size(c + vec2(0.0, 3.0 * k), vec2(10.0 * k, 8.0 * k)),
        0.0,
        p.fg,
    );
    pt.rect_stroke(
        Rect::from_center_size(c + vec2(0.0, -3.0 * k), vec2(6.0 * k, 6.0 * k)),
        0.0,
        Stroke::new(crate::ui::pixel::snap_len(2.0 * k, ppp).max(1.0), p.fg),
        StrokeKind::Inside,
    );
}

/// Details pane text for the hovered/focused skill (name, rank, requirement, description, why-not).
pub struct Detail {
    pub title: String,
    pub rank: String,
    pub lines: Vec<String>,
    pub why: Option<Deny>,
}

/// A build overview for the details pane: shown when no skill is hovered, so the
/// column carries the class, the points spent per tree, and the totals instead
/// of sitting blank.
pub fn build_summary(
    name: &str,
    class_name: &str,
    level: i32,
    trees: &[TreeView<'_>],
    spent: i32,
    available: i32,
) -> Detail {
    let mut lines = Vec::with_capacity(trees.len() + 2);
    for view in trees {
        let pts: i32 = view
            .model
            .tiers
            .iter()
            .flat_map(|t| &t.nodes)
            .map(|n| n.grade.max(0))
            .sum();
        lines.push(format!("{}   {pts}", view.name));
    }
    lines.push(String::new());
    lines.push(format!("Spent {spent}   Available {available}"));
    Detail {
        title: name.to_string(),
        rank: format!("Level {level}   {}", class_name.to_uppercase()),
        lines,
        why: None,
    }
}

/// Paint the details pane (right column). `detail` is the hovered/focused
/// skill, or `None` for a hint. All text goes through the num-font job, so
/// digits ("Rank 3/5", "+6%", "5 Shots or 6") obey the digit gate.
pub fn draw_detail(ui: &mut Ui, p: &Palette, detail: Option<&Detail>, hint: &str) {
    let ppp = ui.ctx().pixels_per_point();
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), 300.0), Sense::hover());
    crate::ui::pixel::panel(ui.painter(), rect, ppp, p);
    let inner = rect.shrink(14.0);
    let wrap = inner.width();
    let mut y = inner.top();
    match detail {
        None => put_text(ui, inner.left(), wrap, hint, p.dim, 13.0, &mut y),
        Some(d) => {
            put_text(ui, inner.left(), wrap, &d.title, p.fg, 18.0, &mut y);
            put_text(ui, inner.left(), wrap, &d.rank, p.gold, 14.0, &mut y);
            for line in &d.lines {
                put_text(ui, inner.left(), wrap, line, p.fg, 13.0, &mut y);
            }
            if let Some(why) = d.why {
                put_text(ui, inner.left(), wrap, &why.message(), p.dim, 13.0, &mut y);
            }
        }
    }
}

fn put_text(ui: &Ui, left: f32, wrap: f32, text: &str, color: Color32, size: f32, y: &mut f32) {
    let mut job = crate::ui::widgets::job_with_nums(text, color, size);
    job.wrap.max_width = wrap;
    let galley = ui.painter().layout_job(job);
    let h = galley.size().y;
    ui.painter().galley(pos2(left, *y), galley, color);
    *y += h + 6.0;
}

pub fn detail_for(
    t: &TreeView,
    ti: usize,
    ni: usize,
    available: i32,
    enforce: bool,
    action_ready: bool,
) -> Detail {
    let n = t.model.tiers[ti].nodes[ni];
    let v = &t.nodes[ti][ni];
    let why = rules::can_add(&t.model, ti, ni, available, enforce, action_ready).err();
    let gate = if ti == 0 {
        "Tier 1 . needs the action skill".to_string()
    } else {
        format!(
            "Tier {} . needs {} points in {}",
            ti + 1,
            rules::needed(t.model.tiers[ti].level),
            t.name
        )
    };
    Detail {
        title: v.name.to_string(),
        rank: format!("Rank {}/{}", n.grade, n.max),
        lines: vec![gate, v.desc.to_string()],
        why,
    }
}
