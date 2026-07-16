use cloudflare::endpoints::dns::dns::{DnsRecord, UpdateDnsRecord, UpdateDnsRecordParams};
use cloudflare::framework::client::async_api::Client;

use crate::commands;

pub async fn call_api(
    client: &Client,
    domain: &str,
    id: &str,
    name: &str,
    record_type: &str,
    value: &str,
    proxied: bool,
    ttl: u32,
) -> anyhow::Result<DnsRecord> {
    let zones = commands::domains::list::call_api(client, Some(domain.to_owned())).await?;

    if zones.is_empty() {
        anyhow::bail!("Domain '{}' not found", domain);
    }

    let zone_id = &zones[0].id;
    let content = crate::commands::dns::add::parse_dns_content(record_type, value, None)?;

    let params = UpdateDnsRecordParams {
        ttl: Some(ttl),
        proxied: Some(proxied),
        name,
        content,
    };

    Ok(client
        .request(&UpdateDnsRecord {
            zone_identifier: zone_id,
            identifier: id,
            params,
        })
        .await?
        .result)
}

pub async fn update(
    client: &Client,
    domain: &str,
    id: &str,
    name: &str,
    record_type: &str,
    value: &str,
    proxied: bool,
    ttl: u32,
) -> anyhow::Result<()> {
    let record = call_api(client, domain, id, name, record_type, value, proxied, ttl).await?;

    println!(
        "Updated DNS record: {} {} {} (ID: {})",
        record.name, record_type, value, record.id,
    );

    Ok(())
}
