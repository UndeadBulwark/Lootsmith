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
    let row = |name: &str, id: i32, mark: i32| ItemRow {
        name: name.into(),
        kind: "Pistol".into(),
        id,
        level: 10,
        mark,
        rarity: lootsmith_core::Rarity::Common,
    };
    // PLAYER_MARKS = ["Trash", "Standard", "Favorite"]
    let rows = vec![
        row("Standard row", 1, 1),
        row("Trash row", 2, 0),
        row("Favorite row", 3, 2),
    ];
    item_table(ui, &p, &rows, None);
}

/// The mockup: only a real **Favorite** (mark 2) draws a diamond; **Standard** (1)
/// and **Trash** (0) draw a small dim dot. (The `bb6c772` render wrongly drew a
/// diamond on every row.) Shapes are read from the frame: a diamond is a filled
/// `Shape::Path` (convex polygon), the dot is a filled `Shape::Circle`.
#[test]
fn mark_is_a_diamond_only_for_a_favourite() {
    let mut h = Harness::builder()
        .with_size(Vec2::new(420.0, 220.0))
        .build_ui_state(build, S);
    h.run();
    let out = h.output();
    let x_mark = 420.0_f32 * 0.88 + 6.0;
    let in_mark_col = |x: f32| (x - x_mark).abs() < 20.0;
    let (mut diamonds, mut dots) = (Vec::new(), Vec::new());
    for cs in &out.shapes {
        match &cs.shape {
            eframe::egui::Shape::Path(p) if p.fill.a() > 0 && !p.points.is_empty() => {
                let cx = p.points.iter().map(|q| q.x).sum::<f32>() / p.points.len() as f32;
                let cy = p.points.iter().map(|q| q.y).sum::<f32>() / p.points.len() as f32;
                if in_mark_col(cx) {
                    diamonds.push(cy);
                }
            }
            eframe::egui::Shape::Circle(c) if c.fill.a() > 0 && in_mark_col(c.center.x) => {
                dots.push(c.center.y);
            }
            _ => {}
        }
    }
    // Row order top-to-bottom: Standard (dot), Trash (dot), Favorite (diamond).    assert_eq!(diamonds.len(), 1, "exactly one row is a favourite: {diamonds:?}");
    assert_eq!(dots.len(), 2, "Standard and Trash draw a dot: {dots:?}");
    let d = diamonds[0];
    assert!(
        d > dots[0] && d > dots[1],
        "the diamond is the last (Favourite) row"
    );
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

/// Gibbed-style part slots: the workbench always shows every weapon slot,
/// including ones the balance does not cover (Stock / Elemental / Accessory 2)
/// — those simply offer only "None" (`BaseWeaponView.xaml`).
#[test]
fn workbench_shows_every_weapon_part_slot() {
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
        "Stock",
        "Elemental",
        "Accessory 1",
        "Accessory 2",
        "Material",
        "Prefix",
        "Title",
    ] {
        assert!(has(present), "expected {present:?} in the parts grid");
    }

    // Each slot's candidates are the resolved balance set (Gibbed's `Create`),
    // not the whole asset library: the Dahl pistol inherits grip/barrel from its
    // base chain, and the list stays small.
    let grip = h.state().slot_choices(
        true,
        Some("GD_Weap_Pistol.A_Weapons.WeaponType_Dahl_Pistol"),
        "GD_Weap_Pistol.A_Weapons.Pistol_Dahl_4_VeryRare",
        "grip",
        lootsmith_core::AssetGroup::WeaponParts,
    );
    assert!(!grip.is_empty(), "grip candidates must resolve");
    assert!(
        grip.len() < 20,
        "grip candidates must be the resolved set, got {} entries",
        grip.len()
    );
    assert!(
        grip.iter().any(|p| p.ends_with("Pistol_Grip_Dahl")),
        "the Dahl grip must be a candidate: {grip:?}"
    );

    // A slot the balance omits still exists, with no candidates of its own.
    let stock = h.state().slot_choices(
        true,
        Some("GD_Weap_Pistol.A_Weapons.WeaponType_Dahl_Pistol"),
        "GD_Weap_Pistol.A_Weapons.Pistol_Dahl_4_VeryRare",
        "stock",
        lootsmith_core::AssetGroup::WeaponParts,
    );
    assert!(stock.is_empty(), "stock has no legal parts here: {stock:?}");
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

/// Warn-only legality: a part outside the balance's resolved list is flagged in
/// the status bar but still written, and a legal part is accepted.
#[test]
fn illegal_parts_are_warned_but_still_written() {
    let save = synthetic("legality");
    let mut h = common::harness_sized(&save, None, Vec2::new(1280.0, 1600.0));
    h.state_mut().set_tab(Tab::Backpack);
    h.state_mut().select_entry(Container::BackpackWeapon, 0);
    h.run();
    // A sniper body is not a legal body for the fixture pistol (only
    // `Pistol_Body_Dahl_4` is), but it must still be written.
    assert!(
        h.state_mut()
            .set_selected_part("body", "GD_Weap_SniperRifles.Body.SR_Body_Dahl")
            .is_ok(),
        "warn-only must not reject the write"
    );
    assert!(
        h.state().status.contains("warning"),
        "an illegal part should warn: {}",
        h.state().status
    );
    // A legal part is accepted with no warning.
    assert!(h
        .state_mut()
        .set_selected_part("body", "GD_Weap_Pistol.Body.Pistol_Body_Dahl_4")
        .is_ok());
}
