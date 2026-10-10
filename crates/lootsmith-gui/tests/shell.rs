//! Stage 2: the shell regions (spec §2.2 — 56 px top bar, 204 px sidebar,
//! 34 px status bar) and every screen rendering inside them.

mod common;

use egui_kittest::kittest::Queryable;
use lootsmith_gui::app::{LootsmithApp, Tab};

const SCREENS: [Tab; 13] = [
    Tab::General,
    Tab::Character,
    Tab::Currency,
    Tab::Ammo,
    Tab::World,
    Tab::Skills,
    Tab::Travel,
    Tab::Vehicles,
    Tab::Missions,
    Tab::Backpack,
    Tab::Bank,
    Tab::Raw,
    Tab::About,
];

fn save_path() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let n = N.fetch_add(1, Ordering::Relaxed);
    let path = std::env::temp_dir().join(format!("lootsmith-shell-{}-{n}.sav", std::process::id()));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    path.display().to_string()
}

/// The sidebar label the shell produces for a tab (Backpack/Bank carry counts).
fn sidebar_label(state: &LootsmithApp, tab: Tab) -> String {
    match tab {
        Tab::Backpack => format!("Backpack ({})", state.backpack.len()),
        Tab::Bank => format!("Bank ({})", state.bank.len()),
        other => [
            (Tab::General, "General"),
            (Tab::Character, "Character"),
            (Tab::Currency, "Currency"),
            (Tab::Ammo, "Ammo"),
            (Tab::World, "World"),
            (Tab::Skills, "Skills"),
            (Tab::Travel, "Fast Travel"),
            (Tab::Vehicles, "Vehicles"),
            (Tab::Missions, "Missions"),
            (Tab::Raw, "Raw"),
            (Tab::About, "About"),
        ]
        .iter()
        .find(|(t, _)| *t == other)
        .map(|(_, l)| (*l).to_string())
        .expect("known tab"),
    }
}

fn approx(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

#[test]
fn shell_rects_match_the_spec_sizes() {
    let mut h = common::harness_isolated(&save_path());
    h.run();
    let r = h.state().shell;
    assert!(approx(r.top.height(), 56.0), "top bar {:?}", r.top);
    assert!(approx(r.sidebar.width(), 204.0), "sidebar {:?}", r.sidebar);
    assert!(approx(r.status.height(), 34.0), "status bar {:?}", r.status);
    // The sidebar sits under the top bar and above the status bar.
    assert!(approx(r.sidebar.min.y, r.top.max.y), "sidebar top");
    assert!(approx(r.sidebar.max.y, r.status.min.y), "sidebar bottom");
    // The content fills the space right of the sidebar, between top and status.
    assert!(approx(r.content.min.x, r.sidebar.max.x), "content left");
    // A 28 px screen header ribbon sits between the top bar and the screen.
    assert!(approx(r.content.min.y, r.top.max.y + 28.0), "content top");
    assert!(approx(r.content.max.y, r.status.min.y), "content bottom");
    assert!(
        r.content.width() > 0.0 && r.content.height() > 0.0,
        "content"
    );
    // The top bar and status bar span the full width.
    assert!(approx(r.top.width(), r.status.width()), "bars share width");
    assert!(approx(r.top.min.x, r.status.min.x), "bars share left");
}

#[test]
fn sidebar_exposes_every_screen_with_count_badges() {
    let mut h = common::harness_isolated(&save_path());
    h.run();
    assert!(
        !h.state().backpack.is_empty(),
        "synthetic save has backpack items"
    );
    for tab in SCREENS {
        let label = sidebar_label(h.state(), tab);
        assert!(
            h.query_all_by_label(&label).count() >= 1,
            "missing sidebar label {label:?}"
        );
    }
}

#[test]
fn every_screen_renders_inside_the_shell() {
    let mut h = common::harness_isolated(&save_path());
    h.run();
    for tab in SCREENS {
        h.state_mut().tab = tab;
        h.run();
        let label = sidebar_label(h.state(), tab);
        let r = h.state().shell;
        assert!(approx(r.top.height(), 56.0), "{label}: shell changed");
        assert!(approx(r.sidebar.width(), 204.0), "{label}: sidebar changed");
        let img = h.render().expect("render");
        assert!(img.width() > 0 && img.height() > 0, "{label}: empty render");
        assert!(
            h.query_all_by_label(&label).count() >= 1,
            "{label}: sidebar label gone"
        );
    }
}
