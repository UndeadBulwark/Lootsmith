//! Packed item/weapon codec gate on the synthetic save: every blob must decode,
//! re-encode semantically, and (for real-content blobs) byte-identically.

mod common;

use common::synthetic_file;
use lootsmith_core::gameinfo::AssetLibraryConfigs;
use lootsmith_core::items;
use lootsmith_core::proto::Message;

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
fn synthetic_items_decode_and_reencode() {
    let configs = AssetLibraryConfigs::pc_default();
    assert_eq!(configs.version, 7, "unexpected asset library version");
    let save = synthetic_file();

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

/// `pc_default()` must match the configs the bundled asset library was built
/// with — no external dumps folder needed.
#[test]
fn pc_default_matches_bundled_configs() {
    let dump = format!(
        "{}/assets/data/asset-library-manager.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let loaded = AssetLibraryConfigs::from_path(std::path::Path::new(&dump)).expect("load bundled");
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
}
