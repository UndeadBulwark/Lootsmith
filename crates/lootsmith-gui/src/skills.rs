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
static DATA: &str = include_str!("../assets/skills/skills.json");

#[derive(Clone, Debug, Deserialize)]
pub struct SkillNode {
    pub skill: String,
    pub max: i32,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub desc: String,
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
    pub trees: Vec<SkillTree>,
}

#[derive(Clone, Debug, Deserialize)]
struct SkillData {
    classes: HashMap<String, SkillClass>,
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

/// Raw PNG bytes for a bundled skill icon.
pub fn icon_bytes(filename: &str) -> Option<&'static [u8]> {
    ICONS.get_file(filename).map(|file| file.contents())
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
        assert_eq!(sentry.max, 5);
        assert!(icon_bytes(&sentry.icon).is_some(), "icon is embedded");

        // All six classes are present.
        assert_eq!(data().classes.len(), 6);
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
