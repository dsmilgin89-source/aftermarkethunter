//! premium.pl scraper. Stub: returns empty until selectors are reverse-engineered
//! against a real fetch. The rest of the pipeline still works for aftermarket.pl.

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
        // TODO: implement HTML scraping for https://premium.pl once selectors stabilised.
        // Skeleton lives here so the trait registry stays complete and the UI can
        // already toggle the source on/off.
        Ok(Vec::new())
    }
}
