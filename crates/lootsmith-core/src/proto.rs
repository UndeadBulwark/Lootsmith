//! A raw-preserving protobuf wire codec.
//!
//! Rather than regenerate a general runtime, Lootsmith keeps every field's
//! exact on-disk bytes. This makes decode -> encode byte-identical for any
//! input, including unknown fields and non-canonical encodings, which is the
//! fidelity gate the item/stat layers need. Typed views are layered on top in
//! [`crate::savegame`].
//!
//! This is a deliberate choice: the reference is protobuf-net, whose emission
//! rules (field order, `ShouldSerialize`, packed repeated, `Guid` shape) would
//! otherwise have to be reproduced exactly. Preserving raw bytes sidesteps
//! that for untouched fields.

use crate::error::{Error, Result};
use std::borrow::Cow;

/// Protobuf wire type.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WireType {
    Varint = 0,
    Fixed64 = 1,
    LengthDelimited = 2,
    StartGroup = 3,
    EndGroup = 4,
    Fixed32 = 5,
}

impl WireType {
    fn from_u8(v: u8) -> WireType {
        match v {
            0 => WireType::Varint,
            1 => WireType::Fixed64,
            2 => WireType::LengthDelimited,
            3 => WireType::StartGroup,
            4 => WireType::EndGroup,
            _ => WireType::Fixed32,
        }
    }
}

/// Read a base-128 varint, advancing `pos`.
pub fn read_varint(buf: &[u8], pos: &mut usize) -> Result<u64> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;
    loop {
        if *pos >= buf.len() {
            return Err(Error::Proto("truncated varint".into()));
        }
        let byte = buf[*pos];
        *pos += 1;
        result |= ((byte & 0x7f) as u64) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 64 {
            return Err(Error::Proto("varint too long".into()));
        }
    }
    Ok(result)
}

/// Encode a value as a base-128 varint.
pub fn write_varint(out: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            break;
        }
        out.push(byte | 0x80);
    }
}

/// Zig-zag decode for `sint32`/`sint64`.
pub fn zigzag_decode(value: u64) -> i64 {
    ((value >> 1) as i64) ^ -((value & 1) as i64)
}

/// One wire field, retaining its exact bytes.
#[derive(Clone, Debug)]
pub struct Field {
    pub number: u32,
    pub wire: WireType,
    /// Key varint followed by the value (length prefix + payload for LEN).
    raw: Vec<u8>,
    /// Offset into `raw` where the value begins (i.e. after the key varint).
    value_start: usize,
}

impl Field {
    fn from_parts(number: u32, wire: WireType, value: Vec<u8>) -> Field {
        let mut key = Vec::new();
        write_varint(&mut key, ((number as u64) << 3) | (wire as u64));
        let value_start = key.len();
        let mut raw = key;
        raw.extend_from_slice(&value);
        Field {
            number,
            wire,
            raw,
            value_start,
        }
    }

    /// Canonical varint field.
    pub fn varint(number: u32, value: u64) -> Field {
        let mut v = Vec::new();
        write_varint(&mut v, value);
        Field::from_parts(number, WireType::Varint, v)
    }

    /// Canonical length-delimited (bytes / string / sub-message) field.
    pub fn bytes(number: u32, payload: &[u8]) -> Field {
        let mut v = Vec::new();
        write_varint(&mut v, payload.len() as u64);
        v.extend_from_slice(payload);
        Field::from_parts(number, WireType::LengthDelimited, v)
    }

    pub fn string(number: u32, value: &str) -> Field {
        Field::bytes(number, value.as_bytes())
    }

    pub fn fixed32(number: u32, value: u32) -> Field {
        Field::from_parts(number, WireType::Fixed32, value.to_le_bytes().to_vec())
    }

    pub fn fixed64(number: u32, value: u64) -> Field {
        Field::from_parts(number, WireType::Fixed64, value.to_le_bytes().to_vec())
    }

    /// The encoded bytes of this field (key + value).
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw
    }

    /// Decode a varint value, if this is a varint field.
    pub fn as_varint(&self) -> Option<u64> {
        if self.wire != WireType::Varint {
            return None;
        }
        let mut pos = self.value_start;
        read_varint(&self.raw, &mut pos).ok()
    }

    pub fn as_bool(&self) -> Option<bool> {
        self.as_varint().map(|v| v != 0)
    }

    pub fn as_i32(&self) -> Option<i32> {
        self.as_varint().map(|v| v as u32 as i32)
    }

    pub fn as_i64(&self) -> Option<i64> {
        self.as_varint().map(|v| v as i64)
    }

    pub fn as_fixed32(&self) -> Option<u32> {
        if self.wire != WireType::Fixed32 {
            return None;
        }
        let b = self.raw.get(self.value_start..self.value_start + 4)?;
        Some(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn as_fixed64(&self) -> Option<u64> {
        if self.wire != WireType::Fixed64 {
            return None;
        }
        let b = self.raw.get(self.value_start..self.value_start + 8)?;
        Some(u64::from_le_bytes([
            b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7],
        ]))
    }

    /// Payload bytes of a length-delimited field.
    pub fn as_bytes(&self) -> Option<&[u8]> {
        if self.wire != WireType::LengthDelimited {
            return None;
        }
        let mut pos = self.value_start;
        let len = read_varint(&self.raw, &mut pos).ok()? as usize;
        self.raw.get(pos..pos + len)
    }

    /// UTF-8 view of a length-delimited field (lossless).
    pub fn as_str(&self) -> Option<Cow<'_, str>> {
        let bytes = self.as_bytes()?;
        match std::str::from_utf8(bytes) {
            Ok(s) => Some(Cow::Borrowed(s)),
            Err(_) => Some(String::from_utf8_lossy(bytes)),
        }
    }

    /// Parse a length-delimited field as a nested message.
    pub fn as_message(&self) -> Result<Option<Message>> {
        match self.as_bytes() {
            Some(bytes) => Ok(Some(Message::parse(bytes)?)),
            None => Ok(None),
        }
    }
}

/// An ordered list of fields, preserving exact bytes and duplicates.
#[derive(Clone, Debug, Default)]
pub struct Message {
    pub fields: Vec<Field>,
}

impl Message {
    pub fn parse(bytes: &[u8]) -> Result<Message> {
        let mut fields = Vec::new();
        let mut pos = 0usize;
        while pos < bytes.len() {
            let start = pos;
            let key = read_varint(bytes, &mut pos)?;
            let value_start = pos - start;
            let number = (key >> 3) as u32;
            let wire = WireType::from_u8((key & 7) as u8);
            if number == 0 {
                return Err(Error::Proto("field number 0 is invalid".into()));
            }
            match wire {
                WireType::Varint => {
                    read_varint(bytes, &mut pos)?;
                }
                WireType::Fixed64 => {
                    pos = pos
                        .checked_add(8)
                        .filter(|&p| p <= bytes.len())
                        .ok_or_else(|| Error::Proto("truncated fixed64".into()))?;
                }
                WireType::Fixed32 => {
                    pos = pos
                        .checked_add(4)
                        .filter(|&p| p <= bytes.len())
                        .ok_or_else(|| Error::Proto("truncated fixed32".into()))?;
                }
                WireType::LengthDelimited => {
                    let len = read_varint(bytes, &mut pos)? as usize;
                    pos = pos
                        .checked_add(len)
                        .filter(|&p| p <= bytes.len())
                        .ok_or_else(|| Error::Proto("truncated length-delimited field".into()))?;
                }
                WireType::StartGroup | WireType::EndGroup => {
                    return Err(Error::Proto("group wire types are not supported".into()));
                }
            }
            fields.push(Field {
                number,
                wire,
                raw: bytes[start..pos].to_vec(),
                value_start,
            });
        }
        Ok(Message { fields })
    }

    /// Re-emit every field's exact bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        for field in &self.fields {
            out.extend_from_slice(&field.raw);
        }
        out
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// First field with the given number.
    pub fn get(&self, number: u32) -> Option<&Field> {
        self.fields.iter().find(|f| f.number == number)
    }

    /// All fields with the given number, in order.
    pub fn get_all(&self, number: u32) -> impl Iterator<Item = &Field> {
        self.fields.iter().filter(move |f| f.number == number)
    }

    pub fn count(&self, number: u32) -> usize {
        self.fields.iter().filter(|f| f.number == number).count()
    }

    /// Replace the first field with `field.number`, or append if absent.
    pub fn set(&mut self, field: Field) {
        if let Some(existing) = self.fields.iter_mut().find(|f| f.number == field.number) {
            *existing = field;
        } else {
            self.fields.push(field);
        }
    }

    pub fn remove_all(&mut self, number: u32) {
        self.fields.retain(|f| f.number != number);
    }

    pub fn push(&mut self, field: Field) {
        self.fields.push(field);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varint_round_trip() {
        for value in [0u64, 1, 127, 128, 300, 16384, u32::MAX as u64, u64::MAX] {
            let mut buf = Vec::new();
            write_varint(&mut buf, value);
            let mut pos = 0;
            assert_eq!(read_varint(&buf, &mut pos).unwrap(), value);
            assert_eq!(pos, buf.len());
        }
    }

    #[test]
    fn parse_reencode_is_exact() {
        // Mixed wire types, including a repeated and an out-of-order field.
        let mut bytes = Vec::new();
        bytes.extend(Field::string(1, "Player").raw_bytes());
        bytes.extend(Field::varint(2, 50).raw_bytes());
        bytes.extend(Field::bytes(3, &[0xde, 0xad, 0xbe, 0xef]).raw_bytes());
        bytes.extend(Field::fixed32(4, 0x1122_3344).raw_bytes());
        bytes.extend(Field::varint(2, 7).raw_bytes()); // duplicate field 2
        bytes.extend(Field::fixed64(9, 0x0102_0304_0506_0708).raw_bytes());

        let message = Message::parse(&bytes).unwrap();
        assert_eq!(message.count(2), 2);
        assert_eq!(message.to_bytes(), bytes);

        let first = message.get(1).unwrap();
        assert_eq!(first.as_str().unwrap(), "Player");
        assert_eq!(message.get(2).unwrap().as_i32(), Some(50));
        assert_eq!(
            message.get(3).unwrap().as_bytes(),
            Some(&[0xde, 0xad, 0xbe, 0xef][..])
        );
        assert_eq!(message.get(4).unwrap().as_fixed32(), Some(0x1122_3344));
    }

    #[test]
    fn truncated_input_errors() {
        assert!(Message::parse(&[0x0a, 0x05, 0x01]).is_err()); // len 5 but only 1 byte
        assert!(Message::parse(&[0x80]).is_err()); // truncated varint key
    }
}
