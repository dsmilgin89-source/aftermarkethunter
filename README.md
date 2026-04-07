# Aftermarket Hunter

Desktop tool for hunting profitable expiring domains on Polish aftermarket
sites (aftermarket.pl, premium.pl, dropped.pl). Type a phrase, get a scored
list of auctions with SEO signals and value-for-money recommendations.

## Stack

- **Tauri 2** (Rust backend + React/TS frontend)
- **Rust:** reqwest + scraper + tokio + rusqlite + governor + strsim
- **React:** Vite + Tailwind + TanStack Table + TanStack Query + Lucide icons

## Features

- Search across multiple marketplaces (aftermarket.pl active; premium.pl /
  dropped.pl stubs ready).
- Free enrichers: WHOIS age, Wayback snapshot history, DNSBL blacklist check,
  linguistic brandability, trademark conflict heuristic, per-TLD value table.
- Optional paid enrichers (Ahrefs, Majestic, Moz, DataForSEO, SerpApi) —
  plug in API keys in Settings.
- Scoring engine with three profiles: **SEO Hunter**, **Brand Builder**,
  **Bargain**.
- Two tables: **Top Recommendations** (top 10 by score) and **All Results**
  (full sort + filter + search).
- Hover any score badge → "Why?" breakdown with sub-scores and explanation.
- Watchlist with real-time countdown (urgent < 1h turns red).
- Saved searches with optional background notifications (scheduler stub).
- CSV export.
- SQLite cache (24h enrichment freshness) in `%APPDATA%/AftermarketHunter`.
- Politeness: per-marketplace rate limiter + honest User-Agent + fixture-based
  scraper tests (`cargo test --lib`).

## Run (dev)

```bash
# Prereqs: Rust + Node 20+ + platform WebView (Windows ships it).
npm install
npx tauri dev
```

## Test

```bash
cd src-tauri
cargo test --lib
```

8 unit tests covering scraper parsing, WHOIS date parsing, linguistic
heuristics, trademark guard and scoring edge cases.

## Architecture

```
src-tauri/src/
├── model.rs         Shared domain types
├── http.rs          Shared reqwest client + User-Agent
├── rate_limit.rs    Per-marketplace governor
├── storage.rs       SQLite cache, watchlist, saved searches, scores
├── scrapers/        Marketplace trait + aftermarket.pl / premium.pl / dropped.pl
├── enrichers/       whois, wayback, blacklist, linguistic, tld_value, trademark
├── scoring.rs       SEO + relevance + value - risk (3 profiles)
├── pipeline.rs      Fan-out, dedupe, enrich (buffer 8), score, sort
└── commands.rs      Tauri IPC exposed to React

src/
├── App.tsx          Sidebar + routing
├── routes/          Hunt · Watchlist · SavedSearches · Settings
├── components/      SearchBar · ResultsTable · ScoreBadge · ScoreBreakdown · Countdown
└── lib/             ipc.ts (typed invoke wrappers) · types.ts (mirror of Rust)
```

## Ethics

This is a **personal tool**. The scraper:

- Identifies itself honestly in `User-Agent`
- Rate-limits to ~1 req/s per marketplace
- Caches enrichment for 24h so the same domain is not re-polled
- Does not parallelize against the same host
- Has no bidding or automation of purchases

## Known limitations

- `premium.pl` / `dropped.pl` scrapers are stubs (trait is wired, the HTML
  selectors need to be pinned against a live fetch before enabling).
- `aftermarket.pl` selectors are heuristic; unit test uses a stable fixture so
  a layout change shows up as a failing test — update the constants at the top
  of `scrapers/aftermarket_pl.rs` when that happens.
- Background polling scheduler for saved-search notifications is scaffolded
  but not yet wired into the Tauri runtime loop.
- Paid enrichers (Ahrefs/Majestic/Moz/DataForSEO/SerpApi) have plumbing in
  place — Settings panel, storage, scoring branches — but the actual HTTP
  calls land in a follow-up once you plug in keys.
