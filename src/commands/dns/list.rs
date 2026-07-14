use crate::commands;

use cloudflare::endpoints::dns::dns::{DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use cloudflare::framework::client::async_api::Client;
use cloudflare::framework::response::ApiFailure;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub async fn list(client: &Client, name: &str) -> anyhow::Result<()> {
    for record in call_api(client, name).await? {
        let record_type = format!("{:?}", record.content);
        let record_type = record_type.split('{').next().unwrap_or(&record_type);
        println!(
            "{:<10} {:<24} {:<8} {}",
            record.id,
            record.name,
            record_type,
            dns_content_to_string(&record.content),
        );
    }
    Ok(())
}

fn dns_content_to_string(content: &DnsContent) -> String {
    match content {
        DnsContent::A { content } => content.to_string(),
        DnsContent::AAAA { content } => content.to_string(),
        DnsContent::CNAME { content } => content.clone(),
        DnsContent::NS { content } => content.clone(),
        DnsContent::MX { content, priority } => format!("{} {}", priority, content),
        DnsContent::TXT { content } => content.clone(),
        DnsContent::SRV { content } => content.clone(),
    }
}

pub async fn call_api(client: &Client, name: &str) -> Result<Vec<DnsRecord>, ApiFailure> {
    let zones = commands::domains::list::call_api(client, Some(name.to_owned())).await?;

    let params = ListDnsRecordsParams {
        name: Some(name.to_owned()),
        direction: None,
        order: None,
        record_type: None,
        search_match: None,
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
    };

    client
        .request(&ListDnsRecords {
            zone_identifier: &zones[0].id,
            params,
        })
        .await
        .map(|r| r.result)
}
