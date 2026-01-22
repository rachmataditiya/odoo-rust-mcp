//! Common test utilities for integration tests.

#![allow(dead_code)]

use serde_json::{Value, json};
use tempfile::TempDir;
use wiremock::matchers::{method, path_regex};
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Test environment that manages temp directories and environment variables.
pub struct TestEnv {
    pub temp_dir: TempDir,
    pub odoo_url: String,
}

impl TestEnv {
    /// Create a new test environment with a mock Odoo server.
    pub async fn new() -> (Self, MockOdooServer) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mock_server = MockOdooServer::start().await;

        let env = Self {
            temp_dir,
            odoo_url: mock_server.uri(),
        };

        (env, mock_server)
    }

    /// Get path for a config file in the temp directory.
    pub fn config_path(&self, name: &str) -> String {
        self.temp_dir
            .path()
            .join(name)
            .to_string_lossy()
            .to_string()
    }

    /// Write a config file to the temp directory.
    pub fn write_config(&self, name: &str, content: &str) {
        std::fs::write(self.temp_dir.path().join(name), content)
            .expect("Failed to write config file");
    }

    /// Set up minimal environment variables for testing.
    /// SAFETY: This uses unsafe env::set_var. Only call in single-threaded test contexts.
    pub unsafe fn setup_env(&self) {
        unsafe {
            std::env::set_var("ODOO_URL", &self.odoo_url);
            std::env::set_var("ODOO_DB", "test_db");
            std::env::set_var("ODOO_API_KEY", "test_api_key");
            std::env::set_var("MCP_TOOLS_JSON", self.config_path("tools.json"));
            std::env::set_var("MCP_PROMPTS_JSON", self.config_path("prompts.json"));
            std::env::set_var("MCP_SERVER_JSON", self.config_path("server.json"));
        }
    }
}

/// Mock Odoo server for testing client operations.
pub struct MockOdooServer {
    pub server: MockServer,
}

impl MockOdooServer {
    /// Start a new mock Odoo server.
    pub async fn start() -> Self {
        let server = MockServer::start().await;
        Self { server }
    }

    /// Get the server URI.
    pub fn uri(&self) -> String {
        self.server.uri()
    }

    /// Mount a mock for the modern JSON-2 API search endpoint.
    pub async fn mock_search(&self, model: &str, response: Vec<i64>) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(r"/json/2/{}/search", escaped_model)))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for the modern JSON-2 API search_read endpoint.
    pub async fn mock_search_read(&self, model: &str, response: Value) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(
                r"/json/2/{}/search_read",
                escaped_model
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for the modern JSON-2 API read endpoint.
    pub async fn mock_read(&self, model: &str, response: Value) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(r"/json/2/{}/read", escaped_model)))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for the modern JSON-2 API create endpoint.
    pub async fn mock_create(&self, model: &str, response_id: i64) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(r"/json/2/{}/create", escaped_model)))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_id))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for the modern JSON-2 API write endpoint.
    pub async fn mock_write(&self, model: &str, success: bool) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(r"/json/2/{}/write", escaped_model)))
            .respond_with(ResponseTemplate::new(200).set_body_json(success))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for the modern JSON-2 API unlink endpoint.
    pub async fn mock_unlink(&self, model: &str, success: bool) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(r"/json/2/{}/unlink", escaped_model)))
            .respond_with(ResponseTemplate::new(200).set_body_json(success))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for search_count endpoint.
    pub async fn mock_search_count(&self, model: &str, count: i64) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(
                r"/json/2/{}/search_count",
                escaped_model
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(count))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for fields_get endpoint.
    pub async fn mock_fields_get(&self, model: &str, response: Value) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(r"/json/2/{}/fields_get", escaped_model)))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for any model method (generic).
    pub async fn mock_method(&self, model: &str, method_name: &str, response: Value) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(
                r"/json/2/{}/{}",
                escaped_model, method_name
            )))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock that returns an error response.
    pub async fn mock_error(&self, model: &str, method_name: &str, status: u16, error_msg: &str) {
        let escaped_model = model.replace('.', r"\.");
        Mock::given(method("POST"))
            .and(path_regex(format!(
                r"/json/2/{}/{}",
                escaped_model, method_name
            )))
            .respond_with(ResponseTemplate::new(status).set_body_json(json!({
                "error": {
                    "message": error_msg,
                    "data": {
                        "name": "odoo.exceptions.AccessDenied",
                        "debug": "Access Denied"
                    }
                }
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for legacy JSON-RPC endpoint.
    pub async fn mock_legacy_jsonrpc(&self, response: Value) {
        Mock::given(method("POST"))
            .and(path_regex(r"/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for legacy authentication.
    pub async fn mock_legacy_auth(&self, uid: i64) {
        Mock::given(method("POST"))
            .and(path_regex(r"/jsonrpc"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": uid
            })))
            .mount(&self.server)
            .await;
    }

    /// Mount a mock for report download.
    pub async fn mock_report_pdf(&self, pdf_content: Vec<u8>) {
        Mock::given(method("GET"))
            .and(path_regex(r"/report/pdf/.*"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "application/pdf")
                    .set_body_bytes(pdf_content),
            )
            .mount(&self.server)
            .await;
    }
}

/// Common JSON response templates for testing.
pub mod responses {
    use serde_json::{Value, json};

    /// Sample partner records.
    pub fn partners() -> Value {
        json!([
            {"id": 1, "name": "Partner 1", "email": "partner1@example.com"},
            {"id": 2, "name": "Partner 2", "email": "partner2@example.com"},
            {"id": 3, "name": "Partner 3", "email": "partner3@example.com"}
        ])
    }

    /// Sample sale order records.
    pub fn sale_orders() -> Value {
        json!([
            {"id": 1, "name": "SO001", "state": "draft", "amount_total": 1000.0},
            {"id": 2, "name": "SO002", "state": "sale", "amount_total": 2500.0}
        ])
    }

    /// Sample fields_get response.
    pub fn fields_get_partner() -> Value {
        json!({
            "id": {"type": "integer", "string": "ID", "readonly": true},
            "name": {"type": "char", "string": "Name", "required": true},
            "email": {"type": "char", "string": "Email"},
            "active": {"type": "boolean", "string": "Active"}
        })
    }

    /// Sample read_group response.
    pub fn read_group_by_state() -> Value {
        json!([
            {"state": "draft", "state_count": 5, "__domain": [["state", "=", "draft"]]},
            {"state": "sale", "state_count": 10, "__domain": [["state", "=", "sale"]]},
            {"state": "done", "state_count": 3, "__domain": [["state", "=", "done"]]}
        ])
    }

    /// Sample name_search response.
    pub fn name_search_results() -> Value {
        json!([[1, "Partner 1"], [2, "Partner 2"], [3, "Partner 3"]])
    }

    /// Sample default_get response.
    pub fn default_get_partner() -> Value {
        json!({
            "active": true,
            "is_company": false,
            "type": "contact"
        })
    }
}

/// Helper to create a minimal tools.json for testing.
pub fn minimal_tools_json() -> &'static str {
    r#"{
        "tools": [
            {
                "name": "odoo_search",
                "description": "Search Odoo records",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "instance": {"type": "string"},
                        "model": {"type": "string"},
                        "domain": {"type": "array"}
                    },
                    "required": ["instance", "model"]
                },
                "op": {
                    "type": "search"
                }
            }
        ]
    }"#
}

/// Helper to create a minimal prompts.json for testing.
pub fn minimal_prompts_json() -> &'static str {
    r#"{
        "prompts": [
            {
                "name": "test_prompt",
                "description": "A test prompt",
                "content": "This is test content"
            }
        ]
    }"#
}

/// Helper to create a minimal server.json for testing.
pub fn minimal_server_json() -> &'static str {
    r#"{
        "serverName": "test-server",
        "instructions": "Test instructions",
        "protocolVersionDefault": "2025-11-05"
    }"#
}
