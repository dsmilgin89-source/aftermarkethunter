//! Aftermarket Hunter — core library.
//!
//! Layout:
//! - `model`        : domain types shared between scrapers, enrichers and UI
//! - `storage`      : SQLite cache + watchlist + saved searches
//! - `rate_limit`   : per-marketplace politeness limiter
//! - `http`         : shared reqwest client (User-Agent, timeouts)
//! - `scrapers`     : marketplace scrapers (aftermarket.pl)
//! - `enrichers`    : free enrichment (whois, wayback, blacklist, linguistic, openpagerank, similarweb, dns)
//! - `scoring`      : SEO + price + relevance + risk → 0..100 score
//! - `pipeline`     : orchestrates search → enrichment → score
//! - `commands`     : Tauri IPC commands invoked from the React UI

pub mod commands;
pub mod enrichers;
pub mod http;
pub mod model;
pub mod pipeline;
pub mod rate_limit;
pub mod scoring;
pub mod scrapers;
pub mod storage;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pipeline::Pipeline;
use crate::storage::Storage;

pub struct AppState {
    pub storage: Arc<Mutex<Storage>>,
    pub pipeline: Arc<Pipeline>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,aftermarket_hunter_lib=debug".into()),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data dir");
            std::fs::create_dir_all(&app_dir).ok();
            let db_path = app_dir.join("aftermarket-hunter.sqlite");

            let storage = Storage::open(&db_path).expect("failed to open SQLite storage");
            let pipeline = Pipeline::new();

            app.manage(AppState {
                storage: Arc::new(Mutex::new(storage)),
                pipeline: Arc::new(pipeline),
            });

            tracing::info!(?db_path, "Aftermarket Hunter ready");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::search,
            commands::list_recent_results,
            commands::get_listing,
            commands::add_to_watchlist,
            commands::remove_from_watchlist,
            commands::list_watchlist,
            commands::save_search,
            commands::list_saved_searches,
            commands::delete_saved_search,
            commands::set_api_key,
            commands::get_api_key_status,
            commands::export_results_csv,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

use tauri::Manager;
