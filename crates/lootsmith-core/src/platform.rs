//! Platform traits: endianness and compression scheme.
//!
//! Ported from `PlatformHelpers.cs` and the `Platform` enum in `GameInfo`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    PC,
    X360,
    PS3,
    PSVita,
    Shield,
    Switch,
}

impl Platform {
    pub fn endian(self) -> Endian {
        match self {
            Platform::PC | Platform::PSVita | Platform::Shield | Platform::Switch => Endian::Little,
            Platform::X360 | Platform::PS3 => Endian::Big,
        }
    }

    pub fn compression_scheme(self) -> CompressionScheme {
        match self {
            Platform::Switch => CompressionScheme::None,
            Platform::PC | Platform::X360 => CompressionScheme::Lzo,
            Platform::PS3 | Platform::PSVita | Platform::Shield => CompressionScheme::Zlib,
        }
    }

    /// Parse a platform identifier used by the CLI (`pc`, `x360`, `ps3`,
    /// `psvita`, `shield`, `switch`).
    pub fn from_id(id: &str) -> Option<Platform> {
        match id.to_ascii_lowercase().as_str() {
            "pc" => Some(Platform::PC),
            "x360" | "xbox360" => Some(Platform::X360),
            "ps3" => Some(Platform::PS3),
            "psvita" | "vita" => Some(Platform::PSVita),
            "shield" => Some(Platform::Shield),
            "switch" => Some(Platform::Switch),
            _ => None,
        }
    }

    pub fn id(self) -> &'static str {
        match self {
            Platform::PC => "pc",
            Platform::X360 => "x360",
            Platform::PS3 => "ps3",
            Platform::PSVita => "psvita",
            Platform::Shield => "shield",
            Platform::Switch => "switch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Endian {
    Little,
    Big,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompressionScheme {
    None,
    Lzo,
    Zlib,
}

impl CompressionScheme {
    pub fn name(self) -> &'static str {
        match self {
            CompressionScheme::None => "none",
            CompressionScheme::Lzo => "lzo",
            CompressionScheme::Zlib => "zlib",
        }
    }
}
