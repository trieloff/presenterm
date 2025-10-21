use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Typewriter animation - Gradual left-to-right reveal effect
pub(crate) struct Typewriter;

impl Animation for Typewriter {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        // Use a longer duration to ensure we have time to reveal all characters
        // and add settling time before animation completes
        let typing_duration = 320.0; // Time to type all characters
        let settle_time = 40.0; // Brief pause after typing completes
        let total_duration = typing_duration + settle_time;

        if ctx.hue_offset > total_duration {
            // Animation fully complete: all text revealed, no caret
            let color = hsl_to_rgb(40.0, 20.0, 85.0);
            return CharAnimationResult::with_color(color);
        }

        let progress = (ctx.hue_offset / typing_duration).clamp(0.0, 1.0);
        // Add 1 to ensure we reach total_chars, not total_chars-1
        let reveal_count = ((progress * ctx.total_chars as f32).floor() as usize).min(ctx.total_chars);

        if ctx.char_index < reveal_count {
            // Already revealed: warm white ink
            let color = hsl_to_rgb(40.0, 20.0, 85.0);
            CharAnimationResult::with_color(color)
        } else if ctx.char_index == reveal_count && reveal_count < ctx.total_chars {
            // Currently being typed: show with caret (only if not at end)
            let color = hsl_to_rgb(200.0, 85.0, 65.0);
            // Draw a block caret instead of the ASCII glyph to emphasize typing
            CharAnimationResult::with_replacement(color, '▌')
        } else if reveal_count >= ctx.total_chars {
            // All characters revealed but still in settling time
            let color = hsl_to_rgb(40.0, 20.0, 85.0);
            CharAnimationResult::with_color(color)
        } else {
            // Not yet revealed: render as space (no ink)
            let color = hsl_to_rgb(0.0, 0.0, 0.0);
            CharAnimationResult::with_replacement(color, ' ')
        }
    }
}
