//! Fast-travel accessors (fields 16/17) and the bundled `Travel Stations.json`
//! lookup, on the synthetic save.

mod common;

use common::synthetic;
use lootsmith_core::{container, Platform};

#[test]
fn synthetic_teleporters_enumerate() {
    let sg = synthetic();
    let visited = sg.visited_teleporters();
    assert_eq!(
        visited.len(),
        3,
        "the synthetic save has 3 visited teleporters"
    );
    assert!(visited.iter().any(|s| s == "Sanctuary"));
    assert_eq!(sg.last_visited_teleporter().as_deref(), Some("Sanctuary"));
}

#[test]
fn add_remove_set_and_reload() {
    let mut sg = synthetic();
    let before = sg.visited_teleporters();

    assert!(
        sg.add_visited_teleporter("NewStation"),
        "first add reports true"
    );
    assert!(
        !sg.add_visited_teleporter("NewStation"),
        "duplicate add is a no-op"
    );
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
fn resolves_against_bundled_stations() {
    let stations = lootsmith_core::bundled::travel_stations().expect("bundled Travel Stations");
    assert!(stations.len() > 100, "expected a large station table");

    let shelf = stations
        .find("SouthernShelfTown")
        .expect("SouthernShelfTown");
    assert_eq!(shelf.display_name, "Southern Shelf");
    let sanctuary = stations.find("Sanctuary").expect("Sanctuary");
    assert_eq!(sanctuary.display_name, "Sanctuary");
    assert!(sanctuary.initially_active);
    assert_eq!(
        stations.label_for("SouthernShelfTown"),
        "Southern Shelf (SouthernShelfTown)"
    );
    assert_eq!(stations.label_for("UnknownStation"), "UnknownStation");

    // Every teleporter in the synthetic save resolves to a known station.
    let sg = synthetic();
    for name in sg.visited_teleporters() {
        assert!(
            stations.find(&name).is_some(),
            "teleporter {name} not found in Travel Stations.json"
        );
    }
}
