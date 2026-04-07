//! Open PageRank API enricher.
//!
//! Free API (no credit card): https://www.domcop.com/openpagerank/
//! Returns page_rank_decimal (0-10) and global rank.
//! Rate limit: 10,000 requests/hour per key.

use anyhow::Result;
use serde::Deserialize;

use crate::http::CLIENT;

#[derive(Debug)]
pub struct PageRankInfo {
    pub score: f32,
    pub rank: Option<u64>,
}

#[derive(Deserialize)]
struct ApiResponse {
    response: Vec<DomainResult>,
}

#[derive(Deserialize)]
struct DomainResult {
    page_rank_decimal: Option<f64>,
    rank: Option<u64>,
}

/// Lookup domain's Open PageRank. Returns None if API key is empty/missing.
pub async fn lookup(domain: &str, api_key: &str) -> Result<Option<PageRankInfo>> {
    if api_key.is_empty() {
        return Ok(None);
    }

    let url = format!(
        "https://openpagerank.com/api/v1.0/getPageRank?domains[0]={}",
        domain
    );

    let resp = CLIENT
        .get(&url)
        .header("API-OPR", api_key)
        .send()
        .await?;

    if !resp.status().is_success() {
        tracing::debug!(domain, status = %resp.status(), "openpagerank request failed");
        return Ok(None);
    }

    let data: ApiResponse = resp.json().await?;

    if let Some(result) = data.response.first() {
        let score = result.page_rank_decimal.unwrap_or(0.0) as f32;
        Ok(Some(PageRankInfo {
            score,
            rank: result.rank,
        }))
    } else {
        Ok(None)
    }
}
