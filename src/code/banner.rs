use figlet_rs::FIGfont;
use crate::markdown::text_style::{Color, TextStyle};
use crate::markdown::elements::Text;
use crate::markdown::text::{WeightedLine, WeightedText};
use crate::render::operation::{AsRenderOperations, BlockLine, Pollable, PollableState, RenderAsync, RenderAsyncStartPolicy, RenderOperation};
use crate::render::properties::WindowSize;
use crate::theme::Alignment;
use crate::code::snippet::BannerAnimationStyle;
use crate::code::animations::{AnimationContext, get_animation};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::panic::{catch_unwind, AssertUnwindSafe};
use once_cell::sync::OnceCell;
use std::collections::HashMap;

/// Embedded FIGlet fonts
mod fonts {
    /// Standard font (embedded)
    pub const STANDARD: &str = include_str!("../../fonts/standard.flf");

    // TODO: Add more embedded fonts like slant, banner, big, etc.
}

/// One-time cache of validated, safe-to-use FIGlet fonts found on the system.
/// Maps lowercased font name (without .flf) to full file path.
static VALID_FONT_PATHS: OnceCell<HashMap<String, String>> = OnceCell::new();

fn scan_figlet_font_dirs() -> Vec<String> {
    let mut dirs = Vec::new();
    // Common install locations
    for d in [
        "/opt/homebrew/share/figlet/fonts",
        "/usr/local/share/figlet",
        "/usr/share/figlet",
        "/usr/share/figlet/fonts",
    ] {
        if std::path::Path::new(d).is_dir() {
            dirs.push(d.to_string());
        }
    }
    dirs
}

fn validate_font_file(path: &str) -> bool {
    // Try to load and convert a simple string, catching panics from the figleter crate
    let loaded = catch_unwind(AssertUnwindSafe(|| FIGfont::from_file(path)));
    let Ok(Ok(font)) = loaded else {
        return false;
    };
    let res = catch_unwind(AssertUnwindSafe(|| font.convert("TEST")));
    matches!(res, Ok(Some(_)))
}

fn build_valid_font_map() -> HashMap<String, String> {
    // Whitelist of fonts tested to work with figlet-rs 0.1.5
    // Tested: 163 total fonts, 149 working, 14 broken
    // Broken fonts (cause panics): banner, big, bubble, digital, dwhistled, gradient,
    // ivrit, l4me, maxfour, morse, pyramid, rot13, term, tsalagi
    const SAFE_FONTS: &[&str] = &[
        "3-d", "3x5", "5lineoblique", "acrobatic", "alligator", "alligator2", "alphabet", "avatar",
        "banner3", "banner3-D", "banner4", "barbwire", "basic", "bell", "bigchief", "binary",
        "block", "broadway", "bulbhead", "calgphy2", "caligraphy", "catwalk", "chunky", "coinstak",
        "colossal", "computer", "contessa", "contrast", "cosmic", "cosmike", "crawford", "cricket",
        "cursive", "cyberlarge", "cybermedium", "cybersmall", "decimal", "diamond", "doh", "doom",
        "dotmatrix", "double", "drpepper", "eftichess", "eftifont", "eftipiti", "eftirobot", "eftitalic",
        "eftiwall", "eftiwater", "epic", "fender", "fourtops", "fraktur", "fuzzy", "goofy",
        "gothic", "graceful", "graffiti", "hex", "hollywood", "invita", "isometric1", "isometric2",
        "isometric3", "isometric4", "italic", "jazmine", "jerusalem", "katakana", "kban", "larry3d",
        "lcd", "lean", "letters", "linux", "lockergnome", "madrid", "marquee", "mike",
        "mini", "mirror", "mnemonic", "moscow", "mshebrew210", "nancyj", "nancyj-fancy", "nancyj-underlined",
        "nipples", "ntgreek", "nvscript", "o8", "octal", "ogre", "os2", "pawp",
        "peaks", "pebbles", "pepper", "poison", "puffy", "rectangles", "relief", "relief2",
        "rev", "roman", "rounded", "rowancap", "rozzo", "runic", "runyc", "sblood",
        "script", "serifcap", "shadow", "short", "slant", "slide", "slscript", "small",
        "smisome1", "smkeyboard", "smscript", "smshadow", "smslant", "smtengwar", "speed", "stacey",
        "stampatello", "standard", "starwars", "stellar", "stop", "straight", "tanja", "tengwar",
        "thick", "thin", "threepoint", "ticks", "ticksslant", "tinker-toy", "tombstone", "trek",
        "twopoint", "univers", "usaflag", "weird", "whimsy"
    ];

    let mut map = HashMap::new();
    for d in scan_figlet_font_dirs() {
        if let Ok(entries) = std::fs::read_dir(&d) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(ext) = path.extension() {
                    if ext == "flf" {
                        if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                            let stem_lower = stem.to_lowercase();
                            // Only include whitelisted fonts
                            if SAFE_FONTS.contains(&stem_lower.as_str()) {
                                let path_str = path.to_string_lossy().to_string();
                                if validate_font_file(&path_str) {
                                    map.entry(stem_lower).or_insert(path_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    map
}

/// Initialize and warn about system FIGlet fonts. Prints summary once at launch.
pub(crate) fn init_figlet_fonts_and_warn() {
    if VALID_FONT_PATHS.get().is_some() {
        return; // already initialized
    }
    let map = build_valid_font_map();
    let count = map.len();
    let _ = VALID_FONT_PATHS.set(map);
    if count == 0 {
        // No system fonts found or none validated — we’ll rely on embedded 'standard'.
        eprintln!("[presenterm] note: no valid FIGlet fonts found on system; using embedded 'standard' font only");
    } else {
        eprintln!("[presenterm] detected {count} valid FIGlet font(s)");
    }
}

fn get_valid_font_path(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    let map = VALID_FONT_PATHS.get_or_init(build_valid_font_map);
    map.get(&lower).cloned()
}


/// Generator for ASCII art banners using FIGlet fonts
pub(crate) struct BannerGenerator {
    font: FIGfont,
}

impl BannerGenerator {
    /// Create a new banner generator with the specified font
    pub(crate) fn new(font_name: &str) -> Result<Self, BannerError> {
        let font = Self::load_font(font_name)?;
        Ok(Self { font })
    }

    /// Generate ASCII art from the given text
    pub(crate) fn generate(&self, text: &str) -> Result<String, BannerError> {
        // Wrapper to guard against panics inside figleter when converting some fonts
        let res = catch_unwind(AssertUnwindSafe(|| self.font.convert(text)));
        match res {
            Ok(Some(figure)) => Ok(figure.to_string()),
            Ok(None) => Err(BannerError::ConversionFailed(text.to_string())),
            Err(_) => Err(BannerError::ConversionFailed(text.to_string())),
        }
    }

    /// Generate a rainbow color for a given character position
    pub(crate) fn rainbow_color(char_index: usize, total_chars: usize) -> Color {
        crate::code::animations::rainbow_color(char_index, total_chars)
    }

    /// Load a FIGlet font by name
    fn load_font(font_name: &str) -> Result<FIGfont, BannerError> {
        // Try embedded fonts first
        match font_name.to_lowercase().as_str() {
            "standard" => {
                // Catch panics from figleter when parsing embedded font (debug overflow issues)
                let result = catch_unwind(AssertUnwindSafe(|| {
                    FIGfont::from_content(fonts::STANDARD)
                }));

                match result {
                    Ok(Ok(font)) => return Ok(font),
                    Ok(Err(e)) => return Err(BannerError::FontLoadFailed(font_name.to_string(), e)),
                    Err(_) => return Err(BannerError::FontLoadFailed(
                        font_name.to_string(),
                        "panic during font loading".to_string()
                    )),
                }
            }
            _ => {}
        }

        // Allow only previously validated fonts (except embedded 'standard')
        if let Some(path) = get_valid_font_path(font_name) {
            // These fonts have already been validated, but catch panics anyway for safety
            let result = catch_unwind(AssertUnwindSafe(|| {
                FIGfont::from_file(&path)
            }));

            match result {
                Ok(Ok(font)) => return Ok(font),
                Ok(Err(e)) => return Err(BannerError::FontLoadFailed(font_name.to_string(), e)),
                Err(_) => return Err(BannerError::FontLoadFailed(
                    font_name.to_string(),
                    "panic during font loading".to_string()
                )),
            }
        }

        // If not found/validated, do NOT fallback — reject the request
        Err(BannerError::FontUnavailable(font_name.to_string()))
    }
}

/// Errors that can occur when generating banners
#[derive(thiserror::Error, Debug)]
pub enum BannerError {
    #[error("failed to load font '{0}': {1}")]
    FontLoadFailed(String, String),

    #[error("failed to convert text '{0}' to ASCII art")]
    ConversionFailed(String),

    #[error("requested figlet font '{0}' is not available or not validated")]
    FontUnavailable(String),
}

/// Animated rainbow banner
#[derive(Debug)]
pub(crate) struct RainbowBannerAnimation {
    /// The banner lines (ASCII art)
    lines: Vec<String>,
    /// Block length for alignment
    block_length: u16,
    /// Alignment settings
    alignment: Alignment,
    /// Font size
    font_size: u8,
    /// Animation state
    state: Arc<Mutex<AnimationState>>,
    /// Whether to loop the animation
    loop_animation: bool,
    /// Duration of a full animation cycle
    duration: Duration,
    /// Animation style
    style: BannerAnimationStyle,
}

#[derive(Debug)]
struct AnimationState {
    /// Current hue offset (0-360)
    hue_offset: f32,
    /// Animation start time
    start_time: Option<Instant>,
    /// Whether animation has completed
    completed: bool,
}

impl RainbowBannerAnimation {
    pub(crate) fn new(
        lines: Vec<String>,
        block_length: u16,
        alignment: Alignment,
        font_size: u8,
        style: BannerAnimationStyle,
        loop_animation: bool,
        duration_millis: u64,
    ) -> Self {
        Self {
            lines,
            block_length,
            alignment,
            font_size,
            state: Arc::new(Mutex::new(AnimationState {
                hue_offset: 0.0,
                start_time: None,
                completed: false,
            })),
            style,
            loop_animation,
            duration: Duration::from_millis(duration_millis.max(1)),
        }
    }

    fn render_with_offset(&self, hue_offset: f32) -> Vec<RenderOperation> {
        let total_chars: usize = self.lines.iter()
            .flat_map(|line| line.chars())
            .filter(|c| !c.is_whitespace())
            .count();

        let total_rows = self.lines.len();
        let mut char_index = 0;
        let mut operations = Vec::new();

        // Get the animation implementation
        let animation = get_animation(self.style.clone());

        for (row_index, line) in self.lines.iter().enumerate() {
            let mut colored_text: Vec<Text> = Vec::new();
            let total_cols = line.chars().count();

            for (col_index, ch) in line.chars().enumerate() {
                let is_whitespace = ch.is_whitespace();

                // Build animation context
                let ctx = AnimationContext {
                    hue_offset,
                    char_index,
                    total_chars,
                    row_index,
                    total_rows,
                    col_index,
                    total_cols,
                    ch,
                };

                let (color, bg_color, replacement_char) = if is_whitespace {
                    // For whitespace, calculate background color (foreground doesn't matter)
                    match self.style {
                        BannerAnimationStyle::Kaleidoscope => {
                            // Calculate background for this whitespace position
                            // Check animation phase for +once mode
                            let animation_duration = 360.0;
                            let fadeout_start = 320.0;
                            let animation_complete = hue_offset > animation_duration;

                            if animation_complete {
                                // Final state: no background
                                (Color::new(0, 0, 0), None, None)
                            } else {
                                // Active animation with fade-out
                                let fade_factor = if hue_offset < fadeout_start {
                                    1.0
                                } else {
                                    1.0 - ((hue_offset - fadeout_start) / (animation_duration - fadeout_start))
                                };

                                let center_y = total_rows / 2.0;
                                let dist_y = (row_index as f32 - center_y).abs();

                                let total_cols = line.chars().count() as f32;
                                let center_x = total_cols / 2.0;
                                let dist_x = (col_index as f32 - center_x).abs();
                                let radial_dist = ((dist_x * dist_x + dist_y * dist_y).sqrt()) / center_x.max(1.0);

                                let angle = ((row_index as f32 - center_y).atan2(col_index as f32 - center_x) * 180.0 / std::f32::consts::PI) + 180.0;

                                let radial_wave = (radial_dist * 6.0 - hue_offset * 0.12).sin();
                                let sector_count = 6.0;
                                let sector_pattern = ((angle + hue_offset * 2.0) * sector_count / 180.0).sin();
                                let bg_complexity = radial_wave * 0.6 + sector_pattern * 0.4;

                                let bg_hue_base = (hue_offset * 2.5) % 360.0;
                                let bg_hue = (bg_hue_base + bg_complexity * 120.0) % 360.0;
                                let bg_saturation = (60.0 + bg_complexity.abs() * 20.0).clamp(60.0, 80.0);

                                let base_bg_lightness = 8.0 + (radial_dist * 6.0);
                                let bg_lightness = (base_bg_lightness + bg_complexity.abs() * 6.0).clamp(8.0, 20.0) * fade_factor;

                                // Return background only if lightness is above threshold
                                let bg_color = if bg_lightness > 0.5 {
                                    Some(rainbow::hsl_to_rgb(bg_hue, bg_saturation, bg_lightness))
                                } else {
                                    None
                                };

                                (Color::new(0, 0, 0), bg_color, None)
                            }
                        }
                        _ => {
                            // Other styles don't use background, return default
                            (Color::new(0, 0, 0), None, None)
                        }
                    }
                } else {
                    // For non-whitespace, calculate full styling
                    match self.style {
                        BannerAnimationStyle::Rainbow => {
                            let base_hue = (char_index as f32 / total_chars as f32) * 360.0;
                            let hue = (base_hue + hue_offset) % 360.0;
                            (rainbow::hsl_to_rgb(hue, 100.0, 50.0), None, None)
                        }
                        BannerAnimationStyle::Flash => {
                            // Single color for all characters, hue cycles with time
                            let hue = hue_offset % 360.0;
                            (rainbow::hsl_to_rgb(hue, 100.0, 50.0), None, None)
                        }
                        BannerAnimationStyle::Wave => {
                            // Hue oscillates around a base using a sine wave along characters
                            let base = 200.0; // blue-ish base
                            let amplitude = 60.0;
                            let freq = 0.35; // chars per cycle
                            let phase = hue_offset.to_radians();
                            let hue = base + amplitude * ((char_index as f32 * freq + phase).sin());
                            (rainbow::hsl_to_rgb(((hue % 360.0) + 360.0) % 360.0, 100.0, 50.0), None, None)
                        }
                        BannerAnimationStyle::Iris => {
                            // Lightness pulse expanding from center
                            let center = (total_chars as f32) / 2.0;
                            let pos = char_index as f32;
                            let radius = (hue_offset / 360.0) * center.max(1.0);
                            let dist = (pos - center).abs();
                            let l = if dist <= radius { 60.0 } else { 35.0 };
                            // Fixed hue rainbow mapping by index for variety
                            let hue = (pos / total_chars as f32) * 360.0;
                            (rainbow::hsl_to_rgb(hue % 360.0, 100.0, l), None, None)
                        }
                        BannerAnimationStyle::Plasma => {
                            // Psychedelic plasma effect using multiple overlapping sine waves
                            let pos = char_index as f32;
                            let time = hue_offset;

                            // Combine multiple sine waves with different frequencies and phases
                            let wave1 = (pos * 0.1 + time * 0.02).sin();
                            let wave2 = (pos * 0.13 + time * 0.03).sin();
                            let wave3 = (pos * 0.08 + time * 0.025).cos();

                            // Average the waves and map from -1..1 to 0..360
                            let hue = ((wave1 + wave2 + wave3) / 3.0 + 1.0) * 180.0;
                            (rainbow::hsl_to_rgb(hue % 360.0, 100.0, 50.0), None, None)
                        }
                        BannerAnimationStyle::Scanner => {
                            // Horizontal scan line effect like KITT from Knight Rider
                            let scan_pos = (hue_offset / 360.0) * total_chars as f32;
                            let dist = (char_index as f32 - scan_pos).abs();
                            let lightness = (70.0 - (dist * 8.0)).max(30.0);
                            // Classic red scanner
                            (rainbow::hsl_to_rgb(0.0, 100.0, lightness), None, None)
                        }
                        BannerAnimationStyle::Neon => {
                            // Bright neon sign colors cycling through classic neon palette
                            // Neon color palette: hot pink, electric blue, lime green, violet
                            let palette = [330.0, 195.0, 85.0, 280.0];
                            let palette_len = palette.len() as f32;

                            // Cycle through palette with character position
                            let cycle_position = (hue_offset + char_index as f32 * 5.0) % (palette_len * 90.0);
                            let palette_index = (cycle_position / 90.0).floor() as usize % palette.len();
                            let next_index = (palette_index + 1) % palette.len();

                            // Interpolate between current and next palette color
                            let t = (cycle_position % 90.0) / 90.0;
                            let hue = palette[palette_index] * (1.0 - t) + palette[next_index] * t;

                            // Bright neon glow with slight pulse effect
                            let base_lightness = 60.0;
                            let pulse = (hue_offset * 0.1).sin() * 5.0;
                            let lightness = base_lightness + pulse;

                            (rainbow::hsl_to_rgb(hue % 360.0, 100.0, lightness), None, None)
                        }
                        BannerAnimationStyle::Matrix => {
                            // Matrix-style digital rain effect with varying green shades and characters
                            let hue = 120.0; // Green base

                            // Create pseudo-random variation per character for authentic look
                            let char_seed = (char_index as f32 * 7.919 + row_index as f32 * 3.141).sin() * 100.0;
                            let char_variation = char_seed.fract();

                            // Cascade effect: brightness falls down the rows over time
                            let time_offset = hue_offset / 8.0;
                            let cascade_pos = row_index as f32 - time_offset;

                            // Animation completion: add settle time after cascade finishes
                            let animation_complete = time_offset > (total_rows + 3.0);
                            let cascade_has_passed = time_offset > row_index as f32;

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
                                let glitch_chance = (char_seed * 37.1 + hue_offset * 0.03).sin();
                                if glitch_chance > 0.95 {
                                    // Rare glitch: briefly show Matrix character
                                    Some(matrix_chars::get_char(char_seed + hue_offset * 10.0))
                                } else {
                                    // Normal: show actual character
                                    None
                                }
                            } else {
                                // Cascade hasn't reached yet - show Matrix characters
                                Some(matrix_chars::get_char(char_seed + hue_offset * 0.5))
                            };

                            (rainbow::hsl_to_rgb(hue, saturation, lightness), None, replacement)
                        }
                        BannerAnimationStyle::Sepia => {
                            // Subdued vintage monochrome sepia tone with gentle brightness wave
                            let hue = 30.0; // Warm brown/sepia tone
                            let saturation = 45.0; // Desaturated vintage look
                            let lightness = 35.0 + 15.0 * (char_index as f32 * 0.15 + hue_offset * 0.05).sin();
                            (rainbow::hsl_to_rgb(hue, saturation, lightness), None, None)
                        }
                        BannerAnimationStyle::Kaleidoscope => {
                            // Enhanced psychedelic symmetrical rotating color patterns
                            // Creates rich, multi-layered kaleidoscope with radial symmetry

                            // Animation phases: active -> fade-out -> complete
                            let animation_duration = 360.0;
                            let fadeout_start = 320.0;
                            let animation_complete = hue_offset > animation_duration;

                            if animation_complete {
                                // Final state: maximum readability with NO background
                                // Use simple rainbow for foreground, transparent background
                                let base_hue = (char_index as f32 / total_chars as f32) * 360.0;
                                let fg_color = rainbow::hsl_to_rgb(base_hue, 85.0, 75.0);
                                (fg_color, None, None)
                            } else {
                                // Active animation state with fade-out

                                // Calculate fade factor: 1.0 during animation, fades to 0.0 during fadeout period
                                let fade_factor = if hue_offset < fadeout_start {
                                    1.0
                                } else {
                                    // Fade out over the remaining period
                                    1.0 - ((hue_offset - fadeout_start) / (animation_duration - fadeout_start))
                                };

                                // Calculate positions relative to center for radial symmetry
                                let center_x = (total_chars as f32) / 2.0;
                                let center_y = total_rows / 2.0;
                                let dist_x = (char_index as f32 - center_x).abs();
                                let dist_y = (row_index as f32 - center_y).abs();

                                // Radial distance from center (for circular patterns)
                                let radial_dist = ((dist_x * dist_x + dist_y * dist_y).sqrt()) / center_x.max(1.0);

                                // Angular position (polar coordinates for rotational symmetry)
                                let angle = ((row_index as f32 - center_y).atan2(char_index as f32 - center_x) * 180.0 / std::f32::consts::PI) + 180.0;

                                // Simplified pattern generation for cleaner look

                                // Layer 1: Radial waves (primary pattern)
                                let radial_wave = (radial_dist * 6.0 - hue_offset * 0.12).sin();

                                // Layer 2: Rotational sectors (kaleidoscope mirrors)
                                let sector_count = 6.0; // 6-fold symmetry
                                let sector_pattern = ((angle + hue_offset * 2.0) * sector_count / 180.0).sin();

                                // Simplified background pattern (less competing visual noise)
                                let bg_complexity = radial_wave * 0.6 + sector_pattern * 0.4;

                                // Background: Full spectrum cycling with simplified modulation
                                let bg_hue_base = (hue_offset * 2.5) % 360.0;
                                let bg_hue = (bg_hue_base + bg_complexity * 120.0) % 360.0;

                                // Desaturated background (60-80%) for less competition with foreground
                                let bg_saturation = (60.0 + bg_complexity.abs() * 20.0).clamp(60.0, 80.0);

                                // Background lightness: Very dark (8-20%) for maximum contrast
                                // Fades darker as fade_factor approaches 0
                                let base_bg_lightness = 8.0 + (radial_dist * 6.0);
                                let bg_lightness = (base_bg_lightness + bg_complexity.abs() * 6.0).clamp(8.0, 20.0) * fade_factor;

                                // Foreground: Pure complementary colors (180° offset) for maximum contrast
                                let fg_pattern = (radial_dist * 5.0 + hue_offset * 0.1).sin() * 0.5 +
                                                (angle * 0.3 - hue_offset * 0.3).cos() * 0.5;

                                // During fade-out, transition foreground to simple rainbow
                                let (fg_hue, fg_saturation) = if hue_offset < fadeout_start {
                                    // Active phase: complementary colors
                                    let fg_hue_base = (bg_hue + 180.0) % 360.0;
                                    let fg_hue = (fg_hue_base + fg_pattern * 60.0) % 360.0;
                                    let fg_saturation = (88.0 + fg_pattern.abs() * 12.0).clamp(88.0, 100.0);
                                    (fg_hue, fg_saturation)
                                } else {
                                    // Fade-out phase: transition to simple rainbow
                                    let target_hue = (char_index as f32 / total_chars as f32) * 360.0;
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

                                let fg_color = rainbow::hsl_to_rgb(fg_hue, fg_saturation, final_fg_lightness);

                                // Return background only if fade_factor > 0
                                let bg_color = if bg_lightness > 0.5 {
                                    Some(rainbow::hsl_to_rgb(bg_hue, bg_saturation, bg_lightness))
                                } else {
                                    None
                                };

                                (fg_color, bg_color, None)
                            }
                        }
                        BannerAnimationStyle::Fire => {
                            // Flame-like effect with red/orange/yellow gradient moving upward
                            // Characters at bottom are red, middle orange, top yellow

                            // Calculate vertical position (0.0 at bottom to 1.0 at top)
                            let vertical_pos = row_index as f32 / total_rows.max(1.0);

                            // Base hue for fire: 0° (red) at bottom to 60° (yellow) at top
                            let base_hue = vertical_pos * 60.0;

                            // Add flickering effect that moves upward
                            let flicker = (hue_offset * 0.1 + char_index as f32 * 0.3).sin() * 10.0;
                            let hue = (base_hue + flicker).max(0.0).min(60.0);

                            // Saturation: 100% for vibrant fire colors
                            let saturation = 100.0;

                            // Lightness: 45-65% with random variations for flicker effect
                            let base_lightness = 55.0;
                            let lightness_flicker = (hue_offset * 0.15 + char_index as f32 * 0.2 + row_index as f32 * 0.4).sin() * 10.0;
                            let lightness = (base_lightness + lightness_flicker).max(45.0).min(65.0);

                            (rainbow::hsl_to_rgb(hue, saturation, lightness), None, None)
                        }
                        BannerAnimationStyle::Glitch => {
                            // Cyberpunk glitch aesthetic with random color flickering and character corruption
                            // Animation stabilizes at the end for +once mode
                            let glitch_duration = 300.0; // Glitch for ~300 degrees of hue_offset
                            let animation_complete = hue_offset > glitch_duration;

                            if animation_complete {
                                // Final stable state: clean green color (like terminal recovered)
                                (rainbow::hsl_to_rgb(120.0, 60.0, 55.0), None, None)
                            } else {
                                // Active glitching phase
                                // Use character index + hue_offset as seed for pseudo-random behavior
                                let glitch_seed = (char_index as f32 * 12.9898 + hue_offset * 78.233).sin() * 43758.5453;

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
                                let char_glitch_seed = (char_index as f32 * 7.321 + hue_offset * 0.5).sin() * 12345.6789;

                                // Increase glitch probability as we approach the end
                                let glitch_intensity = 1.0 - (hue_offset / glitch_duration).min(1.0);
                                let glitch_threshold = 0.65 - (glitch_intensity * 0.3); // 35-65% chance
                                let should_glitch = char_glitch_seed.fract() > glitch_threshold;

                                let replacement = if should_glitch {
                                    glitch_chars::get_glitched_char(ch, char_glitch_seed)
                                } else {
                                    None
                                };

                                (rainbow::hsl_to_rgb(hue, saturation, lightness), None, replacement)
                            }
                        }
                        BannerAnimationStyle::Breathe => {
                            // Subdued, gentle synchronized breathing effect
                            // All characters pulse in lightness together, like breathing in and out
                            let hue = (char_index as f32 / total_chars as f32) * 360.0;
                            let saturation = 65.0; // Calming, not too vibrant
                            // Synchronized breathing: all characters pulse together
                            let lightness = 35.0 + 20.0 * (hue_offset * 0.05).sin();
                            (rainbow::hsl_to_rgb(hue % 360.0, saturation, lightness), None, None)
                        }
                        BannerAnimationStyle::Prism => {
                            // Spectrum split and refraction effect like light through a prism
                            // Creates 3 distinct rainbow beams that shift position over time
                            let beam_width = total_chars as f32 / 3.0;
                            let shifted_pos = char_index as f32 + hue_offset * 1.5; // Faster animation
                            let beam_pos = shifted_pos % beam_width;

                            // Full spectrum within each beam
                            let hue = (beam_pos / beam_width) * 360.0;

                            // Add some variation in brightness to emphasize the beams
                            let beam_center = beam_width / 2.0;
                            let dist_from_center = (beam_pos - beam_center).abs();
                            let lightness = 55.0 - (dist_from_center / beam_center) * 10.0;

                            (rainbow::hsl_to_rgb(hue % 360.0, 100.0, lightness), None, None)
                        }
                        BannerAnimationStyle::Aurora => {
                            // Northern lights effect: flowing vertical curtains of green/teal/purple
                            // Build a smooth field using overlapping sine waves across columns and time
                            let t = hue_offset * 0.02; // slow time
                            let x = col_index as f32;
                            let y = row_index as f32;

                            // Two moving wave fronts with slight vertical parallax
                            let wave1 = (x * 0.12 + t + y * 0.10).sin();
                            let wave2 = (x * 0.07 - t * 0.8 + y * 0.18).cos();
                            let field = (wave1 + wave2) * 0.5; // -1..1

                            // Map field to a hue band between 120° (green) and 280° (purple)
                            let hue_min = 120.0;
                            let hue_max = 280.0;
                            let hue = hue_min + ((field + 1.0) * 0.5) * (hue_max - hue_min);

                            // Shimmer: saturation and lightness gently fluctuate by height
                            let saturation = 70.0 + (y * 0.15 + t * 0.8).sin() * 15.0;
                            let base_lightness = 45.0 + field.abs() * 18.0; // brighter on wave ridges
                            let lightness = base_lightness + (y * 0.25 + t).sin() * 6.0;

                            (rainbow::hsl_to_rgb(hue % 360.0, saturation.clamp(35.0, 100.0), lightness.clamp(30.0, 80.0)), None, None)
                        }
                        BannerAnimationStyle::Crt => {
                            // Retro CRT effect with scanlines, phosphor triads, and rolling highlight
                            let t = hue_offset; // time proxy
                            let y = row_index as f32;
                            let x = col_index as f32;

                            // Base hue cycles subtly to simulate phosphor decay/recharge
                            let base_hue = (180.0 + (x * 0.5 + t * 0.2).sin() * 20.0 + (y * 0.8 + t * 0.15).cos() * 15.0) % 360.0;

                            // Chromatic subpixel triad: tint columns R/G/B in sequence
                            let triad = (col_index % 3) as u8;
                            let hue = match triad {
                                0 => (base_hue + 10.0) % 360.0, // slight red tint
                                1 => (base_hue + 140.0) % 360.0, // green tint
                                _ => (base_hue + 260.0) % 360.0, // blue tint
                            };

                            // Horizontal scanlines: every other row is slightly darker
                            let scanline = if row_index % 2 == 0 { -8.0 } else { 0.0 };

                            // Rolling bright bar moving down (vertical retrace)
                            let bar_pos = (t * 0.35) % 360.0; // 0..360
                            // Map 0..360 to rows cyclically
                            let bar_row = (bar_pos / 360.0) * total_rows.max(1.0);
                            let dist = (y - bar_row).abs();
                            let bar_boost = (1.0 - (dist / 2.5).min(1.0)) * 18.0; // strong near the bar

                            // Subtle noise flicker
                            let noise = ((x * 12.9898 + y * 78.233 + t).sin() * 43758.5453).fract() * 4.0 - 2.0;

                            let saturation = 75.0;
                            let lightness = 42.0 + scanline + bar_boost + noise;

                            (rainbow::hsl_to_rgb(hue, saturation, lightness.clamp(25.0, 80.0)), None, None)
                        }
                        BannerAnimationStyle::Typewriter => {
                            // Gradual left-to-right, top-to-bottom reveal
                            // Use a longer duration to ensure we have time to reveal all characters
                            // and add settling time before animation completes
                            let typing_duration = 320.0; // Time to type all characters
                            let settle_time = 40.0; // Brief pause after typing completes
                            let total_duration = typing_duration + settle_time;

                            if hue_offset > total_duration {
                                // Animation fully complete: all text revealed, no caret
                                let color = rainbow::hsl_to_rgb(40.0, 20.0, 85.0);
                                (color, None, None)
                            } else {
                                let progress = (hue_offset / typing_duration).clamp(0.0, 1.0);
                                // Add 1 to ensure we reach total_chars, not total_chars-1
                                let reveal_count = ((progress * total_chars as f32).floor() as usize).min(total_chars);

                                if char_index < reveal_count {
                                    // Already revealed: warm white ink
                                    let color = rainbow::hsl_to_rgb(40.0, 20.0, 85.0);
                                    (color, None, None)
                                } else if char_index == reveal_count && reveal_count < total_chars {
                                    // Currently being typed: show with caret (only if not at end)
                                    let color = rainbow::hsl_to_rgb(200.0, 85.0, 65.0);
                                    // Draw a block caret instead of the ASCII glyph to emphasize typing
                                    (color, None, Some('▌'))
                                } else if reveal_count >= total_chars {
                                    // All characters revealed but still in settling time
                                    let color = rainbow::hsl_to_rgb(40.0, 20.0, 85.0);
                                    (color, None, None)
                                } else {
                                    // Not yet revealed: render as space (no ink)
                                    let color = rainbow::hsl_to_rgb(0.0, 0.0, 0.0);
                                    (color, None, Some(' '))
                                }
                            }
                        }
                        _ => {
                            // Fallback for unimplemented styles
                            (rainbow::hsl_to_rgb(0.0, 0.0, 50.0), None, None)
                        }
                    }
                };

                // Increment char_index only for non-whitespace characters
                if !is_whitespace {
                    char_index += 1;
                }

                // Use replacement character if provided, otherwise use original
                let display_char = replacement_char.unwrap_or(ch);

                // Apply styling to ALL characters (including whitespace with background)
                let mut text_style = TextStyle::default()
                    .fg_color(color)
                    .size(self.font_size);

                // Apply background color if provided (this works for both fg and whitespace)
                if let Some(bg) = bg_color {
                    text_style = text_style.bg_color(bg);
                }

                colored_text.push(Text::new(display_char.to_string().as_str(), text_style));
            }

            let weighted_line = WeightedLine::from(colored_text);
            operations.push(RenderOperation::RenderBlockLine(BlockLine {
                prefix: WeightedText::from(""),
                right_padding_length: 0,
                repeat_prefix_on_wrap: false,
                text: weighted_line,
                block_length: self.block_length,
                alignment: self.alignment,
                block_color: None,
            }));
            operations.push(RenderOperation::RenderLineBreak);
        }

        operations
    }
}

impl AsRenderOperations for RainbowBannerAnimation {
    fn as_render_operations(&self, _: &WindowSize) -> Vec<RenderOperation> {
        let state = self.state.lock().unwrap();
        self.render_with_offset(state.hue_offset)
    }
}

impl RenderAsync for RainbowBannerAnimation {
    fn pollable(&self) -> Box<dyn Pollable> {
        Box::new(RainbowAnimationPollable {
            state: self.state.clone(),
            loop_animation: self.loop_animation,
            duration: self.duration,
        })
    }

    fn start_policy(&self) -> RenderAsyncStartPolicy {
        // Looping animations use Automatic (they never stop, so timing doesn't matter)
        // Non-looping animations use OnDemand (triggered when slide is first viewed)
        if self.loop_animation {
            RenderAsyncStartPolicy::Automatic
        } else {
            RenderAsyncStartPolicy::OnDemand
        }
    }
}

struct RainbowAnimationPollable {
    state: Arc<Mutex<AnimationState>>,
    loop_animation: bool,
    duration: Duration,
}

impl Pollable for RainbowAnimationPollable {
    fn poll(&mut self) -> PollableState {
        let mut state = self.state.lock().unwrap();

        // Initialize start time on first poll
        if state.start_time.is_none() {
            state.start_time = Some(Instant::now());
            return PollableState::Modified;
        }

        let elapsed = state.start_time.unwrap().elapsed();

        let cycle = self.duration;

        if elapsed >= cycle && !self.loop_animation {
            if !state.completed {
                state.completed = true;
                return PollableState::Done;
            }
            return PollableState::Unmodified;
        }

        // Update hue offset based on time
        let progress = if self.loop_animation {
            (elapsed.as_millis() % cycle.as_millis()) as f32 / cycle.as_millis() as f32
        } else {
            (elapsed.as_millis() as f32 / cycle.as_millis() as f32).min(1.0)
        };

        state.hue_offset = progress * 360.0;

        PollableState::Modified
    }
}

/// Context for multi-line banner display
#[derive(Debug)]
pub(crate) struct MultiBannerContext {
    /// Current line index being displayed (0-based)
    pub(crate) current: usize,
    /// Total number of lines
    pub(crate) total: usize,
}

/// Mutator for multi-line banners that cycles through banner words
#[derive(Debug)]
pub(crate) struct MultiBannerMutator {
    context: Arc<Mutex<MultiBannerContext>>,
}

impl MultiBannerMutator {
    pub(crate) fn new(context: Arc<Mutex<MultiBannerContext>>) -> Self {
        Self { context }
    }
}

impl crate::presentation::ChunkMutator for MultiBannerMutator {
    fn mutate_next(&self) -> bool {
        let mut context = self.context.lock().unwrap();
        if context.current >= context.total - 1 {
            false
        } else {
            context.current += 1;
            true
        }
    }

    fn mutate_previous(&self) -> bool {
        let mut context = self.context.lock().unwrap();
        if context.current == 0 {
            false
        } else {
            context.current -= 1;
            true
        }
    }

    fn reset_mutations(&self) {
        let mut context = self.context.lock().unwrap();
        context.current = 0;
    }

    fn apply_all_mutations(&self) {
        let mut context = self.context.lock().unwrap();
        context.current = context.total - 1;
    }

    fn mutations(&self) -> (usize, usize) {
        let context = self.context.lock().unwrap();
        (context.current, context.total)
    }
}

/// A multi-line banner that renders different words based on the current context
#[derive(Debug)]
pub(crate) struct MultiBannerLine {
    /// All the banner animations, one per word
    animations: Vec<RainbowBannerAnimation>,
    /// Shared context determining which word to display
    context: Arc<Mutex<MultiBannerContext>>,
}

impl MultiBannerLine {
    pub(crate) fn new(
        banners: Vec<RainbowBannerAnimation>,
        context: Arc<Mutex<MultiBannerContext>>,
    ) -> Self {
        Self { animations: banners, context }
    }
}

impl AsRenderOperations for MultiBannerLine {
    fn as_render_operations(&self, window: &WindowSize) -> Vec<RenderOperation> {
        let context = self.context.lock().unwrap();
        let current = context.current;
        drop(context);

        self.animations
            .get(current)
            .map(|anim| anim.as_render_operations(window))
            .unwrap_or_default()
    }
}

impl RenderAsync for MultiBannerLine {
    fn pollable(&self) -> Box<dyn Pollable> {
        // Create pollables for all animations
        let pollables: Vec<Box<dyn Pollable>> = self.animations.iter().map(|a| a.pollable()).collect();
        Box::new(MultiBannerPollable {
            pollables,
            context: self.context.clone(),
        })
    }

    fn start_policy(&self) -> RenderAsyncStartPolicy {
        // Use the policy from the first animation
        self.animations
            .first()
            .map(|a| a.start_policy())
            .unwrap_or(RenderAsyncStartPolicy::OnDemand)
    }
}

struct MultiBannerPollable {
    pollables: Vec<Box<dyn Pollable>>,
    context: Arc<Mutex<MultiBannerContext>>,
}

impl Pollable for MultiBannerPollable {
    fn poll(&mut self) -> PollableState {
        let context = self.context.lock().unwrap();
        let current = context.current;
        drop(context);

        if let Some(pollable) = self.pollables.get_mut(current) {
            pollable.poll()
        } else {
            PollableState::Unmodified
        }
    }
}

/// Static (non-animated) multi-line banner renderer.
/// Renders one of multiple pre-generated ASCII banners (one per input line),
/// with static rainbow coloring, selected by a shared context and cycled via a mutator.
#[derive(Debug)]
pub(crate) struct MultiBannerLineStatic {
    /// All the banner ASCII lines, one per word
    banners: Vec<Vec<String>>,
    /// Block lengths per banner for proper centering
    block_lengths: Vec<u16>,
    /// Alignment settings
    alignment: Alignment,
    /// Font size
    font_size: u8,
    /// Shared context determining which word to display
    context: Arc<Mutex<MultiBannerContext>>,
}

impl MultiBannerLineStatic {
    pub(crate) fn new(
        banners: Vec<Vec<String>>,
        block_lengths: Vec<u16>,
        alignment: Alignment,
        font_size: u8,
        context: Arc<Mutex<MultiBannerContext>>,
    ) -> Self {
        Self { banners, block_lengths, alignment, font_size, context }
    }
}

impl AsRenderOperations for MultiBannerLineStatic {
    fn as_render_operations(&self, _window: &WindowSize) -> Vec<RenderOperation> {
        use crate::markdown::elements::Text;
        use crate::markdown::text::{WeightedLine, WeightedText};
        use crate::markdown::text_style::TextStyle;

        let context = self.context.lock().unwrap();
        let current = context.current;
        drop(context);

        let Some(lines) = self.banners.get(current) else { return vec![]; };
        let block_length = self.block_lengths.get(current).copied().unwrap_or(0);

        // Static plain rendering - no colors, just use theme defaults
        let text_style = TextStyle::default().size(self.font_size);

        let mut operations = Vec::new();
        for line in lines.iter() {
            let text = Text::new(line, text_style);
            let weighted_line = WeightedLine::from(vec![text]);
            operations.push(RenderOperation::RenderBlockLine(BlockLine {
                prefix: WeightedText::from(""),
                right_padding_length: 0,
                repeat_prefix_on_wrap: false,
                text: weighted_line,
                block_length,
                alignment: self.alignment,
                block_color: None,
            }));
            operations.push(RenderOperation::RenderLineBreak);
        }

        operations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_font() {
        let generator = BannerGenerator::new("standard").expect("failed to create generator");
        let result = generator.generate("Hello").expect("failed to generate");
        assert!(!result.is_empty());
        assert!(result.contains("Hello") || result.len() > 10); // ASCII art should be larger
    }

    #[test]
    fn test_fallback_to_standard() {
        let generator = BannerGenerator::new("nonexistent_font_12345").expect("failed to create generator");
        let result = generator.generate("Hi").expect("failed to generate");
        assert!(!result.is_empty());
    }
}
