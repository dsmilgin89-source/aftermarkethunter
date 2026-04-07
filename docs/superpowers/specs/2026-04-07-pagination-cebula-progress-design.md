# Design: Pagination + Cebula Deals + Progress Bar

## Problem

The aftermarket.pl scraper fetches only the first page (30 results) out of potentially hundreds. Users have no visibility into what the pipeline is doing during search. There is no way to surface exceptionally good deals automatically.

## Features

### 1. Full Pagination for aftermarket.pl Scraper

**Mechanism:** aftermarket.pl paginates via `_start=` query param (0, 30, 60, ...) with 30 results per page. The page shows "Pokazuję X - Y z **N** obiektów" revealing the total count.

**Implementation:**

- **`aftermarket_pl.rs` — paginated search loop:**
  - First request: `_start=0`, parse total from HTML regex `z\s+(\d+)\s+obiektów`
  - Loop: increment `_start` by 30, break when `_start >= total`
  - Call `rate_limit::wait()` before each page fetch (1 req/s politeness)
  - Accumulate all `Vec<Listing>` across pages
  - Emit `SearchProgress` event after each page via callback

- **`Marketplace` trait change:**
  - `search()` signature gains `AppHandle` for event emission:
    ```rust
    async fn search(
        &self,
        query: &Query,
        app: &tauri::AppHandle,
    ) -> Result<Vec<Listing>>;
    ```
  - Scrapers emit `SearchProgress` via `app.emit("search-progress", payload)`
  - Existing stubs (premium_pl, dropped_pl) ignore `app` param, return empty

- **HTML total parsing:**
  - Regex: `Pokazuję\s+\d+\s*-\s*\d+\s+z\s+(\d+)\s+obiektów`
  - Fallback: if regex fails, return single-page results (graceful degradation)

- **Safety limits:**
  - Max 50 pages per search (1500 results) — prevents runaway scraping
  - If total > 1500, log warning and stop at page 50

### 2. Interactive Progress Bar (Tauri Events)

**Architecture:** Backend emits `search-progress` events via `tauri::Emitter`. Frontend listens with `@tauri-apps/api/event listen()`.

**Event payload:**
```rust
#[derive(Clone, Serialize)]
struct SearchProgress {
    phase: String,              // "scraping" | "enriching" | "scoring" | "done"
    detail: String,             // human-readable description
    current: u32,               // items processed so far
    total: Option<u32>,         // total items (None if unknown yet)
    marketplace: Option<String>, // marketplace label (scraping phase only)
}
```

**Phase progression:**
1. `scraping` — emitted per page per marketplace: "aftermarket.pl — strona 2/4 (60 znalezionych)"
2. `enriching` — emitted per domain: "Enriching 12/42 domains..."
3. `scoring` — single emit: "Scoring 42 results..."
4. `done` — pipeline complete

**Backend changes:**
- `Pipeline::run()` gains `app_handle: tauri::AppHandle` parameter
- Uses `app_handle.emit("search-progress", payload)` at each stage
- `commands::search()` passes `app.app_handle()` to pipeline
- Enrichment loop: emit after each `buffer_unordered(8)` completion using a counter wrapped in `Arc<AtomicU32>`

**Frontend changes:**
- New component: `src/components/ProgressBar.tsx`
  - Listens to `search-progress` via `listen()` from `@tauri-apps/api/event`
  - Displays: phase label + detail text + progress bar (current/total percentage)
  - Color coding: cyan for scraping, blue for enriching, green for scoring
  - Animated transitions between phases
  - Auto-hides when phase is `done` (after 500ms fade)
- `Hunt.tsx`: render `<ProgressBar />` above results when `searchM.isPending`
- Cleanup: `unlisten()` on component unmount

### 3. Cebula Deals Section

**Definition:** A "Cebula Deal" is a result that passes ALL configured thresholds simultaneously — an exceptionally good domain at an exceptionally good price.

**Configurable thresholds (stored in tauri-plugin-store):**

| Threshold | Key | Default | Type |
|-----------|-----|---------|------|
| Min. total score | `cebula.minScore` | 70 | number (0-100) |
| Max. price (PLN) | `cebula.maxPrice` | 300 | number |
| Min. domain age (years) | `cebula.minAge` | 3 | number |
| Min. Wayback snapshots | `cebula.minWayback` | 10 | number |
| No blacklist hits | `cebula.noBlacklist` | true | boolean |
| No trademark warnings | `cebula.noTrademark` | true | boolean |

**Frontend filtering logic (pure TypeScript, no backend changes):**
```typescript
function isCebulaDeal(row: ResultRow, thresholds: CebulaThresholds): boolean {
  if (row.score.total < thresholds.minScore) return false;
  const price = row.listing.current_price ?? row.listing.buy_now_price;
  if (price == null || price > thresholds.maxPrice) return false;
  if (thresholds.minAge > 0 && (row.enrichment.age_years ?? 0) < thresholds.minAge) return false;
  if (thresholds.minWayback > 0 && (row.enrichment.wayback_snapshots ?? 0) < thresholds.minWayback) return false;
  if (thresholds.noBlacklist && row.enrichment.blacklist_hits > 0) return false;
  if (thresholds.noTrademark && row.enrichment.trademark_warning) return false;
  return true;
}
```

**UI design:**
- Location: above "Top recommendations" on Hunt page, visible only when deals exist
- Header: onion emoji + "Cebula Deals" + count badge
- Style: distinct from regular results — amber/gold accent border, subtle background glow
- Cards (max 5): domain name, score badge, price, age, key metrics — compact horizontal layout
- Each card has watchlist toggle + open URL action
- Collapsible: user can collapse/expand the section
- Empty state: section hidden entirely when no deals match

**Settings UI:**
- New section in `Settings.tsx`: "Cebula Deals — Progi"
- 4 number inputs (score, price, age, wayback) + 2 toggle switches (blacklist, trademark)
- Stored via `tauri-plugin-store` in `settings.json`
- Loaded on Hunt mount, cached in React state

## Files to Modify

### Rust (src-tauri/src/)
| File | Changes |
|------|---------|
| `model.rs` | Add `SearchProgress` struct |
| `scrapers/mod.rs` | Update `Marketplace` trait with progress callback |
| `scrapers/aftermarket_pl.rs` | Pagination loop, total parsing, progress emission |
| `scrapers/premium_pl.rs` | Update trait impl signature |
| `scrapers/dropped_pl.rs` | Update trait impl signature |
| `pipeline.rs` | Accept `AppHandle`, emit progress events per phase |
| `commands.rs` | Pass `AppHandle` to pipeline |
| `lib.rs` | No changes (commands already registered) |

### TypeScript (src/)
| File | Changes |
|------|---------|
| `lib/types.ts` | Add `SearchProgress`, `CebulaThresholds` types |
| `lib/ipc.ts` | No changes (search signature unchanged) |
| `components/ProgressBar.tsx` | New — progress bar with phase display |
| `components/CebulaDeals.tsx` | New — deals section with cards |
| `routes/Hunt.tsx` | Integrate ProgressBar + CebulaDeals |
| `routes/Settings.tsx` | Add Cebula threshold configuration section |

## Verification

1. **Pagination:** Search for "a" on aftermarket.pl (many results) — verify scraper fetches multiple pages, rate-limited at 1/s
2. **Progress bar:** During search, verify phases appear sequentially with correct counts
3. **Cebula Deals:** Set low thresholds (score 40, price 5000), search — verify deals appear. Set high thresholds (score 99, price 1) — verify section hidden
4. **Settings persistence:** Set cebula thresholds, restart app, verify they persist
5. **Graceful degradation:** If total parsing fails, verify single-page results still work
6. **Tests:** `cargo test --lib` — existing tests pass, new pagination parsing test added
