//! The about tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme::{self, Tokens};
use eframe::egui;

pub(crate) fn ui_about(ui: &mut egui::Ui) {
    ui.add_space(10.0);
    card(ui, 620.0, |ui| {
        theme::section(ui, "About");
        ui.label(
            egui::RichText::new("Lootsmith — a cross-platform Borderlands 2 save editor")
                .strong()
                .size(15.0)
                .color(Tokens::ACCENT),
        );
        ui.add_space(6.0);
        theme::muted(
            ui,
            "A Rust port based on Gibbed's Borderlands 2 Save Editor",
        );
        theme::muted(
            ui,
            "(zlib license). Game data is read from Rick Gibbed's dumps.",
        );
        ui.add_space(8.0);
        theme::faint(
            ui,
            "Skill calculator data compressed from the UCP calculator",
        );
        theme::faint(
            ui,
            "(bl2skills.com, /patch variant) and embedded in the app.",
        );
        ui.add_space(8.0);
        theme::muted(
            ui,
            "CLI: lootsmith inspect / verify / dump / items / edit / json / profile …",
        );
    });
}
