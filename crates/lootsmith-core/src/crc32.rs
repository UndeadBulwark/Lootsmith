//! CRC-32 (IEEE 802.3, reflected, init `0xFFFFFFFF`, final xor), matching
//! `Gibbed.Gearbox.Common.CRC32`.

/// Compute the Gibbed/Gearbox CRC-32 hash of a byte slice.
pub fn crc32(data: &[u8]) -> u32 {
    let mut hash: u32 = 0xFFFF_FFFF;
    for &b in data {
        hash ^= b as u32;
        for _ in 0..8 {
            let mask = (hash & 1).wrapping_neg();
            hash = (hash >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !hash
}

#[cfg(test)]
mod tests {
    use super::crc32;

    #[test]
    fn known_vector() {
        // Standard CRC-32 of "123456789".
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }
}
