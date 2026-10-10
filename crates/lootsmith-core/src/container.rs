//! Borderlands 2 save container.
//!
//! Ported from `FileFormats/SaveFile.cs` (deserialize/serialize path through
//! decompression and the inner WSG header).
//!
//! Layout (compressed platforms, e.g. PC):
//!
//! ```text
//! File   = SHA1(20) || u32BE(inner_len) || compress(Inner)
//! Inner  = u32BE(inner_size = len(Inner) - 4) || "WSG" || u32(version)
//!          || u32(crc32) || i32(proto_size) || Huffman(proto)   // endian = platform
//! ```
//!
//! Switch (compression = None) writes `Inner` without the length prefix or the
//! SHA1 wrapper.

use crate::crc32::crc32;
use crate::huffman;
use crate::platform::{CompressionScheme, Endian, Platform};
use crate::savegame::SaveGame;
use crate::{Error, Result};

/// Block size used by the outer block table (`SaveFile.BlockSize`).
pub const BLOCK_SIZE: usize = 0x40000;

const MAGIC: &[u8; 3] = b"WSG";
const X360_CON_MAGIC: u32 = 0x434F_4E20; // "CON "

/// A decoded save's container layers.
#[derive(Debug, Clone)]
pub struct SaveFile {
    pub platform: Platform,
    /// The exact unwrapped inner buffer (WSG header + Huffman bytes). This is
    /// the byte-identity gate.
    pub inner: Vec<u8>,
    pub version: u32,
    /// The CRC-32 stored in the inner header.
    pub crc32: u32,
    /// The Huffman-decoded protobuf payload (`WillowTwoPlayerSaveGame`).
    pub proto: Vec<u8>,
}

impl SaveFile {
    /// Whether the stored CRC-32 matches the decoded protobuf payload.
    pub fn crc32_valid(&self) -> bool {
        crc32(&self.proto) == self.crc32
    }

    /// Whether the Huffman layer is self-consistent: re-encoding the protobuf
    /// and decoding it again returns the same bytes. The game's original tree
    /// need not match ours (the reference tool re-encodes with its own tree),
    /// so this — not raw byte identity — is the Huffman gate.
    pub fn huffman_stable(&self) -> bool {
        match huffman::encode(&self.proto) {
            Ok(encoded) => huffman::decode(&encoded, self.proto.len())
                .map(|decoded| decoded == self.proto)
                .unwrap_or(false),
            Err(_) => false,
        }
    }

    /// Whether the protobuf payload survives a parse/re-emit byte-for-byte.
    /// This is the primary fidelity gate (the reference checks the same thing
    /// via `IgnoreReencodeMismatch`).
    pub fn proto_stable(&self) -> bool {
        match crate::proto::Message::parse(&self.proto) {
            Ok(message) => message.to_bytes() == self.proto,
            Err(_) => false,
        }
    }

    /// Parse the protobuf payload as a typed `WillowTwoPlayerSaveGame`.
    pub fn save_game(&self) -> Result<SaveGame> {
        SaveGame::parse(&self.proto)
    }
}

fn u32_at(buf: &[u8], off: usize, endian: Endian) -> Option<u32> {
    let b = buf.get(off..off + 4)?;
    let arr = [b[0], b[1], b[2], b[3]];
    Some(match endian {
        Endian::Little => u32::from_le_bytes(arr),
        Endian::Big => u32::from_be_bytes(arr),
    })
}

fn push_u32(out: &mut Vec<u8>, value: u32, endian: Endian) {
    match endian {
        Endian::Little => out.extend_from_slice(&value.to_le_bytes()),
        Endian::Big => out.extend_from_slice(&value.to_be_bytes()),
    }
}

fn sha1(data: &[u8]) -> [u8; 20] {
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    hasher.update(data);
    let digest = hasher.finalize();
    let mut out = [0u8; 20];
    out.copy_from_slice(&digest);
    out
}

/// Decompress an outer payload of known uncompressed length.
fn decompress(compressed: &[u8], inner_len: usize, scheme: CompressionScheme) -> Result<Vec<u8>> {
    let mut out = vec![0u8; inner_len];

    if inner_len <= BLOCK_SIZE {
        inflate_block(compressed, &mut out, scheme)?;
    } else {
        let block_count = u32_at(compressed, 0, Endian::Big)
            .ok_or(Error::TooShort("block table header"))? as usize;
        let mut off = 4usize;
        let mut blocks = Vec::with_capacity(block_count);
        for _ in 0..block_count {
            let c = u32_at(compressed, off, Endian::Big)
                .ok_or(Error::TooShort("block table entry"))? as usize;
            off += 4;
            let u = u32_at(compressed, off, Endian::Big)
                .ok_or(Error::TooShort("block table entry"))? as usize;
            off += 4;
            blocks.push((c, u));
        }

        let mut written = 0usize;
        for (c, u) in blocks {
            let end = off
                .checked_add(c)
                .ok_or(Error::TooShort("compressed block"))?;
            let chunk = compressed
                .get(off..end)
                .ok_or(Error::TooShort("compressed block"))?;
            off = end;
            let take = u.min(inner_len - written);
            inflate_block(chunk, &mut out[written..written + take], scheme)?;
            written += take;
        }
        if written != inner_len {
            return Err(Error::SizeMismatch {
                expected: inner_len,
                actual: written,
            });
        }
    }

    Ok(out)
}

fn inflate_block(compressed: &[u8], out: &mut [u8], scheme: CompressionScheme) -> Result<()> {
    match scheme {
        CompressionScheme::Lzo => {
            let n = lzo::decompress_into(compressed, out).map_err(|e| Error::Decompress {
                scheme: "lzo",
                detail: format!("{e:?}"),
            })?;
            if n != out.len() {
                return Err(Error::SizeMismatch {
                    expected: out.len(),
                    actual: n,
                });
            }
            Ok(())
        }
        CompressionScheme::Zlib => {
            use flate2::read::ZlibDecoder;
            use std::io::Read;
            let mut decoder = ZlibDecoder::new(compressed);
            decoder.read_exact(out).map_err(|e| Error::Decompress {
                scheme: "zlib",
                detail: e.to_string(),
            })?;
            Ok(())
        }
        CompressionScheme::None => Err(Error::UnsupportedPlatform),
    }
}

fn parse_inner(inner: &[u8], platform: Platform, has_len_prefix: bool) -> Result<SaveFile> {
    let endian = platform.endian();

    let (inner_size, mut off) = if has_len_prefix {
        let size = u32_at(inner, 0, Endian::Big).ok_or(Error::TooShort("inner size"))? as usize;
        (size, 4usize)
    } else {
        (inner.len(), 0usize)
    };

    if inner.get(off..off + 3) != Some(MAGIC.as_slice()) {
        return Err(Error::BadMagic);
    }
    off += 3;

    let version = u32_at(inner, off, endian).ok_or(Error::TooShort("version"))?;
    if version != 2 {
        return Err(Error::UnsupportedVersion(version));
    }
    off += 4;

    let crc = u32_at(inner, off, endian).ok_or(Error::TooShort("crc32"))?;
    off += 4;

    let proto_size = u32_at(inner, off, endian).ok_or(Error::TooShort("proto size"))? as i32;
    if proto_size < 0 {
        return Err(Error::Huffman("negative proto size".into()));
    }
    off += 4;

    let huff_len = inner_size
        .checked_sub(15)
        .ok_or(Error::TooShort("huffman payload"))?;
    let huff = inner
        .get(off..off + huff_len)
        .ok_or(Error::TooShort("huffman payload"))?;

    let proto = huffman::decode(huff, proto_size as usize)?;

    Ok(SaveFile {
        platform,
        inner: inner.to_vec(),
        version,
        crc32: crc,
        proto,
    })
}

/// Unwrap and decode a save file for the given platform.
pub fn unpack(data: &[u8], platform: Platform) -> Result<SaveFile> {
    if data.len() >= 4 && u32_at(data, 0, Endian::Big) == Some(X360_CON_MAGIC) {
        return Err(Error::Xbox360ConFile);
    }

    match platform.compression_scheme() {
        CompressionScheme::None => parse_inner(data, platform, false),
        scheme => {
            if data.len() < 24 {
                return Err(Error::TooShort("outer header"));
            }
            let stored_sha = &data[0..20];
            let wrapper = &data[20..];

            if sha1(wrapper) != stored_sha {
                return Err(Error::Sha1Mismatch);
            }

            let inner_len =
                u32_at(wrapper, 0, Endian::Big).ok_or(Error::TooShort("outer length"))? as usize;
            let compressed = &wrapper[4..];
            let inner = decompress(compressed, inner_len, scheme)?;
            parse_inner(&inner, platform, true)
        }
    }
}

/// Rebuild the exact inner buffer (WSG header + Huffman bytes) from a protobuf
/// payload. For compressed platforms this includes the leading `u32BE` prefix.
pub fn pack_inner(proto: &[u8], platform: Platform) -> Result<Vec<u8>> {
    let endian = platform.endian();
    let scheme = platform.compression_scheme();

    let huff = huffman::encode(proto)?;

    let mut body = Vec::with_capacity(15 + huff.len());
    body.extend_from_slice(MAGIC);
    push_u32(&mut body, 2, endian);
    push_u32(&mut body, crc32(proto), endian);
    push_u32(&mut body, proto.len() as u32, endian);
    body.extend_from_slice(&huff);

    if scheme == CompressionScheme::None {
        Ok(body)
    } else {
        let mut inner = Vec::with_capacity(4 + body.len());
        push_u32(&mut inner, body.len() as u32, Endian::Big);
        inner.extend_from_slice(&body);
        Ok(inner)
    }
}
