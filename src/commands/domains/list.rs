use cloudflare::endpoints::zones::zone::{ListZones, ListZonesParams, Status, Zone};
use cloudflare::framework::client::async_api::Client;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub async fn list(client: &Client, name: Option<String>) -> anyhow::Result<()> {
    for zone in call_api(client, name).await? {
        println!("{}", &zone.name);
    }
    Ok(())
}

pub async fn call_api(client: &Client, name: Option<String>) -> anyhow::Result<Vec<Zone>> {
    let params = ListZonesParams {
        name,
        direction: None,
        order: None,
        search_match: None,
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
        status: Some(Status::Active),
    };

    client
        .request(&ListZones { params })
        .await
        .map(|r| r.result)
        .map_err(|e| anyhow::anyhow!("{}", e))
}
