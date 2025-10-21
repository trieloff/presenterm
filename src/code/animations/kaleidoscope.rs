use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Kaleidoscope animation - Psychedelic symmetrical rotating color patterns with radial symmetry
pub(crate) struct Kaleidoscope;

impl Animation for Kaleidoscope {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        // Animation phases: active -> fade-out -> complete
        let animation_duration = 360.0;
        let fadeout_start = 320.0;
        let animation_complete = ctx.hue_offset > animation_duration;

        if animation_complete {
            // Final state: maximum readability with NO background
            // Use simple rainbow for foreground, transparent background
            let base_hue = (ctx.char_index as f32 / ctx.total_chars as f32) * 360.0;
            let fg_color = hsl_to_rgb(base_hue, 85.0, 75.0);
            return CharAnimationResult::with_color(fg_color);
        }

        // Active animation state with fade-out

        // Calculate fade factor: 1.0 during animation, fades to 0.0 during fadeout period
        let fade_factor = if ctx.hue_offset < fadeout_start {
            1.0
        } else {
            // Fade out over the remaining period
            1.0 - ((ctx.hue_offset - fadeout_start) / (animation_duration - fadeout_start))
        };

        // Calculate positions relative to center for radial symmetry
        let center_x = (ctx.total_chars as f32) / 2.0;
        let center_y = ctx.total_rows as f32 / 2.0;
        let dist_x = (ctx.char_index as f32 - center_x).abs();
        let dist_y = (ctx.row_index as f32 - center_y).abs();

        // Radial distance from center (for circular patterns)
        let radial_dist = ((dist_x * dist_x + dist_y * dist_y).sqrt()) / center_x.max(1.0);

        // Angular position (polar coordinates for rotational symmetry)
        let angle = ((ctx.row_index as f32 - center_y).atan2(ctx.char_index as f32 - center_x) * 180.0 / std::f32::consts::PI) + 180.0;

        // Simplified pattern generation for cleaner look

        // Layer 1: Radial waves (primary pattern)
        let radial_wave = (radial_dist * 6.0 - ctx.hue_offset * 0.12).sin();

        // Layer 2: Rotational sectors (kaleidoscope mirrors)
        let sector_count = 6.0; // 6-fold symmetry
        let sector_pattern = ((angle + ctx.hue_offset * 2.0) * sector_count / 180.0).sin();

        // Simplified background pattern (less competing visual noise)
        let bg_complexity = radial_wave * 0.6 + sector_pattern * 0.4;

        // Background: Full spectrum cycling with simplified modulation
        let bg_hue_base = (ctx.hue_offset * 2.5) % 360.0;
        let bg_hue = (bg_hue_base + bg_complexity * 120.0) % 360.0;

        // Desaturated background (60-80%) for less competition with foreground
        let bg_saturation = (60.0 + bg_complexity.abs() * 20.0).clamp(60.0, 80.0);

        // Background lightness: Very dark (8-20%) for maximum contrast
        // Fades darker as fade_factor approaches 0
        let base_bg_lightness = 8.0 + (radial_dist * 6.0);
        let bg_lightness = (base_bg_lightness + bg_complexity.abs() * 6.0).clamp(8.0, 20.0) * fade_factor;

        // Foreground: Pure complementary colors (180° offset) for maximum contrast
        let fg_pattern = (radial_dist * 5.0 + ctx.hue_offset * 0.1).sin() * 0.5 +
                        (angle * 0.3 - ctx.hue_offset * 0.3).cos() * 0.5;

        // During fade-out, transition foreground to simple rainbow
        let (fg_hue, fg_saturation) = if ctx.hue_offset < fadeout_start {
            // Active phase: complementary colors
            let fg_hue_base = (bg_hue + 180.0) % 360.0;
            let fg_hue = (fg_hue_base + fg_pattern * 60.0) % 360.0;
            let fg_saturation = (88.0 + fg_pattern.abs() * 12.0).clamp(88.0, 100.0);
            (fg_hue, fg_saturation)
        } else {
            // Fade-out phase: transition to simple rainbow
            let target_hue = (ctx.char_index as f32 / ctx.total_chars as f32) * 360.0;
            let current_fg_hue_base = (bg_hue + 180.0) % 360.0;
            let current_fg_hue = (current_fg_hue_base + fg_pattern * 60.0) % 360.0;

            // Interpolate hue and saturation
            let inverse_fade = 1.0 - fade_factor;
            let fg_hue = current_fg_hue * fade_factor + target_hue * inverse_fade;
            let fg_saturation = 100.0 * fade_factor + 85.0 * inverse_fade;
            (fg_hue, fg_saturation)
        };

        // Foreground lightness: Very bright (75-95%) for maximum readability
        let base_fg_lightness = 75.0 + (radial_dist * 8.0);
        let pattern_boost = fg_pattern.abs() * 6.0;
        let fg_lightness = (base_fg_lightness + pattern_boost).clamp(75.0, 92.0);

        // Subtle sparkle effect (reduced intensity)
        let sparkle = if (radial_wave + sector_pattern).abs() > 0.94 {
            6.0 * fade_factor // Sparkle fades out too
        } else {
            0.0
        };

        let final_fg_lightness = (fg_lightness + sparkle).clamp(75.0, 95.0);

        let fg_color = hsl_to_rgb(fg_hue, fg_saturation, final_fg_lightness);

        // Return background only if fade_factor > 0
        if bg_lightness > 0.5 {
            let bg_color = hsl_to_rgb(bg_hue, bg_saturation, bg_lightness);
            CharAnimationResult::with_bg(fg_color, bg_color)
        } else {
            CharAnimationResult::with_color(fg_color)
        }
    }
}
