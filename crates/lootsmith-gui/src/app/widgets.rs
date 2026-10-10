//! Shared widget helpers and the skill calculator tiles (moved verbatim
//! from the former single-file `app.rs`).

use super::*;
use crate::theme::{self, Tokens};
use eframe::egui;

pub(crate) fn customization_combo(
    ui: &mut egui::Ui,
    customizations: &Option<Arc<Customizations>>,
    usage: Option<&str>,
    kind: &str,
    value: &mut String,
    id: &str,
) {
    let current = match customizations {
        Some(c) if value.as_str() != "None" => c.label_for(value.as_str()),
        _ => value.clone(),
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(current)
        .width(340.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(*value == "None", "None").clicked() {
                *value = "None".to_string();
            }
            if let (Some(c), Some(usage)) = (customizations, usage) {
                for (key, name) in c.type_options(usage, kind) {
                    if ui.selectable_label(*value == key, &name).clicked() {
                        *value = key;
                    }
                }
            }
        });
}

/// Blend `accent` into `base` by `t` (0..1).
pub(crate) fn tint(base: egui::Color32, accent: egui::Color32, t: f32) -> egui::Color32 {
    let mix = |a: u8, b: u8| (a as f32 * (1.0 - t) + b as f32 * t).round() as u8;
    egui::Color32::from_rgb(
        mix(base.r(), accent.r()),
        mix(base.g(), accent.g()),
        mix(base.b(), accent.b()),
    )
}

/// One tree plate: a 3-slot-per-row tier pyramid, uniformly tinted with the
/// tree colour like the community calculator. A tier's single skill centres,
/// a pair spreads to the outer columns, three fill the row — no empty boxes.
pub(crate) fn tree_card(
    ui: &mut egui::Ui,
    tree: &crate::skills::SkillTree,
    grades: &[i32],
    grade_of: &impl Fn(&str) -> Option<usize>,
    textures: &std::collections::HashMap<String, Option<egui::TextureHandle>>,
    edit: &mut Option<(usize, i32)>,
) {
    const TILE: f32 = 62.0;
    const GAP: f32 = 6.0;
    const COLS: usize = 3;

    let accent = crate::skills::tree_accent(&tree.color);
    let tree_total: i32 = tree
        .tiers
        .iter()
        .flat_map(|t| &t.skills)
        .filter_map(|n| grade_of(&n.skill))
        .map(|i| grades[i].max(0))
        .sum();
    let grid_w = COLS as f32 * TILE + (COLS as f32 - 1.0) * GAP;

    egui::Frame::default()
        .fill(tint(Tokens::SURFACE, accent, 0.26))
        .stroke(egui::Stroke::new(1.0, Tokens::BORDER))
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                ui.set_width(grid_w);
                ui.spacing_mut().item_spacing = egui::vec2(GAP, GAP);

                for tier in &tree.tiers {
                    let unlocked = crate::skills::tier_unlocked(tree_total, tier.level);
                    let slots: [Option<&crate::skills::SkillNode>; COLS] =
                        match tier.skills.as_slice() {
                            [only] => [None, Some(only), None],
                            [a, b] => [Some(a), None, Some(b)],
                            [a, b, c, ..] => [Some(a), Some(b), Some(c)],
                            _ => [None, None, None],
                        };
                    ui.horizontal(|ui| {
                        for slot in slots {
                            match slot {
                                Some(node) => {
                                    let idx = grade_of(&node.skill);
                                    let grade = idx.map(|i| grades[i]).unwrap_or(0);
                                    let max = node.max.max(1);
                                    let tex = textures.get(&node.icon).cloned().flatten();
                                    let response = skill_tile(
                                        ui,
                                        &TileState {
                                            node,
                                            texture: tex.as_ref(),
                                            grade,
                                            max,
                                            unlocked,
                                            accent,
                                            size: TILE,
                                        },
                                    );
                                    if unlocked {
                                        if let Some(i) = idx {
                                            if response.clicked() && grade < max {
                                                *edit = Some((i, grade + 1));
                                            } else if response.secondary_clicked() && grade > 0 {
                                                *edit = Some((i, grade - 1));
                                            }
                                        }
                                    }
                                }
                                // Invisible spacer so the pyramid stays centred.
                                None => {
                                    ui.allocate_exact_size(
                                        egui::vec2(TILE, TILE),
                                        egui::Sense::hover(),
                                    );
                                }
                            }
                        }
                    });
                }

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new(tree.name.to_uppercase())
                            .strong()
                            .size(13.0)
                            .color(accent),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(tree_total.to_string())
                                .strong()
                                .size(13.0)
                                .color(accent),
                        );
                    });
                });
            });
        });
}

/// One skill tile: icon-centred with a metal frame and a rank badge, matching the
/// community calculator. Partial/full/locked/hover states are all visualised.
pub(crate) struct TileState<'a> {
    node: &'a crate::skills::SkillNode,
    texture: Option<&'a egui::TextureHandle>,
    grade: i32,
    max: i32,
    unlocked: bool,
    accent: egui::Color32,
    size: f32,
}

pub(crate) fn skill_tile(ui: &mut egui::Ui, t: &TileState) -> egui::Response {
    let (rect, response) = ui.allocate_exact_size(egui::vec2(t.size, t.size), egui::Sense::click());
    let hovered = response.hovered();
    let painter = ui.painter_at(rect);
    let full = t.max > 1 && t.grade >= t.max;
    let unlocked = t.unlocked;
    let accent = t.accent;

    // Soft glow when maxed.
    if full && unlocked {
        painter.rect_filled(
            rect.expand(2.0),
            egui::CornerRadius::same(10),
            accent.gamma_multiply(0.30),
        );
    }
    // Metal edge + inner plate (brightens on hover).
    painter.rect_filled(rect, egui::CornerRadius::same(8), Tokens::BG);
    let plate = if !unlocked {
        Tokens::BG
    } else if hovered {
        tint(Tokens::SURFACE, accent, 0.06)
    } else {
        Tokens::SURFACE_ALT
    };
    painter.rect_filled(rect.shrink(2.0), egui::CornerRadius::same(7), plate);

    // Border: accent when invested, dim when locked.
    let (width, color) = if !unlocked {
        (1.0, Tokens::SURFACE_ALT)
    } else if t.grade > 0 {
        (2.0, accent)
    } else {
        (1.0, Tokens::BORDER)
    };
    painter.rect_stroke(
        rect.shrink(width / 2.0),
        egui::CornerRadius::same(8),
        egui::Stroke::new(width, color),
        egui::StrokeKind::Inside,
    );

    // Icon.
    if let Some(texture) = t.texture {
        let icon = egui::Rect::from_center_size(
            rect.center() - egui::vec2(0.0, 3.0),
            egui::vec2(t.size - 18.0, t.size - 18.0),
        );
        let icon_tint = if unlocked {
            egui::Color32::WHITE
        } else {
            egui::Color32::from_gray(88)
        };
        painter.image(
            texture.id(),
            icon,
            egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
            icon_tint,
        );
    }
    // Locked marker.
    if !unlocked {
        painter.circle_filled(
            rect.left_top() + egui::vec2(10.0, 10.0),
            3.0,
            Tokens::TEXT_FAINT,
        );
    }

    // Rank badge, bottom-right.
    let badge_color = if !unlocked {
        Tokens::TEXT_FAINT
    } else if full {
        accent
    } else {
        Tokens::TEXT
    };
    let galley = painter.layout_no_wrap(
        format!("{}/{}", t.grade, t.max),
        egui::FontId::proportional(11.5),
        badge_color,
    );
    let pad = egui::vec2(4.0, 2.0);
    let badge_size = galley.size() + pad * 2.0;
    let badge = egui::Rect::from_min_size(
        egui::pos2(
            rect.right() - badge_size.x - 3.0,
            rect.bottom() - badge_size.y - 3.0,
        ),
        badge_size,
    );
    painter.rect_filled(
        badge,
        egui::CornerRadius::same(3),
        egui::Color32::from_black_alpha(210),
    );
    painter.galley(badge.min + pad, galley, badge_color);

    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::WidgetType::Button, unlocked, &t.node.skill)
    });
    if t.node.desc.is_empty() {
        response
    } else {
        response.on_hover_text(&t.node.desc)
    }
}

/// Display label for a currency slot.
pub(crate) fn currency_name(index: usize) -> String {
    match index {
        0 => "Money".to_string(),
        1 => "Eridium".to_string(),
        2 => "Seraph crystals".to_string(),
        3 => "Torgue tokens".to_string(),
        n => format!("Slot {n}"),
    }
}

/// A vertical rule between toolbar groups.
pub(crate) fn toolbar_sep(ui: &mut egui::Ui) {
    let brd = theme::active(ui.ctx()).brd;
    ui.add_space(4.0);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(1.0, 18.0), egui::Sense::hover());
    ui.painter()
        .vline(rect.center().x, rect.y_range(), egui::Stroke::new(1.0, brd));
    ui.add_space(4.0);
}

/// A filled status dot.
pub(crate) fn status_dot(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(9.0, 9.0), egui::Sense::hover());
    ui.painter().circle_filled(rect.center(), 4.0, color);
}

/// A grouped surface card of a fixed width.
pub(crate) fn card(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let p = *theme::active(ui.ctx());
    ui.allocate_ui_with_layout(
        egui::vec2(width, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::Frame::group(ui.style())
                .fill(p.pan)
                .stroke(egui::Stroke::new(1.0, p.wdb))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(width - 28.0);
                    add(ui);
                });
        },
    );
}

/// The empty state shown before a save is open.
pub(crate) fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(64.0);
        ui.label(
            egui::RichText::new("Lootsmith")
                .size(26.0)
                .strong()
                .color(theme::active(ui.ctx()).gold),
        );
        ui.add_space(6.0);
        theme::muted(ui, "A cross-platform Borderlands 2 save editor.");
        ui.add_space(14.0);
        theme::faint(
            ui,
            "Open a save (or use Find saves), then pick your game-data folder",
        );
        theme::faint(
            ui,
            "so item parts, missions and travel stations resolve to names.",
        );
    });
}
