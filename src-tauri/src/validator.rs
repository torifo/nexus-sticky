/// ウィンドウサイズ・位置の制約定義
pub struct WindowConstraints {
    pub min_width: u32,
    pub min_height: u32,
    pub max_width_ratio: f64,
    pub max_height_ratio: f64,
}

impl Default for WindowConstraints {
    fn default() -> Self {
        WindowConstraints {
            min_width: 200,   // Requirement 9.3
            min_height: 150,  // Requirement 9.3
            max_width_ratio: 0.8,  // Requirement 9.4
            max_height_ratio: 0.8, // Requirement 9.4
        }
    }
}

/// ウィンドウ位置・サイズの制約を検証するバリデーター (Requirement 9)
pub struct WindowValidator {
    pub screen_width: u32,
    pub screen_height: u32,
    pub constraints: WindowConstraints,
}

impl WindowValidator {
    pub fn new(screen_width: u32, screen_height: u32) -> Self {
        WindowValidator {
            screen_width,
            screen_height,
            constraints: WindowConstraints::default(),
        }
    }

    /// サイズを最小・最大制約内に収める (Requirement 9.3, 9.4)
    pub fn validate_size(&self, width: u32, height: u32) -> (u32, u32) {
        let max_width = (self.screen_width as f64 * self.constraints.max_width_ratio) as u32;
        let max_height = (self.screen_height as f64 * self.constraints.max_height_ratio) as u32;

        let w = width
            .max(self.constraints.min_width)
            .min(max_width.max(self.constraints.min_width));
        let h = height
            .max(self.constraints.min_height)
            .min(max_height.max(self.constraints.min_height));
        (w, h)
    }

    /// 位置をスクリーン境界内に収める (Requirement 9.1, 9.2)
    pub fn validate_position(&self, x: i32, y: i32, width: u32, height: u32) -> (i32, i32) {
        let max_x = (self.screen_width as i32) - (width as i32);
        let max_y = (self.screen_height as i32) - (height as i32);

        let cx = x.max(0).min(max_x.max(0));
        let cy = y.max(0).min(max_y.max(0));
        (cx, cy)
    }

    /// サイズと位置を総合的に検証してスクリーン内に収める
    pub fn clamp_to_screen(
        &self,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    ) -> ((i32, i32), (u32, u32)) {
        let (w, h) = self.validate_size(width, height);
        let (cx, cy) = self.validate_position(x, y, w, h);
        ((cx, cy), (w, h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn validator() -> WindowValidator {
        WindowValidator::new(1920, 1080)
    }

    #[test]
    fn test_size_below_min() {
        let v = validator();
        let (w, h) = v.validate_size(50, 30);
        assert_eq!(w, 200, "Width must be at least 200");
        assert_eq!(h, 150, "Height must be at least 150");
    }

    #[test]
    fn test_size_above_max() {
        let v = validator();
        let (w, h) = v.validate_size(9999, 9999);
        assert!(w <= (1920.0 * 0.8) as u32, "Width must not exceed 80% of screen");
        assert!(h <= (1080.0 * 0.8) as u32, "Height must not exceed 80% of screen");
    }

    #[test]
    fn test_size_valid_range() {
        let v = validator();
        let (w, h) = v.validate_size(300, 250);
        assert_eq!(w, 300);
        assert_eq!(h, 250);
    }

    /// Property 4: 位置制約の保持 (Requirement 9.1, 9.2)
    #[test]
    fn test_position_property_always_within_screen() {
        let v = validator();
        let test_cases = [
            (-500, -500, 300u32, 250u32),
            (0, 0, 300, 250),
            (1920, 1080, 300, 250),
            (3000, 3000, 300, 250),
            (-1, 0, 300, 250),
        ];
        for (px, py, pw, ph) in test_cases {
            let (x, y) = v.validate_position(px, py, pw, ph);
            assert!(x >= 0, "x={} must be >= 0 (input: {}, {}, {}, {})", x, px, py, pw, ph);
            assert!(y >= 0, "y={} must be >= 0", y);
            assert!(
                x + pw as i32 <= 1920,
                "Window right edge {} must be <= screen width",
                x + pw as i32
            );
            assert!(
                y + ph as i32 <= 1080,
                "Window bottom edge {} must be <= screen height",
                y + ph as i32
            );
        }
    }

    /// Property 5: サイズ制約の保持 (Requirement 9.3, 9.4)
    #[test]
    fn test_size_property_always_within_constraints() {
        let v = validator();
        let test_sizes = [0u32, 1, 100, 200, 500, 1000, 2000, 10000];
        for &s in &test_sizes {
            let (w, h) = v.validate_size(s, s);
            assert!(w >= 200, "Width {} must be >= 200", w);
            assert!(h >= 150, "Height {} must be >= 150", h);
            assert!(w <= (1920.0 * 0.8) as u32, "Width {} must be <= 80% screen", w);
            assert!(h <= (1080.0 * 0.8) as u32, "Height {} must be <= 80% screen", h);
        }
    }
}
