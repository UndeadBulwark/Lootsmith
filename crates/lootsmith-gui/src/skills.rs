//! Embedded Borderlands 2 skill-tree layout for the skill calculator.
//!
//! The data (`assets/skills/skills.json`) and icons (`assets/skills/icons/*.png`)
//! are generated once by `scripts/gen-skill-data.py` and committed, so the app and
//! its builds are fully offline and always have the calculator available.

use eframe::egui;
use include_dir::{include_dir, Dir};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

static ICONS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/skills/icons");
static PORTRAITS: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/assets/skills/portraits");
static DATA: &str = include_str!("../assets/skills/skills.json");

#[derive(Clone, Debug, Deserialize)]
pub struct SkillNode {
    pub skill: String,
    /// The save's object path (`SkillData.Skill`), e.g. `GD_Siren_Skills.Motion.Ward`.
    /// This is the authority for matching a tile to a save row.
    #[serde(default)]
    pub id: String,
    pub max: i32,
    /// Grid column (0..=2) from the source page's `pushN` layout. `None` on
    /// older data falls back to the count rule.
    #[serde(default)]
    pub col: Option<u32>,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub desc: String,
}

/// The class action skill: a wide banner, not a tree node.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ActionSkill {
    #[serde(default)]
    pub name: String,
    /// The action skill's save object path (e.g. `GD_Siren_Skills.Phaselock.Skill_Phaselock`).
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillTier {
    pub level: i32,
    pub skills: Vec<SkillNode>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillTree {
    pub name: String,
    pub color: String,
    pub tiers: Vec<SkillTier>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SkillClass {
    pub name: String,
    #[serde(rename = "class")]
    pub class_name: String,
    pub page: String,
    #[serde(default)]
    pub portrait: String,
    #[serde(default)]
    pub action_skill: ActionSkill,
    pub trees: Vec<SkillTree>,
}

#[derive(Clone, Debug, Deserialize)]
struct SkillData {
    classes: HashMap<String, SkillClass>,
}

impl SkillClass {
    /// The canonical `SkillData` list for this class: the action skill first,
    /// then every branch skill branch by branch, tier by tier, left to right.
    /// This is the order the game writes and what a rebuilt save must use.
    pub fn save_order(&self) -> Vec<String> {
        let mut out = Vec::new();
        if !self.action_skill.id.is_empty() {
            out.push(self.action_skill.id.clone());
        }
        for tree in &self.trees {
            for tier in &tree.tiers {
                for node in &tier.skills {
                    if !node.id.is_empty() {
                        out.push(node.id.clone());
                    }
                }
            }
        }
        out
    }
}

/// The whole embedded dataset (parsed once).
fn data() -> &'static SkillData {
    static ONCE: OnceLock<SkillData> = OnceLock::new();
    ONCE.get_or_init(|| serde_json::from_str(DATA).expect("embedded skills.json is valid"))
}

/// The tree layout for a player-class resource path (`GD_Soldier.Character...`).
pub fn for_player_class(player_class: &str) -> Option<&'static SkillClass> {
    data().classes.get(player_class)
}

/// Short display name for a player-class resource path (`Soldier`, `Psycho`).
pub fn class_display_name(player_class: &str) -> String {
    let class_tag = for_player_class(player_class)
        .map(|c| c.class_name.as_str())
        .unwrap_or("?");
    class_tag.to_string()
}

/// All playable class ids, alphabetically, for the new-save picker.
pub fn class_ids() -> Vec<&'static str> {
    let mut ids: Vec<&'static str> = data().classes.keys().map(|s| s.as_str()).collect();
    ids.sort();
    ids
}

/// Raw PNG bytes for a bundled skill icon.
pub fn icon_bytes(filename: &str) -> Option<&'static [u8]> {
    ICONS.get_file(filename).map(|file| file.contents())
}

/// Raw PNG bytes for a bundled class portrait.
pub fn portrait_bytes(filename: &str) -> Option<&'static [u8]> {
    PORTRAITS.get_file(filename).map(|file| file.contents())
}

/// A tier unlocks once the tree has at least `5 * level` points invested.
pub fn tier_unlocked(tree_total: i32, tier_level: i32) -> bool {
    tree_total >= 5 * tier_level
}

/// Normalize a display name or resource leaf for matching (`Laser Sight` == `LaserSight`).
pub fn norm(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .flat_map(|c| c.to_lowercase())
        .collect()
}

/// The accent colour for a tree, matching the community calculator's palette.
pub fn tree_accent(color: &str) -> egui::Color32 {
    match color {
        "green" => egui::Color32::from_rgb(0x6C, 0xC2, 0x6A),
        "blue" => egui::Color32::from_rgb(0x5A, 0x9B, 0xE0),
        "red" => egui::Color32::from_rgb(0xD9, 0x6A, 0x5A),
        _ => egui::Color32::from_rgb(0xA2, 0xA8, 0xB3),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_data_is_complete() {
        let soldier = for_player_class("GD_Soldier.Character.CharClass_Soldier").expect("soldier");
        assert_eq!(soldier.name, "Axton");
        assert_eq!(soldier.trees.len(), 3);
        let names: Vec<&str> = soldier.trees.iter().map(|t| t.name.as_str()).collect();
        assert_eq!(names, ["Guerilla", "Gunpowder", "Survival"]);
        // The Guerilla tree: tiers of 2/2/3/1/1/1 (10 skills) — includes the capstone.
        let counts: Vec<usize> = soldier.trees[0]
            .tiers
            .iter()
            .map(|t| t.skills.len())
            .collect();
        assert_eq!(counts, [2, 2, 3, 1, 1, 1]);
        let sentry = &soldier.trees[0].tiers[0].skills[0];
        assert_eq!(sentry.skill, "Sentry");
        assert_eq!(sentry.id, "GD_Soldier_Skills.Guerrilla.Sentry");
        assert_eq!(sentry.max, 5);
        assert_eq!(sentry.col, Some(0), "Sentry is the left flank");
        assert!(icon_bytes(&sentry.icon).is_some(), "icon is embedded");

        // A flanked pair: tier 0 is Sentry@0, Ready@2 (not 0,1).
        assert_eq!(soldier.trees[0].tiers[0].skills[1].col, Some(2));

        // The action skill is a wide banner, embedded as an icon.
        assert_eq!(soldier.action_skill.name, "Sabre Turret");
        assert_eq!(
            soldier.action_skill.id,
            "GD_Soldier_Skills.Scorpio.Skill_Scorpio"
        );
        assert!(icon_bytes(&soldier.action_skill.icon).is_some());

        // All six classes are present.
        assert_eq!(data().classes.len(), 6);
    }

    #[test]
    fn every_tier_has_distinct_in_range_columns() {
        // The generated `col` is the layout authority; each tier's columns must
        // be distinct and inside the 3-wide grid.
        for class in data().classes.values() {
            for tree in &class.trees {
                for tier in &tree.tiers {
                    let mut cols: Vec<u32> = tier
                        .skills
                        .iter()
                        .map(|s| s.col.expect("generated data carries cols"))
                        .collect();
                    for c in &cols {
                        assert!(*c <= 2, "{}: column {c} out of range", tree.name);
                    }
                    cols.sort_unstable();
                    let before = cols.len();
                    cols.dedup();
                    assert_eq!(before, cols.len(), "{}: duplicate column", tree.name);
                }
            }
        }
    }

    #[test]
    fn salvador_brawn_tier_three_is_adjacent_not_flanked() {
        // Regression for the count rule: this tier is 0,1 in-game, not 0,2.
        let salvador =
            for_player_class("GD_Mercenary.Character.CharClass_Mercenary").expect("salvador");
        let tier = &salvador.trees[2].tiers[3];
        let cols: Vec<Option<u32>> = tier.skills.iter().map(|s| s.col).collect();
        assert_eq!(cols, vec![Some(0), Some(1)]);
    }

    #[test]
    fn every_node_carries_its_save_id() {
        for class in data().classes.values() {
            assert!(
                class.action_skill.id.starts_with("GD_"),
                "{}: action id",
                class.name
            );
            for tree in &class.trees {
                for tier in &tree.tiers {
                    for node in &tier.skills {
                        assert!(
                            node.id.starts_with("GD_"),
                            "{}: {} has no save id",
                            class.name,
                            node.skill
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn save_order_is_the_action_skill_then_the_trees() {
        let axton = for_player_class("GD_Soldier.Character.CharClass_Soldier").expect("axton");
        let order = axton.save_order();
        // 1 action + 10 Guerilla + 11 Gunpowder + 11 Survival.
        assert_eq!(order.len(), 33);
        assert_eq!(order[0], "GD_Soldier_Skills.Scorpio.Skill_Scorpio");
        assert_eq!(order[1], "GD_Soldier_Skills.Guerrilla.Sentry");
        assert_eq!(order[2], "GD_Soldier_Skills.Guerrilla.Ready");
        assert_eq!(order[11], "GD_Soldier_Skills.Gunpowder.Impact");
        // Object paths are unique.
        let mut unique = order.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), order.len());

        let siren = for_player_class("GD_Siren.Character.CharClass_Siren").expect("siren");
        assert_eq!(
            siren.save_order()[0],
            "GD_Siren_Skills.Phaselock.Skill_Phaselock"
        );
    }

    #[test]
    fn gating_rule() {
        assert!(tier_unlocked(0, 0));
        assert!(!tier_unlocked(4, 1));
        assert!(tier_unlocked(5, 1));
        assert!(tier_unlocked(10, 2));
        assert!(!tier_unlocked(9, 2));
    }

    #[test]
    fn normalization_matches_leaves() {
        assert_eq!(norm("Laser Sight"), norm("LaserSight"));
        assert_eq!(norm("Mag-Lock"), norm("Mag-Lock"));
        assert_eq!(norm("Healthy"), norm("HealthY"));
    }
}
