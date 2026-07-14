mod common;

use flary::commands::dns::add::add;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_add_a_record_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_abc", "www.example.com", "A", "1.2.3.4"),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "example.com", "www", "A", "1.2.3.4", false, 1, None).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_add_a_record_proxied() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_abc", "www.example.com", "A", "1.2.3.4"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "example.com", "www", "A", "1.2.3.4", true, 300, None).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_add_mx_record_with_priority() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            serde_json::json!({
                "result": {
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
                },
                "result_info": null,
                "messages": [],
                "errors": [],
                "success": true
            }),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "example.com", "@", "MX", "mail.example.com", false, 1, Some(10)).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_add_cname_record() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_cn", "blog.example.com", "CNAME", "example.com"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "example.com", "blog", "CNAME", "example.com", false, 1, None).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_add_txt_record() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_txt", "example.com", "TXT", "v=spf1 include:_spf.google.com ~all"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "example.com", "@", "TXT", "v=spf1 include:_spf.google.com ~all", false, 1, None).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_add_domain_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::empty_zones_response()))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "nonexistent.com", "www", "A", "1.2.3.4", false, 1, None).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    server.verify().await;
}

#[tokio::test]
async fn test_add_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            &common::error_response(1004, "DNS record already exists"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = add(&client, "example.com", "www", "A", "1.2.3.4", false, 1, None).await;
    assert!(result.is_err());

    server.verify().await;
}
