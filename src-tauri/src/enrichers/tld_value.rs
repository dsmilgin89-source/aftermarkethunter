//! Static TLD value table — Polish-market biased.
//!
//! Returns a 0..100 baseline reflecting how desirable a given TLD is for an
//! SEO/brand purchase in PL. Used as one input into `value_score`.

pub fn score(tld: &str) -> f32 {
    match tld.to_lowercase().as_str() {
        "pl" => 100.0,
        "com" => 95.0,
        "com.pl" => 80.0,
        "io" => 75.0,
        "net" => 65.0,
        "net.pl" => 55.0,
        "org" => 60.0,
        "org.pl" => 50.0,
        "info" => 35.0,
        "info.pl" => 30.0,
        "biz" => 30.0,
        "eu" => 55.0,
        "shop" => 50.0,
        _ => 25.0,
    }
}
