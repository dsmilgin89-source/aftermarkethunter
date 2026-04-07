//! Scoring engine.
//!
//! Inputs: a `Listing`, its `Enrichment`, the user `Query` (for relevance) and
//! the chosen `ScoringProfile`. Output: a 0..100 `Score` with the four
//! sub-scores plus an `explanation` list of short human-readable bullets that
//! the UI's "Why?" popover renders.
//!
//! Sub-scores
//! ----------
//! - **seo**       — domain age + Wayback snapshots + (Ahrefs DR / Majestic TF if present)
//! - **relevance** — string similarity to user phrase + linguistic brandability
//! - **value**     — estimated_worth / current_price (clamped, log-scaled)
//! - **risk**      — blacklist hits + parking heuristics + trademark conflicts
//!
//! The four sub-scores are weighted by a profile-specific weight tuple, then
//! combined: `total = sum(w_i * s_i) - risk_penalty`.

use crate::enrichers::tld_value;
use crate::model::{Enrichment, Listing, Query, Score, ScoreTier, ScoringProfile};

#[derive(Debug, Clone, Copy)]
struct Weights {
    seo: f32,
    relevance: f32,
    value: f32,
}

fn weights(profile: ScoringProfile) -> Weights {
    match profile {
        ScoringProfile::SeoHunter => Weights {
            seo: 0.55,
            relevance: 0.15,
            value: 0.30,
        },
        ScoringProfile::BrandBuilder => Weights {
            seo: 0.15,
            relevance: 0.45,
            value: 0.40,
        },
        ScoringProfile::Bargain => Weights {
            seo: 0.30,
            relevance: 0.10,
            value: 0.60,
        },
    }
}

pub fn score(listing: &Listing, enrichment: &Enrichment, query: &Query) -> Score {
    let w = weights(query.profile);
    let mut explain: Vec<String> = Vec::new();

    // ---- SEO ----
    let seo_score = seo_subscore(enrichment, &mut explain);

    // ---- Relevance ----
    let relevance_score = relevance_subscore(listing, enrichment, &query.phrase, &mut explain);

    // ---- Value ----
    let (value_score, estimated_worth) = value_subscore(listing, enrichment, &mut explain);

    // ---- Risk ----
    let risk_penalty = risk_subscore(enrichment, &mut explain);

    let combined = w.seo * seo_score + w.relevance * relevance_score + w.value * value_score;
    let total = (combined - risk_penalty).clamp(0.0, 100.0);

    if let Some(price) = listing.current_price {
        explain.push(format!(
            "Estimated worth ≈ {} PLN vs current price {} PLN",
            estimated_worth.round(),
            price.round()
        ));
    }

    Score {
        total,
        seo: seo_score,
        relevance: relevance_score,
        value: value_score,
        risk_penalty,
        tier: ScoreTier::from_total(total),
        explanation: explain,
    }
}

fn seo_subscore(e: &Enrichment, explain: &mut Vec<String>) -> f32 {
    let mut s = 0.0;
    let mut weight_sum = 0.0;

    // Age — sigmoid centred on 5 years.
    if let Some(age) = e.age_years {
        let age_score = sigmoid((age - 5.0) / 3.0) * 100.0;
        s += age_score * 0.40;
        weight_sum += 0.40;
        explain.push(format!("Domain age: {:.1}y → {:.0}/100", age, age_score));
    }

    // Wayback snapshots — log scale.
    if let Some(snap) = e.wayback_snapshots {
        let snap_score = ((snap as f32 + 1.0).ln() / 9.0_f32.ln() * 100.0).clamp(0.0, 100.0);
        s += snap_score * 0.30;
        weight_sum += 0.30;
        explain.push(format!("Wayback snapshots: {} → {:.0}/100", snap, snap_score));
    }

    // Paid signals — highest priority when present.
    if let Some(dr) = e.ahrefs_dr {
        s += dr * 0.20;
        weight_sum += 0.20;
        explain.push(format!("Ahrefs DR: {dr:.0}"));
    } else if let Some(tf) = e.majestic_tf {
        s += tf * 0.20;
        weight_sum += 0.20;
        explain.push(format!("Majestic TF: {tf:.0}"));
    } else {
        // Free SEO signals — composite fallback when no paid API keys.
        let free_score = free_seo_composite(e, explain);
        s += free_score * 0.20;
        weight_sum += 0.20;
    }

    // DNS quality bonus (always available, small weight).
    let dns_score = dns_quality_score(e);
    if dns_score > 0.0 {
        s += dns_score * 0.10;
        weight_sum += 0.10;
        let parts: Vec<&str> = [
            if e.has_mx { Some("MX") } else { None },
            if e.has_spf { Some("SPF") } else { None },
            if e.has_dmarc { Some("DMARC") } else { None },
        ]
        .iter()
        .filter_map(|x| *x)
        .collect();
        if !parts.is_empty() {
            explain.push(format!("DNS signals: {} → {:.0}/100", parts.join("+"), dns_score));
        }
    }

    if weight_sum == 0.0 {
        return 30.0; // unknown — neutral-low
    }
    (s / weight_sum).clamp(0.0, 100.0)
}

/// Composite score from free SEO sources (Open PageRank + Similarweb).
/// Used as fallback when no paid API key (Ahrefs/Majestic) is configured.
fn free_seo_composite(e: &Enrichment, explain: &mut Vec<String>) -> f32 {
    let mut total = 0.0;
    let mut count = 0.0;

    // Open PageRank (0-10 scale → 0-100)
    if let Some(pr) = e.openpagerank_score {
        let pr_score = (pr / 10.0 * 100.0).clamp(0.0, 100.0);
        total += pr_score;
        count += 1.0;
        explain.push(format!("Open PageRank: {pr:.1}/10 → {pr_score:.0}/100"));
    }

    // Similarweb rank (lower = better, log-scaled)
    if let Some(rank) = e.similarweb_rank {
        if rank > 0 {
            let rank_score = (100.0 - (rank as f32).log10() * 15.0).clamp(0.0, 100.0);
            total += rank_score;
            count += 1.0;
            explain.push(format!("Similarweb rank: #{rank} → {rank_score:.0}/100"));
        }
    }

    // Similarweb monthly visits bonus
    if let Some(visits) = e.similarweb_monthly_visits {
        if visits > 0 {
            let visits_score = ((visits as f32).log10() * 20.0).clamp(0.0, 100.0);
            total += visits_score;
            count += 1.0;
            explain.push(format!("Monthly visits: {visits} → {visits_score:.0}/100"));
        }
    }

    if count == 0.0 {
        30.0 // no free signals either — neutral fallback
    } else {
        total / count
    }
}

/// DNS quality: MX + SPF + DMARC → configured email = legitimate domain.
fn dns_quality_score(e: &Enrichment) -> f32 {
    let mut score = 0.0;
    if e.has_mx { score += 40.0; }
    if e.has_spf { score += 30.0; }
    if e.has_dmarc { score += 30.0; }
    score
}

fn relevance_subscore(
    listing: &Listing,
    enrichment: &Enrichment,
    phrase: &str,
    explain: &mut Vec<String>,
) -> f32 {
    let phrase = phrase.trim().to_lowercase();
    let sld = listing
        .domain
        .split('.')
        .next()
        .unwrap_or(&listing.domain)
        .to_lowercase();

    let mut similarity = if phrase.is_empty() {
        50.0
    } else {
        let jw = strsim::jaro_winkler(&sld, &phrase) as f32 * 100.0;
        let contains_bonus: f32 = if sld.contains(&phrase) { 25.0 } else { 0.0 };
        (jw + contains_bonus).clamp(0.0, 100.0)
    };
    explain.push(format!("Name similarity to query → {:.0}/100", similarity));

    // Mix in brandability so a horrible name doesn't sneak in just because it
    // matches a phrase.
    similarity = similarity * 0.65 + enrichment.linguistic.brandability * 0.35;
    similarity.clamp(0.0, 100.0)
}

fn value_subscore(
    listing: &Listing,
    enrichment: &Enrichment,
    explain: &mut Vec<String>,
) -> (f32, f32) {
    let estimated = estimate_worth(listing, enrichment);
    let price = listing.current_price.map(|p| p as f32).unwrap_or(estimated);
    if price <= 0.0 {
        return (50.0, estimated);
    }
    // Score = how much cheaper than estimate, on a log scale.
    let ratio = estimated / price;
    let score = ((ratio.ln() + 1.0) * 50.0).clamp(0.0, 100.0);
    explain.push(format!(
        "Value ratio est/price = {:.2} → {:.0}/100",
        ratio, score
    ));
    (score, estimated)
}

/// Heuristic estimated worth in PLN. Calibrated by hand; will need iteration.
fn estimate_worth(listing: &Listing, e: &Enrichment) -> f32 {
    let tld_factor = tld_value::score(&listing.tld) / 100.0;
    let age_bonus = e.age_years.unwrap_or(0.0).clamp(0.0, 25.0) * 40.0;
    let wayback_bonus = e
        .wayback_snapshots
        .unwrap_or(0)
        .min(5_000) as f32
        * 0.5;
    let brand_bonus = e.linguistic.brandability * 4.0;
    let dr_bonus = e.ahrefs_dr.unwrap_or(0.0) * 25.0;
    // Free SEO bonuses
    let pagerank_bonus = e.openpagerank_score.unwrap_or(0.0) * 200.0;
    let traffic_bonus = e
        .similarweb_monthly_visits
        .map(|v| if v > 0 { (v as f32).log10() * 100.0 } else { 0.0 })
        .unwrap_or(0.0);
    let dns_bonus = if e.has_mx { 50.0 } else { 0.0 }
        + if e.has_spf { 30.0 } else { 0.0 }
        + if e.has_dmarc { 20.0 } else { 0.0 };
    let base = 80.0;
    (base + age_bonus + wayback_bonus + brand_bonus + dr_bonus + pagerank_bonus + traffic_bonus + dns_bonus) * tld_factor
}

fn risk_subscore(e: &Enrichment, explain: &mut Vec<String>) -> f32 {
    let mut penalty = 0.0;
    if e.blacklist_hits > 0 {
        penalty += 25.0 + 10.0 * (e.blacklist_hits as f32 - 1.0);
        explain.push(format!("⚠ Blacklist hits: {}", e.blacklist_hits));
    }
    if let Some(t) = &e.trademark_warning {
        penalty += 30.0;
        explain.push(format!("⚠ {t}"));
    }
    if e.linguistic.has_hyphen {
        penalty += 5.0;
    }
    if e.linguistic.length > 18 {
        penalty += 10.0;
        explain.push("⚠ Very long name".to_string());
    }
    penalty.min(60.0)
}

fn sigmoid(x: f32) -> f32 {
    1.0 / (1.0 + (-x).exp())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{AuctionType, LinguisticReport};
    use chrono::Utc;

    fn dummy_listing(domain: &str, price: f64) -> Listing {
        Listing {
            id: format!("test:{domain}"),
            marketplace: "test".into(),
            domain: domain.into(),
            tld: domain.split_once('.').unwrap().1.into(),
            current_price: Some(price),
            buy_now_price: None,
            currency: "PLN".into(),
            auction_type: AuctionType::Auction,
            ends_at: None,
            url: "https://example".into(),
            fetched_at: Utc::now(),
        }
    }

    #[test]
    fn old_cheap_brandable_scores_high() {
        let l = dummy_listing("kawa.pl", 200.0);
        let mut e = Enrichment::default();
        e.age_years = Some(15.0);
        e.wayback_snapshots = Some(2000);
        e.linguistic = LinguisticReport {
            length: 4,
            brandability: 95.0,
            pronounceability: 90.0,
            ..Default::default()
        };
        let q = Query {
            phrase: "kawa".into(),
            ..Default::default()
        };
        let s = score(&l, &e, &q);
        assert!(s.total > 70.0, "expected high score, got {}", s.total);
    }

    #[test]
    fn risky_blacklisted_scores_low() {
        let l = dummy_listing("amazon-deals.pl", 5000.0);
        let mut e = Enrichment::default();
        e.blacklist_hits = 2;
        e.trademark_warning = Some("amazon".into());
        e.linguistic = LinguisticReport {
            length: 13,
            has_hyphen: true,
            brandability: 25.0,
            pronounceability: 50.0,
            ..Default::default()
        };
        let q = Query::default();
        let s = score(&l, &e, &q);
        assert!(s.total < 40.0, "expected low score, got {}", s.total);
    }
}
