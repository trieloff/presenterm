use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Sepia animation - Subdued vintage monochrome sepia tone with gentle wave
pub(crate) struct Sepia;

impl Animation for Sepia {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let hue = 30.0; // Warm brown/sepia tone
        let saturation = 45.0; // Desaturated vintage look
        let lightness = 35.0 + 15.0 * (ctx.char_index as f32 * 0.15 + ctx.hue_offset * 0.05).sin();
        CharAnimationResult::with_color(hsl_to_rgb(hue, saturation, lightness))
    }
}
