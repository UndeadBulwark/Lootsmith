use lootsmith_core::{container, huffman, Platform};

fn main() {
    let path = format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    );
    let data = std::fs::read(&path).expect("read fixture");
    let save = container::unpack(&data, Platform::PC).expect("unpack");

    let inner = &save.inner;
    let inner_size = u32::from_be_bytes([inner[0], inner[1], inner[2], inner[3]]) as usize;
    let huff_len = inner_size - 15;
    let orig = &inner[19..19 + huff_len];

    let enc = huffman::encode(&save.proto).expect("encode");

    println!("proto len        : {}", save.proto.len());
    println!("original huff len: {}", orig.len());
    println!("my encoded len   : {}", enc.len());
    println!(
        "len delta        : {}",
        enc.len() as i64 - orig.len() as i64
    );

    let n = orig.len().min(enc.len());
    match (0..n).find(|&i| orig[i] != enc[i]) {
        Some(i) => println!(
            "first byte diff  : {i} (orig {:02x} enc {:02x})",
            orig[i], enc[i]
        ),
        None => println!("prefix identical : up to {n} bytes"),
    }

    // Does my encoding decode back to the same protobuf?
    let rt = huffman::decode(&enc, save.proto.len()).expect("decode mine");
    println!("my encode decodes back to proto: {}", rt == save.proto);
    let rt_orig = huffman::decode(orig, save.proto.len()).expect("decode orig");
    println!("original decodes to proto      : {}", rt_orig == save.proto);

    // How many distinct byte values and the top frequencies?
    let mut counts = [0usize; 256];
    for &b in &save.proto {
        counts[b as usize] += 1;
    }
    let distinct = counts.iter().filter(|&&c| c > 0).count();
    println!("distinct byte values: {distinct}");
    let mut pairs: Vec<(u8, usize)> = counts
        .iter()
        .enumerate()
        .filter(|(_, &c)| c > 0)
        .map(|(b, &c)| (b as u8, c))
        .collect();
    pairs.sort_by_key(|&(b, c)| (std::cmp::Reverse(c), b));
    println!("top 10 (byte,count): {:?}", &pairs[..10.min(pairs.len())]);
}
