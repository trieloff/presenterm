use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Fire animation - Flame-like effect with red/orange/yellow gradient moving upward
pub(crate) struct Fire;

impl Animation for Fire {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        // Calculate vertical position (0.0 at bottom to 1.0 at top)
        let vertical_pos = ctx.row_index as f32 / ctx.total_rows as f32;

        // Base hue for fire: 0° (red) at bottom to 60° (yellow) at top
        let base_hue = vertical_pos * 60.0;

        // Add flickering effect that moves upward
        let flicker = (ctx.hue_offset * 0.1 + ctx.char_index as f32 * 0.3).sin() * 10.0;
        let hue = (base_hue + flicker).max(0.0).min(60.0);

        // Saturation: 100% for vibrant fire colors
        let saturation = 100.0;

        // Lightness: 45-65% with random variations for flicker effect
        let base_lightness = 55.0;
        let lightness_flicker = (ctx.hue_offset * 0.15 + ctx.char_index as f32 * 0.2 + ctx.row_index as f32 * 0.4).sin() * 10.0;
        let lightness = (base_lightness + lightness_flicker).max(45.0).min(65.0);

        CharAnimationResult::with_color(hsl_to_rgb(hue, saturation, lightness))
    }
}
