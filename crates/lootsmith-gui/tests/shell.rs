//! Stage 2: the shell regions (spec §2.2 — 56 px top bar, 204 px sidebar,
//! 34 px status bar, 16 px gaps) and every screen rendering inside them.

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
            (Tab::Ammo, "Ammo & SDUs"),
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
    (a - b).abs() <= 1.5
}

#[test]
fn shell_rects_match_the_spec_sizes_at_1x_and_2x() {
    for scale in [1.0_f32, 2.0] {
        // A fixed scale of 2× needs twice the physical window so the logical
        // layout stays 1280 pt wide (and the sidebar stays expanded).
        let mut h = common::harness_sized(
            &save_path(),
            None,
            eframe::egui::Vec2::new(1280.0 * scale, 800.0 * scale),
        );
        h.state_mut().set_ui_scale(scale);
        h.run();
        let r = h.state().shell;
        assert!(
            approx(r.top.height(), 56.0),
            "scale {scale}: top {:?}",
            r.top
        );
        assert!(
            approx(r.sidebar.width(), 204.0),
            "scale {scale}: sidebar {:?}",
            r.sidebar
        );
        assert!(
            approx(r.status.height(), 34.0),
            "scale {scale}: status {:?}",
            r.status
        );
        // 16 px gaps: the body sits beside the panels, never under them.
        assert!(
            !r.content.intersects(r.sidebar),
            "scale {scale}: body under sidebar"
        );
        assert!(
            !r.content.intersects(r.top),
            "scale {scale}: body under top"
        );
        assert!(
            !r.content.intersects(r.status),
            "scale {scale}: body under status"
        );
        assert!(
            approx(r.content.min.x, r.sidebar.max.x + 16.0),
            "scale {scale}: left gap"
        );
        assert!(
            approx(r.content.min.y, r.top.max.y + 16.0),
            "scale {scale}: top gap"
        );
        assert!(
            approx(r.content.max.y, r.status.min.y - 16.0),
            "scale {scale}: bottom gap"
        );
        assert!(
            r.content.width() > 0.0 && r.content.height() > 0.0,
            "scale {scale}"
        );
    }
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
    // Command rows live in the sidebar too.
    assert!(h.query_all_by_label("Profile").count() >= 1, "Profile row");
    assert!(h.query_all_by_label("Import").count() >= 1, "Import row");
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

#[test]
fn sidebar_collapses_to_a_narrow_rail() {
    let mut h = common::harness_isolated(&save_path());
    h.run();
    assert!(
        approx(h.state().shell.sidebar.width(), 204.0),
        "expanded by default"
    );
    h.state_mut().set_sidebar_collapsed(true);
    h.run();
    assert!(
        approx(h.state().shell.sidebar.width(), 64.0),
        "collapsed to the rail"
    );
    // Every screen is still reachable from the rail (icons keep the labels).
    for tab in SCREENS {
        let label = sidebar_label(h.state(), tab);
        assert!(
            h.query_all_by_label(&label).count() >= 1,
            "rail missing {label:?}"
        );
    }
}

/// The logo itself is the collapse toggle (no tiny chevron).
#[test]
fn the_logo_toggles_the_sidebar() {
    let mut h = common::harness_isolated(&save_path());
    h.run();
    assert!(!h.state().sidebar_collapsed, "expanded by default");
    h.get_by_label("sidebar toggle").click();
    h.run_steps(30); // let the 180 ms collapse settle
    assert!(h.state().sidebar_collapsed, "clicking the logo collapses");
    h.get_by_label("sidebar toggle").click();
    h.run_steps(30);
    assert!(!h.state().sidebar_collapsed, "clicking it again expands");
}
