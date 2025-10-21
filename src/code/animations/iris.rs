use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Iris animation - Lightness pulse expanding from center outward
pub(crate) struct Iris;

impl Animation for Iris {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let center = (ctx.total_chars as f32) / 2.0;
        let pos = ctx.char_index as f32;
        let radius = (ctx.hue_offset / 360.0) * center.max(1.0);
        let dist = (pos - center).abs();
        let l = if dist <= radius { 60.0 } else { 35.0 };
        // Fixed hue rainbow mapping by index for variety
        let hue = (pos / ctx.total_chars as f32) * 360.0;
        CharAnimationResult::with_color(hsl_to_rgb(hue % 360.0, 100.0, l))
    }
}
