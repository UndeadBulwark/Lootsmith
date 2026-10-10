//! Gibbed's Gearbox Huffman codec (custom inner compression).
//!
//! Faithful port of `Gibbed.Gearbox.Common/Huffman/{Encoder,Decoder}.cs`.
//! Bits are MSB-first within each byte. The tree is serialized in pre-order:
//! a leaf is `1` followed by 8 symbol bits; an internal node is `0` followed by
//! its left then right subtree.
//!
//! The encoder reproduces the reference's exact output size, which includes a
//! deliberate over-allocation (`GetTotalBits` adds 5000 bits; the output is
//! `1 + total_bits / 8` bytes, zero-padded). Matching this matters for
//! byte-identical inner buffers.

use crate::error::{Error, Result};
use std::collections::VecDeque;

struct MsbBits<'a> {
    data: &'a [u8],
    len: usize,
}

impl<'a> MsbBits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            len: data.len() * 8,
        }
    }

    fn bit(&self, i: usize) -> bool {
        (self.data[i >> 3] >> (7 - (i & 7))) & 1 != 0
    }
}

#[derive(Clone, Copy, Default)]
struct Node {
    symbol: u8,
    is_leaf: bool,
    left: i32,
    right: i32,
}

const TREE_CAPACITY: usize = 511;

fn decode_node(
    bits: &MsbBits,
    offset: &mut usize,
    tree: &mut [Node],
    index: &mut usize,
) -> Result<i32> {
    if *index >= tree.len() {
        return Err(Error::Huffman("tree overflow".into()));
    }
    let current = *index;
    *index += 1;

    if *offset >= bits.len {
        return Err(Error::EndOfStream);
    }
    let is_leaf = bits.bit(*offset);
    *offset += 1;

    if is_leaf {
        if *offset + 8 > bits.len {
            return Err(Error::EndOfStream);
        }
        let mut value = 0u8;
        for i in 0..8 {
            value |= (bits.bit(*offset) as u8) << (7 - i);
            *offset += 1;
        }
        tree[current] = Node {
            symbol: value,
            is_leaf: true,
            left: -1,
            right: -1,
        };
    } else {
        let left = decode_node(bits, offset, tree, index)?;
        let right = decode_node(bits, offset, tree, index)?;
        tree[current] = Node {
            symbol: 0,
            is_leaf: false,
            left,
            right,
        };
    }

    Ok(current as i32)
}

/// Decode a Huffman-compressed block into exactly `max_length` bytes.
pub fn decode(input: &[u8], max_length: usize) -> Result<Vec<u8>> {
    let bits = MsbBits::new(input);
    let mut tree = vec![Node::default(); TREE_CAPACITY];
    let mut index = 0usize;
    let mut offset = 0usize;
    decode_node(&bits, &mut offset, &mut tree, &mut index)?;

    let mut out = Vec::with_capacity(max_length);
    while out.len() < max_length {
        let mut idx = 0usize;
        loop {
            if idx >= tree.len() {
                return Err(Error::Huffman("invalid node index".into()));
            }
            let node = tree[idx];
            if node.is_leaf {
                out.push(node.symbol);
                break;
            }
            if offset >= bits.len {
                return Err(Error::EndOfStream);
            }
            let branch = bits.bit(offset);
            offset += 1;
            let next = if branch { node.right } else { node.left };
            if next < 0 {
                return Err(Error::Huffman("dangling branch".into()));
            }
            idx = next as usize;
        }
    }
    Ok(out)
}

struct Enode {
    symbol: u8,
    freq: u64,
    left: Option<usize>,
    right: Option<usize>,
}

fn build_arena(bytes: &[u8]) -> (Vec<Enode>, Vec<usize>) {
    // Frequencies ordered by first occurrence, matching .NET Dictionary
    // insertion order (which drives the reference's tie-breaking).
    let mut arena: Vec<Enode> = Vec::new();
    let mut index_of = std::collections::HashMap::<u8, usize>::new();
    let mut order: Vec<usize> = Vec::new();
    for &b in bytes {
        if let Some(&i) = index_of.get(&b) {
            arena[i].freq += 1;
        } else {
            let i = arena.len();
            arena.push(Enode {
                symbol: b,
                freq: 1,
                left: None,
                right: None,
            });
            index_of.insert(b, i);
            order.push(i);
        }
    }
    (arena, order)
}

fn collect_codes(arena: &[Enode], node: usize, path: &mut Vec<bool>, codes: &mut [Vec<bool>]) {
    let n = &arena[node];
    if n.left.is_none() && n.right.is_none() {
        codes[n.symbol as usize] = path.clone();
        return;
    }
    if let Some(l) = n.left {
        path.push(false);
        collect_codes(arena, l, path, codes);
        path.pop();
    }
    if let Some(r) = n.right {
        path.push(true);
        collect_codes(arena, r, path, codes);
        path.pop();
    }
}

fn total_bits(arena: &[Enode], root: usize) -> u64 {
    let mut queue = VecDeque::new();
    queue.push_back(root);
    let mut total: u64 = 0;
    while let Some(n) = queue.pop_front() {
        let node = &arena[n];
        if node.left.is_none() && node.right.is_none() {
            total += 1 + 8; // leaf: marker bit + symbol byte
            continue;
        }
        total += 1; // internal marker bit
        total += node.freq;
        if let Some(l) = node.left {
            if arena[l].left.is_some() {
                queue.push_back(l);
            }
        }
        if let Some(r) = node.right {
            if arena[r].left.is_some() {
                queue.push_back(r);
            }
        }
    }
    total + 5000
}

fn encode_node(arena: &[Enode], node: usize, bits: &mut [bool], pos: &mut usize) -> Result<()> {
    let n = &arena[node];
    if n.left.is_none() && n.right.is_none() {
        if *pos >= bits.len() {
            return Err(Error::Huffman("bit overflow (tree)".into()));
        }
        bits[*pos] = true;
        *pos += 1;
        for i in 0..8 {
            if *pos >= bits.len() {
                return Err(Error::Huffman("bit overflow (symbol)".into()));
            }
            bits[*pos] = (n.symbol >> (7 - i)) & 1 == 1;
            *pos += 1;
        }
    } else {
        if *pos >= bits.len() {
            return Err(Error::Huffman("bit overflow (branch)".into()));
        }
        bits[*pos] = false;
        *pos += 1;
        let l = n.left.expect("internal node has left");
        let r = n.right.expect("internal node has right");
        encode_node(arena, l, bits, pos)?;
        encode_node(arena, r, bits, pos)?;
    }
    Ok(())
}

/// Encode bytes with the Gearbox Huffman codec, reproducing the reference
/// output length exactly.
pub fn encode(bytes: &[u8]) -> Result<Vec<u8>> {
    if bytes.is_empty() {
        return Err(Error::Huffman("cannot encode empty input".into()));
    }

    let (mut arena, mut nodes) = build_arena(bytes);

    while nodes.len() > 1 {
        let mut ordered = nodes.clone();
        ordered.sort_by_key(|&i| arena[i].freq); // stable sort
        let first = ordered[0];
        let second = ordered[1];

        let parent = arena.len();
        let freq = arena[first].freq + arena[second].freq;
        arena.push(Enode {
            symbol: 0,
            freq,
            left: Some(first),
            right: Some(second),
        });

        nodes.retain(|&x| x != first && x != second);
        nodes.push(parent);
    }

    let root = nodes[0];

    let mut codes = vec![Vec::<bool>::new(); 256];
    collect_codes(&arena, root, &mut Vec::new(), &mut codes);

    let total = total_bits(&arena, root);
    let mut bits = vec![false; total as usize];
    let mut pos = 0usize;
    encode_node(&arena, root, &mut bits, &mut pos)?;

    for &b in bytes {
        for &bit in &codes[b as usize] {
            if pos >= bits.len() {
                return Err(Error::Huffman("bit overflow (payload)".into()));
            }
            bits[pos] = bit;
            pos += 1;
        }
    }

    let out_len = 1 + (total as usize) / 8;
    let mut out = vec![0u8; out_len];
    for (i, &bit) in bits.iter().enumerate() {
        if bit {
            out[i >> 3] |= 1 << (7 - (i & 7));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::{decode, encode};

    #[test]
    fn round_trip_various() {
        for data in [
            b"hello, huffman world".to_vec(),
            vec![0u8; 1000],
            (0..=255u8).collect::<Vec<_>>(),
            b"the quick brown fox jumps over the lazy dog".to_vec(),
        ] {
            let encoded = encode(&data).expect("encode");
            let decoded = decode(&encoded, data.len()).expect("decode");
            assert_eq!(decoded, data);
        }
    }
}
