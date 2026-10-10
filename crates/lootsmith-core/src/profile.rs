//! Borderlands 2 `profile.bin` — golden keys, badass rank & tokens, badass
//! stat bonuses, and customization unlocks.
//!
//! File layout: `SHA-1 (20 bytes)` over the rest, `u32` (big-endian)
//! uncompressed size, `LZO1X` compressed entry stream. Decompressed, it is an
//! entry table: big-endian `u32` count, then entries
//! `[kind:1][id: BE u32][dataType:1][value][separator 0x00]` where kind is 1
//! for the first entry and 2 afterwards (0 marks a broken placeholder).
//!
//! Notable ids: 136 badass rank, 138 badass tokens, 143 badass stats
//! (custom base-32 string), 162 golden keys (binary triplets), 300
//! customization unlock flags (1 byte per customization, 0 = locked).

use crate::error::{Error, Result};
use std::path::Path;

pub const BADASS_RANK_ID: u32 = 136;
pub const BADASS_TOKENS_ID: u32 = 138;
pub const STATS_ID: u32 = 143;
pub const GOLDEN_KEYS_ID: u32 = 162;
pub const CUSTOMIZATIONS_ID: u32 = 300;

/// The 14 badass bonus stats, in file order.
pub const STAT_NAMES: [&str; 14] = [
    "Maximum health",
    "Shield capacity",
    "Shield recharge delay",
    "Shield recharge rate",
    "Melee damage",
    "Grenade damage",
    "Gun accuracy",
    "Gun damage",
    "Fire rate",
    "Recoil reduction",
    "Reload speed",
    "Elemental effect chance",
    "Elemental effect damage",
    "Critical hit damage",
];

const MAXIMUM_STAT_VALUE: f64 = 9_975_792.3;

/// The payload of one profile entry.
#[derive(Clone, Debug, PartialEq)]
pub enum ProfileValue {
    /// `Int32`: unsigned payload of a big-endian int32.
    I32(u32),
    /// `String`: US-ASCII text.
    Str(String),
    /// `Float`: big-endian f32.
    F32(f32),
    /// `Binary`: raw bytes.
    Bin(Vec<u8>),
    /// `Int8`: one byte.
    I8(u8),
    /// A "broken" placeholder entry (raw bytes, kind 0).
    Broken([u8; 6]),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProfileEntry {
    /// 1 for the first entry, 2 later, 0 for broken placeholders.
    pub kind: u8,
    pub id: u32,
    pub value: ProfileValue,
}

/// A parsed profile: the entry list, kept verbatim.
#[derive(Clone, Debug, Default)]
pub struct Profile {
    pub entries: Vec<ProfileEntry>,
}

impl Profile {
    /// Parse a whole `profile.bin` (verifies the header hash; decompresses).
    pub fn parse(bytes: &[u8]) -> Result<Profile> {
        if bytes.len() < 28 {
            return Err(Error::TooShort("profile header"));
        }
        let hash = &bytes[..20];
        let expected = hash_of(&bytes[20..]);
        if expected != *hash {
            return Err(Error::Sha1Mismatch);
        }
        let size = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]) as usize;
        let mut out = vec![0u8; size];
        lzo::decompress_into(&bytes[24..], &mut out).map_err(|e| Error::Decompress {
            scheme: "lzo",
            detail: e.to_string(),
        })?;
        Self::parse_entries(&out)
    }

    /// Read a `profile.bin` from disk.
    pub fn from_path(path: &Path) -> Result<Profile> {
        let bytes = std::fs::read(path).map_err(|e| Error::Io(e.to_string()))?;
        Self::parse(&bytes)
    }

    /// Parse just the decompressed entry table.
    pub fn parse_entries(data: &[u8]) -> Result<Profile> {
        if data.len() < 4 {
            return Err(Error::TooShort("profile entry count"));
        }
        let count = be_u32(data, 0) as usize;
        let mut pos = 4usize;
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            if pos >= data.len() {
                return Err(Error::TooShort("profile entry"));
            }
            let kind = data[pos];
            pos += 1;
            if kind == 0 {
                if pos + 6 > data.len() {
                    return Err(Error::TooShort("broken profile entry"));
                }
                let mut raw = [0u8; 6];
                raw.copy_from_slice(&data[pos..pos + 6]);
                pos += 6;
                entries.push(ProfileEntry {
                    kind,
                    id: 0,
                    value: ProfileValue::Broken(raw),
                });
                continue;
            }
            if pos + 5 > data.len() {
                return Err(Error::TooShort("profile entry header"));
            }
            let id = be_u32(data, pos);
            pos += 4;
            let data_type = data[pos];
            pos += 1;
            let value = match data_type {
                1 => {
                    if pos + 4 > data.len() {
                        return Err(Error::TooShort("profile int32"));
                    }
                    pos += 4;
                    ProfileValue::I32(be_u32(data, pos - 4))
                }
                4 => {
                    if pos + 4 > data.len() {
                        return Err(Error::TooShort("profile string length"));
                    }
                    let len = be_u32(data, pos) as usize;
                    pos += 4;
                    if pos + len > data.len() {
                        return Err(Error::TooShort("profile string"));
                    }
                    let text = String::from_utf8_lossy(&data[pos..pos + len]).into_owned();
                    pos += len;
                    ProfileValue::Str(text)
                }
                5 => {
                    if pos + 4 > data.len() {
                        return Err(Error::TooShort("profile float"));
                    }
                    let bits = be_u32(data, pos);
                    pos += 4;
                    ProfileValue::F32(f32::from_bits(bits))
                }
                6 => {
                    if pos + 4 > data.len() {
                        return Err(Error::TooShort("profile binary length"));
                    }
                    let len = be_u32(data, pos) as usize;
                    pos += 4;
                    if pos + len > data.len() {
                        return Err(Error::TooShort("profile binary"));
                    }
                    let bin = data[pos..pos + len].to_vec();
                    pos += len;
                    ProfileValue::Bin(bin)
                }
                8 => {
                    if pos >= data.len() {
                        return Err(Error::TooShort("profile int8"));
                    }
                    let byte = data[pos];
                    pos += 1;
                    ProfileValue::I8(byte)
                }
                other => {
                    return Err(Error::Proto(format!(
                        "unsupported profile data type {other}"
                    )));
                }
            };
            // One 0x00 separator byte after every real entry.
            if pos >= data.len() || data[pos] != 0 {
                return Err(Error::Proto("missing profile entry separator".into()));
            }
            pos += 1;
            entries.push(ProfileEntry { kind, id, value });
        }
        Ok(Profile { entries })
    }

    /// Serialize the decompressed entry table.
    pub fn entries_to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(self.entries.len() as u32).to_be_bytes());
        for entry in &self.entries {
            out.push(entry.kind);
            match &entry.value {
                ProfileValue::Broken(raw) => {
                    out.extend_from_slice(raw);
                }
                ProfileValue::I32(v) => {
                    out.extend_from_slice(&entry.id.to_be_bytes());
                    out.push(1);
                    out.extend_from_slice(&v.to_be_bytes());
                    out.push(0);
                }
                ProfileValue::Str(text) => {
                    out.extend_from_slice(&entry.id.to_be_bytes());
                    out.push(4);
                    out.extend_from_slice(&(text.len() as u32).to_be_bytes());
                    out.extend_from_slice(text.as_bytes());
                    out.push(0);
                }
                ProfileValue::F32(v) => {
                    out.extend_from_slice(&entry.id.to_be_bytes());
                    out.push(5);
                    out.extend_from_slice(&v.to_bits().to_be_bytes());
                    out.push(0);
                }
                ProfileValue::Bin(bin) => {
                    out.extend_from_slice(&entry.id.to_be_bytes());
                    out.push(6);
                    out.extend_from_slice(&(bin.len() as u32).to_be_bytes());
                    out.extend_from_slice(bin);
                    out.push(0);
                }
                ProfileValue::I8(v) => {
                    out.extend_from_slice(&entry.id.to_be_bytes());
                    out.push(8);
                    out.push(*v);
                    out.push(0);
                }
            }
        }
        out
    }

    /// Serialize a whole `profile.bin` (fresh hash, fresh LZO stream).
    pub fn to_bytes(&self) -> Vec<u8> {
        let inner = self.entries_to_bytes();
        let compressed = lzokay::compress::compress(&inner).expect("lzo compress");
        let mut body = Vec::with_capacity(4 + compressed.len());
        body.extend_from_slice(&(inner.len() as u32).to_be_bytes());
        body.extend_from_slice(&compressed);
        let hash = hash_of(&body);
        let mut out = Vec::with_capacity(20 + body.len());
        out.extend_from_slice(&hash);
        out.extend_from_slice(&body);
        out
    }

    fn entry(&self, id: u32) -> Option<&ProfileEntry> {
        self.entries.iter().find(|e| e.id == id)
    }

    fn mut_entry(&mut self, id: u32) -> Option<&mut ProfileEntry> {
        self.entries.iter_mut().find(|e| e.id == id)
    }

    /// Golden keys (id 162, binary triplets `[index, a, b]`; keys = `(a - b)`).
    pub fn golden_keys(&self) -> Option<i64> {
        match &self.entry(GOLDEN_KEYS_ID)?.value {
            ProfileValue::Bin(data) => Some(golden_keys_decode(data)),
            _ => None,
        }
    }

    /// Set golden keys (preserving any unused-key bookkeeping pattern).
    pub fn set_golden_keys(&mut self, keys: i64) {
        let data = golden_keys_encode(keys);
        match self.mut_entry(GOLDEN_KEYS_ID) {
            Some(entry) => entry.value = ProfileValue::Bin(data),
            None => self.entries.push(ProfileEntry {
                kind: 2,
                id: GOLDEN_KEYS_ID,
                value: ProfileValue::Bin(data),
            }),
        }
    }

    /// Badass rank (id 136).
    pub fn badass_rank(&self) -> Option<u32> {
        match &self.entry(BADASS_RANK_ID)?.value {
            ProfileValue::I32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn set_badass_rank(&mut self, rank: u32) {
        self.set_i32(BADASS_RANK_ID, rank);
    }

    /// Unspent badass tokens (id 138).
    pub fn badass_tokens(&self) -> Option<u32> {
        match &self.entry(BADASS_TOKENS_ID)?.value {
            ProfileValue::I32(v) => Some(*v),
            _ => None,
        }
    }

    pub fn set_badass_tokens(&mut self, tokens: u32) {
        self.set_i32(BADASS_TOKENS_ID, tokens);
    }

    fn set_i32(&mut self, id: u32, value: u32) {
        match self.mut_entry(id) {
            Some(entry) => entry.value = ProfileValue::I32(value),
            None => self.entries.push(ProfileEntry {
                kind: 2,
                id,
                value: ProfileValue::I32(value),
            }),
        }
    }

    /// The 14 badass bonus stat values, decoded (roughly "percent" units).
    pub fn stats(&self) -> Result<[f64; 14]> {
        let entry = self
            .entry(STATS_ID)
            .ok_or_else(|| Error::Proto("no badass stats entry".into()))?;
        match &entry.value {
            ProfileValue::Str(text) => stats_decode(text),
            _ => Err(Error::Proto("badass stats entry is not a string".into())),
        }
    }

    /// Whether the profile carries a badass-stats entry (id 143).
    pub fn has_stats(&self) -> bool {
        matches!(
            self.entry(STATS_ID).map(|e| &e.value),
            Some(ProfileValue::Str(_))
        )
    }

    /// Replace the badass bonus stats.
    pub fn set_stats(&mut self, stats: &[f64; 14]) -> Result<()> {
        let text = stats_encode(stats);
        match self.mut_entry(STATS_ID) {
            Some(entry) => entry.value = ProfileValue::Str(text),
            None => {
                return Err(Error::Proto(
                    "no badass stats entry to replace (add one by editing in game first)".into(),
                ))
            }
        }
        Ok(())
    }

    /// Every customization unlock byte (id 300; 0 = locked, 255 = unlocked).
    pub fn customizations(&self) -> Option<&[u8]> {
        match &self.entry(CUSTOMIZATIONS_ID)?.value {
            ProfileValue::Bin(data) => Some(data),
            _ => None,
        }
    }

    /// Unlock every head/skin/customization (all bytes 255, same length).
    pub fn unlock_all_customizations(&mut self) -> Result<()> {
        let Some(data) = self.customizations().map(<[u8]>::to_vec) else {
            return Err(Error::Proto("no customizations entry".into()));
        };
        let data = data.into_iter().map(|_| 255u8).collect();
        if let Some(entry) = self.mut_entry(CUSTOMIZATIONS_ID) {
            entry.value = ProfileValue::Bin(data);
        }
        Ok(())
    }

    /// Whether every customization is unlocked (all bytes non-zero).
    pub fn all_customizations_unlocked(&self) -> bool {
        self.customizations()
            .is_some_and(|d| !d.is_empty() && d.iter().all(|b| *b != 0))
    }
}

/// Big-endian u32 at `pos`.
fn be_u32(data: &[u8], pos: usize) -> u32 {
    u32::from_be_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
}

/// SHA-1 of `data` (the profile hash covers size + payload).
fn hash_of(data: &[u8]) -> [u8; 20] {
    use sha1::Digest;
    sha1::Sha1::new_with_prefix(data).finalize().into()
}

/// Sum of `(bytes[i + 1] - bytes[i + 2])` per `[index, a, b]` triplet.
fn golden_keys_decode(data: &[u8]) -> i64 {
    let mut keys = 0i64;
    for chunk in data.chunks_exact(3) {
        keys += chunk[1] as i64 - chunk[2] as i64;
    }
    keys
}

/// Encode golden keys as `[index, value, 0]` triplets (255 per entry).
fn golden_keys_encode(keys: i64) -> Vec<u8> {
    let keys = keys.clamp(0, i32::MAX as i64);
    let mut count = 1usize;
    if keys > 255 {
        count = (keys as f64 / 255.0).ceil() as usize;
    }
    count = count.max(1);
    let mut data = vec![0u8; count * 3];
    for (index, chunk) in data.chunks_exact_mut(3).enumerate() {
        chunk[0] = index as u8;
        if index + 1 == count {
            chunk[1] = (keys - (index as i64) * 255).clamp(0, 255) as u8;
        } else {
            chunk[1] = 255;
        }
    }
    data
}

const STATS_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const STATS_BASE_VALUE: u32 = 2_587_251_417;

/// Decode the badass-stats string into 14 bonus values.
fn stats_decode(text: &str) -> Result<[f64; 14]> {
    if text.is_empty() {
        // A fresh profile with badass rank 0 stores an empty string.
        return Ok([0.0; 14]);
    }
    let mut num1 = STATS_BASE_VALUE;
    let mut num2 = 0u32;
    let mut words: Vec<u32> = Vec::new();
    for c in text.chars() {
        let Some(digit) = STATS_ALPHABET.iter().position(|b| *b == c as u8) else {
            return Err(Error::Proto(format!("bad badass-stats character '{c}'")));
        };
        num1 ^= (digit as u32).wrapping_shl(num2 % 32);
        num2 += 5;
        if num2 > 31 {
            words.push(num1.swap_bytes());
            num2 &= 7;
            num1 = STATS_BASE_VALUE
                ^ (digit as u32).wrapping_shr((5i32 - num2 as i32).rem_euclid(32) as u32);
        }
    }
    if words.len() != 14 {
        return Err(Error::Proto(format!(
            "badass stats decode produced {} values, expected 14",
            words.len()
        )));
    }
    let mut out = [0.0; 14];
    for (i, word) in words.iter().enumerate() {
        let unrounded = (*word as f64).powf(0.75);
        out[i] = unrounded.min(MAXIMUM_STAT_VALUE);
    }
    Ok(out)
}

/// Encode 14 bonus values into the badass-stats string.
fn stats_encode(stats: &[f64; 14]) -> String {
    let mut values: Vec<u32> = Vec::with_capacity(14);
    for &stat in stats {
        let unsigned = (stat.powf(4.0 / 3.0).round() as u64).clamp(0, u32::MAX as u64) as u32;
        values.push(unsigned.swap_bytes());
    }
    let mut num1 = 0u32;
    let mut num2 = 0i32;
    let mut out = String::new();
    for &val in &values {
        let num3 = val ^ STATS_BASE_VALUE;
        if num2 > 0 {
            let num4 = (num1 | num3.wrapping_shl(num2.rem_euclid(32) as u32)) & 31;
            out.push(STATS_ALPHABET[num4 as usize] as char);
            num2 = 5 - num2;
        }
        while num2 < 28 {
            out.push(STATS_ALPHABET[(num3.wrapping_shr(num2.rem_euclid(32) as u32) & 31) as usize]
                as char);
            num2 += 5;
        }
        num1 = num3.wrapping_shr(num2.rem_euclid(32) as u32);
        num2 = 32 - num2;
    }
    if num2 > 0 {
        out.push(STATS_ALPHABET[(num1 & 31) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Path of a real profile.bin for the in-place round-trip test, if present.
    fn real_profile() -> Option<std::path::PathBuf> {
        let read = std::fs::read_dir(home_savedata()?).ok()?;
        for entry in read.flatten() {
            let path = entry.path().join("profile.bin");
            if path.exists() {
                return Some(path);
            }
        }
        None
    }

    fn home_savedata() -> Option<std::path::PathBuf> {
        let base = std::env::var("HOME").ok()?;
        let mut path = std::path::PathBuf::from(base);
        for part in [
            ".local/share",
            "aspyr-media",
            "borderlands 2",
            "willowgame",
            "savedata",
        ] {
            path.push(part);
        }
        Some(path)
    }

    #[test]
    fn empty_entry_table_round_trips() {
        let profile = Profile::default();
        let table = profile.entries_to_bytes();
        let parsed = Profile::parse_entries(&table).expect("parse");
        assert!(parsed.entries.is_empty());
    }

    #[test]
    fn synthetic_profile_round_trips() {
        let mut profile = Profile::default();
        profile.entries.push(ProfileEntry {
            kind: 1,
            id: BADASS_RANK_ID,
            value: ProfileValue::I32(1_315),
        });
        profile.entries.push(ProfileEntry {
            kind: 2,
            id: GOLDEN_KEYS_ID,
            value: ProfileValue::Bin(golden_keys_encode(52)),
        });
        assert_eq!(profile.golden_keys(), Some(52));
        assert_eq!(profile.badass_rank(), Some(1_315));

        let table = profile.entries_to_bytes();
        let parsed = Profile::parse_entries(&table).expect("parse");
        assert_eq!(parsed.entries, profile.entries);
    }

    #[test]
    fn golden_keys_above_255_round_trip() {
        let data = golden_keys_encode(765);
        assert_eq!(golden_keys_decode(&data), 765);
        let data = golden_keys_encode(255);
        assert_eq!(golden_keys_decode(&data), 255);
        let data = golden_keys_encode(0);
        assert_eq!(golden_keys_decode(&data), 0);
        let data = golden_keys_encode(300);
        assert_eq!(golden_keys_decode(&data), 300);
    }

    #[test]
    fn stats_codec_round_trips_and_stays_in_range() {
        let mut stats = [0.0f64; 14];
        for (i, v) in stats.iter_mut().enumerate() {
            *v = 5.0 + i as f64 * 1.25;
        }
        let text = stats_encode(&stats);
        let decoded = stats_decode(&text).expect("decode");
        for (a, b) in stats.iter().zip(decoded.iter()) {
            assert!((a - b).abs() < 1.0, "{a} vs {b}");
        }
    }

    #[test]
    fn real_profile_round_trips() {
        let Some(path) = real_profile() else {
            eprintln!("no profile.bin found; skipping");
            return;
        };
        let profile = Profile::from_path(&path).expect("parse real profile");
        // Re-serialize: entries must survive the trip unchanged.
        let table = profile.entries_to_bytes();
        let parsed = Profile::parse_entries(&table).expect("re-parse");
        assert_eq!(parsed.entries, profile.entries);
        // Typed values are sane.
        if let Some(keys) = profile.golden_keys() {
            assert!((0..=100_000).contains(&keys), "golden keys {keys}");
        }
        if let Ok(stats) = profile.stats() {
            assert_eq!(stats.len(), 14);
            assert!(stats.iter().all(|v| v.is_finite() && *v >= 0.0));
        }
        let _ = std::fs::write("/tmp/opencode/profile_rt.bin", profile.to_bytes());
        let reloaded = Profile::from_path(Path::new("/tmp/opencode/profile_rt.bin"))
            .expect("round-trip file parses");
        assert_eq!(reloaded.entries, profile.entries);
    }
}