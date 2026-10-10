//! Shared helpers for the core integration tests.
//!
//! Every test uses the **synthetic** save (`lootsmith_core::fixture`), never a
//! personal `tests/fixtures/pc_Save0001.sav`, so the suite runs identically on a
//! clean checkout with no fixtures and no environment.

#![allow(dead_code)]

use lootsmith_core::{container, fixture, Platform};

/// The synthetic save game.
pub fn synthetic() -> lootsmith_core::SaveGame {
    fixture::synthetic_save()
}

/// The synthetic save as a PC [`lootsmith_core::SaveFile`] (unpacked bytes).
pub fn synthetic_file() -> lootsmith_core::SaveFile {
    let bytes = fixture::synthetic_save_bytes().expect("synthetic save bytes");
    container::unpack(&bytes, Platform::PC).expect("unpack synthetic save")
}
