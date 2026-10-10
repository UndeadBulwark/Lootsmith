//! Phase 4 gate for the packed item/weapon codec.
//!
//! Every blob in a real save must decode, and re-encoding must round-trip
//! semantically. For blobs with real content (non-zero unique id, i.e. actually
//! obfuscated), re-encoding must also be byte-identical to the original — this
//! exercises the full pipeline: bit layout, checksum fold, obfuscation and
//! trailing trim. Blank 40-byte placeholders (unique id 0) are zero-filled by
//! the game and are only semantically stable; the C# reference cannot reproduce
//! their trailing fill byte either.

use lootsmith_core::gameinfo::AssetLibraryConfigs;
use lootsmith_core::items;
use lootsmith_core::{container, proto::Message, Platform};
use std::path::Path;

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn load_configs() -> AssetLibraryConfigs {
    let candidates = [
        std::env::var("LOOTSMITH_ASSET_LIBRARY").ok(),
        Some(format!(
            "{}/../bl2-rogue/data/gibbed/Asset Library Manager.json",
            env!("CARGO_MANIFEST_DIR")
        )),
    ];
    for candidate in candidates.into_iter().flatten() {
        let path = Path::new(&candidate);
        if path.exists() {
            if let Ok(configs) = AssetLibraryConfigs::from_path(path) {
                return configs;
            }
        }
    }
    AssetLibraryConfigs::pc_default()
}

fn serial_numbers(save: &lootsmith_core::SaveFile, field: u32) -> Vec<Vec<u8>> {
    let top = Message::parse(&save.proto).expect("parse top-level protobuf");
    let mut out = Vec::new();
    for entry in top.get_all(field) {
        let sub = entry
            .as_message()
            .expect("parse sub-message")
            .expect("bytes");
        if let Some(serial) = sub.get(1).and_then(|f| f.as_bytes()) {
            out.push(serial.to_vec());
        }
    }
    out
}

#[test]
fn real_save_items_decode_and_reencode() {
    let path = fixture_path();
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("fixture not present; skipping");
            return;
        }
    };

    let configs = load_configs();
    assert_eq!(configs.version, 7, "unexpected asset library version");
    let save = container::unpack(&data, Platform::PC).expect("unpack");

    let mut total = 0usize;
    let mut byte_identical = 0usize;
    let mut real_content = 0usize;
    let mut real_content_identical = 0usize;

    for (field, is_weapon) in [(53u32, false), (54u32, true)] {
        for serial in serial_numbers(&save, field) {
            let decoded = items::unpack(&serial, &configs)
                .unwrap_or_else(|e| panic!("decode {serial:02x?}: {e}"));
            assert_eq!(decoded.is_weapon, is_weapon);

            let reencoded =
                items::pack(&decoded, &configs).unwrap_or_else(|e| panic!("encode: {e}"));

            // Semantic gate: the re-encoded blob must decode to the same data.
            let again =
                items::unpack(&reencoded, &configs).unwrap_or_else(|e| panic!("re-decode: {e}"));
            assert_eq!(again, decoded, "semantic round trip changed the item");

            total += 1;
            if reencoded == serial {
                byte_identical += 1;
            }
            if decoded.unique_id != 0 {
                real_content += 1;
                if reencoded == serial {
                    real_content_identical += 1;
                }
            }
        }
    }

    assert!(total > 0, "no packed items found");
    assert_eq!(
        real_content, real_content_identical,
        "{}/{real_content} real-content blobs were not byte-identical",
        real_content_identical
    );
    assert!(real_content > 0, "expected real-content blobs to validate");
    eprintln!(
        "OK: {total} blobs decoded; {byte_identical}/{total} byte-identical, \
         {real_content_identical}/{real_content} real-content byte-identical"
    );
}

#[test]
fn pc_default_matches_dump_configs() {
    let Some(dump) = [
        std::env::var("LOOTSMITH_ASSET_LIBRARY").ok(),
        Some(format!(
            "{}/../bl2-rogue/data/gibbed/Asset Library Manager.json",
            env!("CARGO_MANIFEST_DIR")
        )),
    ]
    .into_iter()
    .flatten()
    .map(std::path::PathBuf::from)
    .find(|p| p.exists()) else {
        eprintln!("no dumps file; skipping cross-check");
        return;
    };

    let loaded = AssetLibraryConfigs::from_path(&dump).expect("load dump");
    let default = AssetLibraryConfigs::pc_default();
    assert_eq!(loaded.version, default.version);
    for group in lootsmith_core::AssetGroup::ALL {
        assert_eq!(
            loaded.config(group),
            default.config(group),
            "config mismatch for {}",
            group.key()
        );
    }
    eprintln!("pc_default() matches {}", dump.display());
}
