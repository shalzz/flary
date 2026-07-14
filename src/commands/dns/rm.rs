use cloudflare::endpoints::dns::dns::{DeleteDnsRecord, ListDnsRecords, ListDnsRecordsParams};
use cloudflare::framework::client::async_api::Client;

use crate::commands;

pub async fn rm(client: &Client, id: &str, domain: &str, yes: bool) -> anyhow::Result<()> {
    let zones = commands::domains::list::call_api(client, Some(domain.to_owned())).await?;

    if zones.is_empty() {
        anyhow::bail!("Domain '{}' not found", domain);
    }

    let zone_id = &zones[0].id;

    // Look up the record to show what will be deleted
    let params = ListDnsRecordsParams {
        name: None,
        direction: None,
        order: None,
        record_type: None,
        search_match: None,
        page: None,
        per_page: None,
    };

    let records = client
        .request(&ListDnsRecords {
            zone_identifier: zone_id,
            params,
        })
        .await?
        .result;

    let record = records.iter().find(|r| r.id == id);

    let record = match record {
        Some(r) => r,
        None => anyhow::bail!("DNS record with ID '{}' not found", id),
    };

    let record_type = format!("{:?}", record.content);
    let record_type = record_type.split('{').next().unwrap_or(&record_type);

    println!(
        "Record to delete: {} {} (ID: {})",
        record.name, record_type, record.id,
    );

    if !yes {
        eprint!("Are you sure you want to delete this record? [y/N] ");
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    client
        .request(&DeleteDnsRecord {
            zone_identifier: zone_id,
            identifier: id,
        })
        .await?
        .result;

    println!("Deleted DNS record (ID: {})", id);

    Ok(())
}
