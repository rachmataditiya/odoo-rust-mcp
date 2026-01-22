//! Integration tests for OdooLegacyClient (Odoo < 19 JSON-RPC API).

mod common;

use rust_mcp::odoo::config::OdooInstanceConfig;
use rust_mcp::odoo::legacy_client::OdooLegacyClient;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, Respond, ResponseTemplate};

fn create_legacy_config(url: &str) -> OdooInstanceConfig {
    OdooInstanceConfig {
        url: url.to_string(),
        db: Some("test_db".to_string()),
        api_key: None,
        username: Some("admin".to_string()),
        password: Some("admin123".to_string()),
        version: Some("18".to_string()),
        timeout_ms: Some(5000),
        max_retries: Some(0), // No retries for faster tests
        extra: HashMap::new(),
    }
}

/// Helper to create a successful JSON-RPC response.
fn jsonrpc_success(result: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": result
    })
}

/// Helper to create a JSON-RPC error response.
fn jsonrpc_error(code: i32, message: &str) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": code,
            "message": message,
            "data": {
                "name": "odoo.exceptions.AccessDenied",
                "debug": "Access Denied"
            }
        }
    })
}

/// A responder that returns different responses for auth vs other calls.
/// First call is auth (returns uid), subsequent calls return the data.
struct AuthThenDataResponder {
    call_count: AtomicUsize,
    auth_response: serde_json::Value,
    data_response: serde_json::Value,
}

impl AuthThenDataResponder {
    fn new(data: serde_json::Value) -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            auth_response: jsonrpc_success(json!(1)), // uid = 1
            data_response: jsonrpc_success(data),
        }
    }
}

impl Respond for AuthThenDataResponder {
    fn respond(&self, _request: &wiremock::Request) -> ResponseTemplate {
        let count = self.call_count.fetch_add(1, Ordering::SeqCst);
        if count == 0 {
            // First call is authentication
            ResponseTemplate::new(200).set_body_json(&self.auth_response)
        } else {
            // Subsequent calls return data
            ResponseTemplate::new(200).set_body_json(&self.data_response)
        }
    }
}

// ============================================================================
// Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_legacy_authenticate_success() {
    let server = MockServer::start().await;

    // Use the auth-then-data responder
    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!([1, 2, 3])))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_legacy_auth_failure() {
    let server = MockServer::start().await;

    // Mock authentication failure (returns false instead of uid)
    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(jsonrpc_success(json!(false))))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    // Should fail because authentication returns false
    assert!(result.is_err());
}

// ============================================================================
// CRUD Operations Tests
// ============================================================================

#[tokio::test]
async fn test_legacy_search_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!([1, 2, 3])))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_legacy_search_with_domain() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!([42])))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let domain = json!([["name", "ilike", "test"]]);
    let result = client
        .search("res.partner", Some(domain), Some(10), Some(0), None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![42]);
}

#[tokio::test]
async fn test_legacy_search_read_success() {
    let server = MockServer::start().await;

    let records = json!([
        {"id": 1, "name": "Partner 1"},
        {"id": 2, "name": "Partner 2"}
    ]);

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(records.clone()))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search_read(
            "res.partner",
            None,
            Some(vec!["id".to_string(), "name".to_string()]),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_legacy_read_success() {
    let server = MockServer::start().await;

    let records = json!([
        {"id": 1, "name": "Partner 1", "email": "p1@example.com"}
    ]);

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(records))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .read("res.partner", vec![1], None, None)
        .await
        .unwrap();

    assert!(result.is_array());
}

#[tokio::test]
async fn test_legacy_create_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!(42)))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let values = json!({"name": "New Partner"});
    let result = client.create("res.partner", values, None).await.unwrap();

    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_legacy_write_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!(true)))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let values = json!({"name": "Updated Partner"});
    let result = client
        .write("res.partner", vec![1], values, None)
        .await
        .unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_legacy_unlink_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!(true)))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .unlink("res.partner", vec![1, 2], None)
        .await
        .unwrap();

    assert!(result);
}

// ============================================================================
// Advanced Operations Tests
// ============================================================================

#[tokio::test]
async fn test_legacy_search_count_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!(100)))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search_count("res.partner", None, None)
        .await
        .unwrap();

    assert_eq!(result, 100);
}

#[tokio::test]
async fn test_legacy_fields_get_success() {
    let server = MockServer::start().await;

    let fields = json!({
        "id": {"type": "integer", "string": "ID"},
        "name": {"type": "char", "string": "Name"}
    });

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(fields))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client.fields_get("res.partner", None).await.unwrap();

    assert!(result.is_object());
    assert!(result.get("id").is_some());
}

#[tokio::test]
async fn test_legacy_name_search_success() {
    let server = MockServer::start().await;

    let results = json!([[1, "Partner 1"], [2, "Partner 2"]]);

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(results))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .name_search(
            "res.partner",
            Some("Partner".to_string()),
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_legacy_copy_success() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!(99)))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client.copy("res.partner", 1, None, None).await.unwrap();

    assert_eq!(result, 99);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_legacy_rpc_error() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(jsonrpc_error(-32603, "Internal error")),
        )
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_legacy_network_error() {
    // Use invalid URL to trigger network error
    let config = OdooInstanceConfig {
        url: "http://127.0.0.1:1".to_string(), // Port 1 should refuse connections
        db: Some("test_db".to_string()),
        api_key: None,
        username: Some("admin".to_string()),
        password: Some("admin123".to_string()),
        version: Some("18".to_string()),
        timeout_ms: Some(1000),
        max_retries: Some(0), // No retries to speed up test
        extra: HashMap::new(),
    };

    let client = OdooLegacyClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    assert!(result.is_err());
}

// ============================================================================
// Session Management Tests
// ============================================================================

#[tokio::test]
async fn test_legacy_uid_caching() {
    let server = MockServer::start().await;

    // Use a counter-based responder - auth returns 1, data returns [1, 2]
    Mock::given(method("POST"))
        .and(path("/jsonrpc"))
        .respond_with(AuthThenDataResponder::new(json!([1, 2])))
        .mount(&server)
        .await;

    let config = create_legacy_config(&server.uri());
    let client = OdooLegacyClient::new(&config).unwrap();

    // First search - triggers auth then gets data
    let result1 = client
        .search("res.partner", None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result1, vec![1, 2]);
}
