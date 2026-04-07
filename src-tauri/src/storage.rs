//! SQLite storage: cache, watchlist, saved searches, scores.
//!
//! Single-writer model: the whole DB sits behind a Tokio Mutex managed by
//! Tauri. Personal-tool scale — we don't need a connection pool.

use std::path::Path;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::model::{Enrichment, LinguisticReport, Listing, Query, SavedSearch, Score, ScoreTier, WatchlistEntry};

pub struct Storage {
    conn: Connection,
}

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS listings (
    id TEXT PRIMARY KEY,
    marketplace TEXT NOT NULL,
    domain TEXT NOT NULL,
    tld TEXT NOT NULL,
    current_price REAL,
    buy_now_price REAL,
    currency TEXT NOT NULL,
    auction_type TEXT NOT NULL,
    ends_at TEXT,
    url TEXT NOT NULL,
    fetched_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_listings_domain ON listings(domain);
CREATE INDEX IF NOT EXISTS idx_listings_ends_at ON listings(ends_at);

CREATE TABLE IF NOT EXISTS enrichment (
    domain TEXT PRIMARY KEY,
    payload TEXT NOT NULL,
    fetched_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS scores (
    listing_id TEXT PRIMARY KEY REFERENCES listings(id) ON DELETE CASCADE,
    payload TEXT NOT NULL,
    computed_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS watchlist (
    listing_id TEXT PRIMARY KEY REFERENCES listings(id) ON DELETE CASCADE,
    max_bid REAL,
    notes TEXT,
    added_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS saved_searches (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    query_json TEXT NOT NULL,
    notify INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);
"#;

impl Storage {
    pub fn open(path: &Path) -> Result<Self> {
        let conn = Connection::open(path).with_context(|| format!("opening {:?}", path))?;
        conn.execute_batch(SCHEMA)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
        Ok(Self { conn })
    }

    // ---------- Listings ----------

    pub fn upsert_listing(&self, l: &Listing) -> Result<()> {
        self.conn.execute(
            r#"INSERT INTO listings (id,marketplace,domain,tld,current_price,buy_now_price,currency,auction_type,ends_at,url,fetched_at)
               VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11)
               ON CONFLICT(id) DO UPDATE SET
                 current_price=excluded.current_price,
                 buy_now_price=excluded.buy_now_price,
                 ends_at=excluded.ends_at,
                 fetched_at=excluded.fetched_at"#,
            params![
                l.id,
                l.marketplace,
                l.domain,
                l.tld,
                l.current_price,
                l.buy_now_price,
                l.currency,
                serde_json::to_string(&l.auction_type)?,
                l.ends_at.map(|t| t.to_rfc3339()),
                l.url,
                l.fetched_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn get_listing(&self, id: &str) -> Result<Option<Listing>> {
        let row = self.conn.query_row(
            "SELECT id,marketplace,domain,tld,current_price,buy_now_price,currency,auction_type,ends_at,url,fetched_at FROM listings WHERE id=?1",
            params![id],
            |r| {
                let auction_type: String = r.get(7)?;
                let ends_at: Option<String> = r.get(8)?;
                let fetched_at: String = r.get(10)?;
                Ok(Listing {
                    id: r.get(0)?,
                    marketplace: r.get(1)?,
                    domain: r.get(2)?,
                    tld: r.get(3)?,
                    current_price: r.get(4)?,
                    buy_now_price: r.get(5)?,
                    currency: r.get(6)?,
                    auction_type: serde_json::from_str(&auction_type).unwrap_or(crate::model::AuctionType::Unknown),
                    ends_at: ends_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                    url: r.get(9)?,
                    fetched_at: DateTime::parse_from_rfc3339(&fetched_at).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
                })
            },
        ).optional()?;
        Ok(row)
    }

    pub fn list_recent_listings(&self, limit: i64) -> Result<Vec<Listing>> {
        let mut stmt = self.conn.prepare(
            "SELECT id,marketplace,domain,tld,current_price,buy_now_price,currency,auction_type,ends_at,url,fetched_at FROM listings ORDER BY fetched_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |r| {
            let auction_type: String = r.get(7)?;
            let ends_at: Option<String> = r.get(8)?;
            let fetched_at: String = r.get(10)?;
            Ok(Listing {
                id: r.get(0)?,
                marketplace: r.get(1)?,
                domain: r.get(2)?,
                tld: r.get(3)?,
                current_price: r.get(4)?,
                buy_now_price: r.get(5)?,
                currency: r.get(6)?,
                auction_type: serde_json::from_str(&auction_type).unwrap_or(crate::model::AuctionType::Unknown),
                ends_at: ends_at.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|d| d.with_timezone(&Utc))),
                url: r.get(9)?,
                fetched_at: DateTime::parse_from_rfc3339(&fetched_at).map(|d| d.with_timezone(&Utc)).unwrap_or_else(|_| Utc::now()),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    // ---------- Enrichment cache (24h) ----------

    pub fn put_enrichment(&self, e: &Enrichment) -> Result<()> {
        let payload = serde_json::to_string(e)?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO enrichment (domain,payload,fetched_at) VALUES (?1,?2,?3)
             ON CONFLICT(domain) DO UPDATE SET payload=excluded.payload, fetched_at=excluded.fetched_at",
            params![e.domain, payload, now],
        )?;
        Ok(())
    }

    pub fn get_fresh_enrichment(&self, domain: &str, max_age_hours: i64) -> Result<Option<Enrichment>> {
        let row: Option<(String, String)> = self
            .conn
            .query_row(
                "SELECT payload,fetched_at FROM enrichment WHERE domain=?1",
                params![domain],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        if let Some((payload, fetched_at)) = row {
            let fetched = DateTime::parse_from_rfc3339(&fetched_at)
                .map(|d| d.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());
            let age = Utc::now().signed_duration_since(fetched);
            if age.num_hours() < max_age_hours {
                let mut e: Enrichment = serde_json::from_str(&payload)?;
                e.fetched_at = Some(fetched);
                return Ok(Some(e));
            }
        }
        Ok(None)
    }

    // ---------- Scores ----------

    pub fn put_score(&self, listing_id: &str, score: &Score) -> Result<()> {
        let payload = serde_json::to_string(score)?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO scores (listing_id,payload,computed_at) VALUES (?1,?2,?3)
             ON CONFLICT(listing_id) DO UPDATE SET payload=excluded.payload, computed_at=excluded.computed_at",
            params![listing_id, payload, now],
        )?;
        Ok(())
    }

    pub fn get_score(&self, listing_id: &str) -> Result<Option<Score>> {
        let row: Option<String> = self
            .conn
            .query_row(
                "SELECT payload FROM scores WHERE listing_id=?1",
                params![listing_id],
                |r| r.get(0),
            )
            .optional()?;
        match row {
            Some(p) => Ok(Some(serde_json::from_str(&p)?)),
            None => Ok(None),
        }
    }

    // ---------- Watchlist ----------

    pub fn add_watch(&self, e: &WatchlistEntry) -> Result<()> {
        self.conn.execute(
            "INSERT INTO watchlist (listing_id,max_bid,notes,added_at) VALUES (?1,?2,?3,?4)
             ON CONFLICT(listing_id) DO UPDATE SET max_bid=excluded.max_bid, notes=excluded.notes",
            params![e.listing_id, e.max_bid, e.notes, e.added_at.to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn remove_watch(&self, listing_id: &str) -> Result<()> {
        self.conn
            .execute("DELETE FROM watchlist WHERE listing_id=?1", params![listing_id])?;
        Ok(())
    }

    pub fn list_watch(&self) -> Result<Vec<WatchlistEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT listing_id,max_bid,notes,added_at FROM watchlist ORDER BY added_at DESC")?;
        let rows = stmt.query_map([], |r| {
            let added_at: String = r.get(3)?;
            Ok(WatchlistEntry {
                listing_id: r.get(0)?,
                max_bid: r.get(1)?,
                notes: r.get(2)?,
                added_at: DateTime::parse_from_rfc3339(&added_at)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    // ---------- Saved searches ----------

    pub fn save_search(&self, name: &str, query: &Query, notify: bool) -> Result<i64> {
        let q = serde_json::to_string(query)?;
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO saved_searches (name,query_json,notify,created_at) VALUES (?1,?2,?3,?4)",
            params![name, q, notify as i32, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn list_saved_searches(&self) -> Result<Vec<SavedSearch>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id,name,query_json,notify,created_at FROM saved_searches ORDER BY created_at DESC")?;
        let rows = stmt.query_map([], |r| {
            let q: String = r.get(2)?;
            let n: i64 = r.get(3)?;
            let created: String = r.get(4)?;
            Ok(SavedSearch {
                id: r.get(0)?,
                name: r.get(1)?,
                query: serde_json::from_str(&q).unwrap_or_default(),
                notify: n != 0,
                created_at: DateTime::parse_from_rfc3339(&created)
                    .map(|d| d.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            })
        })?;
        Ok(rows.collect::<Result<Vec<_>, _>>()?)
    }

    pub fn delete_saved_search(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM saved_searches WHERE id=?1", params![id])?;
        Ok(())
    }
}

#[allow(dead_code)]
fn _assert(_t: ScoreTier, _l: LinguisticReport) {}
