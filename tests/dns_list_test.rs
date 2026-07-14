mod common;

use flary::commands::dns::list::list;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_list_dns_records_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .expect(1)
        .mount(&server)
        .await;

    let records = vec![
        common::dns_record_json("rec_a", "www.example.com", "A", "1.2.3.4"),
        common::dns_record_json("rec_cname", "blog.example.com", "CNAME", "example.com"),
    ];

    Mock::given(method("GET"))
        .and(path("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::list_dns_records_response(&records)))
        .expect(1)
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = list(&client, "example.com").await;
    match &result {
        Ok(()) => println!("LIST OK"),
        Err(e) => println!("LIST ERROR: {:?}\nDisplay: {}", e, e),
    }
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_list_dns_records_empty() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::list_dns_records_response(&[])))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = list(&client, "example.com").await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_list_dns_records_mx_record() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    let records = vec![
        serde_json::json!({
            "id": "rec_mx",
            "name": "example.com",
            "ttl": 1,
            "proxied": false,
            "proxiable": true,
            "created_on": "2024-01-01T00:00:00Z",
            "modified_on": "2024-01-01T00:00:00Z",
            "meta": {},
            "type": "MX",
            "content": "mail.example.com",
            "priority": 10
        }),
    ];

    Mock::given(method("GET"))
        .and(path("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::list_dns_records_response(&records)))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = list(&client, "example.com").await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_list_dns_records_domain_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::empty_zones_response()))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = list(&client, "nonexistent.com").await;
    assert!(result.is_err());

    server.verify().await;
}

#[tokio::test]
async fn test_list_dns_records_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(500).set_body_json(
            &common::error_response(7000, "Internal server error"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = list(&client, "example.com").await;
    assert!(result.is_err());

    server.verify().await;
}
