//! Domain types shared across the backend and exposed to the frontend via IPC.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User search input.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Query {
    /// Free-text phrase or exact name (e.g. "seo", "kawa", "premium").
    pub phrase: String,
    /// Restrict to TLDs ("pl", "com.pl", "com", ...). Empty = any.
    #[serde(default)]
    pub tlds: Vec<String>,
    /// Maximum current bid / buy-now price (in PLN, converted if needed).
    #[serde(default)]
    pub max_price: Option<f64>,
    /// Minimum domain age in years (uses WHOIS).
    #[serde(default)]
    pub min_age_years: Option<u32>,
    /// Minimum Wayback snapshots count.
    #[serde(default)]
    pub min_wayback_snapshots: Option<u32>,
    /// Sources to query (marketplace ids). Empty = all enabled.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Recommendation profile to score against.
    #[serde(default)]
    pub profile: ScoringProfile,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ScoringProfile {
    /// SEO Hunter — heavy weight on age + wayback + (DR/TF if available).
    #[default]
    SeoHunter,
    /// Brand Builder — favours short, brandable names regardless of history.
    BrandBuilder,
    /// Bargain — value-for-money obsessed.
    Bargain,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuctionType {
    Auction,
    BuyNow,
    Both,
    Unknown,
}

/// A single listing scraped from a marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Listing {
    /// Stable id "{marketplace}:{external_id|domain}".
    pub id: String,
    pub marketplace: String,
    pub domain: String,
    pub tld: String,
    pub current_price: Option<f64>,
    pub buy_now_price: Option<f64>,
    pub currency: String,
    pub auction_type: AuctionType,
    pub ends_at: Option<DateTime<Utc>>,
    pub url: String,
    pub fetched_at: DateTime<Utc>,
}

/// Free + paid enrichment data attached to a domain.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Enrichment {
    pub domain: String,
    pub whois_created: Option<DateTime<Utc>>,
    pub age_years: Option<f32>,
    pub wayback_first: Option<DateTime<Utc>>,
    pub wayback_last: Option<DateTime<Utc>>,
    pub wayback_snapshots: Option<u32>,
    pub blacklist_hits: u32,
    pub google_indexed_estimate: Option<u32>,
    pub linguistic: LinguisticReport,
    pub trademark_warning: Option<String>,
    /// Optional, only when API keys are configured.
    pub ahrefs_dr: Option<f32>,
    pub majestic_tf: Option<f32>,
    pub majestic_cf: Option<f32>,
    pub moz_da: Option<f32>,
    pub fetched_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LinguisticReport {
    pub length: u32,
    pub has_hyphen: bool,
    pub has_digit: bool,
    pub vowel_ratio: f32,
    pub syllable_estimate: u32,
    pub brandability: f32, // 0..100
    pub pronounceability: f32, // 0..100
}

/// Score breakdown returned to UI for the "Why?" popover.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Score {
    pub total: f32,
    pub seo: f32,
    pub relevance: f32,
    pub value: f32,
    pub risk_penalty: f32,
    pub tier: ScoreTier,
    pub explanation: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScoreTier {
    Excellent, // 80+
    Good,      // 60-79
    Fair,      // 40-59
    Poor,      // <40
}

impl ScoreTier {
    pub fn from_total(total: f32) -> Self {
        if total >= 80.0 {
            Self::Excellent
        } else if total >= 60.0 {
            Self::Good
        } else if total >= 40.0 {
            Self::Fair
        } else {
            Self::Poor
        }
    }
}

/// Combined row sent to the UI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultRow {
    pub listing: Listing,
    pub enrichment: Enrichment,
    pub score: Score,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    pub listing_id: String,
    pub max_bid: Option<f64>,
    pub notes: Option<String>,
    pub added_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedSearch {
    pub id: i64,
    pub name: String,
    pub query: Query,
    pub notify: bool,
    pub created_at: DateTime<Utc>,
}

/// Progress event emitted to the frontend via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProgress {
    pub phase: String,
    pub detail: String,
    pub current: u32,
    pub total: Option<u32>,
    pub marketplace: Option<String>,
}
