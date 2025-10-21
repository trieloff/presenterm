use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Flash animation - Single color that cycles through hue spectrum
pub(crate) struct Flash;

impl Animation for Flash {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let hue = ctx.hue_offset % 360.0;
        CharAnimationResult::with_color(hsl_to_rgb(hue, 100.0, 50.0))
    }
}
