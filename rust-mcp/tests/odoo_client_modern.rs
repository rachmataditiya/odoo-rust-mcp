//! Integration tests for OdooHttpClient (Odoo 19+ JSON-2 API).

mod common;

use common::{MockOdooServer, responses};
use rust_mcp::odoo::client::OdooHttpClient;
use rust_mcp::odoo::config::OdooInstanceConfig;
use serde_json::json;
use std::collections::HashMap;
use wiremock::matchers::{header, method, path_regex};
use wiremock::{Mock, ResponseTemplate};

fn create_config(url: &str) -> OdooInstanceConfig {
    OdooInstanceConfig {
        url: url.to_string(),
        db: Some("test_db".to_string()),
        api_key: Some("test_api_key".to_string()),
        username: None,
        password: None,
        version: Some("19".to_string()),
        timeout_ms: Some(5000),
        max_retries: Some(3),
        extra: HashMap::new(),
    }
}

// ============================================================================
// CRUD Operations Tests
// ============================================================================

#[tokio::test]
async fn test_search_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_search("res.partner", vec![1, 2, 3]).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_search_with_domain() {
    let mock = MockOdooServer::start().await;
    mock.mock_search("res.partner", vec![1]).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let domain = json!([["active", "=", true]]);
    let result = client
        .search("res.partner", Some(domain), Some(10), Some(0), None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![1]);
}

#[tokio::test]
async fn test_search_read_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_search_read("res.partner", responses::partners())
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

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
    assert_eq!(result.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_read_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_read("res.partner", responses::partners()).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .read("res.partner", vec![1, 2, 3], None, None)
        .await
        .unwrap();

    assert!(result.is_array());
}

#[tokio::test]
async fn test_create_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_create("res.partner", 42).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let values = json!({"name": "New Partner", "email": "new@example.com"});
    let result = client.create("res.partner", values, None).await.unwrap();

    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_write_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_write("res.partner", true).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let values = json!({"name": "Updated Partner"});
    let result = client
        .write("res.partner", vec![1], values, None)
        .await
        .unwrap();

    assert!(result);
}

#[tokio::test]
async fn test_unlink_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_unlink("res.partner", true).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

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
async fn test_search_count_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_search_count("res.partner", 42).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search_count("res.partner", None, None)
        .await
        .unwrap();

    assert_eq!(result, 42);
}

#[tokio::test]
async fn test_fields_get_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_fields_get("res.partner", responses::fields_get_partner())
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client.fields_get("res.partner", None).await.unwrap();

    assert!(result.is_object());
    assert!(result.get("id").is_some());
    assert!(result.get("name").is_some());
}

#[tokio::test]
async fn test_read_group_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_method("sale.order", "read_group", responses::read_group_by_state())
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .read_group(
            "sale.order",
            None,
            vec!["state".to_string()],
            vec!["state".to_string()],
            None,
            None,
            None,
            None,
            None,
        )
        .await
        .unwrap();

    assert!(result.is_array());
    assert_eq!(result.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_name_search_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_method(
        "res.partner",
        "name_search",
        responses::name_search_results(),
    )
    .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

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
    assert_eq!(result.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_name_get_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_method("res.partner", "name_get", responses::name_search_results())
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .name_get("res.partner", vec![1, 2, 3], None)
        .await
        .unwrap();

    assert!(result.is_array());
}

#[tokio::test]
async fn test_default_get_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_method(
        "res.partner",
        "default_get",
        responses::default_get_partner(),
    )
    .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .default_get(
            "res.partner",
            vec!["active".to_string(), "is_company".to_string()],
            None,
        )
        .await
        .unwrap();

    assert!(result.is_object());
    assert_eq!(result.get("active"), Some(&json!(true)));
}

#[tokio::test]
async fn test_copy_success() {
    let mock = MockOdooServer::start().await;
    mock.mock_method("res.partner", "copy", json!(99)).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client.copy("res.partner", 1, None, None).await.unwrap();

    assert_eq!(result, 99);
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_api_error_401_unauthorized() {
    let mock = MockOdooServer::start().await;
    mock.mock_error("res.partner", "search", 401, "Unauthorized")
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("401") || err.to_string().contains("Unauthorized"));
}

#[tokio::test]
async fn test_api_error_403_forbidden() {
    let mock = MockOdooServer::start().await;
    mock.mock_error("res.partner", "search", 403, "Forbidden")
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_error_404_not_found() {
    let mock = MockOdooServer::start().await;
    mock.mock_error("res.partner", "search", 404, "Not Found")
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_api_error_500_server_error() {
    let mock = MockOdooServer::start().await;
    mock.mock_error("res.partner", "search", 500, "Internal Server Error")
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await;

    assert!(result.is_err());
}

// ============================================================================
// Header Validation Tests
// ============================================================================

#[tokio::test]
async fn test_authorization_header_sent() {
    let mock = MockOdooServer::start().await;

    // Verify Authorization header is sent
    Mock::given(method("POST"))
        .and(path_regex(r"/json/2/res\.partner/search"))
        .and(header("authorization", "bearer test_api_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([1, 2, 3])))
        .mount(&mock.server)
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![1, 2, 3]);
}

#[tokio::test]
async fn test_content_type_header_sent() {
    let mock = MockOdooServer::start().await;

    // The content-type header includes charset, so we just check it starts with application/json
    Mock::given(method("POST"))
        .and(path_regex(r"/json/2/res\.partner/search"))
        .and(wiremock::matchers::header_regex(
            "content-type",
            "application/json.*",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([1])))
        .mount(&mock.server)
        .await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .search("res.partner", None, None, None, None, None)
        .await
        .unwrap();

    assert_eq!(result, vec![1]);
}

// ============================================================================
// Report Download Tests
// ============================================================================

#[tokio::test]
async fn test_download_report_pdf() {
    let mock = MockOdooServer::start().await;
    let pdf_content = b"%PDF-1.4 test content".to_vec();
    mock.mock_report_pdf(pdf_content.clone()).await;

    let config = create_config(&mock.uri());
    let client = OdooHttpClient::new(&config).unwrap();

    let result = client
        .download_report_pdf("account.report_invoice", &[1])
        .await
        .unwrap();

    assert_eq!(result, pdf_content);
}
