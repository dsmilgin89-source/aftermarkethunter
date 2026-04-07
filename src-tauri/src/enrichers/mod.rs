//! Domain enrichers — pluggable signals merged into a single `Enrichment`.
//!
//! Each enricher is independent, async, and best-effort: a failing enricher
//! never aborts the others. Free enrichers run by default; paid ones (Ahrefs,
//! Majestic, Moz, DataForSEO, SerpApi) only run when their API key is present
//! in the user settings (`Settings` panel → stored securely).

pub mod blacklist;
pub mod dns_quality;
pub mod linguistic;
pub mod openpagerank;
pub mod similarweb;
pub mod tld_value;
pub mod trademark;
pub mod wayback;
pub mod whois;

use chrono::Utc;

use crate::model::Enrichment;

/// Run all free enrichers for a domain. Errors are swallowed per-enricher and
/// surfaced as missing fields — UI shows "—" so the user knows what we don't know.
///
/// `openpagerank_key` is read from plugin-store settings; empty string = skip.
pub async fn enrich_free(domain: &str, openpagerank_key: &str) -> Enrichment {
    let mut e = Enrichment {
        domain: domain.to_string(),
        ..Default::default()
    };
    e.linguistic = linguistic::analyze(domain);

    // Run network enrichers concurrently.
    let (whois_res, wayback_res, blacklist_res, similarweb_res, openpagerank_res, dns_res) =
        tokio::join!(
            whois::lookup(domain),
            wayback::lookup(domain),
            blacklist::check(domain),
            similarweb::lookup(domain),
            openpagerank::lookup(domain, openpagerank_key),
            dns_quality::check(domain),
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
    if let Ok(sw) = similarweb_res {
        e.similarweb_rank = sw.global_rank;
        e.similarweb_monthly_visits = sw.monthly_visits;
    }
    if let Ok(Some(pr)) = openpagerank_res {
        e.openpagerank_score = Some(pr.score);
        e.openpagerank_rank = pr.rank;
    }
    e.has_mx = dns_res.has_mx;
    e.has_spf = dns_res.has_spf;
    e.has_dmarc = dns_res.has_dmarc;

    e.trademark_warning = trademark::check(domain);
    e.fetched_at = Some(Utc::now());
    e
}
