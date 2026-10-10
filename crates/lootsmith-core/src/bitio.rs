//! Little-endian (LSB-first) bit reader and writer.
//!
//! Faithful ports of `Gibbed.Gearbox.Common/BitReader.cs` and `BitWriter.cs`.
//! These are the primitives used by the item codec (Phase 4). The container's
//! Huffman codec uses MSB-first bits and lives in `huffman.rs`.

use crate::error::{Error, Result};
use std::cmp::min;

/// LSB-first bit reader over a borrowed byte slice.
pub struct BitReader<'a> {
    buffer: &'a [u8],
    length: usize,
    position: usize,
}

impl<'a> BitReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            length: buffer.len() * 8,
            position: 0,
        }
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    pub fn read_uint32(&mut self, mut bits: usize) -> Result<u32> {
        if bits > 32 {
            return Err(Error::EndOfStream);
        }
        let mut result: u32 = 0;
        let mut shift: usize = 0;
        while bits > 0 {
            if self.position >= self.length {
                return Err(Error::EndOfStream);
            }
            let offset = self.position % 8;
            let left = min(8 - offset, bits);
            let mask: u32 = (1u32 << left) - 1;
            let value = (self.buffer[self.position >> 3] as u32) >> offset;
            bits -= left;
            result |= (mask & value) << shift;
            shift += left;
            self.position += left;
        }
        Ok(result)
    }

    pub fn read_int32(&mut self, bits: usize) -> Result<i32> {
        Ok(self.read_uint32(bits)? as i32)
    }

    pub fn read_boolean(&mut self) -> Result<bool> {
        Ok(self.read_uint32(1)? != 0)
    }
}

/// LSB-first bit writer.
///
/// Mirrors `BitWriter` including its `Resize(128)` fill semantics: newly
/// allocated bytes are filled with `0xFF` and only written bits are cleared.
/// Unwritten high bits in the final byte therefore remain set, matching the
/// reference implementation.
pub struct BitWriter {
    buffer: Vec<u8>,
    allocated: usize,
    position: usize,
}

impl Default for BitWriter {
    fn default() -> Self {
        Self::new()
    }
}

impl BitWriter {
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            allocated: 0,
            position: 0,
        }
    }

    fn resize(&mut self, new_size: usize) {
        let start = self.buffer.len();
        self.buffer.resize(new_size, 0x00);
        if new_size > start {
            for byte in &mut self.buffer[start..] {
                *byte = 0xFF;
            }
        }
        self.allocated = self.buffer.len() * 8;
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn write_uint32(&mut self, mut value: u32, mut bits: usize) -> Result<()> {
        if bits > 32 {
            return Err(Error::EndOfStream);
        }
        while bits > 0 {
            if self.position >= self.allocated {
                self.resize(128);
            }
            let offset = self.position % 8;
            let left = min(8 - offset, bits);
            let mask: u32 = (1u32 << left) - 1;
            let idx = self.position >> 3;
            self.buffer[idx] &= !(((mask << offset) & 0xFF) as u8);
            self.buffer[idx] |= ((value & mask) << offset) as u8;
            bits -= left;
            value >>= left;
            self.position += left;
        }
        Ok(())
    }

    pub fn write_uint64(&mut self, mut value: u64, mut bits: usize) -> Result<()> {
        if bits > 64 {
            return Err(Error::EndOfStream);
        }
        while bits > 0 {
            if self.position >= self.allocated {
                // Reference extends by 128 bytes without the 0xFF fill here.
                let new_len = self.buffer.len() + 128;
                self.buffer.resize(new_len, 0x00);
                self.allocated = self.buffer.len() * 8;
            }
            let offset = self.position % 8;
            let left = min(8 - offset, bits);
            let mask: u64 = (1u64 << left) - 1;
            let idx = self.position >> 3;
            self.buffer[idx] |= ((value & mask) << offset) as u8;
            bits -= left;
            value >>= left;
            self.position += left;
        }
        Ok(())
    }

    pub fn write_boolean(&mut self, value: bool) -> Result<()> {
        self.write_uint32(if value { 1 } else { 0 }, 1)
    }

    /// Return the written prefix, rounded up to whole bytes.
    pub fn get_buffer(&self) -> Vec<u8> {
        let length = (self.position + 7) >> 3;
        self.buffer[..length.min(self.buffer.len())].to_vec()
    }
}
