//! Fast-travel accessors (fields 16/17) and the `Travel Stations.json` lookup.

use lootsmith_core::{container, Platform, TravelStations};

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn dumps_dir() -> std::path::PathBuf {
    std::env::var("LOOTSMITH_DUMPS")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| {
            std::path::PathBuf::from(format!(
                "{}/../../../bl2-rogue/data/gibbed",
                env!("CARGO_MANIFEST_DIR")
            ))
        })
}

fn fixture_save() -> Option<lootsmith_core::SaveGame> {
    let data = std::fs::read(fixture_path()).ok()?;
    let save = container::unpack(&data, Platform::PC).ok()?;
    save.save_game().ok()
}

#[test]
fn fixture_teleporters_enumerate() {
    let Some(sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let visited = sg.visited_teleporters();
    assert_eq!(visited.len(), 23, "the fixture has 23 visited teleporters");
    assert!(visited.iter().any(|s| s == "Sanctuary"));
    assert_eq!(sg.last_visited_teleporter().as_deref(), Some("SouthernShelfTown"));
}

#[test]
fn add_remove_set_and_reload() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let before = sg.visited_teleporters();

    assert!(sg.add_visited_teleporter("NewStation"), "first add reports true");
    assert!(!sg.add_visited_teleporter("NewStation"), "duplicate add is a no-op");
    assert_eq!(sg.visited_teleporters().len(), before.len() + 1);
    assert!(sg.remove_visited_teleporter("NewStation"));
    assert!(!sg.remove_visited_teleporter("NewStation"));
    assert_eq!(sg.visited_teleporters(), before);

    sg.set_last_visited_teleporter("NewStation");

    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());
    let sg2 = reopened.save_game().expect("reparse");
    assert_eq!(sg2.visited_teleporters(), before, "list survived a reload");
    assert_eq!(sg2.last_visited_teleporter().as_deref(), Some("NewStation"));
}

#[test]
fn resolves_against_real_stations() {
    let path = dumps_dir().join("Travel Stations.json");
    if !path.exists() {
        eprintln!("no Travel Stations.json at {}; skipping", path.display());
        return;
    }
    let stations = TravelStations::from_path(&path).expect("parse Travel Stations.json");
    assert!(stations.len() > 100, "expected a large station table");

    let shelf = stations.find("SouthernShelfTown").expect("SouthernShelfTown");
    assert_eq!(shelf.display_name, "Southern Shelf");
    let sanctuary = stations.find("Sanctuary").expect("Sanctuary");
    assert_eq!(sanctuary.display_name, "Sanctuary");
    assert!(sanctuary.initially_active);
    assert_eq!(stations.label_for("SouthernShelfTown"), "Southern Shelf (SouthernShelfTown)");
    assert_eq!(stations.label_for("UnknownStation"), "UnknownStation");

    // Every teleporter in the fixture resolves to a known station.
    if let Some(sg) = fixture_save() {
        for name in sg.visited_teleporters() {
            assert!(
                stations.find(&name).is_some(),
                "teleporter {name} not found in Travel Stations.json"
            );
        }
    }
}
