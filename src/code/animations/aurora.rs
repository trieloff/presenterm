use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Aurora animation - Northern lights effect with flowing vertical curtains
pub(crate) struct Aurora;

impl Animation for Aurora {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        // Build a smooth field using overlapping sine waves across columns and time
        let t = ctx.hue_offset * 0.02; // slow time
        let x = ctx.col_index as f32;
        let y = ctx.row_index as f32;

        // Two moving wave fronts with slight vertical parallax
        let wave1 = (x * 0.12 + t + y * 0.10).sin();
        let wave2 = (x * 0.07 - t * 0.8 + y * 0.18).cos();
        let field = (wave1 + wave2) * 0.5; // -1..1

        // Map field to a hue band between 120° (green) and 280° (purple)
        let hue_min = 120.0;
        let hue_max = 280.0;
        let hue = hue_min + ((field + 1.0) * 0.5) * (hue_max - hue_min);

        // Shimmer: saturation and lightness gently fluctuate by height
        let saturation = 70.0 + (y * 0.15 + t * 0.8).sin() * 15.0;
        let base_lightness = 45.0 + field.abs() * 18.0; // brighter on wave ridges
        let lightness = base_lightness + (y * 0.25 + t).sin() * 6.0;

        CharAnimationResult::with_color(hsl_to_rgb(hue % 360.0, saturation.clamp(35.0, 100.0), lightness.clamp(30.0, 80.0)))
    }
}
