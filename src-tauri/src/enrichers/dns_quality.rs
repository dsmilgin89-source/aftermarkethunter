//! DNS quality signals — MX, SPF, DMARC presence.
//!
//! A domain with configured email infrastructure (MX + SPF + DMARC) is likely
//! a real, active business domain — a strong quality signal for SEO scoring.

use std::net::ToSocketAddrs;
use tokio::task;

pub struct DnsQuality {
    pub has_mx: bool,
    pub has_spf: bool,
    pub has_dmarc: bool,
}

impl Default for DnsQuality {
    fn default() -> Self {
        Self {
            has_mx: false,
            has_spf: false,
            has_dmarc: false,
        }
    }
}

pub async fn check(domain: &str) -> DnsQuality {
    let domain = domain.to_string();
    // DNS lookups are blocking — run on blocking thread pool
    task::spawn_blocking(move || check_sync(&domain))
        .await
        .unwrap_or_default()
}

fn check_sync(domain: &str) -> DnsQuality {
    let has_mx = has_any_record(&format!("{domain}:25"));

    let has_spf = check_txt_contains(domain, "v=spf1");

    let dmarc_host = format!("_dmarc.{domain}");
    let has_dmarc = check_txt_contains(&dmarc_host, "v=DMARC1");

    tracing::debug!(domain, has_mx, has_spf, has_dmarc, "DNS quality check");

    DnsQuality {
        has_mx,
        has_spf,
        has_dmarc,
    }
}

fn has_any_record(host_port: &str) -> bool {
    host_port.to_socket_addrs().is_ok()
}

fn check_txt_contains(domain: &str, needle: &str) -> bool {
    // Use DNS TXT lookup via system resolver
    // We check if the domain resolves at all as a proxy for TXT records
    // Full TXT parsing requires a DNS library; for now we use a lightweight check
    use std::process::Command;
    let output = Command::new("nslookup")
        .args(["-type=TXT", domain])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains(needle)
        }
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_all_false() {
        let d = DnsQuality::default();
        assert!(!d.has_mx);
        assert!(!d.has_spf);
        assert!(!d.has_dmarc);
    }
}
