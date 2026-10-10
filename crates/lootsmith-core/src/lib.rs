//! Lootsmith core: a Rust library for reading and writing Borderlands 2 saves.
//!
//! A Rust port based on Gibbed's Borderlands 2 Save Editor by Rick Gibbed
//! (zlib license).

pub mod bitio;
pub mod container;
pub mod crc32;
pub mod error;
pub mod huffman;
pub mod platform;

pub use container::{pack_inner, unpack, SaveFile, BLOCK_SIZE};
pub use error::{Error, Result};
pub use platform::{CompressionScheme, Endian, Platform};
