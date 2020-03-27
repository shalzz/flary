use crate::commands;

use anyhow::Result;
use cloudflare::endpoints::dns::{DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use cloudflare::framework::async_api::ApiClient;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub async fn list(client: &impl ApiClient, name: &str) -> Result<()> {
    for record in call_api(client, name).await? {
        println!("{:?}", &record);
    }
    Ok(())
}

pub async fn call_api(client: &impl ApiClient, name: &str) -> Result<Vec<DnsRecord>> {
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

    let result: Vec<DnsRecord> = client
        .request(&ListDnsRecords {
            zone_identifier: &zones[0].id,
            params,
        })
        .await?
        .result;
    Ok(result)
}
