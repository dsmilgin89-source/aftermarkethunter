//! Wayback Machine signal — the strongest free SEO proxy we have.
//!
//! Uses the public CDX API which returns one row per snapshot.
//!   http://web.archive.org/cdx/search/cdx?url=DOMAIN&output=json&fl=timestamp&limit=10000
//!
//! From that we derive: total snapshot count, first and last snapshot.
//! High snapshot counts spread over many years = real, long-lived site
//! (great for SEO). Few snapshots clustered in recent years = parking page.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDateTime, Utc};

use crate::http::CLIENT;

#[derive(Debug, Default, Clone)]
pub struct WaybackInfo {
    pub snapshots: u32,
    pub first: Option<DateTime<Utc>>,
    pub last: Option<DateTime<Utc>>,
}

pub async fn lookup(domain: &str) -> Result<WaybackInfo> {
    crate::rate_limit::wait("wayback", 2).await;
    let url = format!(
        "https://web.archive.org/cdx/search/cdx?url={domain}&output=json&fl=timestamp&limit=10000"
    );
    let resp = CLIENT
        .get(&url)
        .send()
        .await
        .with_context(|| format!("GET {url}"))?;
    if !resp.status().is_success() {
        anyhow::bail!("wayback CDX HTTP {}", resp.status());
    }
    let body: serde_json::Value = resp.json().await.context("decode wayback json")?;
    let arr = body.as_array().cloned().unwrap_or_default();
    // First row is the header.
    let timestamps: Vec<String> = arr
        .into_iter()
        .skip(1)
        .filter_map(|row| row.as_array().and_then(|r| r.get(0).cloned()))
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();
    let snapshots = timestamps.len() as u32;
    let parse_ts = |s: &str| -> Option<DateTime<Utc>> {
        NaiveDateTime::parse_from_str(s, "%Y%m%d%H%M%S")
            .ok()
            .map(|d| DateTime::from_naive_utc_and_offset(d, Utc))
    };
    let first = timestamps.first().and_then(|s| parse_ts(s));
    let last = timestamps.last().and_then(|s| parse_ts(s));
    Ok(WaybackInfo {
        snapshots,
        first,
        last,
    })
}
