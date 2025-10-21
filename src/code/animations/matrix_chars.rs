/// Matrix digital rain characters: katakana, Latin, numbers, and symbols
pub(crate) const MATRIX_CHARS: &[char] = &[
    // Half-width katakana
    'ｦ', 'ｱ', 'ｲ', 'ｳ', 'ｴ', 'ｵ', 'ｶ', 'ｷ', 'ｸ', 'ｹ', 'ｺ', 'ｻ', 'ｼ', 'ｽ', 'ｾ', 'ｿ',
    'ﾀ', 'ﾁ', 'ﾂ', 'ﾃ', 'ﾄ', 'ﾅ', 'ﾆ', 'ﾇ', 'ﾈ', 'ﾉ', 'ﾊ', 'ﾋ', 'ﾌ', 'ﾍ', 'ﾎ', 'ﾏ',
    'ﾐ', 'ﾑ', 'ﾒ', 'ﾓ', 'ﾔ', 'ﾕ', 'ﾖ', 'ﾗ', 'ﾘ', 'ﾙ', 'ﾚ', 'ﾛ', 'ﾜ', 'ﾝ',
    // Numbers
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
    // Latin letters (reversed/mirrored for effect)
    'Z', 'Ǝ', 'Ɔ', 'T', 'Ǝ', 'X', 'Ɔ',
    // Symbols
    ':', '.', '=', '*', '+', '-', '<', '>', '¦', '|', '¬',
];

/// Get a pseudo-random character based on seed
pub(crate) fn get_char(seed: f32) -> char {
    let index = ((seed.fract().abs() * MATRIX_CHARS.len() as f32) as usize) % MATRIX_CHARS.len();
    MATRIX_CHARS[index]
}
