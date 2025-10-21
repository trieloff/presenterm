use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Rainbow animation - Full spectrum colors cycling through characters
pub(crate) struct Rainbow;

impl Animation for Rainbow {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let base_hue = (ctx.char_index as f32 / ctx.total_chars as f32) * 360.0;
        let hue = (base_hue + ctx.hue_offset) % 360.0;
        CharAnimationResult::with_color(hsl_to_rgb(hue, 100.0, 50.0))
    }
}
