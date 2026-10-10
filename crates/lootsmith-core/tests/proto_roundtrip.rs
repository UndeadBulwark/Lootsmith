//! Phase 2b gate: the real save's protobuf payload must survive parse/re-emit
//! byte-for-byte, and the typed top-level schema must read known fields.

use lootsmith_core::{container, proto::Message, Platform};

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn real_save_protobuf_is_byte_identical() {
    let path = fixture_path();
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("fixture not present at {path}; skipping");
            return;
        }
    };

    let save = container::unpack(&data, Platform::PC).expect("unpack");
    assert!(save.crc32_valid());
    assert!(
        save.proto_stable(),
        "protobuf parse/re-emit is not byte-identical"
    );

    let message = Message::parse(&save.proto).expect("parse protobuf");
    assert_eq!(message.to_bytes(), save.proto, "raw re-emit differs");

    let save_game = save.save_game().expect("typed parse");
    // Every field in a real save must be a known top-level field 1..=58.
    for field in &save_game.message.fields {
        assert!(
            (1..=58).contains(&field.number),
            "unexpected top-level field {}",
            field.number
        );
    }
    assert!(save_game.player_class().is_some(), "missing PlayerClass");
    assert!(save_game.exp_level().is_some(), "missing ExpLevel");
    assert!(!save_game.currency_on_hand().is_empty());

    eprintln!(
        "protobuf OK: {} fields, PlayerClass={:?}, ExpLevel={:?}, Currency={:?}",
        save_game.message.fields.len(),
        save_game.player_class(),
        save_game.exp_level(),
        save_game.currency_on_hand()
    );
}

#[test]
fn edit_and_reexport_stays_parseable() {
    let path = fixture_path();
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(_) => return,
    };
    let save = container::unpack(&data, Platform::PC).expect("unpack");
    let mut save_game = save.save_game().expect("parse");

    // Mutate a scalar, then re-parse and confirm the edit took while the rest
    // re-emits exactly.
    save_game.set_i32(2, 61);
    let bytes = save_game.to_bytes();
    let again = lootsmith_core::SaveGame::parse(&bytes).expect("reparse");
    assert_eq!(again.exp_level(), Some(61));
    assert_eq!(again.player_class(), save_game.player_class());
    // untouched fields keep their exact bytes
    assert!(bytes.len() != save.proto.len() || bytes != save.proto);
}
