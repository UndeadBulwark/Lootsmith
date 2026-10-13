//! End-to-end: the app settles the **Auto** UI scale on a simulated 4K monitor
//! and then never keeps resizing (spec §2.1 rule 7).

mod common;

use eframe::egui::{vec2, ViewportId};
use lootsmith_gui::app::ScaleMode;

fn synthetic(tag: &str) -> String {
    let path = std::env::temp_dir().join(format!("lootsmith-autoscale-{tag}.sav"));
    lootsmith_core::fixture::write_synthetic_save(&path).expect("write synthetic save");
    path.display().to_string()
}

#[test]
fn auto_scale_settles_on_4k_and_does_not_keep_resizing() {
    let save = synthetic("4k");
    // A 4K monitor at 2× means a 2560×1600 pt logical window; give the harness that
    // physical size so the fit does not cap the scale.
    let mut h = common::harness_sized(&save, None, egui::vec2(2560.0, 1600.0));
    h.state_mut().set_scale_mode(ScaleMode::Auto);
    h.run();

    // Simulate a 3840x2160 **physical** monitor. egui-winit reports `monitor_size`
    // in logical points of the zoom in force (physical / pixels_per_point), so feed
    // that each frame the way winit would.
    let physical = (3840.0f32, 2160.0f32);
    let mut changes = 0usize;
    let mut last = h.ctx.pixels_per_point();
    for _ in 0..60 {
        let ppp = h.ctx.pixels_per_point().max(0.1);
        let logical = vec2(physical.0 / ppp, physical.1 / ppp);
        h.input_mut()
            .viewports
            .entry(ViewportId::ROOT)
            .or_default()
            .monitor_size = Some(logical);
        h.step();
        let now = h.ctx.pixels_per_point();
        if (now - last).abs() > 1e-3 {
            changes += 1;
        }
        last = now;
    }
    assert!(
        (last - 2.0).abs() < 1e-3,
        "expected 2x on a 4K monitor, got {last}"
    );
    assert!(changes <= 3, "ppp changed {changes} times (should settle)");
    assert_eq!(
        h.state().ui_scale,
        2.0,
        "ui_scale follows the applied scale"
    );
}
