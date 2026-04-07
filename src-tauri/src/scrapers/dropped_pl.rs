//! dropped.pl scraper. Stub for now (see premium_pl for rationale).

use anyhow::Result;
use async_trait::async_trait;

use crate::model::{Listing, Query};
use crate::scrapers::Marketplace;

pub struct DroppedPl;

#[async_trait]
impl Marketplace for DroppedPl {
    fn id(&self) -> &'static str {
        "dropped_pl"
    }
    fn label(&self) -> &'static str {
        "dropped.pl"
    }

    async fn search(&self, _query: &Query) -> Result<Vec<Listing>> {
        Ok(Vec::new())
    }
}
