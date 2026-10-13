//! Shared widget helpers (moved verbatim from the former single-file `app.rs`).

use super::*;
use crate::theme;
use eframe::egui;

pub(crate) fn customization_combo(
    ui: &mut egui::Ui,
    customizations: &Option<Arc<Customizations>>,
    usage: Option<&str>,
    kind: &str,
    value: &mut String,
    id: &str,
) {
    let current = match customizations {
        Some(c) if value.as_str() != "None" => c.label_for(value.as_str()),
        _ => value.clone(),
    };
    egui::ComboBox::from_id_salt(id)
        .selected_text(current)
        .width(340.0)
        .show_ui(ui, |ui| {
            if ui.selectable_label(*value == "None", "None").clicked() {
                *value = "None".to_string();
            }
            if let (Some(c), Some(usage)) = (customizations, usage) {
                for (key, name) in c.type_options(usage, kind) {
                    if ui.selectable_label(*value == key, &name).clicked() {
                        *value = key;
                    }
                }
            }
        });
}

/// Display label for a currency slot. The array order is Gibbed's
/// `CurrencyOnHandViewModel`: `[Credits, Eridium, SeraphCrystals, ReservedA,
/// TorgueTokens, ReservedC…J]` — Torgue tokens are **index 4**, not 3.
pub(crate) fn currency_name(index: usize) -> String {
    match index {
        0 => "Money".to_string(),
        1 => "Eridium".to_string(),
        2 => "Seraph crystals".to_string(),
        3 => "Reserved A".to_string(),
        4 => "Torgue tokens".to_string(),
        n => {
            let letter = (b'C' + (n as u8).saturating_sub(5)) as char;
            format!("Reserved {letter}")
        }
    }
}

/// Whether a currency slot holds a real currency (money/eridium/seraph/torgue)
/// rather than a reserved placeholder. Reserved slots are hidden unless the
/// user asks to see them.
pub(crate) fn currency_slot_is_known(index: usize) -> bool {
    matches!(index, 0 | 1 | 2 | 4)
}

/// A status glyph (shape carries the meaning too, not just colour).
pub(crate) fn status_dot(
    ui: &mut egui::Ui,
    glyph: crate::ui::widgets::Glyph,
    color: egui::Color32,
) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(12.0, 12.0), egui::Sense::hover());
    crate::ui::widgets::paint_glyph(ui.painter(), rect.center(), glyph, color, 1.0);
}

/// A grouped surface card of a fixed width.
pub(crate) fn card(ui: &mut egui::Ui, width: f32, add: impl FnOnce(&mut egui::Ui)) {
    let p = *theme::active(ui.ctx());
    ui.allocate_ui_with_layout(
        egui::vec2(width, 0.0),
        egui::Layout::top_down(egui::Align::Min),
        |ui| {
            egui::Frame::group(ui.style())
                .fill(p.pan)
                .stroke(egui::Stroke::new(1.0, p.wdb))
                .corner_radius(egui::CornerRadius::same(8))
                .inner_margin(egui::Margin::same(14))
                .show(ui, |ui| {
                    ui.set_width(width - 28.0);
                    add(ui);
                });
        },
    );
}

/// The empty state shown before a save is open.
pub(crate) fn empty_state(ui: &mut egui::Ui) {
    ui.vertical_centered(|ui| {
        ui.add_space(64.0);
        ui.label(
            egui::RichText::new("Lootsmith")
                .size(26.0)
                .strong()
                .color(theme::active(ui.ctx()).gold),
        );
        ui.add_space(6.0);
        theme::muted(ui, "A cross-platform Borderlands 2 save editor.");
        ui.add_space(14.0);
        theme::faint(
            ui,
            "Open a save (or use Find saves), then pick your game-data folder",
        );
        theme::faint(
            ui,
            "so item parts, missions and travel stations resolve to names.",
        );
    });
}

#[cfg(test)]
mod currency_tests {
    use super::{currency_name, currency_slot_is_known};

    #[test]
    fn currency_indices_match_gibbed() {
        // Gibbed `CurrencyOnHandViewModel`: Torgue tokens are index 4; index 3
        // is a reserved slot.
        assert_eq!(currency_name(0), "Money");
        assert_eq!(currency_name(1), "Eridium");
        assert_eq!(currency_name(2), "Seraph crystals");
        assert_eq!(currency_name(3), "Reserved A");
        assert_eq!(currency_name(4), "Torgue tokens");
        assert_eq!(currency_name(5), "Reserved C");
        assert_eq!(currency_name(12), "Reserved J");
    }

    #[test]
    fn only_real_currencies_are_known() {
        assert!(currency_slot_is_known(0));
        assert!(currency_slot_is_known(4), "Torgue tokens must stay visible");
        assert!(!currency_slot_is_known(3));
        assert!(!currency_slot_is_known(5));
    }
}
