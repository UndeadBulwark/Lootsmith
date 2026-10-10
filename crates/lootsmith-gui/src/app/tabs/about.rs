//! The about tab (moved verbatim from the former single-file app.rs).

use crate::app::*;
use crate::theme::{self, Tokens};
use eframe::egui;

pub(crate) fn ui_about(ui: &mut egui::Ui) {
    ui.add_space(10.0);
    card(ui, 620.0, |ui| {
        theme::section(ui, "About");
        crate::ui::widgets::label_nums(
            ui,
            "Lootsmith — a cross-platform Borderlands 2 save editor",
            Tokens::ACCENT,
            15.0,
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
        ui.add_space(4.0);
        theme::muted(
            ui,
            "Unofficial fan project — not affiliated with Gearbox Software or 2K.",
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
        theme::faint(
            ui,
            "Game-data tables are bundled from the community dumps (see assets/data/PROVENANCE.md);",
        );
        theme::faint(
            ui,
            "a folder picked in Game data… overrides them. No Gearbox art is included.",
        );
        ui.add_space(8.0);
        theme::faint(
            ui,
            "Fonts: Pixelify Sans and VT323, both under the SIL Open Font License 1.1;",
        );
        theme::faint(ui, "their licence texts ship in assets/fonts/.");
        ui.add_space(8.0);
        theme::muted(
            ui,
            "CLI: lootsmith inspect / verify / dump / items / edit / json / profile …",
        );
    });
}
