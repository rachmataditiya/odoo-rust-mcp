use std::sync::Arc;
use std::time::Duration;

use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{Value, json};
use tokio::sync::RwLock;
use url::Url;

use super::config::OdooInstanceConfig;
use super::types::{OdooError, OdooErrorBody, OdooResult};

/// Odoo Legacy JSON-RPC client for Odoo < 19.
/// Uses /jsonrpc endpoint with username/password authentication.
#[derive(Clone)]
pub struct OdooLegacyClient {
    base_url: Url,
    db: String,
    username: String,
    password: String,
    http: reqwest::Client,
    max_retries: usize,
    /// Cached user ID after authentication
    uid: Arc<RwLock<Option<i64>>>,
}

impl OdooLegacyClient {
    pub fn new(cfg: &OdooInstanceConfig) -> anyhow::Result<Self> {
        let mut base_url = Url::parse(&cfg.url)
            .map_err(|e| anyhow::anyhow!("Invalid Odoo url '{}': {e}", cfg.url))?;
        base_url.set_path("");
        base_url.set_query(None);
        base_url.set_fragment(None);

        let db = cfg.db.clone().ok_or_else(|| {
            anyhow::anyhow!("Missing db for legacy Odoo instance url={}", cfg.url)
        })?;
        let username = cfg.username.clone().ok_or_else(|| {
            anyhow::anyhow!("Missing username for legacy Odoo instance url={}", cfg.url)
        })?;
        let password = cfg.password.clone().ok_or_else(|| {
            anyhow::anyhow!("Missing password for legacy Odoo instance url={}", cfg.url)
        })?;

        let timeout = Duration::from_millis(cfg.timeout_ms.unwrap_or(30_000));
        let max_retries = cfg.max_retries.unwrap_or(3);

        let http = reqwest::Client::builder()
            .timeout(timeout)
            .cookie_store(true)
            .build()?;

        Ok(Self {
            base_url,
            db,
            username,
            password,
            http,
            max_retries,
            uid: Arc::new(RwLock::new(None)),
        })
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("odoo-mcp-rust/0.1"));
        headers
    }

    fn jsonrpc_endpoint(&self) -> Url {
        let mut url = self.base_url.clone();
        url.set_path("/jsonrpc");
        url
    }

    /// Build a JSON-RPC 2.0 request payload
    fn build_jsonrpc_request(&self, service: &str, method: &str, args: Value) -> Value {
        json!({
            "jsonrpc": "2.0",
            "method": "call",
            "params": {
                "service": service,
                "method": method,
                "args": args
            },
            "id": 1
        })
    }

    /// Send a JSON-RPC request and extract the result
    async fn jsonrpc_call(&self, service: &str, method: &str, args: Value) -> OdooResult<Value> {
        let url = self.jsonrpc_endpoint();
        let headers = self.headers();
        let body = self.build_jsonrpc_request(service, method, args);

        let mut last_err: Option<OdooError> = None;

        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .post(url.clone())
                .headers(headers.clone())
                .json(&body)
                .send()
                .await;

            match resp {
                Ok(r) => {
                    let status = r.status();
                    let text = r.text().await.unwrap_or_default();

                    if status.is_success() {
                        let v: Value = serde_json::from_str(&text).map_err(|e| {
                            OdooError::InvalidResponse(format!(
                                "Failed to parse JSON-RPC response: {e}. Body: {text}"
                            ))
                        })?;

                        // Check for JSON-RPC error
                        if let Some(error) = v.get("error") {
                            let message = error
                                .get("data")
                                .and_then(|d| d.get("message"))
                                .and_then(|m| m.as_str())
                                .or_else(|| error.get("message").and_then(|m| m.as_str()))
                                .unwrap_or("Unknown JSON-RPC error")
                                .to_string();

                            return Err(OdooError::Api {
                                status: 400,
                                message,
                                body: None,
                            });
                        }

                        // Extract result
                        if let Some(result) = v.get("result") {
                            return Ok(result.clone());
                        }

                        return Err(OdooError::InvalidResponse(
                            "JSON-RPC response missing 'result' field".to_string(),
                        ));
                    }

                    let parsed_err: Option<OdooErrorBody> = serde_json::from_str(&text).ok();
                    let message = parsed_err
                        .as_ref()
                        .and_then(|b| b.message.clone())
                        .unwrap_or_else(|| text.clone());
                    let err = OdooError::Api {
                        status: status.as_u16(),
                        message,
                        body: parsed_err,
                    };

                    if status.is_server_error() || status.as_u16() == 429 {
                        last_err = Some(err);
                    } else {
                        return Err(err);
                    }
                }
                Err(e) => {
                    last_err = Some(OdooError::Http(e));
                }
            }

            if attempt < self.max_retries {
                let backoff_ms = 250u64.saturating_mul(2u64.saturating_pow(attempt as u32));
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        Err(last_err.unwrap_or_else(|| {
            OdooError::InvalidResponse("Request failed without error details".to_string())
        }))
    }

    /// Authenticate and get user ID
    pub async fn authenticate(&self) -> OdooResult<i64> {
        // Check cache first
        {
            let cached = self.uid.read().await;
            if let Some(uid) = *cached {
                return Ok(uid);
            }
        }

        // Authenticate via common service
        let args = json!([self.db, self.username, self.password, {}]);
        let result = self.jsonrpc_call("common", "authenticate", args).await?;

        let uid = result.as_i64().ok_or_else(|| OdooError::Api {
            status: 401,
            message: format!(
                "Authentication failed for user '{}'. Check username/password.",
                self.username
            ),
            body: None,
        })?;

        if uid == 0 {
            return Err(OdooError::Api {
                status: 401,
                message: format!(
                    "Authentication failed for user '{}'. Invalid credentials.",
                    self.username
                ),
                body: None,
            });
        }

        // Cache the uid
        {
            let mut cached = self.uid.write().await;
            *cached = Some(uid);
        }

        Ok(uid)
    }

    /// Call execute_kw on the object service
    async fn execute_kw(
        &self,
        model: &str,
        method: &str,
        args: Value,
        kwargs: Option<Value>,
    ) -> OdooResult<Value> {
        let uid = self.authenticate().await?;

        // execute_kw always expects 7 arguments: [db, uid, password, model, method, args, kwargs]
        // kwargs must be an object (even if empty) for proper Odoo execution
        let call_args = vec![
            json!(self.db),
            json!(uid),
            json!(self.password),
            json!(model),
            json!(method),
            args,
            kwargs.unwrap_or_else(|| json!({})),
        ];

        self.jsonrpc_call("object", "execute_kw", json!(call_args))
            .await
    }

    pub async fn search(
        &self,
        model: &str,
        domain: Option<Value>,
        limit: Option<i64>,
        offset: Option<i64>,
        order: Option<String>,
        _context: Option<Value>,
    ) -> OdooResult<Vec<i64>> {
        let domain = domain.unwrap_or(json!([]));
        let mut kwargs = serde_json::Map::new();
        if let Some(v) = limit {
            kwargs.insert("limit".to_string(), json!(v));
        }
        if let Some(v) = offset {
            kwargs.insert("offset".to_string(), json!(v));
        }
        if let Some(v) = order {
            kwargs.insert("order".to_string(), json!(v));
        }

        let result = self
            .execute_kw(model, "search", json!([domain]), Some(json!(kwargs)))
            .await?;

        serde_json::from_value(result).map_err(|e| {
            OdooError::InvalidResponse(format!("Expected array of ids from search: {e}"))
        })
    }

    pub async fn search_read(
        &self,
        model: &str,
        domain: Option<Value>,
        fields: Option<Vec<String>>,
        limit: Option<i64>,
        offset: Option<i64>,
        order: Option<String>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        let domain = domain.unwrap_or(json!([]));
        let mut kwargs = serde_json::Map::new();
        if let Some(v) = fields {
            kwargs.insert("fields".to_string(), json!(v));
        }
        if let Some(v) = limit {
            kwargs.insert("limit".to_string(), json!(v));
        }
        if let Some(v) = offset {
            kwargs.insert("offset".to_string(), json!(v));
        }
        if let Some(v) = order {
            kwargs.insert("order".to_string(), json!(v));
        }

        self.execute_kw(model, "search_read", json!([domain]), Some(json!(kwargs)))
            .await
    }

    pub async fn read(
        &self,
        model: &str,
        ids: Vec<i64>,
        fields: Option<Vec<String>>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut kwargs = serde_json::Map::new();
        if let Some(v) = fields {
            kwargs.insert("fields".to_string(), json!(v));
        }

        self.execute_kw(model, "read", json!([ids]), Some(json!(kwargs)))
            .await
    }

    pub async fn create(
        &self,
        model: &str,
        values: Value,
        _context: Option<Value>,
    ) -> OdooResult<i64> {
        let result = self
            .execute_kw(model, "create", json!([values]), None)
            .await?;
        serde_json::from_value(result).map_err(|e| {
            OdooError::InvalidResponse(format!("Expected created id (number) from create: {e}"))
        })
    }

    pub async fn write(
        &self,
        model: &str,
        ids: Vec<i64>,
        values: Value,
        _context: Option<Value>,
    ) -> OdooResult<bool> {
        let result = self
            .execute_kw(model, "write", json!([ids, values]), None)
            .await?;
        serde_json::from_value(result)
            .map_err(|e| OdooError::InvalidResponse(format!("Expected boolean from write: {e}")))
    }

    pub async fn unlink(
        &self,
        model: &str,
        ids: Vec<i64>,
        _context: Option<Value>,
    ) -> OdooResult<bool> {
        let result = self.execute_kw(model, "unlink", json!([ids]), None).await?;
        serde_json::from_value(result)
            .map_err(|e| OdooError::InvalidResponse(format!("Expected boolean from unlink: {e}")))
    }

    pub async fn search_count(
        &self,
        model: &str,
        domain: Option<Value>,
        _context: Option<Value>,
    ) -> OdooResult<i64> {
        let domain = domain.unwrap_or(json!([]));
        let result = self
            .execute_kw(model, "search_count", json!([domain]), None)
            .await?;
        serde_json::from_value(result).map_err(|e| {
            OdooError::InvalidResponse(format!("Expected count (number) from search_count: {e}"))
        })
    }

    pub async fn fields_get(&self, model: &str, _context: Option<Value>) -> OdooResult<Value> {
        self.execute_kw(model, "fields_get", json!([]), Some(json!({"attributes": ["string", "type", "help", "required", "readonly", "relation", "selection"]})))
            .await
    }

    pub async fn call_named(
        &self,
        model: &str,
        method: &str,
        ids: Option<Vec<i64>>,
        params: serde_json::Map<String, Value>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        let args = if let Some(ids) = ids {
            json!([ids])
        } else {
            json!([])
        };

        let kwargs = if params.is_empty() {
            None
        } else {
            Some(json!(params))
        };

        self.execute_kw(model, method, args, kwargs).await
    }

    pub async fn download_report_pdf(&self, report_name: &str, ids: &[i64]) -> OdooResult<Vec<u8>> {
        let _uid = self.authenticate().await?;

        // For legacy Odoo, we use the web controller for reports
        let mut url = self.base_url.clone();
        let ids_csv = ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        url.set_path(&format!("/report/pdf/{report_name}/{ids_csv}"));

        // We need to authenticate via session first
        // This is a simplified approach - in production you might need session cookies
        let mut last_err: Option<OdooError> = None;
        for attempt in 0..=self.max_retries {
            // First, establish session via web/session/authenticate
            let session_url = {
                let mut u = self.base_url.clone();
                u.set_path("/web/session/authenticate");
                u
            };

            let session_body = json!({
                "jsonrpc": "2.0",
                "method": "call",
                "params": {
                    "db": self.db,
                    "login": self.username,
                    "password": self.password
                },
                "id": 1
            });

            let session_resp = self
                .http
                .post(session_url)
                .headers(self.headers())
                .json(&session_body)
                .send()
                .await;

            if let Err(e) = session_resp {
                last_err = Some(OdooError::Http(e));
                if attempt < self.max_retries {
                    let backoff_ms = 250u64.saturating_mul(2u64.saturating_pow(attempt as u32));
                    tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
                }
                continue;
            }

            // Now fetch the report
            let resp = self.http.get(url.clone()).send().await;
            match resp {
                Ok(r) => {
                    let status = r.status();
                    if status.is_success() {
                        let bytes = r.bytes().await.map_err(OdooError::Http)?;
                        return Ok(bytes.to_vec());
                    }

                    let text = r.text().await.unwrap_or_default();
                    let err = OdooError::Api {
                        status: status.as_u16(),
                        message: text,
                        body: None,
                    };

                    if status.is_server_error() || status.as_u16() == 429 {
                        last_err = Some(err);
                    } else {
                        return Err(err);
                    }
                }
                Err(e) => {
                    last_err = Some(OdooError::Http(e));
                }
            }

            if attempt < self.max_retries {
                let backoff_ms = 250u64.saturating_mul(2u64.saturating_pow(attempt as u32));
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        Err(last_err.unwrap_or_else(|| {
            OdooError::InvalidResponse("Request failed without error details".to_string())
        }))
    }

    /// read_group - Aggregate records with GROUP BY
    pub async fn read_group(
        &self,
        model: &str,
        domain: Option<Value>,
        fields: Vec<String>,
        groupby: Vec<String>,
        offset: Option<i64>,
        limit: Option<i64>,
        orderby: Option<String>,
        lazy: Option<bool>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        let domain = domain.unwrap_or(json!([]));
        let mut kwargs = json!({});
        if let Some(v) = offset {
            kwargs["offset"] = json!(v);
        }
        if let Some(v) = limit {
            kwargs["limit"] = json!(v);
        }
        if let Some(v) = orderby {
            kwargs["orderby"] = json!(v);
        }
        if let Some(v) = lazy {
            kwargs["lazy"] = json!(v);
        }
        self.execute_kw(
            model,
            "read_group",
            json!([domain, fields, groupby]),
            Some(kwargs),
        )
        .await
    }

    /// name_search - Search by name with autocomplete-style matching
    pub async fn name_search(
        &self,
        model: &str,
        name: Option<String>,
        args: Option<Value>,
        operator: Option<String>,
        limit: Option<i64>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        let name = name.unwrap_or_default();
        let args = args.unwrap_or(json!([]));
        let operator = operator.unwrap_or_else(|| "ilike".to_string());
        let limit = limit.unwrap_or(100);
        self.execute_kw(
            model,
            "name_search",
            json!([name, args, operator, limit]),
            None,
        )
        .await
    }

    /// name_get - Get display names for records
    pub async fn name_get(
        &self,
        model: &str,
        ids: Vec<i64>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        self.execute_kw(model, "name_get", json!([ids]), None).await
    }

    /// default_get - Get default values for new records
    pub async fn default_get(
        &self,
        model: &str,
        fields_list: Vec<String>,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        self.execute_kw(model, "default_get", json!([fields_list]), None)
            .await
    }

    /// copy - Duplicate a record
    pub async fn copy(
        &self,
        model: &str,
        id: i64,
        default: Option<Value>,
        _context: Option<Value>,
    ) -> OdooResult<i64> {
        let kwargs = default.map(|d| json!({ "default": d }));
        let result = self.execute_kw(model, "copy", json!([id]), kwargs).await?;
        serde_json::from_value(result)
            .map_err(|e| OdooError::InvalidResponse(format!("Expected id from copy: {e}")))
    }

    /// onchange - Simulate form onchange behavior
    pub async fn onchange(
        &self,
        model: &str,
        ids: Vec<i64>,
        values: Value,
        field_name: Vec<String>,
        field_onchange: Value,
        _context: Option<Value>,
    ) -> OdooResult<Value> {
        self.execute_kw(
            model,
            "onchange",
            json!([ids, values, field_name, field_onchange]),
            None,
        )
        .await
    }

    /// Health check: perform a minimal operation to verify Odoo is reachable.
    /// Uses search_count on ir.model with empty domain as a cheap probe.
    pub async fn health_check(&self) -> bool {
        self.search_count("ir.model", Some(json!([])), None)
            .await
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_legacy_config(
        url: &str,
        db: Option<&str>,
        username: Option<&str>,
        password: Option<&str>,
    ) -> OdooInstanceConfig {
        OdooInstanceConfig {
            url: url.to_string(),
            db: db.map(|s| s.to_string()),
            api_key: None,
            username: username.map(|s| s.to_string()),
            password: password.map(|s| s.to_string()),
            version: Some("18".to_string()),
            timeout_ms: Some(5000),
            max_retries: Some(2),
            extra: HashMap::new(),
        }
    }

    #[test]
    fn test_legacy_client_new_success() {
        let cfg = make_legacy_config(
            "http://localhost:8069",
            Some("mydb"),
            Some("admin"),
            Some("admin"),
        );
        let client = OdooLegacyClient::new(&cfg);
        assert!(client.is_ok());
    }

    #[test]
    fn test_legacy_client_new_invalid_url() {
        let cfg = make_legacy_config("not a valid url", Some("db"), Some("user"), Some("pass"));
        let client = OdooLegacyClient::new(&cfg);
        assert!(client.is_err());
    }

    #[test]
    fn test_legacy_client_new_missing_db() {
        let cfg = make_legacy_config("http://localhost:8069", None, Some("admin"), Some("admin"));
        let client = OdooLegacyClient::new(&cfg);
        assert!(client.is_err());
    }

    #[test]
    fn test_legacy_client_new_missing_username() {
        let cfg = make_legacy_config("http://localhost:8069", Some("db"), None, Some("pass"));
        let client = OdooLegacyClient::new(&cfg);
        assert!(client.is_err());
    }

    #[test]
    fn test_legacy_client_new_missing_password() {
        let cfg = make_legacy_config("http://localhost:8069", Some("db"), Some("user"), None);
        let client = OdooLegacyClient::new(&cfg);
        assert!(client.is_err());
    }

    #[test]
    fn test_legacy_client_normalizes_url() {
        let cfg = make_legacy_config(
            "http://localhost:8069/some/path?query=1",
            Some("db"),
            Some("user"),
            Some("pass"),
        );
        let client = OdooLegacyClient::new(&cfg).unwrap();
        // The base_url should be normalized without path/query
        assert_eq!(client.base_url.path(), "/");
        assert!(client.base_url.query().is_none());
    }

    #[test]
    fn test_legacy_client_stores_credentials() {
        let cfg = make_legacy_config(
            "http://localhost:8069",
            Some("mydb"),
            Some("admin"),
            Some("secret"),
        );
        let client = OdooLegacyClient::new(&cfg).unwrap();
        assert_eq!(client.db, "mydb");
        assert_eq!(client.username, "admin");
        assert_eq!(client.password, "secret");
    }

    #[test]
    fn test_legacy_client_default_max_retries() {
        let cfg = OdooInstanceConfig {
            url: "http://localhost:8069".to_string(),
            db: Some("db".to_string()),
            api_key: None,
            username: Some("user".to_string()),
            password: Some("pass".to_string()),
            version: None,
            timeout_ms: None,
            max_retries: None,
            extra: HashMap::new(),
        };
        let client = OdooLegacyClient::new(&cfg).unwrap();
        assert_eq!(client.max_retries, 3); // default
    }

    #[test]
    fn test_legacy_client_custom_max_retries() {
        let cfg = make_legacy_config(
            "http://localhost:8069",
            Some("db"),
            Some("user"),
            Some("pass"),
        );
        let client = OdooLegacyClient::new(&cfg).unwrap();
        assert_eq!(client.max_retries, 2); // from config
    }

    #[test]
    fn test_legacy_client_jsonrpc_endpoint() {
        let cfg = make_legacy_config(
            "http://localhost:8069",
            Some("db"),
            Some("user"),
            Some("pass"),
        );
        let client = OdooLegacyClient::new(&cfg).unwrap();
        let endpoint = client.jsonrpc_endpoint();
        assert_eq!(endpoint.path(), "/jsonrpc");
    }

    #[test]
    fn test_legacy_client_build_jsonrpc_request() {
        let cfg = make_legacy_config(
            "http://localhost:8069",
            Some("db"),
            Some("user"),
            Some("pass"),
        );
        let client = OdooLegacyClient::new(&cfg).unwrap();
        let request = client.build_jsonrpc_request("common", "version", json!([]));

        assert_eq!(request["jsonrpc"], "2.0");
        assert_eq!(request["method"], "call");
        assert_eq!(request["params"]["service"], "common");
        assert_eq!(request["params"]["method"], "version");
    }
}
