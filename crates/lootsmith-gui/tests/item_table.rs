//! Item-list behaviour: middle-elision, balance fallback names, the favourite
//! mark, and the inspector's full scroll (spec §2.2 item 4).

mod common;

use common::frame::extract;
use eframe::egui::Vec2;
use egui_kittest::kittest::Queryable;
use egui_kittest::Harness;
use lootsmith_core::savegame::player_mark_name;
use lootsmith_core::Container;
use lootsmith_gui::app::Tab;
use lootsmith_gui::theme::{self, Theme};
use lootsmith_gui::ui::item_table::{balance_to_name, elide_middle, item_table, ItemRow};

#[test]
fn elides_a_60_char_name_in_the_middle() {
    let name = "GD_Weapon_Manufacturer_Something_A_Really_Long_Balance_Name_Xyz";
    assert!(name.chars().count() >= 60, "{}", name.chars().count());
    let e = elide_middle(name, 24);
    assert!(e.chars().count() <= 24, "{e:?}");
    assert!(e.contains("..."), "{e:?}");
    assert!(e.starts_with("GD_"), "{e:?}");
    assert!(e.ends_with("Xyz"), "{e:?}");
    // Short names are untouched.
    assert_eq!(elide_middle("Pistol", 24), "Pistol");
    // Balance fallback: last dotted segment, underscores -> spaces.
    assert_eq!(
        balance_to_name("GD_Weap_Pistol.A_Weapons.Pistol_Dahl_4_VeryRare"),
        "Pistol Dahl 4"
    );
}

#[test]
fn favourite_mark_is_only_the_favourite_value() {
    assert_eq!(player_mark_name(2), "Favorite");
    assert_ne!(player_mark_name(0), "Favorite"); // Trash
    assert_ne!(player_mark_name(1), "Favorite"); // Standard
}

struct S;
fn build(ui: &mut egui::Ui, _: &mut S) {
    theme::apply(ui.ctx(), Theme::Dark);
    let p = *theme::active(ui.ctx());
    ui.painter().rect_filled(ui.max_rect(), 0.0, p.bg);
    let rows = vec![
        ItemRow {
            name: "Standard row".into(),
            kind: "Pistol".into(),
            id: 1,
            level: 10,
            marked: false,
            rarity: lootsmith_core::Rarity::Common,
        },
        ItemRow {
            name: "Favorite row".into(),
            kind: "Pistol".into(),
            id: 2,
            level: 10,
            marked: true,
            rarity: lootsmith_core::Rarity::Common,
        },
    ];
    item_table(ui, &p, &rows, None);
}

#[test]
fn mark_diamond_is_drawn_only_on_the_favourite_row() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(420.0, 160.0))
        .build_ui_state(build, S);
    h.run();
    let img = h.render().expect("render");
    let (w, _h) = img.dimensions();
    let xlo = (w as f32 * 0.80) as u32;
    let x0 = (w as f32 * 0.86) as u32;
    let x1 = (w as f32 * 0.94) as u32;
    // A mark is a compact diamond; the dotted row dividers span the full width,
    // so ignore any scanline that is mostly bright.
    let diamond = |y0: u32, y1: u32| {
        let mut n = 0u32;
        for y in y0..y1 {
            let full = (xlo..w).filter(|&x| img.get_pixel(x, y)[0] > 60).count();
            if full > 25 {
                continue; // divider line
            }
            for x in x0..x1 {
                if img.get_pixel(x, y)[0] > 60 {
                    n += 1;
                }
            }
        }
        n
    };
    let standard = diamond(30, 68); // row 0 band
    let favourite = diamond(70, 126); // row 1 band
    assert_eq!(
        standard, 0,
        "Standard row must not draw a mark ({standard} px)"
    );
    assert!(favourite > 0, "Favorite row must draw a mark");
}

fn synthetic(tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("lootsmith-list-{tag}.sav"));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    path.display().to_string()
}

/// The parts list must reach the last slot ("Material"). The inspector body is one
/// scroll area, so with a viewport tall enough for the whole body every slot renders;
/// at 1x/2x/3x the inspector must still render its first part rows.
#[test]
fn inspector_parts_reach_the_last_slot() {
    // Tall viewport: nothing is culled, so every part row must be present.
    let save = synthetic("material");
    let mut h = common::harness_sized(&save, None, Vec2::new(1280.0, 2000.0));
    h.state_mut().set_tab(Tab::Backpack);
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let runs = {
        let out = h.output();
        extract(out).0
    };
    assert!(
        runs.iter().any(|r| r.text.trim() == "Material"),
        "the parts list must include the last slot 'Material'"
    );

    // At 1x/2x/3x the inspector still renders (its scroll area is bounded).
    for scale in [1.0_f32, 2.0, 3.0] {
        let save = synthetic(&format!("reach-{scale}"));
        // Physical size scaled so the logical window stays 1280x800.
        let mut h = common::harness_sized(&save, None, Vec2::new(1280.0 * scale, 800.0 * scale));
        h.state_mut().set_ui_scale(scale);
        h.state_mut().set_tab(Tab::Backpack);
        h.state_mut().select_entry(Container::BackpackWeapon, 0);
        h.run();
        let runs = {
            let out = h.output();
            extract(out).0
        };
        assert!(
            runs.iter().any(|r| r.text.trim() == "GRADE"),
            "{scale}x: the inspector must render"
        );
    }
}

/// The workbench part list must include the last slot ("Material") at 1x, 2x and
/// 3x. The body is one scroll area; with a window tall enough for the whole body
/// every slot renders, so the last row is reachable at every scale.
#[test]
fn workbench_lists_the_last_part_row_at_every_scale() {
    for scale in [1.0_f32, 2.0, 3.0] {
        let save = synthetic(&format!("last-{scale}"));
        let mut h = common::harness_sized(&save, None, Vec2::new(1280.0 * scale, 1400.0 * scale));
        h.state_mut().set_ui_scale(scale);
        h.state_mut().set_tab(Tab::Backpack);
        h.state_mut().select_entry(Container::BackpackWeapon, 0);
        h.run();
        let runs = {
            let out = h.output();
            extract(out).0
        };
        assert!(
            runs.iter().any(|r| r.text.trim() == "Material"),
            "{scale}x: the last part row 'Material' is missing"
        );
    }
}

/// Gibbed-style part exclusion: the workbench shows the item's own parts **plus**
/// the balance's candidates, with type/balance/manufacturer pinned. The fixture
/// Dahl pistol carries grip/barrel/material/prefix/title and its balance offers
/// {accessory1, body, material, sight}, so Stock/Elemental/Accessory 2 must not show.
#[test]
fn workbench_shows_the_items_parts_and_balance_candidates() {
    let save = synthetic("parts");
    let mut h = common::harness_sized(&save, None, Vec2::new(1280.0, 1600.0));
    h.state_mut().set_tab(Tab::Backpack);
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let runs = {
        let out = h.output();
        extract(out).0
    };
    let has = |t: &str| runs.iter().any(|r| r.text.trim() == t);
    for present in [
        "Type",
        "Balance",
        "Manufacturer",
        "Body",
        "Grip",
        "Barrel",
        "Sight",
        "Accessory 1",
        "Material",
        "Prefix",
        "Title",
    ] {
        assert!(has(present), "expected {present:?} in the parts grid");
    }
    for absent in ["Stock", "Elemental", "Accessory 2"] {
        assert!(!has(absent), "{absent:?} must not appear for this item");
    }

    // The item's own parts win: an elemental part shows even though the balance
    // omits it.
    let _ = h.state_mut().set_selected_part(
        "elemental",
        "GD_Weap_Pistol.elemental.Pistol_Elemental_Fire",
    );
    h.run();
    let runs = {
        let out = h.output();
        extract(out).0
    };
    assert!(
        runs.iter().any(|r| r.text.trim() == "Elemental"),
        "an existing elemental part must be editable"
    );
}

/// The workbench stat blocks are LEVEL / GRADE / SLOT (or STATUS) / MARK, editable,
/// with no duplicate GAME STAGE / Quick slot row.
#[test]
fn workbench_stat_blocks_are_editable_and_deduplicated() {
    let save = synthetic("stats");
    let mut h = common::harness_sized(&save, None, Vec2::new(1280.0, 1600.0));
    h.state_mut().set_tab(Tab::Backpack);
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    let runs = {
        let out = h.output();
        extract(out).0
    };
    let has = |t: &str| runs.iter().any(|r| r.text.trim() == t);
    for block in ["LEVEL", "GRADE", "SLOT", "MARK"] {
        assert!(has(block), "stat block {block:?} missing");
    }
    assert!(!has("GAME STAGE"), "GAME STAGE duplicates LEVEL");
    assert!(
        h.query_all_by_label("Quick slot").count() == 0,
        "the redundant Quick slot row must be gone"
    );
}
