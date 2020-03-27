use anyhow::Result;
use cloudflare::endpoints::zone::{ListZones, ListZonesParams, Status, Zone};
use cloudflare::framework::async_api::ApiClient;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub async fn list(client: &impl ApiClient, name: Option<String>) -> Result<()> {
    for zone in call_api(client, name).await? {
        println!("{}", &zone.name);
    }
    Ok(())
}

pub async fn call_api(client: &impl ApiClient, name: Option<String>) -> Result<Vec<Zone>> {
    let params = ListZonesParams {
        name,
        direction: None,
        order: None,
        search_match: None,
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
        status: Some(Status::Active),
    };

    let result: Vec<Zone> = client.request(&ListZones { params }).await?.result;
    Ok(result)
}
