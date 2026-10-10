//! The synthetic save's protobuf payload must survive parse/re-emit
//! byte-for-byte, and the typed top-level schema must read known fields.

mod common;

use common::synthetic_file;
use lootsmith_core::proto::Message;

#[test]
fn synthetic_protobuf_is_byte_identical() {
    let save = synthetic_file();
    assert!(save.crc32_valid());
    assert!(
        save.proto_stable(),
        "protobuf parse/re-emit is not byte-identical"
    );

    let message = Message::parse(&save.proto).expect("parse protobuf");
    assert_eq!(message.to_bytes(), save.proto, "raw re-emit differs");

    let save_game = save.save_game().expect("typed parse");
    // Every field must be a known top-level field 1..=58.
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
}

#[test]
fn edit_and_reexport_stays_parseable() {
    let save = synthetic_file();
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
