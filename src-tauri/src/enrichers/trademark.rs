//! Tiny in-memory trademark/brand watchlist. Cheap heuristic — flags an obvious
//! infringement risk so the user can think twice before bidding.
//!
//! In a future iteration this can be backed by EUIPO TMview API.

const PROTECTED: &[&str] = &[
    "google", "youtube", "facebook", "instagram", "tiktok", "twitter", "x.com",
    "amazon", "apple", "microsoft", "netflix", "spotify", "tesla", "nike",
    "adidas", "samsung", "huawei", "sony",
    // PL brands
    "allegro", "olx", "empik", "biedronka", "lidl", "orlen", "pkn", "pko",
    "mbank", "ing", "santander", "millennium", "interia", "wp", "onet",
    "coca-cola", "cocacola", "nestle", "ikea",
];

pub fn check(domain: &str) -> Option<String> {
    let sld = domain
        .split('.')
        .next()
        .unwrap_or(domain)
        .to_lowercase()
        .replace('-', "");
    for brand in PROTECTED {
        if sld.contains(brand) {
            return Some(format!("Potential trademark conflict: '{brand}'"));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flags_known_brand() {
        assert!(check("amazon-shop.pl").is_some());
        assert!(check("kawa-z-mleka.pl").is_none());
    }
}
