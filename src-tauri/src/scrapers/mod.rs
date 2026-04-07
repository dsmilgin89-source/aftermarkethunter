//! Marketplace scraper trait + registry.
//!
//! Each scraper:
//! - declares its `id()` (also used for rate-limit bucketing)
//! - implements `search()` and (optionally) `fetch_details()`
//! - keeps selectors in module-level constants so layout breakage = compile-friendly diffs
//! - has a fixture-based unit test in `tests/fixtures/` so we don't depend on the network

pub mod aftermarket_pl;

use anyhow::Result;
use async_trait::async_trait;

use crate::model::{Listing, Query};

#[async_trait]
pub trait Marketplace: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn rps(&self) -> u32 {
        1
    }
    async fn search(&self, query: &Query, app: &tauri::AppHandle) -> Result<Vec<Listing>>;
}

pub fn registry() -> Vec<Box<dyn Marketplace>> {
    vec![
        Box::new(aftermarket_pl::AftermarketPl),
    ]
}

/// Filter the registry by user-provided source ids; empty = all.
pub fn registry_for(sources: &[String]) -> Vec<Box<dyn Marketplace>> {
    let all = registry();
    if sources.is_empty() {
        return all;
    }
    all.into_iter()
        .filter(|m| sources.iter().any(|s| s == m.id()))
        .collect()
}
