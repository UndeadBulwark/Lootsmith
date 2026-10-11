//! Auto UI scale: pick a whole-number scale from the monitor's **physical** size
//! and apply it **once**, without oscillating (spec §2.1 rule 7).
//!
//! egui-winit reports `viewport().monitor_size` in **logical points** (physical /
//! `pixels_per_point`). Because changing the scale changes the reported logical
//! size, recomputing the scale from `monitor_size * native` every frame makes the
//! window resize back and forth (a resize spam loop). This module fixes that:
//! it converts via the **current** `pixels_per_point` (zoom-invariant), latches the
//! result, and only reports a change when it actually differs.
use eframe::egui::Vec2;

/// 1280x800 reference layout. Whole numbers only (spec §2.1 rule 7).
pub fn scale_for(physical_w: u32, physical_h: u32) -> u32 {
    (physical_w / 1280).min(physical_h / 800).clamp(1, 4)
}

/// The zoom factor that makes the total `pixels_per_point` an exact integer `n`
/// regardless of the OS scale (`ppp = native * zoom`).
pub fn zoom_for(n: u32, native_ppp: f32) -> f32 {
    n as f32 / native_ppp.max(0.1)
}

/// Latching auto-scale. Decides at most once per distinct physical monitor size.
#[derive(Debug, Clone, Default)]
pub struct AutoScale {
    last_physical: Option<(u32, u32)>,
    applied: Option<u32>,
}

impl AutoScale {
    /// Seed from a persisted scale so frame 0 is already at the right scale (no jump).
    pub fn with_persisted(scale: Option<u32>) -> Self {
        Self {
            last_physical: None,
            applied: scale.map(|s| s.clamp(1, 4)),
        }
    }

    /// The scale last decided (or seeded), if any.
    pub fn applied(&self) -> Option<u32> {
        self.applied
    }

    /// Call once per frame. Returns `Some(n)` **only** when the scale must change.
    ///
    /// * `monitor_logical` — `ctx.input(|i| i.viewport().monitor_size)`
    /// * `current_ppp` — `ctx.pixels_per_point()` (== native * zoom)
    /// * `native_ppp` — `ctx.native_pixels_per_point().unwrap_or(1.0)`
    pub fn update(
        &mut self,
        monitor_logical: Option<Vec2>,
        current_ppp: f32,
        native_ppp: f32,
    ) -> Option<u32> {
        let target = match monitor_logical {
            Some(m) if m.x > 0.0 && m.y > 0.0 => {
                // Zoom-invariant physical size: logical * current ppp.
                let phys = (
                    (m.x * current_ppp).round() as u32,
                    (m.y * current_ppp).round() as u32,
                );
                if self.last_physical == Some(phys) {
                    return None; // same monitor — nothing to decide
                }
                self.last_physical = Some(phys);
                scale_for(phys.0, phys.1)
            }
            _ => {
                // Monitor size unknown (some Wayland setups, first frames). Decide
                // once from the OS scale and never oscillate.
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
    use eframe::egui::vec2;

    #[test]
    fn table() {
        for (w, h, n) in [
            (1280u32, 800u32, 1u32),
            (1920, 1080, 1),
            (2560, 1440, 1),
            (3840, 2160, 2),
            (5120, 2880, 3),
            (7680, 4320, 4),
        ] {
            assert_eq!(scale_for(w, h), n, "{w}x{h}");
        }
    }

    /// Simulates egui: `monitor_size` is reported in logical points of the
    /// **current** ppp, which changes when we apply a new scale.
    fn run(phys: (f32, f32), native: f32, frames: usize) -> Vec<u32> {
        let mut a = AutoScale::default();
        let mut ppp = native; // zoom starts at 1
        let mut changes = vec![];
        for _ in 0..frames {
            let logical = vec2(phys.0 / ppp, phys.1 / ppp);
            if let Some(n) = a.update(Some(logical), ppp, native) {
                changes.push(n);
                ppp = n as f32; // = zoom_for(n, native) * native
            }
        }
        changes
    }

    #[test]
    fn settles_once_on_4k_for_any_os_scale() {
        for native in [1.0, 1.5, 2.0] {
            assert_eq!(
                run((3840.0, 2160.0), native, 100),
                vec![2],
                "native {native}"
            );
        }
    }

    #[test]
    fn does_not_oscillate_for_common_layouts() {
        // Every reference size settles on exactly one change, for any OS scale.
        for (w, h, n) in [
            (1280.0f32, 800.0f32, 1u32),
            (1920.0, 1080.0, 1),
            (2560.0, 1440.0, 1),
            (5120.0, 2880.0, 3),
            (7680.0, 4320.0, 4),
        ] {
            for native in [1.0, 1.25, 1.5, 2.0] {
                assert_eq!(run((w, h), native, 50), vec![n], "{w}x{h} native {native}");
            }
        }
    }

    #[test]
    fn unknown_monitor_decides_once() {
        let mut a = AutoScale::default();
        assert_eq!(a.update(None, 1.5, 1.5), Some(2));
        for _ in 0..50 {
            assert_eq!(a.update(None, 2.0, 1.5), None);
        }
    }

    #[test]
    fn persisted_scale_means_no_first_frame_jump() {
        let mut a = AutoScale::with_persisted(Some(2));
        let m = vec2(1920.0, 1080.0); // 4K at ppp 2
        assert_eq!(a.update(Some(m), 2.0, 1.0), None);
    }
}
