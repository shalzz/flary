use std::net::Ipv4Addr;
use std::str::FromStr;

use cloudflare::endpoints::dns::dns::{
    CreateDnsRecord, CreateDnsRecordParams, DnsContent, DnsRecord,
};
use cloudflare::framework::client::async_api::Client;

use crate::commands;

pub async fn call_api(
    client: &Client,
    domain: &str,
    name: &str,
    record_type: &str,
    value: &str,
    proxied: bool,
    ttl: u32,
    priority: Option<u16>,
) -> anyhow::Result<DnsRecord> {
    let zones = commands::domains::list::call_api(client, Some(domain.to_owned())).await?;

    if zones.is_empty() {
        anyhow::bail!("Domain '{}' not found", domain);
    }

    let zone_id = &zones[0].id;
    let content = parse_dns_content(record_type, value, priority)?;

    let params = CreateDnsRecordParams {
        ttl: Some(ttl),
        priority,
        proxied: Some(proxied),
        name,
        content,
    };

    Ok(client
        .request(&CreateDnsRecord {
            zone_identifier: zone_id,
            params,
        })
        .await?
        .result)
}

pub async fn add(
    client: &Client,
    domain: &str,
    name: &str,
    record_type: &str,
    value: &str,
    proxied: bool,
    ttl: u32,
    priority: Option<u16>,
) -> anyhow::Result<()> {
    let record = call_api(client, domain, name, record_type, value, proxied, ttl, priority).await?;

    println!(
        "Created DNS record: {} {} {} (ID: {})",
        record.name,
        record_type,
        value,
        record.id,
    );

    Ok(())
}

pub fn parse_dns_content(
    record_type: &str,
    value: &str,
    priority: Option<u16>,
) -> anyhow::Result<DnsContent> {
    match record_type.to_uppercase().as_str() {
        "A" => Ok(DnsContent::A {
            content: Ipv4Addr::from_str(value)
                .map_err(|e| anyhow::anyhow!("Invalid IPv4 address '{}': {}", value, e))?,
        }),
        "AAAA" => Ok(DnsContent::AAAA {
            content: std::net::Ipv6Addr::from_str(value)
                .map_err(|e| anyhow::anyhow!("Invalid IPv6 address '{}': {}", value, e))?,
        }),
        "CNAME" => Ok(DnsContent::CNAME {
            content: value.to_string(),
        }),
        "NS" => Ok(DnsContent::NS {
            content: value.to_string(),
        }),
        "MX" => Ok(DnsContent::MX {
            content: value.to_string(),
            priority: priority.unwrap_or(0),
        }),
        "TXT" => Ok(DnsContent::TXT {
            content: value.to_string(),
        }),
        "SRV" => Ok(DnsContent::SRV {
            content: value.to_string(),
        }),
        _ => anyhow::bail!(
            "Unsupported record type '{}'. Supported types: A, AAAA, CNAME, NS, MX, TXT, SRV",
            record_type
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_a_record() {
        let result = parse_dns_content("A", "1.2.3.4", None).unwrap();
        match result {
            DnsContent::A { content } => assert_eq!(content, "1.2.3.4".parse::<Ipv4Addr>().unwrap()),
            _ => panic!("Expected A record"),
        }
    }

    #[test]
    fn parse_a_record_case_insensitive() {
        let result = parse_dns_content("a", "10.0.0.1", None).unwrap();
        match result {
            DnsContent::A { content } => assert_eq!(content, "10.0.0.1".parse::<Ipv4Addr>().unwrap()),
            _ => panic!("Expected A record"),
        }
    }

    #[test]
    fn parse_a_record_invalid_ip() {
        assert!(parse_dns_content("A", "not-an-ip", None).is_err());
    }

    #[test]
    fn parse_a_record_partial_ip() {
        assert!(parse_dns_content("A", "1.2.3", None).is_err());
    }

    #[test]
    fn parse_aaaa_record() {
        let result = parse_dns_content("AAAA", "::1", None).unwrap();
        match result {
            DnsContent::AAAA { content } => {
                assert_eq!(content, "::1".parse::<std::net::Ipv6Addr>().unwrap())
            }
            _ => panic!("Expected AAAA record"),
        }
    }

    #[test]
    fn parse_aaaa_record_full() {
        let result = parse_dns_content("AAAA", "2001:0db8:85a3:0000:0000:8a2e:0370:7334", None)
            .unwrap();
        match result {
            DnsContent::AAAA { content } => assert_eq!(
                content,
                "2001:0db8:85a3:0000:0000:8a2e:0370:7334"
                    .parse::<std::net::Ipv6Addr>()
                    .unwrap()
            ),
            _ => panic!("Expected AAAA record"),
        }
    }

    #[test]
    fn parse_aaaa_record_invalid() {
        assert!(parse_dns_content("AAAA", "not-ipv6", None).is_err());
    }

    #[test]
    fn parse_cname_record() {
        let result = parse_dns_content("CNAME", "example.com", None).unwrap();
        match result {
            DnsContent::CNAME { content } => assert_eq!(content, "example.com"),
            _ => panic!("Expected CNAME record"),
        }
    }

    #[test]
    fn parse_ns_record() {
        let result = parse_dns_content("NS", "ns1.example.com", None).unwrap();
        match result {
            DnsContent::NS { content } => assert_eq!(content, "ns1.example.com"),
            _ => panic!("Expected NS record"),
        }
    }

    #[test]
    fn parse_mx_record_with_priority() {
        let result = parse_dns_content("MX", "mail.example.com", Some(10)).unwrap();
        match result {
            DnsContent::MX { content, priority } => {
                assert_eq!(content, "mail.example.com");
                assert_eq!(priority, 10);
            }
            _ => panic!("Expected MX record"),
        }
    }

    #[test]
    fn parse_mx_record_without_priority() {
        let result = parse_dns_content("MX", "mail.example.com", None).unwrap();
        match result {
            DnsContent::MX { content, priority } => {
                assert_eq!(content, "mail.example.com");
                assert_eq!(priority, 0);
            }
            _ => panic!("Expected MX record"),
        }
    }

    #[test]
    fn parse_txt_record() {
        let result = parse_dns_content("TXT", "v=spf1 include:_spf.google.com ~all", None).unwrap();
        match result {
            DnsContent::TXT { content } => {
                assert_eq!(content, "v=spf1 include:_spf.google.com ~all")
            }
            _ => panic!("Expected TXT record"),
        }
    }

    #[test]
    fn parse_srv_record() {
        let result = parse_dns_content("SRV", "10 0 5060 sip.example.com", None).unwrap();
        match result {
            DnsContent::SRV { content } => assert_eq!(content, "10 0 5060 sip.example.com"),
            _ => panic!("Expected SRV record"),
        }
    }

    #[test]
    fn parse_unsupported_type() {
        let err = parse_dns_content("PTR", "1.2.3.4", None).unwrap_err();
        assert!(err.to_string().contains("Unsupported record type"));
    }

    #[test]
    fn parse_unsupported_type_caa() {
        assert!(parse_dns_content("CAA", "0 issue \"letsencrypt.org\"", None).is_err());
    }

    #[test]
    fn parse_case_insensitive_types() {
        assert!(parse_dns_content("a", "1.2.3.4", None).is_ok());
        assert!(parse_dns_content("AaAa", "::1", None).is_ok());
        assert!(parse_dns_content("cname", "example.com", None).is_ok());
        assert!(parse_dns_content("Mx", "mail.example.com", None).is_ok());
    }
}
