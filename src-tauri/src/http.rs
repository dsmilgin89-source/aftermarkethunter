//! Shared HTTP client. Identifies the app honestly in the User-Agent so
//! marketplaces can contact us if our scraping behaviour becomes a problem.

use once_cell::sync::Lazy;
use reqwest::Client;
use std::time::Duration;

pub static USER_AGENT: &str = concat!(
    "AftermarketHunter/",
    env!("CARGO_PKG_VERSION"),
    " (+personal-tool; respectful-scraper)"
);

pub static CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(7))
        .gzip(true)
        .brotli(true)
        .build()
        .expect("failed to build reqwest client")
});
