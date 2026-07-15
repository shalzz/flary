use crate::commands;

use cloudflare::endpoints::dns::dns::{DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use cloudflare::framework::client::async_api::Client;

const MAX_NAMESPACES_PER_PAGE: u32 = 100;
const PAGE_NUMBER: u32 = 1;

pub fn print_records_to_writer(records: &[DnsRecord], writer: &mut dyn std::io::Write) -> std::io::Result<()> {
    for record in records {
        let record_type = format!("{:?}", record.content);
        let record_type = record_type.split('{').next().unwrap_or(&record_type);
        writeln!(
            writer,
            "{:<10} {:<24} {:<8} {}{}",
            record.id,
            record.name,
            record_type,
            dns_content_to_string(&record.content),
            if record.proxied { " proxied" } else { "" },
        )?;
    }
    Ok(())
}

pub fn print_records(records: &[DnsRecord]) {
    let _ = print_records_to_writer(records, &mut std::io::stdout());
}

pub async fn list(client: &Client, name: &str) -> anyhow::Result<()> {
    print_records(&call_api(client, name).await?);
    Ok(())
}

pub fn dns_content_to_string(content: &DnsContent) -> String {
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
        name: None,
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

    fn a_record(id: &str, name: &str, ip: &str, proxied: bool) -> DnsRecord {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": name,
            "ttl": 1,
            "proxied": proxied,
            "proxiable": true,
            "created_on": "2024-01-01T00:00:00Z",
            "modified_on": "2024-01-01T00:00:00Z",
            "meta": {},
            "type": "A",
            "content": ip,
        })).unwrap()
    }

    fn cname_record(id: &str, name: &str, target: &str, proxied: bool) -> DnsRecord {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": name,
            "ttl": 1,
            "proxied": proxied,
            "proxiable": true,
            "created_on": "2024-01-01T00:00:00Z",
            "modified_on": "2024-01-01T00:00:00Z",
            "meta": {},
            "type": "CNAME",
            "content": target,
        })).unwrap()
    }

    fn mx_record(id: &str, name: &str, target: &str, priority: u16) -> DnsRecord {
        serde_json::from_value(serde_json::json!({
            "id": id,
            "name": name,
            "ttl": 1,
            "proxied": false,
            "proxiable": true,
            "created_on": "2024-01-01T00:00:00Z",
            "modified_on": "2024-01-01T00:00:00Z",
            "meta": {},
            "type": "MX",
            "content": target,
            "priority": priority,
        })).unwrap()
    }

    #[test]
    fn print_records_single_a() {
        let records = vec![a_record("rec123", "www.example.com", "1.2.3.4", false)];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "rec123     www.example.com          A        1.2.3.4\n");
    }

    #[test]
    fn print_records_single_a_proxied() {
        let records = vec![a_record("rec123", "www.example.com", "1.2.3.4", true)];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output,
            "rec123     www.example.com          A        1.2.3.4 proxied\n"
        );
    }

    #[test]
    fn print_records_multiple_types() {
        let records = vec![
            a_record("rec_a", "example.com", "192.168.1.1", false),
            cname_record("rec_cname", "www.example.com", "example.com", false),
        ];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output,
            "rec_a      example.com              A        192.168.1.1\n\
             rec_cname  www.example.com          CNAME    example.com\n"
        );
    }

    #[test]
    fn print_records_multiple_types_mixed_proxy() {
        let records = vec![
            a_record("rec_a", "example.com", "192.168.1.1", false),
            cname_record("rec_cname", "www.example.com", "example.com", true),
        ];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output,
            "rec_a      example.com              A        192.168.1.1\n\
             rec_cname  www.example.com          CNAME    example.com proxied\n"
        );
    }

    #[test]
    fn print_records_mx() {
        let records = vec![mx_record("rec_mx", "example.com", "mail.example.com", 10)];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output,
            "rec_mx     example.com              MX       10 mail.example.com\n"
        );
    }

    #[test]
    fn print_records_empty() {
        let records = vec![];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(output, "");
    }

    #[test]
    fn print_records_long_values() {
        let records = vec![a_record("rec-with-long-id-12345", "a-very-long-subdomain-name.example.com", "255.255.255.255", false)];
        let mut buf = Vec::new();
        print_records_to_writer(&records, &mut buf).unwrap();
        let output = String::from_utf8(buf).unwrap();
        assert_eq!(
            output,
            "rec-with-long-id-12345 a-very-long-subdomain-name.example.com A        255.255.255.255\n"
        );
    }
}
