//! Per-marketplace rate limiter.
//!
//! Politeness is non-negotiable: we hit each marketplace at most once per
//! second by default, and we *never* run two requests in parallel against the
//! same host. Personal-tool scale, not commercial scraping.

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Mutex;

type Limiter = RateLimiter<NotKeyed, InMemoryState, DefaultClock>;

static LIMITERS: Lazy<Mutex<HashMap<String, std::sync::Arc<Limiter>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Get (or create) a limiter that allows `per_sec` requests/second for the
/// given key (typically a marketplace id).
pub fn limiter(key: &str, per_sec: u32) -> std::sync::Arc<Limiter> {
    let mut map = LIMITERS.lock().unwrap();
    map.entry(key.to_string())
        .or_insert_with(|| {
            let quota = Quota::per_second(NonZeroU32::new(per_sec.max(1)).unwrap());
            std::sync::Arc::new(RateLimiter::direct(quota))
        })
        .clone()
}

/// Awaits politely until the limiter releases a permit.
pub async fn wait(key: &str, per_sec: u32) {
    let l = limiter(key, per_sec);
    l.until_ready().await;
}
