use crate::commands;

use cloudflare::endpoints::dns::dns::{DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use cloudflare::framework::client::async_api::Client;

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

pub async fn call_api(client: &Client, name: &str) -> anyhow::Result<Vec<DnsRecord>> {
    let zones = commands::domains::list::call_api(client, Some(name.to_owned())).await?;

    if zones.is_empty() {
        anyhow::bail!("Domain '{}' not found", name);
    }

    let params = ListDnsRecordsParams {
        name: Some(name.to_owned()),
        direction: None,
        order: None,
        record_type: None,
        search_match: None,
        page: Some(PAGE_NUMBER),
        per_page: Some(MAX_NAMESPACES_PER_PAGE),
    };

    Ok(client
        .request(&ListDnsRecords {
            zone_identifier: &zones[0].id,
            params,
        })
        .await
        .map(|r| r.result)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;
    use std::net::Ipv6Addr;

    #[test]
    fn dns_content_to_string_a() {
        let content = DnsContent::A {
            content: "1.2.3.4".parse::<Ipv4Addr>().unwrap(),
        };
        assert_eq!(dns_content_to_string(&content), "1.2.3.4");
    }

    #[test]
    fn dns_content_to_string_aaaa() {
        let content = DnsContent::AAAA {
            content: "::1".parse::<Ipv6Addr>().unwrap(),
        };
        assert_eq!(dns_content_to_string(&content), "::1");
    }

    #[test]
    fn dns_content_to_string_cname() {
        let content = DnsContent::CNAME {
            content: "example.com".to_string(),
        };
        assert_eq!(dns_content_to_string(&content), "example.com");
    }

    #[test]
    fn dns_content_to_string_ns() {
        let content = DnsContent::NS {
            content: "ns1.example.com".to_string(),
        };
        assert_eq!(dns_content_to_string(&content), "ns1.example.com");
    }

    #[test]
    fn dns_content_to_string_mx() {
        let content = DnsContent::MX {
            content: "mail.example.com".to_string(),
            priority: 10,
        };
        assert_eq!(dns_content_to_string(&content), "10 mail.example.com");
    }

    #[test]
    fn dns_content_to_string_txt() {
        let content = DnsContent::TXT {
            content: "v=spf1 include:_spf.google.com ~all".to_string(),
        };
        assert_eq!(
            dns_content_to_string(&content),
            "v=spf1 include:_spf.google.com ~all"
        );
    }

    #[test]
    fn dns_content_to_string_srv() {
        let content = DnsContent::SRV {
            content: "10 0 5060 sip.example.com".to_string(),
        };
        assert_eq!(dns_content_to_string(&content), "10 0 5060 sip.example.com");
    }
}
