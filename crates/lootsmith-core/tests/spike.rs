//! Container + Huffman codec gate on the synthetic save.
//!
//! Gate note: the *protobuf payload* is the byte-identity target. The Huffman
//! layer is verified by decode->encode->decode stability plus CRC32, because the
//! reference tool re-encodes Huffman with its own (different but valid) tree.

mod common;

use common::synthetic_file;
use lootsmith_core::{container, huffman, Platform};

#[test]
fn synthetic_save_unwraps_crc_and_huffman_is_stable() {
    let save = synthetic_file();

    assert_eq!(save.version, 2, "unexpected inner version");
    assert!(save.crc32_valid(), "stored CRC-32 does not match protobuf");
    assert!(!save.proto.is_empty(), "empty protobuf payload");
    assert!(!save.inner.is_empty(), "empty inner buffer");

    // The original Huffman stream must decode to the protobuf whose CRC-32 we
    // just verified.
    let inner_size =
        u32::from_be_bytes([save.inner[0], save.inner[1], save.inner[2], save.inner[3]]) as usize;
    let huff = &save.inner[19..19 + (inner_size - 15)];
    let decoded = huffman::decode(huff, save.proto.len()).expect("decode original huffman");
    assert_eq!(decoded, save.proto, "original huffman stream decoded wrong");

    // Our encoder must produce a stream that decodes back to the same bytes.
    let mine = huffman::encode(&save.proto).expect("encode");
    let round = huffman::decode(&mine, save.proto.len()).expect("decode ours");
    assert_eq!(round, save.proto, "huffman encode/decode is not stable");
}

#[test]
fn container_round_trips_through_serialize() {
    let save = synthetic_file();
    let bytes = container::serialize(&save.proto, Platform::PC).expect("serialize");
    let again = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(again.crc32_valid() && again.proto_stable() && again.huffman_stable());
    assert_eq!(again.proto, save.proto);
}
