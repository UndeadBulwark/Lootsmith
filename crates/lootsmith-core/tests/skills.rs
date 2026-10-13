//! SkillData (field 8) accessors and write path, on the synthetic save.
//!
//! Each entry is `{1: Skill, 2: Grade, 3: GradePoints, 4: EquippedSlotIndex}`.

mod common;

use common::synthetic;
use lootsmith_core::{container, Platform};

#[test]
fn synthetic_skills_enumerate() {
    let sg = synthetic();
    let skills = sg.skills();
    assert_eq!(skills.len(), 5, "the synthetic save has 5 skill entries");
    assert!(
        skills.iter().all(|s| s.skill.starts_with("GD_")),
        "skill names are resource paths"
    );
    assert!(
        skills.iter().any(|s| s.grade > 0),
        "a leveled character has invested ranks"
    );
    assert!(
        skills.iter().any(|s| s.grade == 0),
        "one skill is still at rank 0"
    );
}

#[test]
fn edits_a_skill_grade_and_reloads() {
    let mut sg = synthetic();
    let before = sg.skills();
    // Pick a skill at rank 0 and raise it.
    let index = before
        .iter()
        .position(|s| s.grade == 0)
        .expect("a rank-0 skill");
    let target = before[index].skill.clone();

    sg.set_skill_grade(index, 5).expect("set grade");
    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());

    let after = reopened.save_game().expect("reparse").skills();
    assert_eq!(after.len(), before.len(), "no skills added or removed");
    assert_eq!(after[index].skill, target);
    assert_eq!(after[index].grade, 5);

    // Every other skill is untouched.
    for (i, (old, new)) in before.iter().zip(after.iter()).enumerate() {
        if i != index {
            assert_eq!(old, new, "skill {i} changed unexpectedly");
        }
    }
}

#[test]
fn out_of_range_skill_errors() {
    let mut sg = synthetic();
    assert!(sg.set_skill_grade(9999, 5).is_err());
}

#[test]
fn rebuilds_the_skill_list() {
    let mut sg = synthetic();
    let paths: Vec<String> = vec![
        "GD_Soldier_Skills.Scorpio.Skill_Scorpio".into(),
        "GD_Soldier_Skills.Guerrilla.Sentry".into(),
        "GD_Soldier_Skills.Guerrilla.Ready".into(),
    ];
    let others_before: Vec<u32> = sg
        .message
        .fields
        .iter()
        .map(|f| f.number)
        .filter(|n| *n != 8)
        .collect();

    sg.set_skill_list(&paths);
    let skills = sg.skills();
    assert_eq!(skills.len(), paths.len());
    assert_eq!(skills[0].skill, paths[0]);
    assert!(
        skills
            .iter()
            .all(|s| s.grade == 0 && s.grade_points == 0 && s.equipped_slot_index == 0),
        "a rebuilt list starts clean"
    );

    // Only the skill list changed.
    let others_after: Vec<u32> = sg
        .message
        .fields
        .iter()
        .map(|f| f.number)
        .filter(|n| *n != 8)
        .collect();
    assert_eq!(others_before, others_after);

    // Round-trips through the container byte-for-byte cleanly.
    let bytes = container::serialize(&sg.to_bytes(), Platform::PC).expect("serialize");
    let reopened = container::unpack(&bytes, Platform::PC).expect("reopen");
    assert!(reopened.crc32_valid() && reopened.proto_stable() && reopened.huffman_stable());
    let ids: Vec<String> = reopened
        .save_game()
        .expect("reparse")
        .skills()
        .into_iter()
        .map(|s| s.skill)
        .collect();
    assert_eq!(ids, paths);
}
