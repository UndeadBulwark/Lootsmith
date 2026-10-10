//! Item rarity, classified from the balance-definition name.
//!
//! The dumps carry **no rarity field** (checked `Weapon Balance.json`,
//! `Item Balance.json`, `Items.json`, `Weapon Types.json` — none expose one),
//! so rarity is derived from the balance name with the rules stored in
//! `assets/rarity.json` (data, not code). Anything unmatched is
//! [`Rarity::Unknown`] — we never guess, and the UI shows a neutral diamond
//! plus the text "Unknown".
//!
//! Rules are evaluated **in file order**, so the file must list the most
//! specific patterns first (`VeryRare` before `Rare`).

use crate::error::{Error, Result};
use std::collections::BTreeMap;

/// Item rarity, strongest last.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
    Etech,
    Legendary,
    Seraph,
    Pearlescent,
    /// No rule matched; shown neutrally, never guessed.
    Unknown,
}

impl Rarity {
    pub fn label(self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::VeryRare => "Very Rare",
            Rarity::Etech => "E-tech",
            Rarity::Legendary => "Legendary",
            Rarity::Seraph => "Seraph",
            Rarity::Pearlescent => "Pearlescent",
            Rarity::Unknown => "Unknown",
        }
    }

    /// The rarity for its label (case-insensitive), for data files and tests.
    pub fn from_label(label: &str) -> Option<Rarity> {
        const ALL: [Rarity; 9] = [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::VeryRare,
            Rarity::Etech,
            Rarity::Legendary,
            Rarity::Seraph,
            Rarity::Pearlescent,
            Rarity::Unknown,
        ];
        let normalize = |text: &str| -> String {
            text.chars()
                .filter(char::is_ascii_alphanumeric)
                .flat_map(char::to_lowercase)
                .collect()
        };
        let wanted = normalize(label);
        ALL.into_iter().find(|r| normalize(r.label()) == wanted)
    }
}

#[derive(serde::Deserialize)]
struct Rule {
    rarity: String,
    #[serde(default)]
    contains: Vec<String>,
}

#[derive(serde::Deserialize)]
struct Override {
    /// Balance asset name (exact) → rarity label.
    balance: String,
    rarity: String,
}

#[derive(serde::Deserialize)]
struct BalanceEntry {
    rarity: String,
}

#[derive(serde::Deserialize)]
struct RulesFile {
    version: u32,
    patterns: Vec<Rule>,
    #[serde(default)]
    overrides: Vec<Override>,
    #[serde(default)]
    balances: std::collections::HashMap<String, BalanceEntry>,
}

#[derive(Clone, Debug)]
pub struct Rules {
    version: u32,
    patterns: Vec<(Rarity, Vec<String>)>,
    overrides: Vec<(String, Rarity)>,
    /// Per-balance rarities sourced from real data (Borderlens, exact join).
    balances: Vec<(String, Rarity)>,
}

impl Rules {
    /// Parse a `rarity.json` document.
    pub fn from_json_str(text: &str) -> Result<Rules> {
        let file: RulesFile = serde_json::from_str(text)
            .map_err(|e| Error::GameData(format!("bad rarity.json: {e}")))?;
        let mut patterns = Vec::with_capacity(file.patterns.len());
        for rule in file.patterns {
            let rarity = Rarity::from_label(&rule.rarity).ok_or_else(|| {
                Error::GameData(format!("rarity.json: unknown rarity '{}'", rule.rarity))
            })?;
            patterns.push((rarity, rule.contains));
        }
        let mut overrides = Vec::with_capacity(file.overrides.len());
        for entry in file.overrides {
            let rarity = Rarity::from_label(&entry.rarity).ok_or_else(|| {
                Error::GameData(format!("rarity.json: unknown rarity '{}'", entry.rarity))
            })?;
            overrides.push((entry.balance, rarity));
        }
        let mut balances = Vec::with_capacity(file.balances.len());
        for (name, entry) in file.balances {
            let rarity = Rarity::from_label(&entry.rarity).ok_or_else(|| {
                Error::GameData(format!("rarity.json: unknown rarity '{}'", entry.rarity))
            })?;
            balances.push((name, rarity));
        }
        Ok(Rules {
            version: file.version,
            patterns,
            overrides,
            balances,
        })
    }

    /// The embedded rules (`assets/rarity.json`).
    pub fn embedded() -> &'static Rules {
        static ONCE: std::sync::OnceLock<Rules> = std::sync::OnceLock::new();
        ONCE.get_or_init(|| {
            Rules::from_json_str(include_str!("../assets/rarity.json"))
                .expect("embedded rarity.json is valid")
        })
    }

    pub fn version(&self) -> u32 {
        self.version
    }

    /// Classify one balance asset name.
    pub fn classify(&self, balance: &str) -> Rarity {
        // 1. real data (Borderlens, exact unique-name join), 2. reviewed
        // overrides, 3. generated-gear name patterns, else Unknown.
        if let Some((_, rarity)) = self.balances.iter().find(|(name, _)| name == balance) {
            return *rarity;
        }
        if let Some((_, rarity)) = self.overrides.iter().find(|(name, _)| name == balance) {
            return *rarity;
        }
        for (rarity, needles) in &self.patterns {
            if needles
                .iter()
                .any(|needle| balance.contains(needle.as_str()))
            {
                return *rarity;
            }
        }
        Rarity::Unknown
    }
}

/// Counts per rarity plus every name that matched no rule.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RarityReport {
    pub counts: BTreeMap<Rarity, usize>,
    pub unclassified: Vec<String>,
    pub total: usize,
}

impl RarityReport {
    pub fn classified(&self) -> usize {
        self.total - self.unclassified.len()
    }
}

/// Classify a whole list of balance names.
pub fn classify_all<'a>(names: impl IntoIterator<Item = &'a str>) -> RarityReport {
    let rules = Rules::embedded();
    let mut report = RarityReport::default();
    for name in names {
        report.total += 1;
        match rules.classify(name) {
            Rarity::Unknown => report.unclassified.push(name.to_string()),
            rarity => *report.counts.entry(rarity).or_default() += 1,
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_rules_load() {
        let rules = Rules::embedded();
        assert_eq!(rules.version(), 2, "generated by scripts/gen-rarity.py");
    }

    /// Real, dump-sourced names pinned to their expected rarity.
    #[test]
    fn known_items_classify() {
        let rules = Rules::embedded();
        for (balance, expected) in [
            (
                "GD_Anemone_Weap_SniperRifles.A_Weapons.Sniper_Jakobs_2_Uncommon",
                Rarity::Uncommon,
            ),
            (
                "GD_Anemone_Weap_SniperRifles.A_Weapons.Sniper_Jakobs_3_Rare",
                Rarity::Rare,
            ),
            (
                "GD_Anemone_Weap_SniperRifles.A_Weapons.Sniper_Jakobs_4_VeryRare",
                Rarity::VeryRare,
            ),
            (
                "GD_Weap_Pistol.A_Weapons.Pistol_Dahl_4_VeryRare",
                Rarity::VeryRare,
            ),
            ("GD_Weap_Pistol.A_Weapons.Pistol_Dahl", Rarity::Unknown),
            (
                "GD_Weap_Pistol.A_Weapons_Legendary.Pistol_Dahl_5_Emperor",
                Rarity::Legendary,
            ),
        ] {
            assert_eq!(rules.classify(balance), expected, "{balance}");
        }
    }

    #[test]
    fn real_data_mapping_is_used_first() {
        let rules = Rules::embedded();
        // A Borderlens-sourced balance from the generated file.
        assert_eq!(
            rules.classify("GD_Weap_Launchers.A_Weapons_Unique.RL_Maliwan_3_TheHive"),
            Rarity::Rare
        );
    }

    #[test]
    fn unknown_is_never_guessed() {
        let rules = Rules::embedded();
        assert_eq!(
            rules.classify("GD_Totally_Made_Up.Balance_Thing"),
            Rarity::Unknown
        );
    }

    #[test]
    fn report_counts_and_lists_unclassified() {
        let report = classify_all(["A_Rare", "B_Rare", "C_Mystery"]);
        assert_eq!(report.total, 3);
        assert_eq!(report.counts.get(&Rarity::Rare), Some(&2));
        assert_eq!(report.unclassified, ["C_Mystery"]);
        assert_eq!(report.classified(), 2);
    }

    #[test]
    fn overrides_win_over_patterns() {
        let json = r#"{
          "version": 1,
          "patterns": [{"rarity": "Rare", "contains": ["Rare"]}],
          "overrides": [{"balance": "GD_X.Special_Rare_Thing", "rarity": "Legendary"}]
        }"#;
        let rules = Rules::from_json_str(json).expect("parse");
        assert_eq!(rules.classify("GD_X.Special_Rare_Thing"), Rarity::Legendary);
        assert_eq!(rules.classify("GD_X.Other_Rare_Thing"), Rarity::Rare);
    }
}
