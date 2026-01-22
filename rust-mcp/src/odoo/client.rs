use std::time::Duration;

use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use serde_json::{Value, json};
use url::Url;

use super::config::OdooInstanceConfig;
use super::types::{OdooError, OdooErrorBody, OdooResult};

#[derive(Clone)]
pub struct OdooHttpClient {
    base_url: Url, // e.g. https://mycompany.example.com
    db: Option<String>,
    api_key: String,
    http: reqwest::Client,
    max_retries: usize,
}

impl OdooHttpClient {
    pub fn new(cfg: &OdooInstanceConfig) -> anyhow::Result<Self> {
        let mut base_url = Url::parse(&cfg.url)
            .map_err(|e| anyhow::anyhow!("Invalid Odoo url '{}': {e}", cfg.url))?;
        // Normalize to origin (strip path/query), but keep scheme/host/port.
        base_url.set_path("");
        base_url.set_query(None);
        base_url.set_fragment(None);

        let api_key = cfg
            .api_key
            .clone()
            .ok_or_else(|| anyhow::anyhow!("Missing api key for instance url={}", cfg.url))?;

        let timeout = Duration::from_millis(cfg.timeout_ms.unwrap_or(30_000));
        let max_retries = cfg.max_retries.unwrap_or(3);

        let http = reqwest::Client::builder()
            .timeout(timeout)
            .cookie_store(true)
            .build()?;

        Ok(Self {
            base_url,
            db: cfg.db.clone(),
            api_key,
            http,
            max_retries,
        })
    }

    fn headers(&self) -> anyhow::Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("bearer {}", self.api_key))
                .map_err(|e| anyhow::anyhow!("Invalid Authorization header value: {e}"))?,
        );
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        headers.insert(USER_AGENT, HeaderValue::from_static("odoo-mcp-rust/0.1"));
        if let Some(db) = &self.db
            && !db.trim().is_empty()
        {
            headers.insert(
                "X-Odoo-Database",
                HeaderValue::from_str(db)
                    .map_err(|e| anyhow::anyhow!("Invalid X-Odoo-Database header: {e}"))?,
            );
        }
        Ok(headers)
    }

    fn endpoint(&self, model: &str, method: &str) -> anyhow::Result<Url> {
        let mut url = self.base_url.clone();
        url.set_path(&format!("/json/2/{model}/{method}"));
        Ok(url)
    }

    async fn post_json2_raw(&self, model: &str, method: &str, body: Value) -> OdooResult<Value> {
        let url = self
            .endpoint(model, method)
            .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
        let headers = self
            .headers()
            .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

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
                                "Failed to parse JSON response: {e}. Body: {text}"
                            ))
                        })?;
                        return Ok(v);
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

                    // Retry on 5xx and 429; do not retry auth/4xx.
                    if status.is_server_error() || status.as_u16() == 429 {
                        last_err = Some(err);
                    } else {
                        return Err(err);
                    }
                }
                Err(e) => {
                    // Retry on transport-level errors.
                    last_err = Some(OdooError::Http(e));
                }
            }

            if attempt < self.max_retries {
                // Exponential backoff: 250ms, 500ms, 1s, 2s...
                let backoff_ms = 250u64.saturating_mul(2u64.saturating_pow(attempt as u32));
                tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
            }
        }

        Err(last_err.unwrap_or_else(|| {
            OdooError::InvalidResponse("Request failed without error details".to_string())
        }))
    }

    pub async fn download_report_pdf(&self, report_name: &str, ids: &[i64]) -> OdooResult<Vec<u8>> {
        let mut url = self.base_url.clone();
        let ids_csv = ids
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(",");
        url.set_path(&format!("/report/pdf/{report_name}/{ids_csv}"));

        let headers = self
            .headers()
            .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

        let mut last_err: Option<OdooError> = None;
        for attempt in 0..=self.max_retries {
            let resp = self
                .http
                .get(url.clone())
                .headers(headers.clone())
                .send()
                .await;
            match resp {
                Ok(r) => {
                    let status = r.status();
                    if status.is_success() {
                        let bytes = r.bytes().await.map_err(OdooError::Http)?;
                        return Ok(bytes.to_vec());
                    }

                    let text = r.text().await.unwrap_or_default();
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

    pub async fn search(
        &self,
        model: &str,
        domain: Option<Value>,
        limit: Option<i64>,
        offset: Option<i64>,
        order: Option<String>,
        context: Option<Value>,
    ) -> OdooResult<Vec<i64>> {
        let mut body = json!({});
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(d) = domain {
            body["domain"] = d;
        }
        if let Some(v) = limit {
            body["limit"] = json!(v);
        }
        if let Some(v) = offset {
            body["offset"] = json!(v);
        }
        if let Some(v) = order {
            body["order"] = json!(v);
        }

        let v = self.post_json2_raw(model, "search", body).await?;
        serde_json::from_value(v).map_err(|e| {
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
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({});
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(d) = domain {
            body["domain"] = d;
        }
        if let Some(v) = fields {
            body["fields"] = json!(v);
        }
        if let Some(v) = limit {
            body["limit"] = json!(v);
        }
        if let Some(v) = offset {
            body["offset"] = json!(v);
        }
        if let Some(v) = order {
            body["order"] = json!(v);
        }

        self.post_json2_raw(model, "search_read", body).await
    }

    pub async fn read(
        &self,
        model: &str,
        ids: Vec<i64>,
        fields: Option<Vec<String>>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({ "ids": ids });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(v) = fields {
            body["fields"] = json!(v);
        }

        self.post_json2_raw(model, "read", body).await
    }

    pub async fn create(
        &self,
        model: &str,
        values: Value,
        context: Option<Value>,
    ) -> OdooResult<i64> {
        // Odoo signature is create(vals_list)
        let mut body = json!({ "vals_list": values });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        let v = self.post_json2_raw(model, "create", body).await?;
        serde_json::from_value(v).map_err(|e| {
            OdooError::InvalidResponse(format!("Expected created id (number) from create: {e}"))
        })
    }

    pub async fn write(
        &self,
        model: &str,
        ids: Vec<i64>,
        values: Value,
        context: Option<Value>,
    ) -> OdooResult<bool> {
        // Odoo signature is write(vals)
        let mut body = json!({ "ids": ids, "vals": values });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        let v = self.post_json2_raw(model, "write", body).await?;
        serde_json::from_value(v)
            .map_err(|e| OdooError::InvalidResponse(format!("Expected boolean from write: {e}")))
    }

    pub async fn unlink(
        &self,
        model: &str,
        ids: Vec<i64>,
        context: Option<Value>,
    ) -> OdooResult<bool> {
        let mut body = json!({ "ids": ids });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        let v = self.post_json2_raw(model, "unlink", body).await?;
        serde_json::from_value(v)
            .map_err(|e| OdooError::InvalidResponse(format!("Expected boolean from unlink: {e}")))
    }

    pub async fn search_count(
        &self,
        model: &str,
        domain: Option<Value>,
        context: Option<Value>,
    ) -> OdooResult<i64> {
        let mut body = json!({});
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(d) = domain {
            body["domain"] = d;
        }
        let v = self.post_json2_raw(model, "search_count", body).await?;
        serde_json::from_value(v).map_err(|e| {
            OdooError::InvalidResponse(format!("Expected count (number) from search_count: {e}"))
        })
    }

    pub async fn fields_get(&self, model: &str, context: Option<Value>) -> OdooResult<Value> {
        // fields_get(allfields=None, attributes=None)
        let mut body = json!({});
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        self.post_json2_raw(model, "fields_get", body).await
    }

    pub async fn call_named(
        &self,
        model: &str,
        method: &str,
        ids: Option<Vec<i64>>,
        params: serde_json::Map<String, Value>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = Value::Object(params);
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(ids) = ids {
            body["ids"] = json!(ids);
        }
        self.post_json2_raw(model, method, body).await
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
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({
            "fields": fields,
            "groupby": groupby
        });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(d) = domain {
            body["domain"] = d;
        }
        if let Some(v) = offset {
            body["offset"] = json!(v);
        }
        if let Some(v) = limit {
            body["limit"] = json!(v);
        }
        if let Some(v) = orderby {
            body["orderby"] = json!(v);
        }
        if let Some(v) = lazy {
            body["lazy"] = json!(v);
        }
        self.post_json2_raw(model, "read_group", body).await
    }

    /// name_search - Search by name with autocomplete-style matching
    pub async fn name_search(
        &self,
        model: &str,
        name: Option<String>,
        args: Option<Value>,
        operator: Option<String>,
        limit: Option<i64>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({});
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(n) = name {
            body["name"] = json!(n);
        }
        if let Some(a) = args {
            body["args"] = a;
        }
        if let Some(op) = operator {
            body["operator"] = json!(op);
        }
        if let Some(l) = limit {
            body["limit"] = json!(l);
        }
        self.post_json2_raw(model, "name_search", body).await
    }

    /// name_get - Get display names for records
    pub async fn name_get(
        &self,
        model: &str,
        ids: Vec<i64>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({ "ids": ids });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        self.post_json2_raw(model, "name_get", body).await
    }

    /// default_get - Get default values for new records
    pub async fn default_get(
        &self,
        model: &str,
        fields_list: Vec<String>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({ "fields_list": fields_list });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        self.post_json2_raw(model, "default_get", body).await
    }

    /// copy - Duplicate a record
    pub async fn copy(
        &self,
        model: &str,
        id: i64,
        default: Option<Value>,
        context: Option<Value>,
    ) -> OdooResult<i64> {
        let mut body = json!({ "ids": [id] });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        if let Some(d) = default {
            body["default"] = d;
        }
        let v = self.post_json2_raw(model, "copy", body).await?;
        serde_json::from_value(v)
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
        context: Option<Value>,
    ) -> OdooResult<Value> {
        let mut body = json!({
            "ids": ids,
            "values": values,
            "field_name": field_name,
            "field_onchange": field_onchange
        });
        if let Some(ctx) = context {
            body["context"] = ctx;
        }
        self.post_json2_raw(model, "onchange", body).await
    }
}
