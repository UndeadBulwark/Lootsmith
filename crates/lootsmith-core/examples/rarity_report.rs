//! Classify every balance name in the game-data dumps and write a report.
//!
//! Usage: cargo run -p lootsmith-core --example rarity_report -- <Asset Library Manager.json> [out.md]
//!
//! Prints per-rarity counts, then the unclassified names grouped by their
//! package marker, and writes the same to markdown.

use lootsmith_core::{classify_all, AssetGroup, AssetLibraryManager, Rarity};
use std::collections::BTreeMap;

fn main() {
    let mut args = std::env::args().skip(1);
    let library = args
        .next()
        .or_else(|| std::env::var("LOOTSMITH_ASSET_LIBRARY").ok())
        .expect("pass Asset Library Manager.json (or set LOOTSMITH_ASSET_LIBRARY)");
    let out = args
        .next()
        .unwrap_or_else(|| "/home/undeadbulwark/Downloads/lootsmith-rarity-report.md".to_string());

    let manager =
        AssetLibraryManager::from_path(std::path::Path::new(&library)).expect("load asset library");
    let names = manager.choices(AssetGroup::BalanceDefs, 0);
    let report = classify_all(names.iter().map(String::as_str));

    let mut text = String::new();
    text.push_str("# Lootsmith rarity classification report\n\n");
    text.push_str(&format!("Source: `{library}`\n\n"));
    text.push_str(&format!(
        "Balance names: **{}** — classified **{}**, unclassified **{}**\n\n",
        report.total,
        report.classified(),
        report.unclassified.len()
    ));
    text.push_str("## Counts per rarity\n\n| Rarity | Count |\n|---|---|\n");
    for rarity in [
        Rarity::Common,
        Rarity::Uncommon,
        Rarity::Rare,
        Rarity::VeryRare,
        Rarity::Etech,
        Rarity::Legendary,
        Rarity::Seraph,
        Rarity::Pearlescent,
    ] {
        let count = report.counts.get(&rarity).copied().unwrap_or(0);
        text.push_str(&format!("| {} | {count} |\n", rarity.label()));
    }
    text.push_str(&format!("| Unknown | {} |\n", report.unclassified.len()));

    // Group the unclassified names by a coarse marker for review.
    let mut groups: BTreeMap<&str, Vec<&String>> = BTreeMap::new();
    for name in &report.unclassified {
        let marker = if name.contains(".BalanceDefs.") {
            "non-gear balance defs (mission/pickup items)"
        } else if name.contains("Unique") {
            "Unique weapons/items (awaiting the override list)"
        } else if name.contains("ClassMods") {
            "class mods"
        } else if name.contains("A_Weapons") || name.contains("A_Item") {
            "gear without a rarity marker in its name"
        } else {
            "other"
        };
        groups.entry(marker).or_default().push(name);
    }
    text.push_str("\n## Unclassified\n\n");
    for (marker, entries) in &groups {
        text.push_str(&format!("### {marker} — {}\n\n", entries.len()));
        for name in entries {
            text.push_str(&format!("- `{name}`\n"));
        }
        text.push('\n');
    }

    std::fs::write(&out, &text).expect("write report");
    println!("{}", text.lines().take(14).collect::<Vec<_>>().join("\n"));
    println!("… full report: {out}");
    for (marker, entries) in &groups {
        println!("  {marker}: {}", entries.len());
    }
}
