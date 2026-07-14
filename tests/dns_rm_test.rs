mod common;

use flary::commands::dns::rm::rm;
use wiremock::matchers::{method, path, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_rm_record_with_yes_flag() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .expect(1)
        .mount(&server)
        .await;

    let records = vec![
        common::dns_record_json("rec_abc", "www.example.com", "A", "1.2.3.4"),
    ];

    Mock::given(method("GET"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::list_dns_records_response(&records)))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path_regex("/zones/zone123/dns_records/rec_abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(
            &common::delete_dns_record_response("rec_abc"),
        ))
        .expect(1)
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = rm(&client, "rec_abc", "example.com", true).await;
    assert!(result.is_ok());

    server.verify().await;
}

#[tokio::test]
async fn test_rm_record_domain_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::empty_zones_response()))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = rm(&client, "rec_abc", "nonexistent.com", true).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    server.verify().await;
}

#[tokio::test]
async fn test_rm_record_not_found_in_zone() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::list_dns_records_response(&[])))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = rm(&client, "nonexistent", "example.com", true).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));

    server.verify().await;
}

#[tokio::test]
async fn test_rm_record_list_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(500).set_body_json(
            &common::error_response(7000, "Internal server error"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = rm(&client, "rec_abc", "example.com", true).await;
    assert!(result.is_err());

    server.verify().await;
}

#[tokio::test]
async fn test_rm_record_delete_api_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/zones"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::zones_response("zone123", "example.com")))
        .mount(&server)
        .await;

    let records = vec![
        common::dns_record_json("rec_abc", "www.example.com", "A", "1.2.3.4"),
    ];

    Mock::given(method("GET"))
        .and(path_regex("/zones/zone123/dns_records"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&common::list_dns_records_response(&records)))
        .mount(&server)
        .await;

    Mock::given(method("DELETE"))
        .and(path_regex("/zones/zone123/dns_records/rec_abc"))
        .respond_with(ResponseTemplate::new(403).set_body_json(
            &common::error_response(9106, "You do not have permission to delete this record"),
        ))
        .mount(&server)
        .await;

    let client = common::create_mock_client(&server).await;

    let result = rm(&client, "rec_abc", "example.com", true).await;
    assert!(result.is_err());

    server.verify().await;
}
