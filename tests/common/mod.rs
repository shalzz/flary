#![allow(dead_code)]

use cloudflare::framework::auth::Credentials;
use cloudflare::framework::client::async_api::Client;
use cloudflare::framework::client::ClientConfig;
use cloudflare::framework::Environment;
use wiremock::MockServer;

pub async fn create_mock_client(server: &MockServer) -> Client {
    Client::new(
        Credentials::UserAuthToken {
            token: "test-token".to_string(),
        },
        ClientConfig::default(),
        Environment::Custom(server.uri()),
    )
    .unwrap()
}

pub fn zones_response(zone_id: &str, zone_name: &str) -> serde_json::Value {
    serde_json::json!({
        "result": [
            {
                "id": zone_id,
                "name": zone_name,
                "status": "active",
                "name_servers": [],
                "original_name_servers": [],
                "permissions": [],
                "plan": null,
                "plan_pending": null,
                "type": "full",
                "account": {"id": "acc123", "name": "test"},
                "activated_on": "2024-01-01T00:00:00Z",
                "created_on": "2024-01-01T00:00:00Z",
                "modified_on": "2024-01-01T00:00:00Z",
                "development_mode": 0,
                "meta": {
                    "custom_certificate_quota": 0,
                    "page_rule_quota": 0,
                    "phishing_detected": false
                },
                "owner": {"type": "user", "id": "user1", "email": "test@test.com"},
                "paused": false,
                "vanity_name_servers": null,
                "betas": null,
                "deactivation_reason": null,
                "host": null,
                "original_dnshost": null,
                "original_registrar": null
            }
        ],
        "result_info": {
            "page": 1,
            "per_page": 100,
            "total_pages": 1,
            "count": 1,
            "total_count": 1
        },
        "messages": [],
        "errors": [],
        "success": true
    })
}

pub fn empty_zones_response() -> serde_json::Value {
    serde_json::json!({
        "result": [],
        "result_info": {
            "page": 1,
            "per_page": 100,
            "total_pages": 1,
            "count": 0,
            "total_count": 0
        },
        "messages": [],
        "errors": [],
        "success": true
    })
}

pub fn create_dns_record_response(record_id: &str, name: &str, record_type: &str, value: &str) -> serde_json::Value {
    serde_json::json!({
        "result": {
            "id": record_id,
            "name": name,
            "ttl": 1,
            "proxied": false,
            "proxiable": true,
            "created_on": "2024-01-01T00:00:00Z",
            "modified_on": "2024-01-01T00:00:00Z",
            "meta": {},
            "type": record_type,
            "content": value
        },
        "result_info": null,
        "messages": [],
        "errors": [],
        "success": true
    })
}

pub fn delete_dns_record_response(record_id: &str) -> serde_json::Value {
    serde_json::json!({
        "result": {
            "id": record_id
        },
        "result_info": null,
        "messages": [],
        "errors": [],
        "success": true
    })
}

pub fn list_dns_records_response(records: &[serde_json::Value]) -> serde_json::Value {
    serde_json::json!({
        "result": records,
        "result_info": {
            "page": 1,
            "per_page": 100,
            "total_pages": 1,
            "count": records.len(),
            "total_count": records.len()
        },
        "messages": [],
        "errors": [],
        "success": true
    })
}

pub fn dns_record_json(record_id: &str, name: &str, record_type: &str, value: &str) -> serde_json::Value {
    serde_json::json!({
        "id": record_id,
        "name": name,
        "ttl": 1,
        "proxied": false,
        "proxiable": true,
        "created_on": "2024-01-01T00:00:00Z",
        "modified_on": "2024-01-01T00:00:00Z",
        "meta": {},
        "type": record_type,
        "content": value
    })
}

pub fn error_response(code: u16, message: &str) -> serde_json::Value {
    serde_json::json!({
        "success": false,
        "errors": [{"code": code, "message": message}],
        "messages": [],
        "result": null
    })
}
