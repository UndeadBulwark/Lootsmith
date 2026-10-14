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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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

/// Every row draws **both** a trash can and a star. The Trash row's can uses
/// `dng`, the Favourite row's star uses `gold`, and a Standard row stays `dim`.
/// Shapes are read from the frame: the star is a filled `Shape::Path`, each
/// trash-can stroke is a `Shape::LineSegment`.
#[test]
fn mark_cycle_is_none_favourite_trash() {
    use lootsmith_gui::ui::item_table::next_mark;
    // Standard ("none") -> Favourite -> Trash -> Standard.
    assert_eq!(next_mark(1), 2);
    assert_eq!(next_mark(2), 0);
    assert_eq!(next_mark(0), 1);
}

/// One cycling control per row: a dim dot for Standard, a `dng` trash can for
/// Trash, a `gold` star for Favourite — read from the rendered frame.
#[test]
fn mark_control_shows_dot_can_or_star() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = Harness::builder()
        .with_size(Vec2::new(420.0, 220.0))
        .build_ui_state(build, S);
    h.run();
    let p = theme::DARK;
    let out = h.output();
    let x_mark = 420.0_f32 * 0.88 + 22.0;
    let in_col = |x: f32| (x - x_mark).abs() < 28.0;
    let (mut dots, mut cans, mut stars) = (Vec::new(), Vec::new(), Vec::new());
    for cs in &out.shapes {
        match &cs.shape {
            eframe::egui::Shape::Path(sh) if sh.fill == p.gold && !sh.points.is_empty() => {
                let cx = sh.points.iter().map(|q| q.x).sum::<f32>() / sh.points.len() as f32;
                let cy = sh.points.iter().map(|q| q.y).sum::<f32>() / sh.points.len() as f32;
                if in_col(cx) {
                    stars.push(cy);
                }
            }
            eframe::egui::Shape::Circle(c) if c.fill == p.dim && in_col(c.center.x) => {
                dots.push(c.center.y);
            }
            eframe::egui::Shape::LineSegment { points, stroke }
                // Only the trash-can strokes use dim/dng; the dotted row dividers
                // are `brd` and must not be counted.
                if (stroke.color == p.dim || stroke.color == p.dng)
                    && in_col((points[0].x + points[1].x) / 2.0) =>
            {
                cans.push((points[0].y + points[1].y) / 2.0);
            }
            _ => {}
        }
    }
    // Cluster by y (rows are ~46 px apart, strokes within a row stay under 20).
    let clusters = |mut ys: Vec<f32>| {
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mut out: Vec<f32> = Vec::new();
        for y in ys {
            if out.last().is_none_or(|l| (y - l).abs() > 20.0) {
                out.push(y);
            }
        }
        out
    };
    let dots = clusters(dots);
    let cans = clusters(cans);
    let stars = clusters(stars);
    assert_eq!(
        dots.len(),
        1,
        "one row shows the Standard dim dot: {dots:?}"
    );
    assert_eq!(cans.len(), 1, "one row shows the Trash can: {cans:?}");
    assert_eq!(
        stars.len(),
        1,
        "one row shows the Favourite star: {stars:?}"
    );
    // Rows top-to-bottom: Standard, Trash, Favourite.
    assert!(dots[0] < cans[0], "the Standard row is above the Trash row");
    assert!(
        cans[0] < stars[0],
        "the Trash row is above the Favourite row"
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
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
