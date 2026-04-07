//! Domain enrichers — pluggable signals merged into a single `Enrichment`.
//!
//! Each enricher is independent, async, and best-effort: a failing enricher
//! never aborts the others. Free enrichers run by default; paid ones (Ahrefs,
//! Majestic, Moz, DataForSEO, SerpApi) only run when their API key is present
//! in the user settings (`Settings` panel → stored securely).

pub mod blacklist;
pub mod linguistic;
pub mod tld_value;
pub mod trademark;
pub mod wayback;
pub mod whois;

use chrono::Utc;

use crate::model::Enrichment;

/// Run all free enrichers for a domain. Errors are swallowed per-enricher and
/// surfaced as missing fields — UI shows "—" so the user knows what we don't know.
pub async fn enrich_free(domain: &str) -> Enrichment {
    let mut e = Enrichment {
        domain: domain.to_string(),
        ..Default::default()
    };
    e.linguistic = linguistic::analyze(domain);

    // Run network enrichers concurrently.
    let (whois_res, wayback_res, blacklist_res) = tokio::join!(
        whois::lookup(domain),
        wayback::lookup(domain),
        blacklist::check(domain),
    );

    if let Ok(w) = whois_res {
        e.whois_created = w.created;
        e.age_years = w.age_years;
    }
    if let Ok(w) = wayback_res {
        e.wayback_first = w.first;
        e.wayback_last = w.last;
        e.wayback_snapshots = Some(w.snapshots);
    }
    if let Ok(hits) = blacklist_res {
        e.blacklist_hits = hits;
    }

    e.trademark_warning = trademark::check(domain);
    e.fetched_at = Some(Utc::now());
    e
}
