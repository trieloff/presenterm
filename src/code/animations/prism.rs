use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// Prism animation - Spectrum split and refraction effect like light through a prism
pub(crate) struct Prism;

impl Animation for Prism {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        // Creates 3 distinct rainbow beams that shift position over time
        let beam_width = ctx.total_chars as f32 / 3.0;
        let shifted_pos = ctx.char_index as f32 + ctx.hue_offset * 1.5; // Faster animation
        let beam_pos = shifted_pos % beam_width;

        // Full spectrum within each beam
        let hue = (beam_pos / beam_width) * 360.0;

        // Add some variation in brightness to emphasize the beams
        let beam_center = beam_width / 2.0;
        let dist_from_center = (beam_pos - beam_center).abs();
        let lightness = 55.0 - (dist_from_center / beam_center) * 10.0;

        CharAnimationResult::with_color(hsl_to_rgb(hue % 360.0, 100.0, lightness))
    }
}
