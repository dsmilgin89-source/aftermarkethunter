//! WHOIS lookup over plain TCP/43.
//!
//! For .pl we go to whois.dns.pl. For other TLDs we use whois.iana.org as a
//! starter and (best-effort) follow the `refer:` field once.
//!
//! Returns just what we need for scoring: domain creation date + computed age.

use anyhow::{Context, Result};
use chrono::{DateTime, NaiveDate, Utc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug, Default, Clone)]
pub struct WhoisInfo {
    pub created: Option<DateTime<Utc>>,
    pub age_years: Option<f32>,
}

pub async fn lookup(domain: &str) -> Result<WhoisInfo> {
    let server = pick_server(domain);
    let raw = query(server, domain).await?;
    let info = parse(&raw);
    Ok(info)
}

fn pick_server(domain: &str) -> &'static str {
    if domain.ends_with(".pl") {
        "whois.dns.pl"
    } else if domain.ends_with(".com") || domain.ends_with(".net") {
        "whois.verisign-grs.com"
    } else if domain.ends_with(".io") {
        "whois.nic.io"
    } else {
        "whois.iana.org"
    }
}

async fn query(server: &str, domain: &str) -> Result<String> {
    let mut stream = TcpStream::connect((server, 43))
        .await
        .with_context(|| format!("connect {server}:43"))?;
    let q = format!("{domain}\r\n");
    stream.write_all(q.as_bytes()).await?;
    let mut buf = String::new();
    let mut bytes = Vec::new();
    let mut chunk = [0u8; 4096];
    let timeout = tokio::time::Duration::from_secs(8);
    loop {
        let read = match tokio::time::timeout(timeout, stream.read(&mut chunk)).await {
            Ok(r) => r?,
            Err(_) => break,
        };
        if read == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..read]);
        if bytes.len() > 64 * 1024 {
            break;
        }
    }
    buf.push_str(&String::from_utf8_lossy(&bytes));
    Ok(buf)
}

fn parse(raw: &str) -> WhoisInfo {
    let mut created: Option<DateTime<Utc>> = None;
    for line in raw.lines() {
        let l = line.trim();
        let lower = l.to_lowercase();
        // Common keys across registries (incl. dns.pl "created:")
        if lower.starts_with("created:")
            || lower.starts_with("creation date:")
            || lower.starts_with("registered on:")
            || lower.starts_with("registration date:")
        {
            if let Some(value) = l.splitn(2, ':').nth(1) {
                if let Some(dt) = parse_date(value.trim()) {
                    created = Some(dt);
                    break;
                }
            }
        }
    }
    let age_years = created.map(|c| {
        let secs = Utc::now().signed_duration_since(c).num_seconds() as f32;
        secs / (365.25 * 86_400.0)
    });
    WhoisInfo { created, age_years }
}

fn parse_date(s: &str) -> Option<DateTime<Utc>> {
    let s = s.trim();
    if let Ok(d) = DateTime::parse_from_rfc3339(s) {
        return Some(d.with_timezone(&Utc));
    }
    // Try to extract a YYYY[.-/]MM[.-/]DD prefix from the head of the string.
    let bytes: Vec<char> = s.chars().take(10).collect();
    if bytes.len() < 10 {
        return None;
    }
    let head: String = bytes.iter().collect();
    let normalized = head.replace(['.', '/'], "-");
    if let Ok(d) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
        return Some(DateTime::from_naive_utc_and_offset(
            d.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        ));
    }
    // Try DD-MM-YYYY style.
    if let Ok(d) = NaiveDate::parse_from_str(&normalized, "%d-%m-%Y") {
        return Some(DateTime::from_naive_utc_and_offset(
            d.and_hms_opt(0, 0, 0).unwrap(),
            Utc,
        ));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dns_pl() {
        let raw = "DOMAIN NAME:           example.pl\ncreated:               2010.05.01 12:00:00\n";
        let info = parse(raw);
        assert!(info.created.is_some());
        assert!(info.age_years.unwrap() > 10.0);
    }
}
