mod common;
mod glitch_chars;
mod matrix_chars;

// Individual animation modules
mod aurora;
mod breathe;
mod crt;
mod fire;
mod flash;
mod glitch;
mod iris;
mod kaleidoscope;
mod matrix;
mod neon;
mod plasma;
mod prism;
mod rainbow;
mod scanner;
mod sepia;
mod typewriter;
mod wave;

pub(crate) use common::{Animation, AnimationContext, CharAnimationResult, hsl_to_rgb, rainbow_color};

use crate::code::snippet::BannerAnimationStyle;

/// Get the animation implementation for a given style
pub(crate) fn get_animation(style: BannerAnimationStyle) -> Box<dyn Animation> {
    match style {
        BannerAnimationStyle::Rainbow => Box::new(rainbow::Rainbow),
        BannerAnimationStyle::Flash => Box::new(flash::Flash),
        BannerAnimationStyle::Wave => Box::new(wave::Wave),
        BannerAnimationStyle::Iris => Box::new(iris::Iris),
        BannerAnimationStyle::Plasma => Box::new(plasma::Plasma),
        BannerAnimationStyle::Scanner => Box::new(scanner::Scanner),
        BannerAnimationStyle::Matrix => Box::new(matrix::Matrix),
        BannerAnimationStyle::Neon => Box::new(neon::Neon),
        BannerAnimationStyle::Kaleidoscope => Box::new(kaleidoscope::Kaleidoscope),
        BannerAnimationStyle::Sepia => Box::new(sepia::Sepia),
        BannerAnimationStyle::Prism => Box::new(prism::Prism),
        BannerAnimationStyle::Glitch => Box::new(glitch::Glitch),
        BannerAnimationStyle::Breathe => Box::new(breathe::Breathe),
        BannerAnimationStyle::Fire => Box::new(fire::Fire),
        BannerAnimationStyle::Aurora => Box::new(aurora::Aurora),
        BannerAnimationStyle::Crt => Box::new(crt::Crt),
        BannerAnimationStyle::Typewriter => Box::new(typewriter::Typewriter),
    }
}
