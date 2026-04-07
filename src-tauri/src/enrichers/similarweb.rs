//! Similarweb free (undocumented) API enricher.
//!
//! Endpoint: https://data.similarweb.com/api/v1/data?domain={domain}
//! No auth needed. Returns traffic estimates and global rank.
//! Undocumented — may break at any time; all errors are swallowed.

use anyhow::Result;
use serde::Deserialize;
use std::collections::HashMap;

use crate::http::CLIENT;

#[derive(Debug)]
pub struct SimilarwebInfo {
    pub global_rank: Option<u64>,
    pub monthly_visits: Option<u64>,
}

#[derive(Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
struct ApiResponse {
    global_rank: Option<GlobalRank>,
    estimated_monthly_visits: Option<HashMap<String, f64>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct GlobalRank {
    rank: Option<u64>,
}

pub async fn lookup(domain: &str) -> Result<SimilarwebInfo> {
    let url = format!(
        "https://data.similarweb.com/api/v1/data?domain={}",
        domain
    );

    let resp = CLIENT
        .get(&url)
        .header("Accept", "application/json")
        .send()
        .await?;

    if !resp.status().is_success() {
        return Ok(SimilarwebInfo {
            global_rank: None,
            monthly_visits: None,
        });
    }

    let data: ApiResponse = resp.json().await.unwrap_or_default();

    let global_rank = data.global_rank.and_then(|g| g.rank);

    // Sum all months' visits to get the most recent estimate
    let monthly_visits = data.estimated_monthly_visits.and_then(|visits| {
        visits.values().last().map(|v| *v as u64)
    });

    Ok(SimilarwebInfo {
        global_rank,
        monthly_visits,
    })
}
