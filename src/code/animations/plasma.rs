use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Plasma animation - Psychedelic plasma effect with overlapping sine waves
pub(crate) struct Plasma;

impl Animation for Plasma {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let pos = ctx.char_index as f32;
        let time = ctx.hue_offset;

        // Combine multiple sine waves with different frequencies and phases
        let wave1 = (pos * 0.1 + time * 0.02).sin();
        let wave2 = (pos * 0.13 + time * 0.03).sin();
        let wave3 = (pos * 0.08 + time * 0.025).cos();

        // Average the waves and map from -1..1 to 0..360
        let hue = ((wave1 + wave2 + wave3) / 3.0 + 1.0) * 180.0;
        CharAnimationResult::with_color(hsl_to_rgb(hue % 360.0, 100.0, 50.0))
    }
}
