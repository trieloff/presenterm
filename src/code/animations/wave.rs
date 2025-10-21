use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Wave animation - Hue oscillates along a sine wave with character position
pub(crate) struct Wave;

impl Animation for Wave {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let base = 200.0; // blue-ish base
        let amplitude = 60.0;
        let freq = 0.35; // chars per cycle
        let phase = ctx.hue_offset.to_radians();
        let hue = base + amplitude * ((ctx.char_index as f32 * freq + phase).sin());
        CharAnimationResult::with_color(hsl_to_rgb(((hue % 360.0) + 360.0) % 360.0, 100.0, 50.0))
    }
}
