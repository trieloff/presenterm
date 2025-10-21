use super::common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb};

/// CRT animation - Retro CRT effect with scanlines, phosphor triads, and rolling highlight
pub(crate) struct Crt;

impl Animation for Crt {
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult {
        let t = ctx.hue_offset; // time proxy
        let y = ctx.row_index as f32;
        let x = ctx.col_index as f32;

        // Base hue cycles subtly to simulate phosphor decay/recharge
        let base_hue = (180.0 + (x * 0.5 + t * 0.2).sin() * 20.0 + (y * 0.8 + t * 0.15).cos() * 15.0) % 360.0;

        // Chromatic subpixel triad: tint columns R/G/B in sequence
        let triad = (ctx.col_index % 3) as u8;
        let hue = match triad {
            0 => (base_hue + 10.0) % 360.0, // slight red tint
            1 => (base_hue + 140.0) % 360.0, // green tint
            _ => (base_hue + 260.0) % 360.0, // blue tint
        };

        // Horizontal scanlines: every other row is slightly darker
        let scanline = if ctx.row_index % 2 == 0 { -8.0 } else { 0.0 };

        // Rolling bright bar moving down (vertical retrace)
        let bar_pos = (t * 0.35) % 360.0; // 0..360
        // Map 0..360 to rows cyclically
        let bar_row = (bar_pos / 360.0) * ctx.total_rows as f32;
        let dist = (y - bar_row).abs();
        let bar_boost = (1.0 - (dist / 2.5).min(1.0)) * 18.0; // strong near the bar

        // Subtle noise flicker
        let noise = ((x * 12.9898 + y * 78.233 + t).sin() * 43758.5453).fract() * 4.0 - 2.0;

        let saturation = 75.0;
        let lightness = 42.0 + scanline + bar_boost + noise;

        CharAnimationResult::with_color(hsl_to_rgb(hue, saturation, lightness.clamp(25.0, 80.0)))
    }
}
