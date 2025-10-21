use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};
use super::matrix_chars;

/// Matrix animation - Matrix-style digital rain with authentic green shades
pub(crate) struct Matrix;

impl Animation for Matrix {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let hue = 120.0; // Green base

        // Create pseudo-random variation per character for authentic look
        let char_seed = (ctx.char_index as f32 * 7.919 + ctx.row_index as f32 * 3.141).sin() * 100.0;
        let char_variation = char_seed.fract();

        // Cascade effect: brightness falls down the rows over time
        let time_offset = ctx.hue_offset / 8.0;
        let cascade_pos = ctx.row_index as f32 - time_offset;

        // Animation completion: add settle time after cascade finishes
        let animation_complete = time_offset > (ctx.total_rows as f32 + 3.0);
        let cascade_has_passed = time_offset > ctx.row_index as f32;

        // Characters near the cascade front are brightest (white-green)
        // Then fade to bright green, then stable after reveal
        let distance_from_front = cascade_pos.abs();

        let (saturation, lightness) = if animation_complete {
            // Animation fully complete: stable final state
            (90.0, 55.0)
        } else if distance_from_front < 1.0 && !cascade_has_passed {
            // Leading edge: bright white-green (low saturation, high lightness)
            (40.0 + char_variation * 20.0, 75.0 + char_variation * 15.0)
        } else if distance_from_front < 3.0 && !cascade_has_passed {
            // Bright green trail
            (80.0 + char_variation * 20.0, 55.0 + char_variation * 15.0)
        } else if cascade_has_passed {
            // After cascade passes: stable bright green (revealed state)
            (90.0, 50.0 + char_variation * 10.0)
        } else {
            // Dark green / not yet revealed
            (70.0, 15.0 + char_variation * 10.0)
        };

        // Character replacement logic:
        // - Animation complete: ALWAYS show actual character (final clean state)
        // - Before cascade: show Matrix glyphs
        // - After cascade but before complete: show actual character with rare glitches
        let replacement = if animation_complete {
            // Final frame: no replacements, show actual text
            None
        } else if cascade_has_passed {
            // Cascade has passed - show actual character with occasional glitch
            let glitch_chance = (char_seed * 37.1 + ctx.hue_offset * 0.03).sin();
            if glitch_chance > 0.95 {
                // Rare glitch: briefly show Matrix character
                Some(matrix_chars::get_char(char_seed + ctx.hue_offset * 10.0))
            } else {
                // Normal: show actual character
                None
            }
        } else {
            // Cascade hasn't reached yet - show Matrix characters
            Some(matrix_chars::get_char(char_seed + ctx.hue_offset * 0.5))
        };

        let color = hsl_to_rgb(hue, saturation, lightness);
        if let Some(ch) = replacement {
            CharAnimationResult::with_replacement(color, ch)
        } else {
            CharAnimationResult::with_color(color)
        }
    }
}
