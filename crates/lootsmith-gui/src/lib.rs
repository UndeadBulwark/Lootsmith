//! Lootsmith GUI library: application state and UI, shared by the native binary
//! and the `egui_kittest` harness.

pub mod app;

pub use app::{slots_for, Form, LootsmithApp, ReadInfo, Row, SkillRow, Tab};
