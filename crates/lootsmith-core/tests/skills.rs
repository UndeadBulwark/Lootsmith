//! SkillData (field 8) accessors and write path.
//!
//! Each entry is `{1: Skill, 2: Grade, 3: GradePoints, 4: EquippedSlotIndex}`.

use lootsmith_core::{container, Platform};

fn fixture_path() -> String {
    format!(
        "{}/../../tests/fixtures/pc_Save0001.sav",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn fixture_save() -> Option<lootsmith_core::SaveGame> {
    let data = std::fs::read(fixture_path()).ok()?;
    let save = container::unpack(&data, Platform::PC).ok()?;
    save.save_game().ok()
}

#[test]
fn fixture_skills_enumerate() {
    let Some(sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let skills = sg.skills();
    assert_eq!(skills.len(), 33, "the soldier fixture has 33 skill entries");
    assert!(
        skills.iter().all(|s| s.skill.starts_with("GD_")),
        "skill names are resource paths"
    );
    assert!(
        skills.iter().any(|s| s.grade > 0),
        "a leveled character has invested ranks"
    );
}

#[test]
fn edits_a_skill_grade_and_reloads() {
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    let before = sg.skills();
    // Pick a skill at rank 0 and raise it.
    let index = before.iter().position(|s| s.grade == 0).expect("a rank-0 skill");
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
    let Some(mut sg) = fixture_save() else {
        eprintln!("fixture not present; skipping");
        return;
    };
    assert!(sg.set_skill_grade(9999, 5).is_err());
}
