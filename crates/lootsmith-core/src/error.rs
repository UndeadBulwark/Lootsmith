use std::fmt;

/// Errors returned by Lootsmith. Malformed input must never panic.
#[derive(Debug)]
pub enum Error {
    Io(String),
    TooShort(&'static str),
    BadMagic,
    UnsupportedVersion(u32),
    Crc32Mismatch {
        expected: u32,
        computed: u32,
    },
    Sha1Mismatch,
    Decompress {
        scheme: &'static str,
        detail: String,
    },
    Compress {
        scheme: &'static str,
        detail: String,
    },
    SizeMismatch {
        expected: usize,
        actual: usize,
    },
    Huffman(String),
    EndOfStream,
    UnsupportedPlatform,
    Xbox360ConFile,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(m) => write!(f, "io error: {m}"),
            Error::TooShort(what) => write!(f, "not enough data for {what}"),
            Error::BadMagic => write!(f, "invalid magic (expected \"WSG\")"),
            Error::UnsupportedVersion(v) => write!(f, "invalid or unsupported version: {v}"),
            Error::Crc32Mismatch { expected, computed } => write!(
                f,
                "invalid CRC32 hash: stored {expected:#010x}, computed {computed:#010x}"
            ),
            Error::Sha1Mismatch => write!(f, "invalid SHA1 hash"),
            Error::Decompress { scheme, detail } => {
                write!(f, "{scheme} decompression failure ({detail})")
            }
            Error::Compress { scheme, detail } => {
                write!(f, "{scheme} compression failure ({detail})")
            }
            Error::SizeMismatch { expected, actual } => {
                write!(f, "size mismatch: expected {expected} bytes, got {actual}")
            }
            Error::Huffman(m) => write!(f, "huffman codec error: {m}"),
            Error::EndOfStream => write!(f, "unexpected end of bit stream"),
            Error::UnsupportedPlatform => write!(f, "unsupported platform"),
            Error::Xbox360ConFile => write!(
                f,
                "cannot load XBOX 360 CON files; extract the save with Modio or equivalent"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
