//! Phase 0.5 spike: validate the container + Huffman codec against a real PC
//! save. The fixture is git-ignored; the test is skipped when it is absent.
//!
//! Gate note: the *protobuf payload* is the byte-identity target. The Huffman
//! layer is verified by decode->encode->decode stability plus CRC32, because
//! the reference tool re-encodes Huffman with its own (different but valid)
//! tree and only checks protobuf reencode equality (`IgnoreReencodeMismatch`).

use lootsmith_core::{container, huffman, Platform};

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

#[test]
fn pc_save_unwraps_crc_and_huffman_is_stable() {
    let path = fixture_path();
    let data = match std::fs::read(&path) {
        Ok(d) => d,
        Err(_) => {
            eprintln!("fixture not present at {path}; skipping spike test");
            return;
        }
    };

    let save = container::unpack(&data, Platform::PC).expect("unpack PC save");

    assert_eq!(save.version, 2, "unexpected inner version");
    assert!(save.crc32_valid(), "stored CRC-32 does not match protobuf");
    assert!(!save.proto.is_empty(), "empty protobuf payload");
    assert!(!save.inner.is_empty(), "empty inner buffer");

    // The original Huffman stream must decode to the protobuf whose CRC-32 we
    // just verified. (Re-decoding from the raw inner bytes proves the decoder
    // path end to end.)
    let inner_size =
        u32::from_be_bytes([save.inner[0], save.inner[1], save.inner[2], save.inner[3]]) as usize;
    let huff = &save.inner[19..19 + (inner_size - 15)];
    let decoded = huffman::decode(huff, save.proto.len()).expect("decode original huffman");
    assert_eq!(decoded, save.proto, "original huffman stream decoded wrong");

    // Our encoder must produce a stream that decodes back to the same bytes.
    let mine = huffman::encode(&save.proto).expect("encode");
    let round = huffman::decode(&mine, save.proto.len()).expect("decode ours");
    assert_eq!(round, save.proto, "huffman encode/decode is not stable");

    eprintln!(
        "OK: proto={} bytes, inner={} bytes, huffman orig={} mine={}, crc32={:#010x}",
        save.proto.len(),
        save.inner.len(),
        huff.len(),
        mine.len(),
        save.crc32
    );
}
