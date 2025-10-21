use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Neon animation - Bright neon sign colors cycling through classic neon palette
pub(crate) struct Neon;

impl Animation for Neon {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        // Neon color palette: hot pink, electric blue, lime green, violet
        let palette = [330.0, 195.0, 85.0, 280.0];
        let palette_len = palette.len() as f32;

        // Cycle through palette with character position
        let cycle_position = (ctx.hue_offset + ctx.char_index as f32 * 5.0) % (palette_len * 90.0);
        let palette_index = (cycle_position / 90.0).floor() as usize % palette.len();
        let next_index = (palette_index + 1) % palette.len();

        // Interpolate between current and next palette color
        let t = (cycle_position % 90.0) / 90.0;
        let hue = palette[palette_index] * (1.0 - t) + palette[next_index] * t;

        // Bright neon glow with slight pulse effect
        let base_lightness = 60.0;
        let pulse = (ctx.hue_offset * 0.1).sin() * 5.0;
        let lightness = base_lightness + pulse;

        CharAnimationResult::with_color(hsl_to_rgb(hue % 360.0, 100.0, lightness))
    }
}
