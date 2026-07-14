use cloudflare::endpoints::dns::dns::{UpdateDnsRecord, UpdateDnsRecordParams};
use cloudflare::framework::client::async_api::Client;

use crate::commands;

pub async fn update(
    client: &Client,
    id: &str,
    domain: &str,
    name: &str,
    record_type: &str,
    value: &str,
    proxied: bool,
    ttl: u32,
) -> anyhow::Result<()> {
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

    let record = client
        .request(&UpdateDnsRecord {
            zone_identifier: zone_id,
            identifier: id,
            params,
        })
        .await?
        .result;

    println!(
        "Updated DNS record: {} {} {} (ID: {})",
        record.name, record_type, value, record.id,
    );

    Ok(())
}
