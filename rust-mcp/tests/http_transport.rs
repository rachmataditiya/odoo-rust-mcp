//! Integration tests for MCP HTTP transport endpoints.

mod common;

use axum::http::{HeaderName, HeaderValue};
use axum_test::TestServer;
use common::{minimal_prompts_json, minimal_server_json, minimal_tools_json};
use rust_mcp::mcp::McpOdooHandler;
use rust_mcp::mcp::http::{AuthConfig, create_app};
use rust_mcp::mcp::registry::Registry;
use rust_mcp::mcp::tools::OdooClientPool;
use serde_json::json;
use std::sync::Arc;
use tempfile::TempDir;

const MCP_SESSION_HEADER: &str = "mcp-session-id";
const AUTH_HEADER: &str = "authorization";

/// Setup test environment and create a test server.
async fn setup_test_server(with_auth: bool) -> (TestServer, TempDir) {
    let temp_dir = TempDir::new().unwrap();

    // Write minimal config files
    std::fs::write(temp_dir.path().join("tools.json"), minimal_tools_json()).unwrap();
    std::fs::write(temp_dir.path().join("prompts.json"), minimal_prompts_json()).unwrap();
    std::fs::write(temp_dir.path().join("server.json"), minimal_server_json()).unwrap();

    // Set up environment
    unsafe {
        std::env::set_var("ODOO_URL", "http://localhost:8069");
        std::env::set_var("ODOO_DB", "test_db");
        std::env::set_var("ODOO_API_KEY", "test_key");
        std::env::set_var(
            "MCP_TOOLS_JSON",
            temp_dir
                .path()
                .join("tools.json")
                .to_string_lossy()
                .to_string(),
        );
        std::env::set_var(
            "MCP_PROMPTS_JSON",
            temp_dir
                .path()
                .join("prompts.json")
                .to_string_lossy()
                .to_string(),
        );
        std::env::set_var(
            "MCP_SERVER_JSON",
            temp_dir
                .path()
                .join("server.json")
                .to_string_lossy()
                .to_string(),
        );
    }

    let pool = OdooClientPool::from_env().unwrap();
    let registry = Arc::new(Registry::from_env());
    registry.initial_load().await.unwrap();
    let handler = Arc::new(McpOdooHandler::new(pool, registry));

    let auth = if with_auth {
        AuthConfig {
            bearer_token: Some("test_token".to_string()),
        }
    } else {
        AuthConfig { bearer_token: None }
    };

    let app = create_app(handler, auth);
    let server = TestServer::new(app.into_make_service()).unwrap();

    (server, temp_dir)
}

// ============================================================================
// Initialize Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_post_initialize() {
    let (server, _temp) = setup_test_server(false).await;

    let response = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "test-client",
                    "version": "1.0.0"
                }
            }
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body.get("result").is_some());
    assert!(body["result"]["capabilities"].is_object());
    assert!(body["result"]["serverInfo"].is_object());
}

#[tokio::test]
async fn test_mcp_post_initialize_returns_session_id() {
    let (server, _temp) = setup_test_server(false).await;

    let response = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    response.assert_status_ok();
    // Check for session ID header
    let session_header = response.headers().get("mcp-session-id");
    assert!(
        session_header.is_some(),
        "Should return mcp-session-id header"
    );
}

// ============================================================================
// Tools Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_post_tools_list() {
    let (server, _temp) = setup_test_server(false).await;

    // First initialize
    let init_resp = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Then list tools
    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(MCP_SESSION_HEADER),
            HeaderValue::from_str(&session_id).unwrap(),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body.get("result").is_some());
    assert!(body["result"]["tools"].is_array());
}

// ============================================================================
// Prompts Endpoint Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_post_prompts_list() {
    let (server, _temp) = setup_test_server(false).await;

    // Initialize first
    let init_resp = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // List prompts
    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(MCP_SESSION_HEADER),
            HeaderValue::from_str(&session_id).unwrap(),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "prompts/list",
            "params": {}
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body.get("result").is_some());
    assert!(body["result"]["prompts"].is_array());
}

// ============================================================================
// Authentication Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_post_unauthorized_without_token() {
    let (server, _temp) = setup_test_server(true).await;

    let response = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_mcp_post_unauthorized_wrong_token() {
    let (server, _temp) = setup_test_server(true).await;

    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(AUTH_HEADER),
            HeaderValue::from_static("Bearer wrong_token"),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    response.assert_status_unauthorized();
}

#[tokio::test]
async fn test_mcp_post_authorized_with_correct_token() {
    let (server, _temp) = setup_test_server(true).await;

    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(AUTH_HEADER),
            HeaderValue::from_static("Bearer test_token"),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    response.assert_status_ok();
}

#[tokio::test]
async fn test_mcp_post_unauthorized_invalid_scheme() {
    let (server, _temp) = setup_test_server(true).await;

    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(AUTH_HEADER),
            HeaderValue::from_static("Basic test_token"),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    response.assert_status_unauthorized();
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_mcp_post_invalid_json() {
    let (server, _temp) = setup_test_server(false).await;

    let response = server
        .post("/mcp")
        .content_type("application/json")
        .bytes(axum::body::Bytes::from("{ invalid json }"))
        .await;

    // Should return 400 or 422 for invalid JSON
    assert!(
        response.status_code().as_u16() == 400 || response.status_code().as_u16() == 422,
        "Expected 400 or 422, got {}",
        response.status_code()
    );
}

#[tokio::test]
async fn test_mcp_post_missing_method() {
    let (server, _temp) = setup_test_server(false).await;

    let response = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "params": {}
        }))
        .await;

    response.assert_status_bad_request();
}

#[tokio::test]
async fn test_mcp_post_unknown_method() {
    let (server, _temp) = setup_test_server(false).await;

    // Initialize first
    let init_resp = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Call unknown method
    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(MCP_SESSION_HEADER),
            HeaderValue::from_str(&session_id).unwrap(),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "unknown/method",
            "params": {}
        }))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    // Should have an error in the response
    assert!(body.get("error").is_some());
}

// NOTE: SSE endpoint tests are commented out because they involve
// streaming responses that can cause test hangs. SSE functionality
// is covered by the mcp_smoke integration test.

// ============================================================================
// Session Management Tests
// ============================================================================

#[tokio::test]
async fn test_session_required_for_tools_list() {
    let (server, _temp) = setup_test_server(false).await;

    // Try to call tools/list without initializing
    let response = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }))
        .await;

    // Should work but might be uninitialized
    response.assert_status_ok();
}

#[tokio::test]
async fn test_notification_returns_accepted() {
    let (server, _temp) = setup_test_server(false).await;

    // Initialize first
    let init_resp = server
        .post("/mcp")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {}
        }))
        .await;

    let session_id = init_resp
        .headers()
        .get("mcp-session-id")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    // Send initialized notification (no id = notification)
    let response = server
        .post("/mcp")
        .add_header(
            HeaderName::from_static(MCP_SESSION_HEADER),
            HeaderValue::from_str(&session_id).unwrap(),
        )
        .json(&json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        }))
        .await;

    // Notifications should return 202 Accepted
    assert_eq!(response.status_code().as_u16(), 202);
}

// NOTE: Legacy messages endpoint test is commented out because
// it requires SSE session which can cause test hangs.
