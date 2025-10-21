use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Breathe animation - Gentle synchronized breathing effect
pub(crate) struct Breathe;

impl Animation for Breathe {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let hue = (ctx.char_index as f32 / ctx.total_chars as f32) * 360.0;
        let saturation = 65.0; // Calming, not too vibrant
        // Synchronized breathing: all characters pulse together
        let lightness = 35.0 + 20.0 * (ctx.hue_offset * 0.05).sin();
        CharAnimationResult::with_color(hsl_to_rgb(hue % 360.0, saturation, lightness))
    }
}
