//! Skill-board parity tests: the five jobs from the redesign doc, driven
//! through the real UI on the synthetic save (Axton).

mod common;

use common::{harness, reopen};
use egui_kittest::kittest::Queryable;
use lootsmith_gui::app::SkillRow;

fn sentry_index(h: &egui_kittest::Harness<'_, lootsmith_gui::app::LootsmithApp>) -> usize {
    h.state()
        .skills
        .iter()
        .position(|s| s.name.contains("Sentry"))
        .expect("the synthetic save has Sentry")
}

fn action_index(h: &egui_kittest::Harness<'_, lootsmith_gui::app::LootsmithApp>) -> usize {
    h.state()
        .skills
        .iter()
        .position(|s| s.name.contains("Skill_Scorpio"))
        .expect("the synthetic save has the action skill")
}

/// Left-click adds, right-click removes, and the grade survives a save/reload.
#[test]
fn adds_and_removes_a_rank_and_writes_it() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();

    // Sentry starts at 3/5. Exact labels: the tile ("Sentry, rank 3 of 5"), not
    // the detail line (which is just "Sentry").
    h.get_by_label("Sentry, rank 3 of 5").click();
    h.run();
    assert!(h.state().dirty, "adding a rank marks the save dirty");
    let i = sentry_index(&h);
    assert_eq!(h.state().skills[i].grade, 4, "left-click adds a rank");

    h.get_by_label("Sentry, rank 4 of 5").click_secondary();
    h.run();
    assert_eq!(h.state().skills[i].grade, 3, "right-click removes a rank");

    // Add again and prove it round-trips through the file.
    h.get_by_label("Sentry, rank 3 of 5").click();
    h.run();
    let out = std::env::temp_dir().join("lootsmith-skill-roundtrip.sav");
    h.state_mut().save_to(out.to_str().unwrap());
    let save = reopen(&out).save_game().expect("reparse");
    let sentry = save
        .skills()
        .into_iter()
        .find(|s| s.skill.contains("Sentry"))
        .expect("sentry");
    assert_eq!(sentry.grade, 4);
    let _ = std::fs::remove_file(&out);
}

/// With no points available, an add is blocked (nothing changes).
#[test]
fn add_is_blocked_without_points() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();
    let i = sentry_index(&h);
    let before = h.state().skills[i].grade;
    h.state_mut().form.general_skill_points = 0;
    h.run();

    h.get_by_label_contains("Sentry").click();
    h.run();
    assert_eq!(
        h.state().skills[i].grade,
        before,
        "no points available must block the add"
    );
}

/// Removing a point that would lock a higher tier (that still holds points) is
/// blocked.
#[test]
fn removal_that_would_lock_a_higher_tier_is_blocked() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();

    // Guerilla total 10 = Sentry (tier 0) 5 + Able (tier 2) 5; tier 2 needs 10.
    for s in h.state_mut().skills.iter_mut() {
        s.grade = 0;
    }
    let a = action_index(&h);
    h.state_mut().skills[a].grade = 1;
    let i = sentry_index(&h);
    h.state_mut().skills[i].grade = 5;
    h.state_mut().skills.push(SkillRow {
        name: "GD_Soldier_Skills.Guerrilla.Able".into(),
        grade: 5,
        points: 0,
        slot: 0,
        orig_grade: 5,
        orig_points: 0,
    });
    h.run();

    h.get_by_label("Sentry, rank 5 of 5").click_secondary();
    h.run();
    assert_eq!(
        h.state().skills[i].grade,
        5,
        "a removal that would lock tier 3 is blocked"
    );
}

/// The wide action-skill banner toggles the save's action skill (0 -> 1).
#[test]
fn action_skill_banner_toggles() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();

    // The synthetic save has the action skill at rank 1; clear it first, then
    // prove the banner sets it again.
    let a = action_index(&h);
    h.state_mut().skills[a].grade = 0;
    h.run();

    h.get_by_label_contains("Sabre Turret").click();
    h.run();
    assert_eq!(
        h.state().skills[a].grade,
        1,
        "clicking the action-skill banner sets it"
    );
}

/// The action skill gates tier 1: with none set, a tree point is refused.
#[test]
fn tier_one_needs_the_action_skill() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();
    let i = sentry_index(&h);
    let before = h.state().skills[i].grade;
    // Clear the action point: tier 1 must refuse tree points.
    let a = action_index(&h);
    h.state_mut().skills[a].grade = 0;
    h.run();

    h.get_by_label_contains("Sentry").click();
    h.run();
    assert_eq!(
        h.state().skills[i].grade,
        before,
        "tier 1 is locked until the action skill is filled"
    );
}

/// Clearing the action point is refused while tier 1 still holds ranks.
#[test]
fn clearing_the_action_point_is_blocked_while_tier_one_has_ranks() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();
    for s in h.state_mut().skills.iter_mut() {
        s.grade = 0;
    }
    let a = action_index(&h);
    h.state_mut().skills[a].grade = 1;
    h.run();

    let i = sentry_index(&h);
    h.state_mut().skills[i].grade = 1;
    h.run();

    let action = |h: &egui_kittest::Harness<'_, lootsmith_gui::app::LootsmithApp>| {
        h.state()
            .skills
            .iter()
            .find(|s| s.name.contains("Skill_Scorpio"))
            .map(|s| s.grade)
            .expect("action skill")
    };

    h.get_by_label_contains("Sabre Turret").click_secondary();
    h.run();
    assert_eq!(action(&h), 1, "the action point is held by tier 1");
    assert!(
        h.state().status.contains("tier 1"),
        "the refusal is explained: {}",
        h.state().status
    );

    // Once tier 1 is empty the action point clears.
    h.state_mut().skills[i].grade = 0;
    h.run();
    h.get_by_label_contains("Sabre Turret").click_secondary();
    h.run();
    assert_eq!(
        action(&h),
        0,
        "the action point clears once tier 1 is empty"
    );
}

/// A pasted `LS2:` build applies (with the budget kept), and Respec refunds
/// every point back.
#[test]
fn pasted_build_applies_and_respec_refunds() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.get_by_label("Skills").click();
    h.run();

    // Axton Guerilla is 10 nodes; Sentry is the first. Rank it 1.
    h.state_mut().skill_build = "LS2:Commando:1000000000-00000000000-00000000000".into();
    h.run();
    h.get_by_label("Apply").click();
    h.run();
    let sentry = h
        .state()
        .skills
        .iter()
        .find(|s| s.name.contains("Sentry"))
        .unwrap();
    assert_eq!(sentry.grade, 1, "the pasted build was applied");
    assert!(
        h.state()
            .skills
            .iter()
            .filter(|s| !s.name.contains("Sentry") && !s.name.contains("Skill_Scorpio"))
            .all(|s| s.grade == 0),
        "the build replaces the other tree ranks (the action skill is kept)"
    );

    let spent: i32 = h.state().skills.iter().map(|s| s.grade.max(0)).sum();
    let avail = h.state().form.general_skill_points;
    h.state_mut().respec_skills();
    h.run();
    assert!(h.state().skills.iter().all(|s| s.grade == 0));
    assert_eq!(
        h.state().form.general_skill_points,
        avail + spent,
        "respec refunds the spent points"
    );
}

/// After a class change the new class's tree is editable (the original bug:
/// only the action skill was, because the save still held the old class's list).
#[test]
fn changed_class_tree_is_editable() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    h.state_mut()
        .change_class("GD_Siren.Character.CharClass_Siren");
    let a = h
        .state()
        .skills
        .iter()
        .position(|s| s.name.contains("Skill_Phaselock"))
        .expect("Maya's action skill");
    // A real character has the action point; fill it so tier 1 opens.
    h.state_mut().skills[a].grade = 1;
    h.get_by_label("Skills").click();
    h.run();

    h.get_by_label_contains("Ward").click();
    h.run();
    let ward = h
        .state()
        .skills
        .iter()
        .find(|s| s.name == "GD_Siren_Skills.Motion.Ward")
        .expect("Maya's Ward");
    assert_eq!(ward.grade, 1, "Maya's tier-1 tile is editable");
    assert!(
        h.state()
            .skills
            .iter()
            .any(|s| s.name == "GD_Siren_Skills.Phaselock.Skill_Phaselock"),
        "the action skill is Maya's, not Axton's"
    );
}

/// A save whose skill list belongs to another class (e.g. changed by an older
/// build) is repaired when the Skills screen opens.
#[test]
fn a_mismatched_skill_list_is_rebuilt_on_open() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    // The class says Siren, but the skill list is still Axton's.
    h.state_mut().read_info.player_class = "GD_Siren.Character.CharClass_Siren".into();
    h.run();
    h.get_by_label("Skills").click();
    h.run();

    assert!(
        h.state()
            .skills
            .iter()
            .any(|s| s.name == "GD_Siren_Skills.Phaselock.Skill_Phaselock"),
        "the list was rebuilt for the class"
    );
    assert!(
        h.state().skills.iter().all(|s| s.grade == 0),
        "the rebuilt list starts clean"
    );
    assert!(
        h.state().status.contains("rebuilt"),
        "status explains the repair: {}",
        h.state().status
    );
}

/// Changing the class clears the old class's ranks and refunds the points so
/// the board is consistent for the new hero.
#[test]
fn changing_class_refunds_skill_points() {
    if std::env::var_os("LOOTSMITH_NO_GPU").is_some() {
        eprintln!("skipped: no GPU adapter in this environment");
        return;
    }
    let mut h = harness();
    h.run();
    let before: i32 = h.state().skills.iter().map(|s| s.grade.max(0)).sum();
    let avail = h.state().form.general_skill_points;
    assert!(before > 0, "the synthetic save has spent points");

    h.state_mut()
        .change_class("GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass");
    h.run();
    assert!(
        h.state().skills.iter().all(|s| s.grade == 0),
        "the old class's ranks are cleared"
    );
    assert_eq!(
        h.state().form.general_skill_points,
        avail + before,
        "spent points are refunded"
    );
    assert_eq!(
        h.state().read_info.player_class,
        "GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass"
    );
    // The save's skill list is rebuilt to the new class's canonical objects.
    assert_eq!(
        h.state().skills.len(),
        36,
        "Krieg's list: 35 tree skills + the action skill"
    );
    assert!(
        h.state()
            .skills
            .iter()
            .any(|s| s.name == "GD_Lilac_SkillsBase.ActionSkill.Skill_Psycho"),
        "Krieg's action skill is present"
    );
    assert!(
        h.state()
            .skills
            .iter()
            .any(|s| s.name == "GD_Lilac_Skills_Mania.Skills.FeedTheMeat"),
        "Krieg's tree objects are present"
    );
}
