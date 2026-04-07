//! Blacklist signal — DNS-based lookups against well-known DNSBLs.
//!
//! We resolve `<domain>.<dnsbl>` and count any successful resolution as a hit.
//! `tokio::net::lookup_host` will return an error for NXDOMAIN, which is what
//! "not listed" looks like — so error == clean.

const DNSBLS: &[&str] = &[
    "dbl.spamhaus.org",
    "multi.surbl.org",
    "uribl.com",
];

pub async fn check(domain: &str) -> anyhow::Result<u32> {
    let mut hits = 0u32;
    for bl in DNSBLS {
        let host = format!("{domain}.{bl}");
        // 5s budget per lookup, fail-soft.
        let res = tokio::time::timeout(
            tokio::time::Duration::from_secs(5),
            tokio::net::lookup_host((host.as_str(), 0)),
        )
        .await;
        if let Ok(Ok(mut iter)) = res {
            if iter.next().is_some() {
                hits += 1;
            }
        }
    }
    Ok(hits)
}
