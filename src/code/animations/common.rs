use crate::markdown::text_style::Color;

/// Animation context passed to all animation renderers
#[derive(Debug, Clone)]
pub(crate) struct AnimationContext {
    /// Current hue offset (0-360) - drives animation progression
    pub hue_offset: f32,
    /// Character index within non-whitespace characters
    pub char_index: usize,
    /// Total non-whitespace characters in the banner
    pub total_chars: usize,
    /// Current row index
    pub row_index: usize,
    /// Total rows in the banner
    pub total_rows: usize,
    /// Current column index
    pub col_index: usize,
    /// Total columns in current row
    pub total_cols: usize,
    /// The character being rendered
    pub ch: char,
}

/// Result from rendering a single character with animation
#[derive(Debug)]
pub(crate) struct CharAnimationResult {
    /// Foreground color
    pub color: Color,
    /// Optional background color
    pub bg_color: Option<Color>,
    /// Optional replacement character
    pub replacement_char: Option<char>,
}

impl CharAnimationResult {
    /// Create a simple result with just a foreground color
    pub fn with_color(color: Color) -> Self {
        Self {
            color,
            bg_color: None,
            replacement_char: None,
        }
    }

    /// Create a result with foreground and background colors
    pub fn with_bg(color: Color, bg_color: Color) -> Self {
        Self {
            color,
            bg_color: Some(bg_color),
            replacement_char: None,
        }
    }

    /// Create a result with a replacement character
    pub fn with_replacement(color: Color, replacement_char: char) -> Self {
        Self {
            color,
            bg_color: None,
            replacement_char: Some(replacement_char),
        }
    }
}

/// Trait for animation styles
pub(crate) trait Animation {
    /// Render a single character with the animation style
    fn render_char(&self, ctx: &AnimationContext) -> CharAnimationResult;
}

/// Convert HSL to RGB color
/// H: hue (0-360), S: saturation (0-100), L: lightness (0-100)
pub(crate) fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
    let s = s / 100.0;
    let l = l / 100.0;

    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h {
        h if h < 60.0 => (c, x, 0.0),
        h if h < 120.0 => (x, c, 0.0),
        h if h < 180.0 => (0.0, c, x),
        h if h < 240.0 => (0.0, x, c),
        h if h < 300.0 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    Color::new(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

/// Generate a rainbow color for a given position
/// total: total number of positions, index: current position (0-based)
pub(crate) fn rainbow_color(index: usize, total: usize) -> Color {
    let hue = (index as f32 / total as f32) * 360.0;
    hsl_to_rgb(hue, 100.0, 50.0)
}
