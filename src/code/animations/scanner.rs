use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Scanner animation - Horizontal scan line effect like KITT from Knight Rider
pub(crate) struct Scanner;

impl Animation for Scanner {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let scan_pos = (ctx.hue_offset / 360.0) * ctx.total_chars as f32;
        let dist = (ctx.char_index as f32 - scan_pos).abs();
        let lightness = (70.0 - (dist * 8.0)).max(30.0);
        // Classic red scanner
        CharAnimationResult::with_color(hsl_to_rgb(0.0, 100.0, lightness))
    }
}
