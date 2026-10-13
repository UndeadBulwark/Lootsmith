//! Visual baselines for every class's skill tree (dark, 1x). Regenerate with
//! `UPDATE_SNAPSHOTS=1 cargo test -p lootsmith-gui --test skills_snapshots`.
//!
//! One test per class: egui_kittest wants a single Harness per snapshot.

mod common;

use eframe::egui::Vec2;
use lootsmith_gui::app::Tab;

/// A fixed per-class path: the save filename is part of the pixels, so it must
/// not contain the process id.
fn synthetic_at(tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("lootsmith-skill-{tag}.sav"));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    common::freeze_mtime(&path);
    path.display().to_string()
}

macro_rules! skill_shot {
    ($name:ident, $class:literal) => {
        #[test]
        fn $name() {
            let save = synthetic_at(stringify!($name));
            let mut h = common::harness_sized(&save, None, Vec2::new(1280.0, 1180.0));
            h.state_mut().read_info.player_class = $class.to_string();
            h.state_mut().set_tab(Tab::Skills);
            h.run();
            h.snapshot(concat!("skills_", stringify!($name), "_dark_1x"));
        }
    };
}

skill_shot!(soldier, "GD_Soldier.Character.CharClass_Soldier");
skill_shot!(gunzerker, "GD_Mercenary.Character.CharClass_Mercenary");
skill_shot!(assassin, "GD_Assassin.Character.CharClass_Assassin");
skill_shot!(siren, "GD_Siren.Character.CharClass_Siren");
skill_shot!(
    mechromancer,
    "GD_Tulip_Mechromancer.Character.CharClass_Mechromancer"
);
skill_shot!(
    psycho,
    "GD_Lilac_PlayerClass.Character.CharClass_LilacPlayerClass"
);
