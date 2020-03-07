use anyhow::Result;
use cloudflare::endpoints::dns::{DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use cloudflare::framework::async_api::ApiClient;
use cloudflare::framework::response::{ApiFailure, ApiSuccess};

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub async fn list(client: &impl ApiClient) -> Result<()> {
    let result = call_api(client).await;

    match result {
        Ok(success) => {
            let records = success.result;
            println!("{:?}", &records);
        }
        Err(e) => println!("{:?}", e),
    }
    Ok(())
}

pub async fn call_api(client: &impl ApiClient) -> Result<ApiSuccess<Vec<DnsRecord>>, ApiFailure> {
    let params = ListDnsRecordsParams {
        name: None,
        direction: None,
        order: None,
        record_type: None,
        search_match: None,
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
    };

    client
        .request(&ListDnsRecords {
            zone_identifier: "",
            params,
        })
        .await
}
