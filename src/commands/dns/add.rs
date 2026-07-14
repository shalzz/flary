use std::net::Ipv4Addr;
use std::str::FromStr;

use cloudflare::endpoints::dns::dns::{
    CreateDnsRecord, CreateDnsRecordParams, DnsContent,
};
use cloudflare::framework::client::async_api::Client;

use crate::commands;

pub async fn add(
    client: &Client,
    domain: &str,
    name: &str,
    record_type: &str,
    value: &str,
    proxied: bool,
    ttl: u32,
    priority: Option<u16>,
) -> anyhow::Result<()> {
    let zones = commands::domains::list::call_api(client, Some(domain.to_owned())).await?;

    if zones.is_empty() {
        anyhow::bail!("Domain '{}' not found", domain);
    }

    let zone_id = &zones[0].id;
    let content = parse_dns_content(record_type, value, priority)?;

    let params = CreateDnsRecordParams {
        ttl: Some(ttl),
        priority,
        proxied: Some(proxied),
        name,
        content,
    };

    let record = client
        .request(&CreateDnsRecord {
            zone_identifier: zone_id,
            params,
        })
        .await?
        .result;

    println!(
        "Created DNS record: {} {} {} (ID: {})",
        record.name,
        record_type,
        value,
        record.id,
    );

    Ok(())
}

pub fn parse_dns_content(
    record_type: &str,
    value: &str,
    priority: Option<u16>,
) -> anyhow::Result<DnsContent> {
    match record_type.to_uppercase().as_str() {
        "A" => Ok(DnsContent::A {
            content: Ipv4Addr::from_str(value)
                .map_err(|e| anyhow::anyhow!("Invalid IPv4 address '{}': {}", value, e))?,
        }),
        "AAAA" => Ok(DnsContent::AAAA {
            content: std::net::Ipv6Addr::from_str(value)
                .map_err(|e| anyhow::anyhow!("Invalid IPv6 address '{}': {}", value, e))?,
        }),
        "CNAME" => Ok(DnsContent::CNAME {
            content: value.to_string(),
        }),
        "NS" => Ok(DnsContent::NS {
            content: value.to_string(),
        }),
        "MX" => Ok(DnsContent::MX {
            content: value.to_string(),
            priority: priority.unwrap_or(0),
        }),
        "TXT" => Ok(DnsContent::TXT {
            content: value.to_string(),
        }),
        "SRV" => Ok(DnsContent::SRV {
            content: value.to_string(),
        }),
        _ => anyhow::bail!(
            "Unsupported record type '{}'. Supported types: A, AAAA, CNAME, NS, MX, TXT, SRV",
            record_type
        ),
    }
}
