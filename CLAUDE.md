# Aftermarket Hunter — Developer Guide

## Quick Commands

```bash
npm run tauri dev              # Full-stack dev (Vite HMR on :1420 + Rust backend)
npm run tauri build            # Production build (frontend + Rust, optimised binary)
cd src-tauri && cargo test --lib   # Rust unit tests (fixture-based scrapers + scoring)
npx tsc --noEmit               # Frontend type-check only (no emit)
RUST_LOG=debug npm run tauri dev   # Verbose backend logging
```

No frontend test runner is configured. TypeScript strict mode is on (`noUnusedLocals`/`noUnusedParameters` are off).

---

## Architecture

Tauri 2 desktop app. Rust backend does all I/O (scraping, enrichment, storage). React frontend is a thin presentation layer communicating via typed IPC.

**Data flow:**

```
SearchBar → ipc.search() → commands::search → Pipeline::run
  → [scrapers fan-out ‖] → dedupe by domain → [enrichers (buffer 8)] → scoring
  → SQLite persist → Vec<ResultRow>
  → React Query cache → ResultsTable
```

**Key decisions:**
- Single-writer SQLite behind `Arc<Mutex<Storage>>` — personal-tool scale, no connection pool
- Pipeline is stateless; all persistence lives in `Storage`
- Scrapers fail soft — errors logged, empty vec returned, other scrapers continue
- Enrichers fail soft — missing fields shown as "—" in UI
- No router library — `useState<Route>` in `App.tsx` drives view switching
- React Query manages server state; no Redux/Zustand
- Dark theme only (CSS variables in `globals.css`, mapped to Tailwind tokens)

---

## Project Structure

```
src/                          # React/TypeScript frontend
  components/                 #   SearchBar, ResultsTable, ScoreBadge, ScoreBreakdown, Countdown, ProgressBar, CebulaDeals
  routes/                     #   Hunt, Watchlist, SavedSearches, Settings
  lib/
    ipc.ts                    #   Typed Tauri invoke wrappers + cn() utility
    types.ts                  #   TS types mirroring model.rs (hand-maintained)
  styles/globals.css          #   CSS custom properties, dark theme, scrollbar
  App.tsx                     #   Root: sidebar nav + route switch
  main.tsx                    #   Entry: React + QueryClient setup

src-tauri/                    # Rust backend
  src/
    lib.rs                    #   AppState, plugin setup, command registration
    main.rs                   #   Binary entry point
    model.rs                  #   Domain types (Query, Listing, Enrichment, Score, ResultRow)
    commands.rs               #   12 Tauri IPC commands
    pipeline.rs               #   search → enrich → score orchestration
    storage.rs                #   SQLite (WAL, 5 tables)
    scoring.rs                #   3 profiles, sub-scores, risk penalty
    http.rs                   #   Shared reqwest client, honest User-Agent
    rate_limit.rs             #   governor-based per-marketplace limiter
    scrapers/
      mod.rs                  #   Marketplace trait + registry
      aftermarket_pl.rs       #   Active — HTML scraper with fixture test
      premium_pl.rs           #   Stub (returns empty)
      dropped_pl.rs           #   Stub (returns empty)
    enrichers/
      mod.rs                  #   Enricher composition, enrich_free()
      whois.rs                #   TCP/43 WHOIS → domain age
      wayback.rs              #   Wayback CDX API → snapshot count/dates
      blacklist.rs            #   DNSBL lookups (spamhaus, surbl, uribl)
      linguistic.rs           #   Brandability/pronounceability heuristics
      trademark.rs            #   In-memory brand list substring check
      tld_value.rs            #   Static TLD score table (.pl=100, .com=95, ...)
  tests/fixtures/             #   HTML fixtures for scraper tests
```

---

## Type Sync Contract — types.ts ↔ model.rs

**This is the most important invariant.** `src/lib/types.ts` mirrors `src-tauri/src/model.rs` by hand.

Serde conventions to remember:
- Enums use `#[serde(rename_all = "snake_case")]` → TS union types are `"snake_case"` string literals
- `DateTime<Utc>` serialises to RFC 3339 string → TS type is `string`
- `Option<T>` → `T | null` in TypeScript
- `DEFAULT_QUERY` in `types.ts` must match the `Default` impl in `model.rs`

**When changing a data type:**
1. Update struct/enum in `model.rs`
2. Mirror the change in `types.ts`
3. If it affects a command signature, update `ipc.ts`
4. If it's a new field displayed in UI, update the relevant component

---

## IPC Boundary

All frontend→backend calls go through `src/lib/ipc.ts` which wraps `@tauri-apps/api/core invoke()`.

- Tauri auto-converts camelCase JS args to snake_case Rust params (e.g. `listingId` → `listing_id`)
- Rust commands return `Result<T, String>` — `anyhow` errors are `.to_string()`'d at the boundary
- API keys are stored via `tauri-plugin-store` on the frontend; `set_api_key` command is a placeholder

**Adding a new command:**
1. Add `pub async fn` in `commands.rs` with `#[tauri::command]`
2. Register in `lib.rs` → `tauri::generate_handler![..., commands::new_cmd]`
3. Add typed wrapper in `ipc.ts`

---

## Extension Recipes

### Adding a Scraper

1. Create `src-tauri/src/scrapers/{name}.rs` implementing `Marketplace` trait
2. Add `pub mod {name};` to `scrapers/mod.rs`
3. Add `Box::new({Name})` to `registry()` in `scrapers/mod.rs`
4. Call `rate_limit::wait(self.id(), self.rps())` before any HTTP request
5. Use `crate::http::CLIENT` for requests (shared User-Agent + timeouts)
6. Keep CSS selectors as `const` at module top (not inline strings)
7. Separate `parse_listings(html)` from HTTP fetch for fixture testing
8. Add fixture HTML in `tests/fixtures/` with `#[test]` using `include_str!`

### Adding an Enricher

1. Create `src-tauri/src/enrichers/{name}.rs`
2. Add `pub mod {name};` to `enrichers/mod.rs`
3. If free: wire into `enrich_free()` (add to the `tokio::join!`)
4. If paid: will need API key check (pattern not yet established)
5. Add fields to `Enrichment` in `model.rs` → mirror in `types.ts`
6. Enrichers must never panic — return `Result`, let `enrich_free` swallow errors

---

## Scoring System

Three profiles with different weight tuples:

| Profile | SEO | Relevance | Value |
|---------|-----|-----------|-------|
| SeoHunter | 0.55 | 0.15 | 0.30 |
| BrandBuilder | 0.15 | 0.45 | 0.40 |
| Bargain | 0.30 | 0.10 | 0.60 |

**Sub-scores** (each 0–100):
- **SEO**: domain age (sigmoid, center=5y) + Wayback snapshots (log scale) + DR/TF if available
- **Relevance**: Jaro-Winkler name similarity (65%) + brandability (35%)
- **Value**: `log(estimated_worth / current_price)` scaled — `estimate_worth` is a hand-calibrated heuristic (base 80 PLN + age/wayback/brand/DR bonuses × TLD factor)
- **Risk penalty** (0–60 cap): blacklist hits, trademark, length >18, hyphens

**Formula**: `total = (w.seo×seo + w.rel×rel + w.val×val - risk).clamp(0, 100)`

**Tiers**: Excellent ≥80, Good ≥60, Fair ≥40, Poor <40

---

## Frontend Patterns

- **Path alias**: `@/*` → `./src/*` (configured in `tsconfig.json` + `vite.config.ts`)
- **Class merging**: `cn()` in `src/lib/ipc.ts` — simple `filter(Boolean).join(" ")`
- **Theming**: CSS custom properties in `globals.css` → Tailwind tokens in `tailwind.config.ts`
- **Icons**: `lucide-react` exclusively, imported individually
- **Routing**: sidebar nav array in `App.tsx`; adding a view = add to `Route` union type + `NAV` array + render branch
- **Server state**: React Query (`staleTime: 60s`, `retry: 1`, `refetchOnWindowFocus: false`)
- **Persistent settings**: `tauri-plugin-store` → `settings.json` file
- **Tables**: TanStack Table v8 with sorting + global filter
- **Dates from Rust**: arrive as RFC 3339 strings; use `date-fns` for formatting
- **Fonts**: Inter (sans), JetBrains Mono (mono)

---

## Coding Conventions

**Rust:**
- Module per file, `snake_case` naming
- `anyhow::Result` internally; `Result<T, String>` at IPC boundary
- No `.unwrap()` in command handlers — always `.map_err(|e| e.to_string())?`
- Scraper selectors as `const` at module top
- Rate limiting mandatory before any marketplace HTTP request
- `tracing::info/warn/debug` for logging (not `println!`)
- Release profile: `opt-level = "s"`, LTO, strip, `panic = "abort"`

**TypeScript/React:**
- Components: PascalCase `.tsx`, one component per file
- Routes in `src/routes/`, reusable components in `src/components/`
- Utilities and types in `src/lib/`
- Tailwind utility classes; no CSS modules or CSS-in-JS
- No unused imports/variables enforcement is off — but keep code clean

**General:**
- SQLite DB at `$APPDATA/aftermarket-hunter.sqlite`
- Enrichment cache TTL: 24 hours
- Rate limit default: 1 req/s per marketplace
- User-Agent: `"AftermarketHunter/{VERSION} (+personal-tool; respectful-scraper)"`

---

## Current State & Stubs

**Active:**
- Scraper: `aftermarket_pl` (fixture-tested, full pagination via `_start=` param, max 50 pages)
- Progress bar: Tauri event-based (`search-progress`), phases: scraping → enriching → scoring → done
- Cebula Deals: configurable thresholds in Settings, displayed on Hunt page above results
- Enrichers: whois, wayback, blacklist, linguistic, trademark, tld_value
- Views: Hunt, Watchlist, SavedSearches, Settings
- Storage: full SQLite schema (listings, enrichment, scores, watchlist, saved_searches)
- CSV export

**Stubs / Placeholders:**
- Scrapers: `premium_pl`, `dropped_pl` (trait wired, return empty)
- Paid enrichers: Ahrefs, Majestic, Moz, DataForSEO, SerpApi (fields in model, no HTTP calls)
- `set_api_key` command (no-op; keys stored via plugin-store on frontend)
- `get_api_key_status` (hardcoded all `false`)
- Background saved-search scheduler / notifications

---

## Staleness Checklist

> When modifying the project, check if any of these apply and update the relevant section above:
>
> - **Added/removed a Tauri command?** → Update "IPC Boundary" section, verify `lib.rs` `generate_handler!`
> - **Changed a field in `model.rs`?** → Update `types.ts`, note in "Type Sync Contract"
> - **Added a scraper or enricher?** → Update "Current State & Stubs", verify "Extension Recipes" still accurate
> - **Changed scoring weights or formula?** → Update "Scoring System" table and formula
> - **Changed dev/build commands?** → Update "Quick Commands"
> - **Enabled a stub?** → Move from "Stubs" to "Active" in "Current State"
> - **Added a frontend route?** → Update "Frontend Patterns" routing note
> - **Changed theme/styling approach?** → Update "Frontend Patterns"
