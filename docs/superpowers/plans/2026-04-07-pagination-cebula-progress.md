# Pagination + Progress Bar + Cebula Deals — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add full pagination for aftermarket.pl scraper, a real-time progress bar using Tauri events, and a "Cebula Deals" section showing exceptionally good offers with configurable thresholds.

**Architecture:** The `Marketplace` trait gains an `AppHandle` parameter so scrapers can emit progress events. The pipeline emits phase-based events (scraping/enriching/scoring/done) that the frontend listens to via `@tauri-apps/api/event`. Cebula Deals is a pure frontend filter over search results with thresholds persisted in `tauri-plugin-store`.

**Tech Stack:** Rust (Tauri 2 events, reqwest, scraper, regex), React 18 (TanStack Query, Tailwind CSS, lucide-react), `@tauri-apps/api/event` for event listening, `@tauri-apps/plugin-store` for threshold persistence.

---

## File Map

| File | Action | Responsibility |
|------|--------|----------------|
| `src-tauri/src/model.rs` | Modify | Add `SearchProgress` struct |
| `src-tauri/src/scrapers/mod.rs` | Modify | Update `Marketplace` trait with `AppHandle` |
| `src-tauri/src/scrapers/aftermarket_pl.rs` | Modify | Pagination loop, total parsing, progress emission |
| `src-tauri/src/scrapers/premium_pl.rs` | Modify | Update trait impl signature |
| `src-tauri/src/scrapers/dropped_pl.rs` | Modify | Update trait impl signature |
| `src-tauri/src/pipeline.rs` | Modify | Accept `AppHandle`, emit enriching/scoring/done events |
| `src-tauri/src/commands.rs` | Modify | Pass `AppHandle` to pipeline |
| `src/lib/types.ts` | Modify | Add `SearchProgress`, `CebulaThresholds` types |
| `src/components/ProgressBar.tsx` | Create | Progress bar component with Tauri event listener |
| `src/components/CebulaDeals.tsx` | Create | Deals section with card layout |
| `src/routes/Hunt.tsx` | Modify | Integrate ProgressBar + CebulaDeals |
| `src/routes/Settings.tsx` | Modify | Add Cebula threshold configuration section |

---

### Task 1: Add `SearchProgress` to Rust model

**Files:**
- Modify: `src-tauri/src/model.rs`

- [ ] **Step 1: Add the `SearchProgress` struct**

Add at the end of `src-tauri/src/model.rs`, before the closing of the file:

```rust
/// Progress event emitted to the frontend via Tauri events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchProgress {
    pub phase: String,
    pub detail: String,
    pub current: u32,
    pub total: Option<u32>,
    pub marketplace: Option<String>,
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/model.rs
git commit -m "feat(model): add SearchProgress struct for pipeline events"
```

---

### Task 2: Update `Marketplace` trait with `AppHandle`

**Files:**
- Modify: `src-tauri/src/scrapers/mod.rs`
- Modify: `src-tauri/src/scrapers/premium_pl.rs`
- Modify: `src-tauri/src/scrapers/dropped_pl.rs`

- [ ] **Step 1: Update the trait in `scrapers/mod.rs`**

Replace the entire file content of `src-tauri/src/scrapers/mod.rs`:

```rust
//! Marketplace scraper trait + registry.

pub mod aftermarket_pl;
pub mod premium_pl;
pub mod dropped_pl;

use anyhow::Result;
use async_trait::async_trait;

use crate::model::{Listing, Query};

#[async_trait]
pub trait Marketplace: Send + Sync {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    /// Suggested politeness budget (requests per second).
    fn rps(&self) -> u32 {
        1
    }
    async fn search(&self, query: &Query, app: &tauri::AppHandle) -> Result<Vec<Listing>>;
}

/// Returns all built-in marketplaces.
pub fn registry() -> Vec<Box<dyn Marketplace>> {
    vec![
        Box::new(aftermarket_pl::AftermarketPl),
        Box::new(premium_pl::PremiumPl),
        Box::new(dropped_pl::DroppedPl),
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
```

- [ ] **Step 2: Update `premium_pl.rs` stub**

Replace the `search` signature in `src-tauri/src/scrapers/premium_pl.rs`:

```rust
//! premium.pl scraper. Stub: returns empty until selectors are reverse-engineered.

use anyhow::Result;
use async_trait::async_trait;

use crate::model::{Listing, Query};
use crate::scrapers::Marketplace;

pub struct PremiumPl;

#[async_trait]
impl Marketplace for PremiumPl {
    fn id(&self) -> &'static str {
        "premium_pl"
    }
    fn label(&self) -> &'static str {
        "premium.pl"
    }

    async fn search(&self, _query: &Query, _app: &tauri::AppHandle) -> Result<Vec<Listing>> {
        Ok(Vec::new())
    }
}
```

- [ ] **Step 3: Update `dropped_pl.rs` stub**

Replace `src-tauri/src/scrapers/dropped_pl.rs`:

```rust
//! dropped.pl scraper. Stub for now (see premium_pl for rationale).

use anyhow::Result;
use async_trait::async_trait;

use crate::model::{Listing, Query};
use crate::scrapers::Marketplace;

pub struct DroppedPl;

#[async_trait]
impl Marketplace for DroppedPl {
    fn id(&self) -> &'static str {
        "dropped_pl"
    }
    fn label(&self) -> &'static str {
        "dropped.pl"
    }

    async fn search(&self, _query: &Query, _app: &tauri::AppHandle) -> Result<Vec<Listing>> {
        Ok(Vec::new())
    }
}
```

- [ ] **Step 4: Update `aftermarket_pl.rs` — signature only (pagination comes in Task 3)**

In `src-tauri/src/scrapers/aftermarket_pl.rs`, change the `search` method signature from:

```rust
    async fn search(&self, query: &Query) -> Result<Vec<Listing>> {
```

to:

```rust
    async fn search(&self, query: &Query, _app: &tauri::AppHandle) -> Result<Vec<Listing>> {
```

(Just add `_app` — the pagination task will use it.)

- [ ] **Step 5: Verify it compiles (expect errors in pipeline.rs — that's next)**

Run: `cd src-tauri && cargo check 2>&1 | head -20`
Expected: errors in `pipeline.rs` about `search()` argument count — this is correct; Task 4 fixes it.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/scrapers/
git commit -m "feat(scrapers): add AppHandle param to Marketplace trait for progress events"
```

---

### Task 3: Implement pagination in aftermarket.pl scraper

**Files:**
- Modify: `src-tauri/src/scrapers/aftermarket_pl.rs`

- [ ] **Step 1: Add the total-count parser and pagination regex**

Add these after the existing `DATETIME_RE` lazy static in `aftermarket_pl.rs`:

```rust
static TOTAL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"z\s+(\d+)\s+obiekt").unwrap());

const PAGE_SIZE: u32 = 30;
const MAX_PAGES: u32 = 50;
```

- [ ] **Step 2: Add a helper to parse total count from HTML**

Add this function after `parse_datetime`:

```rust
/// Extract total listing count from "Pokazuję 1 - 30 z **98** obiektów".
fn parse_total(html: &str) -> Option<u32> {
    TOTAL_RE.captures(html).and_then(|c| c[1].parse().ok())
}
```

- [ ] **Step 3: Write a test for the total parser**

Add in the `#[cfg(test)] mod tests` block:

```rust
    #[test]
    fn parses_total_count() {
        let html = r#"Pokazuję <strong>1</strong> - <strong>30</strong> z <strong>98</strong> obiektów"#;
        assert_eq!(parse_total(html), Some(98));

        let html_none = "<div>no pagination here</div>";
        assert_eq!(parse_total(html_none), None);
    }
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd src-tauri && cargo test --lib aftermarket_pl::tests::parses_total_count -- --nocapture`
Expected: PASS

- [ ] **Step 5: Rewrite `search()` with pagination loop**

Replace the entire `search` method in the `impl Marketplace for AftermarketPl` block:

```rust
    async fn search(&self, query: &Query, app: &tauri::AppHandle) -> Result<Vec<Listing>> {
        use tauri::Emitter;
        use crate::model::SearchProgress;

        let mut all_listings: Vec<Listing> = Vec::new();
        let mut start: u32 = 0;
        let mut total: Option<u32> = None;
        let mut page: u32 = 0;

        loop {
            page += 1;
            if page > MAX_PAGES {
                tracing::warn!("aftermarket.pl: hit max page limit ({MAX_PAGES}), stopping");
                break;
            }

            rate_limit::wait(self.id(), self.rps()).await;

            let url = reqwest::Url::parse_with_params(
                SEARCH_URL,
                &[
                    ("domain", query.phrase.as_str()),
                    ("length1", ""),
                    ("length2", ""),
                    ("price1", ""),
                    ("price2", ""),
                    ("price3", "PLN"),
                    ("extension", ""),
                    ("category", ""),
                    ("type", ""),
                    ("start1", ""),
                    ("start2", ""),
                    ("idn", "0"),
                    ("seller", ""),
                    ("bin", "0"),
                    ("auction", "0"),
                    ("offers", "0"),
                    ("hire", "0"),
                    ("rental", "0"),
                    ("group", "0"),
                    ("lastminute", "0"),
                    ("is_catch", "0"),
                    ("future", "0"),
                    ("_sort", ""),
                    ("_start", &start.to_string()),
                ],
            )
            .context("building aftermarket.pl url")?;

            tracing::debug!(%url, page, "aftermarket.pl search page");
            let resp = CLIENT
                .get(url.clone())
                .header("Accept", "text/html,application/xhtml+xml")
                .header("Accept-Language", "pl-PL,pl;q=0.9,en;q=0.8")
                .send()
                .await
                .with_context(|| format!("GET {url}"))?;
            let status = resp.status();
            let body = resp.text().await?;
            if !status.is_success() {
                anyhow::bail!("aftermarket.pl returned HTTP {status}");
            }

            // Parse total on first page
            if total.is_none() {
                total = parse_total(&body);
            }

            let page_listings = parse_listings(&body);
            let page_count = page_listings.len();
            all_listings.extend(page_listings);

            // Emit progress
            let total_pages = total.map(|t| (t + PAGE_SIZE - 1) / PAGE_SIZE);
            let _ = app.emit("search-progress", SearchProgress {
                phase: "scraping".to_string(),
                detail: format!(
                    "{} — strona {}/{} ({} znalezionych)",
                    self.label(),
                    page,
                    total_pages.map(|t| t.to_string()).unwrap_or("?".to_string()),
                    all_listings.len(),
                ),
                current: page,
                total: total_pages,
                marketplace: Some(self.label().to_string()),
            });

            tracing::info!(page, page_count, total_so_far = all_listings.len(), "aftermarket.pl page parsed");

            // Stop conditions
            if page_count == 0 {
                break;
            }
            start += PAGE_SIZE;
            if let Some(t) = total {
                if start >= t {
                    break;
                }
            }
        }

        tracing::info!(total = all_listings.len(), "aftermarket.pl all pages parsed");
        Ok(all_listings)
    }
```

- [ ] **Step 6: Verify existing tests still pass**

Run: `cd src-tauri && cargo test --lib aftermarket_pl -- --nocapture`
Expected: `parses_fixture`, `parses_polish_price`, `parses_total_count` all PASS

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/scrapers/aftermarket_pl.rs
git commit -m "feat(aftermarket_pl): paginate through all result pages with progress events"
```

---

### Task 4: Update pipeline to accept `AppHandle` and emit progress events

**Files:**
- Modify: `src-tauri/src/pipeline.rs`
- Modify: `src-tauri/src/commands.rs`

- [ ] **Step 1: Rewrite `pipeline.rs` with progress emission**

Replace the entire content of `src-tauri/src/pipeline.rs`:

```rust
//! Search → enrich → score pipeline with progress events.

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

        // 2. Apply user filters (price, tld)
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

        // Apply post-enrichment filters
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
```

- [ ] **Step 2: Update `commands.rs` to pass `AppHandle`**

In `src-tauri/src/commands.rs`, change the `search` command. Replace:

```rust
#[tauri::command]
pub async fn search(state: State<'_, AppState>, query: Query) -> Result<Vec<ResultRow>, String> {
    let pipeline = state.pipeline.clone();
    let storage = state.storage.clone();

    let rows = pipeline.run(query).await.map_err(|e| e.to_string())?;
```

With:

```rust
#[tauri::command]
pub async fn search(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    query: Query,
) -> Result<Vec<ResultRow>, String> {
    let pipeline = state.pipeline.clone();
    let storage = state.storage.clone();

    let rows = pipeline.run(query, app).await.map_err(|e| e.to_string())?;
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check`
Expected: compiles with no errors

- [ ] **Step 4: Run all tests**

Run: `cd src-tauri && cargo test --lib`
Expected: all 8+ tests pass

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/pipeline.rs src-tauri/src/commands.rs
git commit -m "feat(pipeline): emit search-progress events through all phases"
```

---

### Task 5: Add frontend types for `SearchProgress` and `CebulaThresholds`

**Files:**
- Modify: `src/lib/types.ts`

- [ ] **Step 1: Add types at the end of `types.ts`**

Add at the bottom of `src/lib/types.ts`, before the `DEFAULT_QUERY` export:

```typescript
export interface SearchProgress {
  phase: "scraping" | "enriching" | "scoring" | "done";
  detail: string;
  current: number;
  total: number | null;
  marketplace: string | null;
}

export interface CebulaThresholds {
  minScore: number;
  maxPrice: number;
  minAge: number;
  minWayback: number;
  noBlacklist: boolean;
  noTrademark: boolean;
}

export const DEFAULT_CEBULA_THRESHOLDS: CebulaThresholds = {
  minScore: 70,
  maxPrice: 300,
  minAge: 3,
  minWayback: 10,
  noBlacklist: true,
  noTrademark: true,
};
```

- [ ] **Step 2: Verify types compile**

Run: `npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add src/lib/types.ts
git commit -m "feat(types): add SearchProgress and CebulaThresholds types"
```

---

### Task 6: Create `ProgressBar` component

**Files:**
- Create: `src/components/ProgressBar.tsx`

- [ ] **Step 1: Create the component file**

Create `src/components/ProgressBar.tsx`:

```tsx
import { useEffect, useState } from "react";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { SearchProgress } from "@/lib/types";
import { cn } from "@/lib/ipc";

const PHASE_LABELS: Record<string, string> = {
  scraping: "Scraping",
  enriching: "Enriching",
  scoring: "Scoring",
  done: "Gotowe",
};

const PHASE_COLORS: Record<string, string> = {
  scraping: "bg-accent",
  enriching: "bg-blue-400",
  scoring: "bg-green-400",
  done: "bg-green-400",
};

export function ProgressBar({ visible }: { visible: boolean }) {
  const [progress, setProgress] = useState<SearchProgress | null>(null);
  const [fading, setFading] = useState(false);

  useEffect(() => {
    let unlisten: UnlistenFn | null = null;

    listen<SearchProgress>("search-progress", (event) => {
      const p = event.payload;
      if (p.phase === "done") {
        setProgress(p);
        setFading(true);
        setTimeout(() => {
          setProgress(null);
          setFading(false);
        }, 800);
      } else {
        setFading(false);
        setProgress(p);
      }
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, []);

  // Reset when a new search starts
  useEffect(() => {
    if (visible) {
      setFading(false);
    }
  }, [visible]);

  if (!visible && !progress) return null;
  if (!progress && visible) {
    return (
      <div className="rounded-md border border-border bg-surface p-4">
        <div className="flex items-center gap-3">
          <div className="h-2 w-2 animate-pulse rounded-full bg-accent" />
          <span className="text-sm text-muted">Inicjowanie wyszukiwania...</span>
        </div>
      </div>
    );
  }
  if (!progress) return null;

  const percentage =
    progress.total && progress.total > 0
      ? Math.round((progress.current / progress.total) * 100)
      : null;

  const phaseLabel = PHASE_LABELS[progress.phase] ?? progress.phase;
  const barColor = PHASE_COLORS[progress.phase] ?? "bg-accent";

  return (
    <div
      className={cn(
        "rounded-md border border-border bg-surface p-4 transition-opacity duration-500",
        fading && "opacity-0",
      )}
    >
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          <div
            className={cn(
              "h-2 w-2 rounded-full",
              progress.phase === "done" ? "bg-green-400" : "animate-pulse bg-accent",
            )}
          />
          <span className="text-sm font-medium text-text">{phaseLabel}</span>
        </div>
        {percentage !== null && (
          <span className="text-xs tabular-nums text-muted">{percentage}%</span>
        )}
      </div>
      <p className="text-xs text-muted mb-2">{progress.detail}</p>
      {percentage !== null && (
        <div className="h-1.5 w-full rounded-full bg-surface-2 overflow-hidden">
          <div
            className={cn("h-full rounded-full transition-all duration-300", barColor)}
            style={{ width: `${percentage}%` }}
          />
        </div>
      )}
    </div>
  );
}
```

- [ ] **Step 2: Verify it compiles**

Run: `npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add src/components/ProgressBar.tsx
git commit -m "feat(ui): add ProgressBar component with Tauri event listener"
```

---

### Task 7: Create `CebulaDeals` component

**Files:**
- Create: `src/components/CebulaDeals.tsx`

- [ ] **Step 1: Create the component file**

Create `src/components/CebulaDeals.tsx`:

```tsx
import { ExternalLink, Star, StarOff } from "lucide-react";
import { open } from "@tauri-apps/plugin-opener";
import type { CebulaThresholds, ResultRow } from "@/lib/types";
import { ScoreBadge } from "@/components/ScoreBadge";
import { cn } from "@/lib/ipc";

const MAX_DEALS = 5;

export function isCebulaDeal(
  row: ResultRow,
  t: CebulaThresholds,
): boolean {
  if (row.score.total < t.minScore) return false;
  const price = row.listing.current_price ?? row.listing.buy_now_price;
  if (price == null || price > t.maxPrice) return false;
  if (t.minAge > 0 && (row.enrichment.age_years ?? 0) < t.minAge) return false;
  if (t.minWayback > 0 && (row.enrichment.wayback_snapshots ?? 0) < t.minWayback)
    return false;
  if (t.noBlacklist && row.enrichment.blacklist_hits > 0) return false;
  if (t.noTrademark && row.enrichment.trademark_warning) return false;
  return true;
}

export function CebulaDeals({
  rows,
  thresholds,
  watchedIds,
  onToggleWatch,
}: {
  rows: ResultRow[];
  thresholds: CebulaThresholds;
  watchedIds: Set<string>;
  onToggleWatch: (row: ResultRow) => void;
}) {
  const deals = rows.filter((r) => isCebulaDeal(r, thresholds)).slice(0, MAX_DEALS);

  if (deals.length === 0) return null;

  return (
    <section className="space-y-3">
      <div className="flex items-center gap-2">
        <span className="text-lg" role="img" aria-label="onion">
          🧅
        </span>
        <h2 className="text-base font-medium text-text">Cebula Deals</h2>
        <span className="rounded-full bg-amber-500/15 px-2 py-0.5 text-xs font-medium text-amber-400">
          {deals.length}
        </span>
      </div>
      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5">
        {deals.map((row) => {
          const watched = watchedIds.has(row.listing.id);
          const price = row.listing.current_price ?? row.listing.buy_now_price;
          return (
            <div
              key={row.listing.id}
              className="rounded-lg border border-amber-500/25 bg-gradient-to-b from-amber-500/5 to-transparent p-4 space-y-2"
            >
              <div className="flex items-start justify-between">
                <span className="text-sm font-semibold text-text truncate flex-1">
                  {row.listing.domain}
                </span>
                <ScoreBadge score={row.score} />
              </div>
              <div className="flex items-center gap-3 text-xs text-muted">
                {price != null && (
                  <span className="font-medium text-amber-400">
                    {price.toLocaleString("pl-PL")} {row.listing.currency}
                  </span>
                )}
                {row.enrichment.age_years != null && (
                  <span>{row.enrichment.age_years.toFixed(1)}y</span>
                )}
                {row.enrichment.wayback_snapshots != null && (
                  <span>WB: {row.enrichment.wayback_snapshots}</span>
                )}
              </div>
              <div className="flex items-center gap-1 pt-1">
                <button
                  onClick={() => onToggleWatch(row)}
                  className={cn(
                    "rounded p-1 text-xs hover:bg-surface-2",
                    watched ? "text-accent" : "text-subtle",
                  )}
                  title={watched ? "Usuń z watchlisty" : "Dodaj do watchlisty"}
                >
                  {watched ? (
                    <Star className="h-3.5 w-3.5 fill-current" />
                  ) : (
                    <StarOff className="h-3.5 w-3.5" />
                  )}
                </button>
                <button
                  onClick={() => open(row.listing.url)}
                  className="rounded p-1 text-xs text-subtle hover:bg-surface-2 hover:text-text"
                  title="Otwórz w przeglądarce"
                >
                  <ExternalLink className="h-3.5 w-3.5" />
                </button>
              </div>
            </div>
          );
        })}
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Verify it compiles**

Run: `npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add src/components/CebulaDeals.tsx
git commit -m "feat(ui): add CebulaDeals component with card layout"
```

---

### Task 8: Integrate ProgressBar and CebulaDeals into Hunt view

**Files:**
- Modify: `src/routes/Hunt.tsx`

- [ ] **Step 1: Add imports and cebula state**

At the top of `src/routes/Hunt.tsx`, add these imports (merge with existing):

```tsx
import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { Download, Save, Sparkles, AlertTriangle } from "lucide-react";
import { load } from "@tauri-apps/plugin-store";
import { DEFAULT_QUERY, DEFAULT_CEBULA_THRESHOLDS, type Query, type ResultRow, type CebulaThresholds } from "@/lib/types";
import { ipc } from "@/lib/ipc";
import { SearchBar } from "@/components/SearchBar";
import { ResultsTable } from "@/components/ResultsTable";
import { ProgressBar } from "@/components/ProgressBar";
import { CebulaDeals } from "@/components/CebulaDeals";
```

- [ ] **Step 2: Add cebula thresholds state inside `HuntView`**

After the existing `useState` declarations in `HuntView`, add:

```tsx
  const [cebulaThresholds, setCebulaThresholds] = useState<CebulaThresholds>({
    ...DEFAULT_CEBULA_THRESHOLDS,
  });

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json");
        const stored = await store.get<CebulaThresholds>("cebula");
        if (stored) setCebulaThresholds(stored);
      } catch {
        // plugin not ready
      }
    })();
  }, []);
```

- [ ] **Step 3: Add ProgressBar and CebulaDeals to the JSX**

In the JSX return, add the `ProgressBar` between the error block and the empty state. Add `CebulaDeals` before "Top rekomendacje".

The full return block should become:

```tsx
    return (
    <div className="mx-auto max-w-[1400px] space-y-10 p-10">
      <header className="space-y-1">
        <h1 className="text-2xl font-semibold tracking-tight text-text">
          Polowanie
        </h1>
        <p className="text-sm text-muted">
          Wpisz frazę lub nazwę. Łączymy aftermarket.pl (oraz premium.pl i
          dropped.pl gdy ich scrapery są włączone), wzbogacamy każdą domenę
          danymi Wayback/WHOIS/blacklist i scorujemy pod wybrany profil.
        </p>
      </header>

      <SearchBar
        value={query}
        onChange={setQuery}
        onSubmit={() => searchM.mutate(query)}
        loading={searchM.isPending}
      />

      <ProgressBar visible={searchM.isPending} />

      {error && (
        <div className="flex items-start gap-2 rounded-md border border-danger/30 bg-danger/10 p-3 text-sm text-danger">
          <AlertTriangle className="mt-0.5 h-4 w-4 flex-shrink-0" />
          <div>{error}</div>
        </div>
      )}

      {rows.length === 0 && !searchM.isPending && !error && <EmptyState />}

      {rows.length > 0 && (
        <>
          <CebulaDeals
            rows={rows}
            thresholds={cebulaThresholds}
            watchedIds={watchedIds}
            onToggleWatch={(r) => toggleWatchM.mutate(r)}
          />

          <section className="space-y-4">
            <div className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                <Sparkles className="h-4 w-4 text-accent" />
                <h2 className="text-base font-medium text-text">
                  Top rekomendacje
                </h2>
                <span className="text-xs text-subtle">
                  ({recommendations.length})
                </span>
              </div>
              <div className="flex items-center gap-2">
                <button
                  onClick={() => saveSearchM.mutate()}
                  className="flex items-center gap-1.5 rounded-sm border border-border bg-surface px-2.5 py-1.5 text-xs text-muted hover:text-text"
                >
                  <Save className="h-3.5 w-3.5" />
                  Zapisz query
                </button>
                <button
                  onClick={exportCsv}
                  className="flex items-center gap-1.5 rounded-sm border border-border bg-surface px-2.5 py-1.5 text-xs text-muted hover:text-text"
                >
                  <Download className="h-3.5 w-3.5" />
                  Eksport CSV
                </button>
              </div>
            </div>
            <ResultsTable
              rows={recommendations}
              watchedIds={watchedIds}
              onToggleWatch={(r) => toggleWatchM.mutate(r)}
              compact
            />
          </section>

          <section className="space-y-4">
            <div className="flex items-center gap-2">
              <h2 className="text-base font-medium text-text">Wszystkie wyniki</h2>
              <span className="text-xs text-subtle">({rows.length})</span>
            </div>
            <ResultsTable
              rows={rows}
              watchedIds={watchedIds}
              onToggleWatch={(r) => toggleWatchM.mutate(r)}
            />
          </section>
        </>
      )}
    </div>
  );
```

- [ ] **Step 4: Verify it compiles**

Run: `npx tsc --noEmit`
Expected: no errors

- [ ] **Step 5: Commit**

```bash
git add src/routes/Hunt.tsx
git commit -m "feat(hunt): integrate ProgressBar and CebulaDeals into search view"
```

---

### Task 9: Add Cebula Deals configuration to Settings

**Files:**
- Modify: `src/routes/Settings.tsx`

- [ ] **Step 1: Add cebula state and persistence**

In `src/routes/Settings.tsx`, add the imports and state. Replace the entire file:

```tsx
import { useEffect, useState } from "react";
import { Key, Check } from "lucide-react";
import { load } from "@tauri-apps/plugin-store";
import type { CebulaThresholds } from "@/lib/types";
import { DEFAULT_CEBULA_THRESHOLDS } from "@/lib/types";

type Provider = "ahrefs" | "majestic" | "moz" | "dataforseo" | "serpapi";

const PROVIDERS: {
  id: Provider;
  label: string;
  hint: string;
}[] = [
  {
    id: "ahrefs",
    label: "Ahrefs API",
    hint: "Domain Rating, backlinks, referring domains",
  },
  {
    id: "majestic",
    label: "Majestic",
    hint: "Trust Flow, Citation Flow, topical trust",
  },
  { id: "moz", label: "Moz Links", hint: "Domain Authority, Page Authority" },
  {
    id: "dataforseo",
    label: "DataForSEO",
    hint: "Keyword volume (PL), CPC, SERP snapshots",
  },
  {
    id: "serpapi",
    label: "SerpApi",
    hint: "Real-time Google SERP queries for relevance",
  },
];

export function SettingsView() {
  const [keys, setKeys] = useState<Record<Provider, string>>({
    ahrefs: "",
    majestic: "",
    moz: "",
    dataforseo: "",
    serpapi: "",
  });
  const [saved, setSaved] = useState<Record<Provider, boolean>>({
    ahrefs: false,
    majestic: false,
    moz: false,
    dataforseo: false,
    serpapi: false,
  });

  const [cebula, setCebula] = useState<CebulaThresholds>({
    ...DEFAULT_CEBULA_THRESHOLDS,
  });
  const [cebulaSaved, setCebulaSaved] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const store = await load("settings.json");
        const next: Record<Provider, string> = { ...keys };
        for (const p of PROVIDERS) {
          const v = (await store.get<string>(`apiKeys.${p.id}`)) ?? "";
          next[p.id] = v;
        }
        setKeys(next);

        const storedCebula = await store.get<CebulaThresholds>("cebula");
        if (storedCebula) setCebula(storedCebula);
      } catch {
        // plugin not ready
      }
    })();
  }, []);

  async function save(provider: Provider) {
    try {
      const store = await load("settings.json");
      await store.set(`apiKeys.${provider}`, keys[provider]);
      await store.save();
      setSaved((s) => ({ ...s, [provider]: true }));
      setTimeout(() => setSaved((s) => ({ ...s, [provider]: false })), 1500);
    } catch (e) {
      console.error(e);
    }
  }

  async function saveCebula() {
    try {
      const store = await load("settings.json");
      await store.set("cebula", cebula);
      await store.save();
      setCebulaSaved(true);
      setTimeout(() => setCebulaSaved(false), 1500);
    } catch (e) {
      console.error(e);
    }
  }

  return (
    <div className="mx-auto max-w-[900px] space-y-8 p-10">
      <header>
        <h1 className="text-2xl font-semibold tracking-tight text-text">
          Settings
        </h1>
        <p className="mt-1 text-sm text-muted">
          Klucze API opcjonalnych dostawców. Bez nich aplikacja używa tylko
          darmowych źródeł (Wayback, WHOIS, blacklisty, heurystyki językowe).
          Klucze są przechowywane lokalnie w pliku <code>settings.json</code>.
        </p>
      </header>

      {/* Cebula Deals */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          🧅 Cebula Deals — Progi
        </h2>
        <p className="text-xs text-muted">
          Skonfiguruj kiedy domena kwalifikuje się jako "Cebula Deal" —
          wyjątkowo dobra oferta w wyjątkowo dobrej cenie.
        </p>
        <div className="grid grid-cols-2 gap-4 rounded-md border border-amber-500/25 bg-surface p-4 lg:grid-cols-3">
          <CebulaField label="Min. score (0-100)">
            <input
              type="number"
              min={0}
              max={100}
              value={cebula.minScore}
              onChange={(e) =>
                setCebula((c) => ({ ...c, minScore: Number(e.target.value) }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Max cena (PLN)">
            <input
              type="number"
              min={0}
              value={cebula.maxPrice}
              onChange={(e) =>
                setCebula((c) => ({ ...c, maxPrice: Number(e.target.value) }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Min. wiek (lata)">
            <input
              type="number"
              min={0}
              value={cebula.minAge}
              onChange={(e) =>
                setCebula((c) => ({ ...c, minAge: Number(e.target.value) }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Min. Wayback snapshots">
            <input
              type="number"
              min={0}
              value={cebula.minWayback}
              onChange={(e) =>
                setCebula((c) => ({
                  ...c,
                  minWayback: Number(e.target.value),
                }))
              }
              className="h-9 w-full rounded-sm border border-border bg-surface-2 px-2 text-sm text-text"
            />
          </CebulaField>
          <CebulaField label="Brak blacklist hits">
            <ToggleSwitch
              checked={cebula.noBlacklist}
              onChange={(v) => setCebula((c) => ({ ...c, noBlacklist: v }))}
            />
          </CebulaField>
          <CebulaField label="Brak trademark warnings">
            <ToggleSwitch
              checked={cebula.noTrademark}
              onChange={(v) => setCebula((c) => ({ ...c, noTrademark: v }))}
            />
          </CebulaField>
        </div>
        <button
          onClick={saveCebula}
          className="flex h-9 items-center gap-1.5 rounded-sm bg-white px-4 text-xs font-medium text-black hover:bg-white/90"
        >
          {cebulaSaved ? (
            <>
              <Check className="h-3.5 w-3.5" /> Saved
            </>
          ) : (
            "Zapisz progi"
          )}
        </button>
      </section>

      {/* API Keys */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          Płatne dostawcy SEO
        </h2>
        {PROVIDERS.map((p) => (
          <div
            key={p.id}
            className="flex items-start gap-3 rounded-md border border-border bg-surface p-4"
          >
            <Key className="mt-1 h-4 w-4 text-subtle" />
            <div className="flex-1 space-y-1.5">
              <div className="flex items-baseline justify-between">
                <label htmlFor={p.id} className="text-sm font-medium text-text">
                  {p.label}
                </label>
                <span className="text-[11px] text-subtle">{p.hint}</span>
              </div>
              <div className="flex gap-2">
                <input
                  id={p.id}
                  type="password"
                  value={keys[p.id]}
                  onChange={(e) =>
                    setKeys((k) => ({ ...k, [p.id]: e.target.value }))
                  }
                  placeholder="API key..."
                  className="h-9 flex-1 rounded-sm border border-border bg-surface-2 px-2 font-mono text-xs text-text placeholder:text-subtle focus:border-white/20 focus:outline-none"
                />
                <button
                  onClick={() => save(p.id)}
                  className="flex h-9 items-center gap-1.5 rounded-sm bg-white px-3 text-xs font-medium text-black hover:bg-white/90"
                >
                  {saved[p.id] ? (
                    <>
                      <Check className="h-3.5 w-3.5" /> Saved
                    </>
                  ) : (
                    "Save"
                  )}
                </button>
              </div>
            </div>
          </div>
        ))}
      </section>

      {/* Scraper status */}
      <section className="space-y-4">
        <h2 className="text-sm font-medium uppercase tracking-wider text-subtle">
          Scrapery
        </h2>
        <div className="rounded-md border border-border bg-surface p-4 text-sm text-muted">
          <p>
            Aktywne: <span className="text-text">aftermarket.pl</span> (pełna
            paginacja)
          </p>
          <p className="mt-1">
            W przygotowaniu:{" "}
            <span className="text-subtle">premium.pl, dropped.pl</span> —
            scrapery zostaną uaktywnione gdy ich selektory zostaną dopięte do
            aktualnego layoutu obu serwisów.
          </p>
        </div>
      </section>
    </div>
  );
}

function CebulaField({
  label,
  children,
}: {
  label: string;
  children: React.ReactNode;
}) {
  return (
    <label className="block space-y-1.5">
      <span className="text-[11px] uppercase tracking-wider text-subtle">
        {label}
      </span>
      {children}
    </label>
  );
}

function ToggleSwitch({
  checked,
  onChange,
}: {
  checked: boolean;
  onChange: (v: boolean) => void;
}) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      onClick={() => onChange(!checked)}
      className={`relative h-6 w-11 rounded-full transition-colors ${
        checked ? "bg-accent" : "bg-surface-2 border border-border"
      }`}
    >
      <span
        className={`absolute top-0.5 left-0.5 h-5 w-5 rounded-full bg-white transition-transform ${
          checked ? "translate-x-5" : ""
        }`}
      />
    </button>
  );
}
```

- [ ] **Step 2: Verify it compiles**

Run: `npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add src/routes/Settings.tsx
git commit -m "feat(settings): add Cebula Deals threshold configuration UI"
```

---

### Task 10: Update CLAUDE.md and final verification

**Files:**
- Modify: `CLAUDE.md`

- [ ] **Step 1: Update "Current State & Stubs" in CLAUDE.md**

In `CLAUDE.md`, update the **Current State & Stubs** section. Under **Active**, change aftermarket_pl line and add new entries:

Replace:
```
- Scraper: `aftermarket_pl` (fixture-tested HTML parser)
```
With:
```
- Scraper: `aftermarket_pl` (fixture-tested, full pagination via `_start=` param, max 50 pages)
- Progress bar: Tauri event-based (`search-progress`), phases: scraping → enriching → scoring → done
- Cebula Deals: configurable thresholds in Settings, displayed on Hunt page
```

- [ ] **Step 2: Run full Rust test suite**

Run: `cd src-tauri && cargo test --lib`
Expected: all tests pass (including new `parses_total_count`)

- [ ] **Step 3: Run frontend type check**

Run: `npx tsc --noEmit`
Expected: no errors

- [ ] **Step 4: Run full build to verify everything links**

Run: `cd src-tauri && cargo check`
Expected: compiles clean

- [ ] **Step 5: Commit**

```bash
git add CLAUDE.md
git commit -m "docs: update CLAUDE.md with pagination, progress bar, and Cebula Deals"
```
