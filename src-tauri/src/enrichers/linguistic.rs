//! Pure-Rust linguistic analysis. No network. Always available.
//!
//! Heuristics:
//! - Length penalty (sweet spot 5–9 chars on the SLD).
//! - Hyphens / digits drop brandability hard.
//! - Vowel/consonant balance + naive syllable count → pronounceability proxy.
//! - Brandability = mix of length + vowel ratio + lack of digits/hyphens.

use crate::model::LinguisticReport;

pub fn analyze(domain: &str) -> LinguisticReport {
    let sld = domain.split('.').next().unwrap_or(domain).to_lowercase();
    let length = sld.chars().count() as u32;
    let has_hyphen = sld.contains('-');
    let has_digit = sld.chars().any(|c| c.is_ascii_digit());

    let vowels = "aeiouyąęó";
    let mut v = 0u32;
    let mut c = 0u32;
    for ch in sld.chars() {
        if ch.is_alphabetic() {
            if vowels.contains(ch) {
                v += 1;
            } else {
                c += 1;
            }
        }
    }
    let total = (v + c).max(1);
    let vowel_ratio = v as f32 / total as f32;

    // Naive syllable count = number of vowel groups
    let mut syllables = 0u32;
    let mut in_vowel = false;
    for ch in sld.chars() {
        let is_v = vowels.contains(ch);
        if is_v && !in_vowel {
            syllables += 1;
        }
        in_vowel = is_v;
    }

    // Pronounceability: closer to 0.4–0.55 vowel ratio is the sweet spot.
    let vr_score = (1.0 - (vowel_ratio - 0.45).abs() * 2.0).clamp(0.0, 1.0);
    let mut pron = vr_score * 100.0;
    if has_hyphen {
        pron -= 25.0;
    }
    if has_digit {
        pron -= 15.0;
    }
    pron = pron.clamp(0.0, 100.0);

    // Brandability: short, all-letters, balanced.
    let len_score = match length {
        0..=3 => 70.0,        // very short — premium but rare
        4..=6 => 100.0,       // sweet spot
        7..=9 => 85.0,
        10..=12 => 60.0,
        13..=15 => 35.0,
        _ => 15.0,
    };
    let mut brand = (len_score * 0.55) + (pron * 0.45);
    if has_hyphen {
        brand -= 30.0;
    }
    if has_digit {
        brand -= 20.0;
    }
    brand = brand.clamp(0.0, 100.0);

    LinguisticReport {
        length,
        has_hyphen,
        has_digit,
        vowel_ratio,
        syllable_estimate: syllables,
        brandability: brand,
        pronounceability: pron,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rewards_short_clean_brandable() {
        let r = analyze("kawa.pl");
        assert!(r.brandability > 70.0, "kawa.pl should be brandable");
        assert!(!r.has_digit && !r.has_hyphen);
    }

    #[test]
    fn punishes_hyphens_and_digits() {
        let r = analyze("seo-1234-pro.pl");
        assert!(r.has_hyphen);
        assert!(r.has_digit);
        assert!(r.brandability < 50.0);
    }
}
