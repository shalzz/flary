use anyhow::Result;
use cloudflare::endpoints::zone::{ListZones, ListZonesParams, Status, Zone};
use cloudflare::framework::async_api::ApiClient;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub async fn list(client: &impl ApiClient, name: Option<String>) -> Result<()> {
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

    for zone in result {
        println!("{}", &zone.name);
    }
    Ok(())
}
