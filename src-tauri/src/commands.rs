//! Tauri IPC commands. Each command:
//! - takes JSON-serialised arguments from the React frontend
//! - returns serialisable data or an error string (Tauri converts both)
//! - never panics — every fallible call is mapped to `Result<_, String>`

use std::sync::Arc;

use chrono::Utc;
use tauri::State;

use crate::model::{Query, ResultRow, SavedSearch, WatchlistEntry};
use crate::AppState;

// ---------- Search ----------

#[tauri::command]
pub async fn search(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    query: Query,
    openpagerank_key: Option<String>,
) -> Result<Vec<ResultRow>, String> {
    let pipeline = state.pipeline.clone();
    let storage = state.storage.clone();
    let opr_key = openpagerank_key.unwrap_or_default();

    let rows = pipeline.run(query, app, opr_key).await.map_err(|e| e.to_string())?;

    // Persist for the "Recent" view + watchlist resolution.
    {
        let s = storage.lock().await;
        for r in &rows {
            s.upsert_listing(&r.listing).map_err(|e| e.to_string())?;
            s.put_enrichment(&r.enrichment).map_err(|e| e.to_string())?;
            s.put_score(&r.listing.id, &r.score).map_err(|e| e.to_string())?;
        }
    }

    Ok(rows)
}

#[tauri::command]
pub async fn list_recent_results(
    state: State<'_, AppState>,
    limit: Option<i64>,
) -> Result<Vec<crate::model::Listing>, String> {
    let s = state.storage.lock().await;
    s.list_recent_listings(limit.unwrap_or(200)).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_listing(
    state: State<'_, AppState>,
    id: String,
) -> Result<Option<crate::model::Listing>, String> {
    let s = state.storage.lock().await;
    s.get_listing(&id).map_err(|e| e.to_string())
}

// ---------- Watchlist ----------

#[tauri::command]
pub async fn add_to_watchlist(
    state: State<'_, AppState>,
    listing_id: String,
    max_bid: Option<f64>,
    notes: Option<String>,
) -> Result<(), String> {
    let entry = WatchlistEntry {
        listing_id,
        max_bid,
        notes,
        added_at: Utc::now(),
    };
    let s = state.storage.lock().await;
    s.add_watch(&entry).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn remove_from_watchlist(
    state: State<'_, AppState>,
    listing_id: String,
) -> Result<(), String> {
    let s = state.storage.lock().await;
    s.remove_watch(&listing_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_watchlist(
    state: State<'_, AppState>,
) -> Result<Vec<WatchlistEntry>, String> {
    let s = state.storage.lock().await;
    s.list_watch().map_err(|e| e.to_string())
}

// ---------- Saved searches ----------

#[tauri::command]
pub async fn save_search(
    state: State<'_, AppState>,
    name: String,
    query: Query,
    notify: bool,
) -> Result<i64, String> {
    let s = state.storage.lock().await;
    s.save_search(&name, &query, notify).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_saved_searches(
    state: State<'_, AppState>,
) -> Result<Vec<SavedSearch>, String> {
    let s = state.storage.lock().await;
    s.list_saved_searches().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_saved_search(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    let s = state.storage.lock().await;
    s.delete_saved_search(id).map_err(|e| e.to_string())
}

// ---------- API keys (Settings panel) ----------
//
// Stored via tauri-plugin-store on the frontend side; the backend just exposes
// a "do we have this key" status so the UI can show/hide DR/TF columns.
// Actual key-using HTTP enrichers will read these via the same plugin store.

#[tauri::command]
pub fn set_api_key(_provider: String, _value: String) -> Result<(), String> {
    // Frontend writes directly to the secure store; this is a placeholder
    // for future backend-side key consumption.
    Ok(())
}

#[tauri::command]
pub fn get_api_key_status() -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "ahrefs": false,
        "majestic": false,
        "moz": false,
        "dataforseo": false,
        "serpapi": false,
    }))
}

// ---------- Export ----------

#[tauri::command]
pub fn export_results_csv(rows: Vec<ResultRow>) -> Result<String, String> {
    let mut out = String::new();
    out.push_str("domain,tld,score,tier,price,currency,age_years,wayback_snapshots,blacklist_hits,brandability,marketplace,url\n");
    for r in rows {
        let line = format!(
            "{},{},{:.0},{:?},{},{},{},{},{},{:.0},{},{}\n",
            csv_escape(&r.listing.domain),
            csv_escape(&r.listing.tld),
            r.score.total,
            r.score.tier,
            r.listing.current_price.map(|p| format!("{p:.2}")).unwrap_or_default(),
            csv_escape(&r.listing.currency),
            r.enrichment.age_years.map(|a| format!("{a:.1}")).unwrap_or_default(),
            r.enrichment.wayback_snapshots.map(|n| n.to_string()).unwrap_or_default(),
            r.enrichment.blacklist_hits,
            r.enrichment.linguistic.brandability,
            csv_escape(&r.listing.marketplace),
            csv_escape(&r.listing.url),
        );
        out.push_str(&line);
    }
    Ok(out)
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

// Silence dead-code lint for the unused Arc import in some build configs.
#[allow(dead_code)]
fn _keep_arc(_: Arc<()>) {}
