//! The skills tab: the Homestead board (`ui::skill_board`) driven by the pure
//! rules in `skill_rules`. Tile columns come from the generated `skills.json`,
//! so each tree matches the in-game layout.

use crate::app::*;
use crate::skill_rules::{Node, Tier, Tree};
use crate::theme;
use crate::ui::skill_board::{self as board, Act, NodeView, TreeView};
use eframe::egui;

/// Action-skill banner: ~1.5 tree widths, centred over the row.
const BANNER_W: f32 = 354.0;
const BANNER_H: f32 = 74.0;
/// Tiles may shrink to this fraction before the row is allowed to scroll.
const MIN_TILE_SCALE: f32 = 0.55;

impl LootsmithApp {
    pub(crate) fn ui_skills(&mut self, ui: &mut egui::Ui) {
        let Some(class) = crate::skills::for_player_class(&self.read_info.player_class) else {
            theme::muted(ui, "Unknown player class — the skill tree is unavailable.");
            return;
        };

        // The save's skill list is class-specific. If it belongs to another class
        // (e.g. the class was changed before this was handled), rebuild it to this
        // class's canonical list so the board is editable.
        let want = crate::skills::norm(&class.action_skill.id);
        if !want.is_empty()
            && self.save_game.is_some()
            && !self
                .skills
                .iter()
                .any(|r| crate::skills::norm(&r.name) == want)
        {
            self.rebuild_skill_list(class);
            self.status = format!(
                "skill list rebuilt for {}",
                crate::skills::class_display_name(&self.read_info.player_class)
            );
        }

        // Snapshot grades and index the save's skills by object path (and leaf).
        let grades: Vec<i32> = self.skills.iter().map(|r| r.grade).collect();
        let mut index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for (i, row) in self.skills.iter().enumerate() {
            index.insert(crate::skills::norm(&row.name), i);
            let leaf = row.name.rsplit('.').next().unwrap_or(&row.name);
            index.entry(crate::skills::norm(leaf)).or_insert(i);
        }
        let idx_of = |id: &str| {
            index
                .get(&crate::skills::norm(id))
                .or_else(|| {
                    let leaf = id.rsplit('.').next().unwrap_or(id);
                    index.get(&crate::skills::norm(leaf))
                })
                .copied()
        };

        let spent: i32 = grades.iter().map(|g| (*g).max(0)).sum();
        let available = self.form.general_skill_points;
        let enforce = self.enforce_budget;

        // Decode/cache the textures this class needs (icons, action banner, portrait).
        let portrait = self.skill_texture(ui.ctx(), &class.portrait);
        let action_icon = self.skill_texture(ui.ctx(), &class.action_skill.icon);
        let mut textures: std::collections::HashMap<String, Option<egui::TextureHandle>> =
            std::collections::HashMap::new();
        for tree in &class.trees {
            for tier in &tree.tiers {
                for node in &tier.skills {
                    if !node.icon.is_empty() && !textures.contains_key(&node.icon) {
                        let handle = self.skill_texture(ui.ctx(), &node.icon);
                        textures.insert(node.icon.clone(), handle);
                    }
                }
            }
        }

        let p = *theme::active(ui.ctx());
        let mut header_copy = false;
        let mut header_apply = false;
        let mut header_respec = false;

        // Header + controls.
        ui.add_space(6.0);
        ui.horizontal(|ui| {
            crate::ui::widgets::label_nums(ui, &class.name, p.fg, 24.0);
            ui.add_space(8.0);
            crate::ui::widgets::label_nums(
                ui,
                format!(
                    "LEVEL {} {}",
                    self.form.level,
                    class.class_name.to_uppercase()
                ),
                p.dim,
                14.0,
            );
            ui.add_space(12.0);
            crate::ui::widgets::label_nums(
                ui,
                format!("Spent {spent}   Available {available}"),
                p.gold,
                15.0,
            );
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let (r, _) = ui.allocate_exact_size(egui::vec2(76.0, 24.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r,
                    "Apply",
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                {
                    header_apply = true;
                }
                ui.add_sized(
                    [176.0, 22.0],
                    egui::TextEdit::singleline(&mut self.skill_build).hint_text("build code"),
                );
                let (r, _) = ui.allocate_exact_size(egui::vec2(100.0, 24.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r,
                    "Copy build",
                    crate::ui::widgets::ButtonKind::Normal,
                )
                .clicked()
                {
                    header_copy = true;
                }
                let respec_label = if self.respec_armed {
                    "Respec?"
                } else {
                    "Respec"
                };
                let (r, _) = ui.allocate_exact_size(egui::vec2(90.0, 24.0), egui::Sense::click());
                if crate::ui::widgets::button(
                    ui,
                    r,
                    respec_label,
                    crate::ui::widgets::ButtonKind::Danger,
                )
                .clicked()
                {
                    header_respec = true;
                }
                // Checked = ignore the budget, so the stored flag is inverted.
                let mut ignore_budget = !self.enforce_budget;
                if crate::ui::widgets::checkbox(ui, &mut ignore_budget, "Ignore budget").changed() {
                    self.enforce_budget = !ignore_budget;
                }
            });
        });
        ui.add_space(6.0);

        // The action skill (a 0/1 skill) gates tier 1: no tree point may be spent
        // until it is filled. Match it by the class's object path.
        let action_idx = idx_of(&class.action_skill.id);
        let action_grade = action_idx.map(|i| grades[i].max(0)).unwrap_or(0);
        let action_ready = action_grade >= 1;
        let mut act: Option<Act> = None;

        // Build one board view per tree (model + nodes), plus the save-index map
        // (tree -> tier -> node) a pasted build needs.
        let mut views: Vec<TreeView> = Vec::new();
        let mut index_map: Vec<Vec<Vec<Option<usize>>>> = Vec::new();
        for tree in &class.trees {
            let mut tiers: Vec<Tier> = Vec::new();
            let mut nodes: Vec<Vec<NodeView>> = Vec::new();
            let mut tree_map: Vec<Vec<Option<usize>>> = Vec::new();
            for tier in &tree.tiers {
                let mut tier_nodes = Vec::new();
                let mut tier_views = Vec::new();
                let mut tier_map = Vec::new();
                for node in &tier.skills {
                    let save_idx = idx_of(&node.id);
                    let grade = save_idx.map(|i| grades[i].max(0)).unwrap_or(0);
                    tier_nodes.push(Node {
                        grade,
                        max: node.max,
                    });
                    tier_views.push(NodeView {
                        name: &node.skill,
                        desc: &node.desc,
                        icon: textures.get(&node.icon).and_then(|t| t.as_ref()),
                        col: node.col,
                        save_idx,
                    });
                    tier_map.push(save_idx);
                }
                tiers.push(Tier {
                    level: tier.level,
                    nodes: tier_nodes,
                });
                nodes.push(tier_views);
                tree_map.push(tier_map);
            }
            views.push(TreeView {
                name: &tree.name,
                accent: crate::skills::tree_accent(&tree.color),
                model: Tree { tiers },
                nodes,
            });
            index_map.push(tree_map);
        }

        let mut focus: Option<(usize, usize, usize)> = None;
        let body = ui.available_rect_before_wrap();
        let native_row = views.len() as f32 * board::panel_w(1.0)
            + views.len().saturating_sub(1) as f32 * board::GAP;
        let tiers = views.first().map_or(6, |v| v.model.tiers.len());
        let native_h = board::panel_h(tiers, 1.0);
        // Show the details pane when the row plus it fit at native size.
        let details_w = 300.0;
        let show_details = body.width() >= native_row + details_w + 32.0;
        let avail_w = if show_details {
            body.width() - details_w - 16.0
        } else {
            body.width()
        };
        // Fit the tiles to BOTH axes so every tier is visible with no vertical
        // scroll, and keep the action-skill banner inside the same fitted region so
        // it scales/moves with the trees instead of clipping on its own.
        let avail_h = (body.height() - BANNER_H - 16.0).max(60.0);
        let k = (avail_w / native_row)
            .min(avail_h / native_h)
            .clamp(MIN_TILE_SCALE, 1.0);
        let row_w = k * native_row;
        let trees_max = egui::Rect::from_min_max(
            egui::pos2(body.left(), body.top() + BANNER_H + 16.0),
            egui::pos2(
                if show_details {
                    body.right() - details_w - 16.0
                } else {
                    body.right()
                },
                body.bottom(),
            ),
        );
        let mut panels_left = trees_max.left();
        ui.scope_builder(egui::UiBuilder::new().max_rect(trees_max), |ui| {
            ui.set_clip_rect(trees_max);
            // Vertical + horizontal scroll, so the three fixed-width panels never
            // clip on a narrow window.
            egui::ScrollArea::vertical()
                .id_salt("skill_trees")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    egui::ScrollArea::horizontal()
                        .id_salt("skill_row")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal_top(|ui| {
                                // The portrait column only fits on wide bodies.
                                if body.width() >= 1300.0 {
                                    if let Some(texture) = &portrait {
                                        ui.add(
                                            egui::Image::new(texture)
                                                .max_height(430.0)
                                                .max_width(300.0),
                                        );
                                        ui.add_space(16.0);
                                    }
                                }
                                // One allocation for the row; panels sit on exact rects.
                                let pw = board::panel_w(k);
                                let gap = board::GAP * k;
                                let tiers = views.first().map_or(6, |v| v.model.tiers.len());
                                let ph = board::panel_h(tiers, k);
                                let row_w = views.len() as f32 * pw
                                    + views.len().saturating_sub(1) as f32 * gap;
                                let (row, _) = ui.allocate_exact_size(
                                    egui::vec2(row_w, ph),
                                    egui::Sense::hover(),
                                );
                                panels_left = row.min.x;
                                for (vi, view) in views.iter().enumerate() {
                                    let rect = egui::Rect::from_min_size(
                                        row.min + egui::vec2(vi as f32 * (pw + gap), 0.0),
                                        egui::vec2(pw, ph),
                                    );
                                    let (a, f) = board::draw_tree(
                                        ui,
                                        &p,
                                        rect,
                                        view,
                                        available,
                                        enforce,
                                        action_ready,
                                        k,
                                    );
                                    if a.is_some() {
                                        act = a;
                                    }
                                    if let Some((ti, ni)) = f {
                                        focus = Some((vi, ti, ni));
                                    }
                                }
                            });
                        });
                });
        });

        // Details pane: the skill under the cursor / focus.
        if show_details {
            let details_rect = egui::Rect::from_min_max(
                egui::pos2(body.right() - details_w, body.top()),
                body.max,
            );
            let mut detail = focus.map(|(vi, ti, ni)| {
                board::detail_for(&views[vi], ti, ni, available, enforce, action_ready)
            });
            // Nothing hovered: summarise the build rather than showing a bare hint.
            if detail.is_none() {
                detail = Some(board::build_summary(
                    &class.name,
                    &class.class_name,
                    self.form.level,
                    &views,
                    spent,
                    available,
                ));
            }
            ui.scope_builder(egui::UiBuilder::new().max_rect(details_rect), |ui| {
                ui.set_clip_rect(details_rect);
                board::draw_detail(
                    ui,
                    &p,
                    detail.as_ref(),
                    "Hover or focus a skill to see its details.",
                );
            });
        }
        ui.advance_cursor_after_rect(body);

        // The action-skill banner, centred over the tree panels (painted after the
        // row so `panels_left` is known). Clearing is refused while tier 1 holds ranks.
        let models: Vec<Tree> = views.iter().map(|v| v.model.clone()).collect();
        let clear_ok = crate::skill_rules::action_clear_ok(&models);
        let brect = egui::Rect::from_center_size(
            egui::pos2(panels_left + row_w / 2.0, body.top() + BANNER_H / 2.0 + 8.0),
            egui::vec2(BANNER_W, BANNER_H),
        );
        if let Some(a) = board::action_banner(
            ui,
            &p,
            brect,
            &class.action_skill.name,
            action_icon.as_ref(),
            action_idx,
            action_grade,
            1,
            clear_ok,
        ) {
            act = Some(a);
        }

        // Header actions.
        if header_copy {
            let build = crate::skill_rules::encode(&class.class_name, &models);
            ui.ctx().copy_text(build.clone());
            self.skill_build = build;
            self.status = "skill build copied to the clipboard".into();
        }
        if header_apply {
            let _ = self.apply_skill_build(&class.class_name, &models, &index_map);
        }
        if header_respec {
            if self.respec_armed {
                self.respec_skills();
                self.respec_armed = false;
            } else {
                self.respec_armed = true;
                self.status = "click Respec again to refund every point".into();
            }
        }

        match act {
            Some(Act::Set(i, new_grade)) => self.set_skill_grade(i, new_grade),
            Some(Act::Status(msg)) => self.status = msg,
            Some(Act::Respec) | None => {}
        }
    }

    /// Apply a pasted `LS2:` build (validated against the tree shape and the
    /// total points available).
    pub fn apply_skill_build(
        &mut self,
        class_tag: &str,
        models: &[Tree],
        index_map: &[Vec<Vec<Option<usize>>>],
    ) -> Result<(), String> {
        let decoded = crate::skill_rules::decode(&self.skill_build, class_tag, models)
            .map_err(|e| format!("invalid build ({e:?})"))?;
        let total_points: i32 = self.form.general_skill_points
            + self.skills.iter().map(|s| s.grade.max(0)).sum::<i32>();
        let new_spent: i32 = decoded.iter().flatten().sum();
        if new_spent > total_points {
            let msg = format!("build needs {new_spent} points; only {total_points} total");
            self.status = msg.clone();
            return Err(msg);
        }
        for (vi, tree_grades) in decoded.iter().enumerate() {
            let mut k = 0usize;
            for tier_map in &index_map[vi] {
                for &maybe_idx in tier_map {
                    let g = tree_grades.get(k).copied().unwrap_or(0);
                    k += 1;
                    if let Some(idx) = maybe_idx {
                        if let Some(row) = self.skills.get_mut(idx) {
                            row.grade = g;
                            row.points = 0;
                        }
                    }
                }
            }
        }
        self.form.general_skill_points = (total_points - new_spent).max(0);
        self.dirty = true;
        self.apply_skills();
        self.status = format!("build applied ({new_spent} points)");
        Ok(())
    }

    /// Refund every spent skill point and clear the trees.
    pub fn respec_skills(&mut self) {
        let refund: i32 = self.skills.iter().map(|s| s.grade.max(0)).sum();
        for row in self.skills.iter_mut() {
            row.grade = 0;
            row.points = 0;
        }
        self.form.general_skill_points += refund;
        self.dirty = true;
        self.apply_skills();
        self.status = format!("respec: refunded {refund} point(s)");
    }

    /// Apply a skill grade and keep the point budget consistent, then write.
    pub fn set_skill_grade(&mut self, i: usize, new_grade: i32) {
        let old = self.skills[i].grade;
        if new_grade == old {
            return;
        }
        self.skills[i].grade = new_grade;
        if new_grade > old {
            let delta = new_grade - old;
            self.form.general_skill_points = (self.form.general_skill_points - delta).max(0);
        } else {
            self.form.general_skill_points += old - new_grade;
        }
        self.dirty = true;
        self.apply_skills();
    }
}
