use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};
use super::glitch_chars;

/// Glitch animation - Cyberpunk glitch aesthetic with character corruption
pub(crate) struct Glitch;

impl Animation for Glitch {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let glitch_duration = 300.0; // Glitch for ~300 degrees of hue_offset
        let animation_complete = ctx.hue_offset > glitch_duration;

        if animation_complete {
            // Final stable state: clean green color (like terminal recovered)
            return CharAnimationResult::with_color(hsl_to_rgb(120.0, 60.0, 55.0));
        }

        // Active glitching phase
        // Use character index + hue_offset as seed for pseudo-random behavior
        let glitch_seed = (ctx.char_index as f32 * 12.9898 + ctx.hue_offset * 78.233).sin() * 43758.5453;

        // Pick color: 0=cyan (~180°), 1=magenta (~300°), 2=random
        let hue_choice = (glitch_seed.fract() * 3.0) as i32;
        let hue = match hue_choice {
            0 => 180.0, // Cyan
            1 => 300.0, // Magenta
            _ => glitch_seed.fract() * 360.0, // Random vibrant color
        };

        // Random brightness variations
        let lightness = 35.0 + (glitch_seed.fract() * 40.0);

        // High saturation for vibrant glitch effect
        let saturation = 85.0 + ((glitch_seed * 1.234).fract() * 15.0);

        // Character glitching: replace with visually similar characters
        // Use a different seed for character selection
        let char_glitch_seed = (ctx.char_index as f32 * 7.321 + ctx.hue_offset * 0.5).sin() * 12345.6789;

        // Increase glitch probability as we approach the end
        let glitch_intensity = 1.0 - (ctx.hue_offset / glitch_duration).min(1.0);
        let glitch_threshold = 0.65 - (glitch_intensity * 0.3); // 35-65% chance
        let should_glitch = char_glitch_seed.fract() > glitch_threshold;

        let color = hsl_to_rgb(hue, saturation, lightness);
        if should_glitch {
            if let Some(glitched_ch) = glitch_chars::get_glitched_char(ctx.ch, char_glitch_seed) {
                return CharAnimationResult::with_replacement(color, glitched_ch);
            }
        }
        CharAnimationResult::with_color(color)
    }
}
