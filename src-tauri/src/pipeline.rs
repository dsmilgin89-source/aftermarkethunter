//! Search → enrich → score pipeline with progress events.
//!
//! - Fan-out scrapers in parallel (one per marketplace).
//! - De-duplicate listings by domain across sources.
//! - Run free enrichers concurrently (capped at 8 in-flight) using
//!   `futures::stream::buffer_unordered`.
//! - Compute scores synchronously (CPU-cheap).
//! - Returns the assembled `ResultRow`s in score-desc order.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use anyhow::Result;
use futures::stream::{FuturesUnordered, StreamExt};
use tauri::Emitter;

use crate::enrichers;
use crate::model::{Listing, Query, ResultRow, SearchProgress};
use crate::scoring;
use crate::scrapers;

pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    fn emit_progress(app: &tauri::AppHandle, progress: SearchProgress) {
        let _ = app.emit("search-progress", progress);
    }

    pub async fn run(&self, query: Query, app: tauri::AppHandle) -> Result<Vec<ResultRow>> {
        // 1. Fan-out scrapers
        let scrapers = scrapers::registry_for(&query.sources);
        tracing::info!(n = scrapers.len(), "running scrapers");

        let mut tasks = FuturesUnordered::new();
        for s in scrapers {
            let q = query.clone();
            let app_clone = app.clone();
            tasks.push(async move {
                match s.search(&q, &app_clone).await {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(marketplace = s.id(), error = %e, "scraper failed");
                        Vec::new()
                    }
                }
            });
        }
        let mut all: Vec<Listing> = Vec::new();
        while let Some(batch) = tasks.next().await {
            all.extend(batch);
        }

        // 2. Apply user filters (price, tld) — done here so failing scrapers
        //    can't make filters silently disappear.
        all.retain(|l| {
            if !query.tlds.is_empty() && !query.tlds.iter().any(|t| t == &l.tld) {
                return false;
            }
            if let Some(max) = query.max_price {
                if let Some(p) = l.current_price {
                    if p > max {
                        return false;
                    }
                }
            }
            true
        });

        // 3. Deduplicate by domain
        let mut seen: HashSet<String> = HashSet::new();
        all.retain(|l| seen.insert(l.domain.clone()));
        let total_domains = all.len() as u32;
        tracing::info!(unique = total_domains, "post-dedupe listings");

        // 4. Enrich in parallel (limit 8 in-flight) with progress
        let counter = Arc::new(AtomicU32::new(0));
        let enriched: Vec<(Listing, _)> = futures::stream::iter(all.into_iter())
            .map(|l| {
                let app_clone = app.clone();
                let counter = counter.clone();
                let total = total_domains;
                async move {
                    let e = enrichers::enrich_free(&l.domain).await;
                    let done = counter.fetch_add(1, Ordering::Relaxed) + 1;
                    Self::emit_progress(&app_clone, SearchProgress {
                        phase: "enriching".to_string(),
                        detail: format!("Enriching {done}/{total} domains..."),
                        current: done,
                        total: Some(total),
                        marketplace: None,
                    });
                    (l, e)
                }
            })
            .buffer_unordered(8)
            .collect()
            .await;

        // 5. Score
        Self::emit_progress(&app, SearchProgress {
            phase: "scoring".to_string(),
            detail: format!("Scoring {} results...", enriched.len()),
            current: 0,
            total: Some(enriched.len() as u32),
            marketplace: None,
        });

        let mut rows: Vec<ResultRow> = enriched
            .into_iter()
            .map(|(listing, enrichment)| {
                let score = scoring::score(&listing, &enrichment, &query);
                ResultRow {
                    listing,
                    enrichment,
                    score,
                }
            })
            .collect();

        // Apply post-enrichment filters that depend on enrichment data.
        if let Some(min_age) = query.min_age_years {
            rows.retain(|r| r.enrichment.age_years.map(|a| a >= min_age as f32).unwrap_or(false));
        }
        if let Some(min_snap) = query.min_wayback_snapshots {
            rows.retain(|r| r.enrichment.wayback_snapshots.map(|s| s >= min_snap).unwrap_or(false));
        }

        rows.sort_by(|a, b| b.score.total.partial_cmp(&a.score.total).unwrap_or(std::cmp::Ordering::Equal));

        // 6. Done
        Self::emit_progress(&app, SearchProgress {
            phase: "done".to_string(),
            detail: format!("{} results ready", rows.len()),
            current: rows.len() as u32,
            total: Some(rows.len() as u32),
            marketplace: None,
        });

        Ok(rows)
    }
}
