mod common;

use flary::commands::dns::update::update;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_update_a_record_success() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex("/zones/zone123/dns_records/rec_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_abc", "www.example.com", "A", "5.6.7.8"),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = update(&client, "example.com", "rec_abc", "www", "A", "5.6.7.8", false, 1).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_update_a_record_proxied() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex("/zones/zone123/dns_records/rec_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_abc", "www.example.com", "A", "5.6.7.8"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = update(&client, "example.com", "rec_abc", "www", "A", "5.6.7.8", true, 300).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_update_cname_record() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex("/zones/zone123/dns_records/rec_cn"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::create_dns_record_response("rec_cn", "blog.example.com", "CNAME", "newtarget.example.com"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = update(&client, "example.com", "rec_cn", "blog", "CNAME", "newtarget.example.com", false, 1).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_update_domain_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::empty_zones_response()))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = update(&client, "nonexistent.com", "rec_abc", "www", "A", "1.2.3.4", false, 1).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    server.verify().await;
}

#[tokio::test]
async fn test_update_record_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex("/zones/zone123/dns_records/nonexistent"))
        .respond_with(ResponseTemplate::new(404).set_body_json(
            &common::error_response(7003, "Record does not exist"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = update(&client, "example.com", "nonexistent", "www", "A", "1.2.3.4", false, 1).await;
    assert!(result.is_err());

    server.verify().await;
}

#[tokio::test]
async fn test_update_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("PUT"))
        .and(path_regex("/zones/zone123/dns_records/rec_abc"))
        .respond_with(ResponseTemplate::new(400).set_body_json(
            &common::error_response(1004, "Invalid record type"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = update(&client, "example.com", "rec_abc", "www", "INVALID", "1.2.3.4", false, 1).await;
    assert!(result.is_err());

    server.verify().await;
}
