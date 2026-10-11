//! Auto UI scale: pick a whole-number scale from the monitor's **physical** size
//! and apply it **once**, without oscillating (spec §2.1 rule 7).
//!
//! Verified against egui / egui-winit 0.36:
//!   * egui-winit `update_viewport_info` sets `monitor_size = physical / (native *
//!     ctx.zoom_factor())`, read **before** `begin_pass` of the same frame.
//!   * `Context::set_pixels_per_point(p)` is `set_zoom_factor(p / native)`; a new
//!     zoom is only applied at the NEXT `begin_pass`.
//!
//! So the zoom that produced this frame's `monitor_size` is the ppp the app
//! observed on the **previous** frame (`prev_ppp`), not `ctx.pixels_per_point()`
//! now. Using the wrong one makes the estimate wrong on the first frame after
//! every change, which oscillates (see the simulator tests).
//!
//! Pure std: no egui types, so it compiles and tests anywhere.

/// 1280x800 reference layout. Whole numbers only (spec §2.1 rule 7).
pub fn scale_for(physical_w: u32, physical_h: u32) -> u32 {
    (physical_w / 1280).min(physical_h / 800).clamp(1, 4)
}

/// The zoom factor that makes the total `pixels_per_point` an exact integer `n`
/// whatever the OS scale is.
pub fn zoom_for(n: u32, native_ppp: f32) -> f32 {
    n as f32 / native_ppp.max(0.1)
}

/// Latching auto-scale. Decides at most once per distinct physical monitor size.
#[derive(Debug, Clone, Default)]
pub struct AutoScale {
    prev_ppp: Option<f32>,
    last_physical: Option<(u32, u32)>,
    applied: Option<u32>,
}

impl AutoScale {
    /// Seed from the persisted scale so frame 0 is already right (no jump).
    pub fn with_persisted(scale: Option<u32>) -> Self {
        Self {
            prev_ppp: None,
            last_physical: None,
            applied: scale.map(|s| s.clamp(1, 4)),
        }
    }

    /// The scale last decided (or seeded), if any.
    pub fn applied(&self) -> Option<u32> {
        self.applied
    }

    /// Call exactly once per frame, from `build()`.
    ///
    /// * `monitor_logical` — `ctx.input(|i| i.viewport().monitor_size).map(|m| (m.x, m.y))`
    /// * `ppp_now` — `ctx.pixels_per_point()`
    /// * `native_ppp` — `ctx.native_pixels_per_point().unwrap_or(1.0)`
    ///
    /// Returns `Some(n)` only when the scale must change.
    pub fn update(
        &mut self,
        monitor_logical: Option<(f32, f32)>,
        ppp_now: f32,
        native_ppp: f32,
    ) -> Option<u32> {
        // The zoom winit used for this frame's monitor_size = ppp seen last frame.
        let basis = self.prev_ppp.replace(ppp_now);
        let target = match (monitor_logical, basis) {
            (Some((mx, my)), Some(b)) if mx > 0.0 && my > 0.0 => {
                let phys = ((mx * b).round() as u32, (my * b).round() as u32);
                if self.last_physical == Some(phys) {
                    return None; // same monitor: nothing to decide
                }
                self.last_physical = Some(phys);
                scale_for(phys.0, phys.1)
            }
            (Some(_), None) => return None, // first frame: no basis yet; decide next frame
            _ => {
                // No monitor info (some Wayland setups, test harness): decide once
                // from the OS scale.
                if self.applied.is_some() {
                    return None;
                }
                (native_ppp.round() as u32).clamp(1, 4)
            }
        };
        if self.applied == Some(target) {
            return None;
        }
        self.applied = Some(target);
        Some(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Models egui's frame order: winit builds input with the zoom in force BEFORE
    /// `begin_pass`; `begin_pass` then applies the zoom the app set last frame.
    struct Sim {
        native: f32,
        phys: (u32, u32),
        zoom: f32,
        pending: Option<f32>,
    }
    impl Sim {
        fn new(native: f32, phys: (u32, u32)) -> Self {
            Self {
                native,
                phys,
                zoom: 1.0,
                pending: None,
            }
        }
        fn frame(
            &mut self,
            decide: &mut dyn FnMut(Option<(f32, f32)>, f32, f32) -> Option<u32>,
        ) -> f32 {
            let mon = (
                self.phys.0 as f32 / (self.native * self.zoom),
                self.phys.1 as f32 / (self.native * self.zoom),
            );
            if let Some(z) = self.pending.take() {
                self.zoom = z; // begin_pass
            }
            let ppp_now = self.native * self.zoom;
            if let Some(n) = decide(Some(mon), ppp_now, self.native) {
                self.pending = Some(zoom_for(n, self.native));
            }
            ppp_now
        }
    }
    fn changes(
        native: f32,
        phys: (u32, u32),
        decide: &mut dyn FnMut(Option<(f32, f32)>, f32, f32) -> Option<u32>,
    ) -> (usize, f32) {
        let mut sim = Sim::new(native, phys);
        let (mut last, mut n) = (0.0f32, 0usize);
        for i in 0..300 {
            let p = sim.frame(decide);
            if i > 0 && (p - last).abs() > 1e-4 {
                n += 1;
            }
            last = p;
        }
        (n, last)
    }
    const MONITORS: [(u32, u32); 6] = [
        (1920, 1080),
        (2560, 1440),
        (3840, 2160),
        (1366, 768),
        (5120, 2880),
        (3440, 1440),
    ];
    const NATIVES: [f32; 4] = [1.0, 1.25, 1.5, 2.0];

    #[test]
    fn settles_on_the_right_integer_and_stays() {
        for native in NATIVES {
            for m in MONITORS {
                let mut a = AutoScale::default();
                let (n, ppp) = changes(native, m, &mut |mon, p, nat| a.update(mon, p, nat));
                let want = scale_for(m.0, m.1) as f32;
                assert!(n <= 2, "native {native} monitor {m:?}: {n} ppp changes");
                assert!(
                    (ppp - want).abs() < 1e-3,
                    "native {native} monitor {m:?}: ended at {ppp}, want {want}"
                );
            }
        }
    }

    /// The app applies the persisted scale on frame 0, so a correct seed never changes ppp.
    #[test]
    fn persisted_seed_does_not_jump() {
        let seed = scale_for(3840, 2160); // 2
        let mut a = AutoScale::with_persisted(Some(seed));
        let mut sim = Sim::new(1.0, (3840, 2160));
        sim.zoom = zoom_for(seed, 1.0); // applied at startup by the app
        let mut n = 0;
        let mut last = sim.native * sim.zoom;
        for _ in 0..300 {
            let p = sim.frame(&mut |mon, p, nat| a.update(mon, p, nat));
            if (p - last).abs() > 1e-4 {
                n += 1;
            }
            last = p;
        }
        assert_eq!(n, 0, "a correct persisted seed must not change the scale");
        // A stale seed (monitor changed since last run) corrects itself exactly once.
        let mut a = AutoScale::with_persisted(Some(3));
        let (n, ppp) = changes(1.0, (3840, 2160), &mut |mon, p, nat| a.update(mon, p, nat));
        assert!(n <= 1 && (ppp - 2.0).abs() < 1e-3, "n={n} ppp={ppp}");
    }

    #[test]
    fn no_monitor_info_decides_once() {
        let mut a = AutoScale::default();
        let mut got = vec![];
        for _ in 0..50 {
            got.push(a.update(None, 1.0, 2.0));
        }
        assert_eq!(got.iter().flatten().count(), 1);
    }

    /// Regression proof: the original per-frame `effective_scale` oscillates here.
    #[test]
    fn old_effective_scale_flickers() {
        let (n, _) = changes(1.0, (3840, 2160), &mut |mon, _p, nat| {
            let (mx, my) = mon.unwrap();
            Some(scale_for((mx * nat) as u32, (my * nat) as u32)) // physical = monitor * native
        });
        assert!(n > 50, "expected the old logic to flicker, got {n} changes");
    }

    /// The first drop-in (basis = `ppp_now`) is unstable too: documents why
    /// `prev_ppp` is needed.
    #[test]
    fn using_ppp_now_as_basis_is_unstable() {
        let mut last: Option<(u32, u32)> = None;
        let mut applied: Option<u32> = None;
        let (n, _) = changes(1.0, (3840, 2160), &mut |mon, ppp_now, _| {
            let (mx, my) = mon?;
            let phys = ((mx * ppp_now).round() as u32, (my * ppp_now).round() as u32);
            if last == Some(phys) {
                return None;
            }
            last = Some(phys);
            let t = scale_for(phys.0, phys.1);
            if applied == Some(t) {
                return None;
            }
            applied = Some(t);
            Some(t)
        });
        assert!(n > 4, "expected instability, got {n} changes");
    }
}
