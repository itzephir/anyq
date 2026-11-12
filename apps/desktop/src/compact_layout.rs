use tauri::{PhysicalPosition, PhysicalRect, PhysicalSize};

pub struct CompactLayout {
    pub position: PhysicalPosition<i32>,
    pub size: PhysicalSize<u32>,
}

impl CompactLayout {
    pub fn anchored(
        bounds: PhysicalRect<i32, u32>,
        icon: PhysicalRect<f64, f64>,
        scale: f64,
    ) -> Self {
        let margin = (8.0 * scale).round() as i64;
        let width = (380.0 * scale).round() as u32;
        let height = (355.0 * scale).round() as u32;
        let inset_x = margin.min(i64::from(bounds.size.width.saturating_sub(1)) / 2);
        let inset_y = margin.min(i64::from(bounds.size.height.saturating_sub(1)) / 2);
        let width = width.min((i64::from(bounds.size.width) - 2 * inset_x).max(1) as u32);
        let height = height.min((i64::from(bounds.size.height) - 2 * inset_y).max(1) as u32);
        let left = i64::from(bounds.position.x) + inset_x;
        let top = i64::from(bounds.position.y) + inset_y;
        let right = i64::from(bounds.position.x) + i64::from(bounds.size.width) - inset_x;
        let bottom = i64::from(bounds.position.y) + i64::from(bounds.size.height) - inset_y;

        let x = (icon.position.x + icon.size.width / 2.0).round() as i64 - i64::from(width) / 2;
        let below = (icon.position.y + icon.size.height).round() as i64 + margin;
        let y = if below + i64::from(height) <= bottom {
            below
        } else {
            icon.position.y.round() as i64 - i64::from(height) - margin
        };

        Self {
            position: PhysicalPosition::new(
                x.clamp(left, right - i64::from(width)) as i32,
                y.clamp(top, bottom - i64::from(height)) as i32,
            ),
            size: PhysicalSize::new(width, height),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaled_panels_fit_negative_secondary_monitors_and_every_taskbar_edge() {
        for scale in [1.0, 1.25, 1.5, 2.0] {
            for (x, y, width, height) in [
                (0, 24, 1920, 1016),
                (-2560, -240, 2560, 1400),
                (1920, 80, 1200, 1600),
                (-400, 0, 320, 240),
            ] {
                let bounds = PhysicalRect {
                    position: PhysicalPosition::new(x, y),
                    size: PhysicalSize::new(width, height),
                };
                for (icon_x, icon_y) in [
                    (x, y - 24),
                    (x + width as i32 - 24, y - 24),
                    (x, y + height as i32),
                    (x + width as i32, y + height as i32 / 2),
                ] {
                    let layout = CompactLayout::anchored(
                        bounds,
                        PhysicalRect {
                            position: PhysicalPosition::new(f64::from(icon_x), f64::from(icon_y)),
                            size: PhysicalSize::new(24.0, 24.0),
                        },
                        scale,
                    );

                    assert!(layout.position.x >= x);
                    assert!(layout.position.y >= y);
                    assert!(layout.position.x + layout.size.width as i32 <= x + width as i32);
                    assert!(layout.position.y + layout.size.height as i32 <= y + height as i32);
                }
            }
        }
    }
}
