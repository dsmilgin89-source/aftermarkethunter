//! aftermarket.pl scraper.
//!
//! Endpoint: `https://www.aftermarket.pl/Market/List/?domain={phrase}&...`
//! (the older `/domeny/wyszukaj/?szukaj=...` URL just 302s to the homepage).
//!
//! HTML shape (verified against a live fetch on 2026-04-07):
//! ```text
//! <table class="standardTable table toggle-item ...">
//!   <tbody class="data ..." data-id="rowertour.pl">
//!     <tr class="data featured ..." data-id="rowertour.pl">
//!       <td><a class="table-link" href="...redir.php?...id=2677763...">
//!         <div class="text ellipsis"><span class="domain">rowertour.pl</span></div>
//!       </a></td>
//!       <td>...</td>                                  // watch heart
//!       <td>...8...</td>                              // # offers
//!       <td>...325...</td>                            // traffic
//!       <td><strong>319.00 PLN</strong></td>          // price
//!       <td><i class="fa-gavel">...</i></td>          // type icon
//!       <td><strong>2026-04-09 20:00:00</strong></td> // end time (absolute!)
//!       <td>...</td>                                  // chevron
//!     </tr>
//!     <tr class="mobile-extra-row mobile ...">…</tr>  // mobile duplicates — skipped
//!     <tr class="mobile ...">…</tr>
//!   </tbody>
//! </table>
//! ```
//!
//! Selector strategy: `tr.data[data-id]` matches *only* the primary desktop
//! row (mobile rows have no `data-id` attribute), which avoids duplicates.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use regex::Regex;
use scraper::{Html, Selector};

use crate::http::CLIENT;
use crate::model::{AuctionType, Listing, Query};
use crate::rate_limit;
use crate::scrapers::Marketplace;

const SEARCH_URL: &str = "https://www.aftermarket.pl/Market/List/";

// Selectors — pinned constants. Keep them at the top so a layout change is a
// one-line diff and the fixture test fails loudly.
const SEL_ROW: &str = "tr.data[data-id]";
const SEL_DOMAIN: &str = "span.domain";
const SEL_LINK: &str = "a.table-link";
const SEL_STRONG: &str = "strong";
const SEL_GAVEL: &str = "i.fa-gavel";

static PRICE_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?i)([\d\s.,]+)\s*(PLN|EUR|USD|zł)").unwrap());
static DATETIME_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(\d{4}-\d{2}-\d{2}[\sT]\d{2}:\d{2}(?::\d{2})?)").unwrap());
static TOTAL_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"z\s+(?:<[^>]+>\s*)*(\d+)(?:\s*<[^>]+>)*\s+obiekt").unwrap());

const PAGE_SIZE: u32 = 30;
const MAX_PAGES: u32 = 50;

pub struct AftermarketPl;

#[async_trait]
impl Marketplace for AftermarketPl {
    fn id(&self) -> &'static str {
        "aftermarket_pl"
    }
    fn label(&self) -> &'static str {
        "aftermarket.pl"
    }

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

            let start_str = start.to_string();
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
                    ("_start", &start_str),
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

            if total.is_none() {
                total = parse_total(&body);
            }

            let page_listings = parse_listings(&body);
            let page_count = page_listings.len();
            all_listings.extend(page_listings);

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
}

/// Pure parser, separated for fixture testing.
pub fn parse_listings(html: &str) -> Vec<Listing> {
    let doc = Html::parse_document(html);
    let row_sel = match Selector::parse(SEL_ROW) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    let domain_sel = Selector::parse(SEL_DOMAIN).unwrap();
    let link_sel = Selector::parse(SEL_LINK).unwrap();
    let strong_sel = Selector::parse(SEL_STRONG).unwrap();
    let gavel_sel = Selector::parse(SEL_GAVEL).unwrap();

    let now = Utc::now();
    let mut out = Vec::new();

    for row in doc.select(&row_sel) {
        // Domain
        let domain = match row.select(&domain_sel).next() {
            Some(el) => el.text().collect::<String>().trim().to_string(),
            None => row
                .value()
                .attr("data-id")
                .map(|s| s.to_string())
                .unwrap_or_default(),
        };
        if domain.is_empty() || !domain.contains('.') {
            continue;
        }

        // URL — first table-link href in the row.
        let url = row
            .select(&link_sel)
            .next()
            .and_then(|a| a.value().attr("href"))
            .map(absolutize)
            .unwrap_or_else(|| format!("https://www.aftermarket.pl/?domain={domain}"));

        // Walk every <strong> in the row, parse the first one that looks like
        // a price and the first that looks like a datetime.
        let mut current_price: Option<f64> = None;
        let mut currency: String = "PLN".to_string();
        let mut ends_at: Option<DateTime<Utc>> = None;
        for s in row.select(&strong_sel) {
            let text = s.text().collect::<String>();
            let trimmed = text.trim();
            if current_price.is_none() {
                if let Some(cap) = PRICE_RE.captures(trimmed) {
                    if let Some(p) = parse_price(cap.get(1).unwrap().as_str()) {
                        current_price = Some(p);
                        currency = normalize_currency(cap.get(2).unwrap().as_str());
                    }
                }
            }
            if ends_at.is_none() {
                if let Some(cap) = DATETIME_RE.captures(trimmed) {
                    ends_at = parse_datetime(cap.get(1).unwrap().as_str());
                }
            }
        }

        // Auction vs sales offer — gavel icon is the auction marker.
        let is_auction = row.select(&gavel_sel).next().is_some();
        let auction_type = if is_auction {
            AuctionType::Auction
        } else {
            AuctionType::BuyNow
        };

        // Stable id — prefer the auction id from the redir url, else fall back
        // to the domain so we still de-dupe sensibly across runs.
        let id = extract_id_from_url(&url)
            .map(|i| format!("aftermarket_pl:{i}"))
            .unwrap_or_else(|| format!("aftermarket_pl:{domain}"));

        let tld = domain
            .split_once('.')
            .map(|(_, t)| t.to_string())
            .unwrap_or_default();

        out.push(Listing {
            id,
            marketplace: "aftermarket_pl".to_string(),
            domain,
            tld,
            current_price,
            buy_now_price: None,
            currency,
            auction_type,
            ends_at,
            url,
            fetched_at: now,
        });
    }
    out
}

fn absolutize(href: &str) -> String {
    if href.starts_with("http") {
        href.to_string()
    } else if href.starts_with("//") {
        format!("https:{href}")
    } else if href.starts_with('/') {
        format!("https://www.aftermarket.pl{href}")
    } else {
        format!("https://www.aftermarket.pl/{href}")
    }
}

fn extract_id_from_url(url: &str) -> Option<String> {
    // ...&id=2677763&...  (the redir URL has it twice; either is fine)
    static ID_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"[?&]id=(\d+)").unwrap());
    ID_RE.captures(url).map(|c| c[1].to_string())
}

fn normalize_currency(c: &str) -> String {
    match c.trim().to_uppercase().as_str() {
        "ZŁ" | "ZL" => "PLN".to_string(),
        other => other.to_string(),
    }
}

/// "1 234,56" / "319.00" / "12 000" → number.
pub fn parse_price(s: &str) -> Option<f64> {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.')
        .collect();
    if cleaned.is_empty() {
        return None;
    }
    // If both '.' and ',' appear, the LAST one is the decimal separator.
    let normalized = if cleaned.contains('.') && cleaned.contains(',') {
        if cleaned.rfind(',').unwrap() > cleaned.rfind('.').unwrap() {
            cleaned.replace('.', "").replace(',', ".")
        } else {
            cleaned.replace(',', "")
        }
    } else if cleaned.contains(',') {
        // Polish decimal: "1234,56" → "1234.56".
        cleaned.replace(',', ".")
    } else {
        cleaned
    };
    let parts: Vec<&str> = normalized.split('.').collect();
    let value = if parts.len() <= 1 {
        normalized.clone()
    } else {
        let last = parts.last().unwrap();
        if last.len() <= 2 {
            let int_part: String = parts[..parts.len() - 1].concat();
            format!("{int_part}.{last}")
        } else {
            // No real decimals — '.'s were thousand separators.
            parts.concat()
        }
    };
    value.parse::<f64>().ok()
}

/// Extract total listing count from "Pokazuję 1 - 30 z **98** obiektów".
fn parse_total(html: &str) -> Option<u32> {
    TOTAL_RE.captures(html).and_then(|c| c[1].parse().ok())
}

fn parse_datetime(s: &str) -> Option<DateTime<Utc>> {
    let s = s.replace('T', " ");
    for fmt in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%d %H:%M"] {
        if let Ok(naive) = NaiveDateTime::parse_from_str(&s, fmt) {
            // The page renders Europe/Warsaw — close enough; we treat as UTC
            // for countdown purposes. Off by ≤ 2h is fine for ranking.
            return Some(Utc.from_utc_datetime(&naive));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!("../../tests/fixtures/aftermarket_pl_search.html");

    #[test]
    fn parses_fixture() {
        let listings = parse_listings(FIXTURE);
        assert_eq!(
            listings.len(),
            3,
            "expected 3 listings in fixture (selectors may be stale)"
        );

        let rowertour = &listings[0];
        assert_eq!(rowertour.domain, "rowertour.pl");
        assert_eq!(rowertour.current_price, Some(319.0));
        assert_eq!(rowertour.currency, "PLN");
        assert!(matches!(rowertour.auction_type, AuctionType::Auction));
        assert!(rowertour.ends_at.is_some());
        assert_eq!(rowertour.id, "aftermarket_pl:2677763");

        let rower = &listings[1];
        assert_eq!(rower.domain, "rower.pl");
        assert_eq!(rower.current_price, Some(250000.0));
        assert_eq!(rower.currency, "EUR");
        assert!(matches!(rower.auction_type, AuctionType::BuyNow));

        let bike = &listings[2];
        assert_eq!(bike.domain, "bikeportal.pl");
        assert_eq!(bike.current_price, Some(49900.0));
    }

    #[test]
    fn parses_total_count() {
        // Plain text
        assert_eq!(parse_total("z 98 obiektów"), Some(98));
        // With <strong> tags (actual aftermarket.pl HTML)
        assert_eq!(parse_total("z <strong>98</strong> obiektów"), Some(98));
        assert_eq!(parse_total("<div>no pagination</div>"), None);
    }

    #[test]
    fn parses_polish_price() {
        assert_eq!(parse_price("1 234,56"), Some(1234.56));
        assert_eq!(parse_price("319.00"), Some(319.0));
        assert_eq!(parse_price("12 000"), Some(12000.0));
        assert_eq!(parse_price("250000.00"), Some(250000.0));
        assert_eq!(parse_price(""), None);
    }
}
