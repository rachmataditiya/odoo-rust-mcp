use serde_json::Value;

use super::client::OdooHttpClient;
use super::config::{OdooAuthMode, OdooInstanceConfig};
use super::legacy_client::OdooLegacyClient;
use super::types::OdooResult;

/// Unified Odoo client that supports both Odoo 19+ (JSON-2 API) and Odoo < 19 (JSON-RPC).
/// Automatically selects the appropriate client based on configuration.
#[derive(Clone)]
pub enum OdooClient {
    /// Odoo 19+ with JSON-2 API and API key authentication
    Modern(OdooHttpClient),
    /// Odoo < 19 with JSON-RPC and username/password authentication
    Legacy(OdooLegacyClient),
}

impl OdooClient {
    /// Create a new OdooClient based on the instance configuration.
    /// Automatically selects Modern (Odoo 19+) or Legacy (Odoo < 19) mode.
    pub fn new(cfg: &OdooInstanceConfig) -> anyhow::Result<Self> {
        match cfg.auth_mode() {
            OdooAuthMode::ApiKey => {
                let client = OdooHttpClient::new(cfg)?;
                Ok(OdooClient::Modern(client))
            }
            OdooAuthMode::Password => {
                let client = OdooLegacyClient::new(cfg)?;
                Ok(OdooClient::Legacy(client))
            }
        }
    }

    /// Returns true if using legacy (Odoo < 19) mode
    pub fn is_legacy(&self) -> bool {
        matches!(self, OdooClient::Legacy(_))
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
        match self {
            OdooClient::Modern(c) => c.search(model, domain, limit, offset, order, context).await,
            OdooClient::Legacy(c) => c.search(model, domain, limit, offset, order, context).await,
        }
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
        match self {
            OdooClient::Modern(c) => {
                c.search_read(model, domain, fields, limit, offset, order, context)
                    .await
            }
            OdooClient::Legacy(c) => {
                c.search_read(model, domain, fields, limit, offset, order, context)
                    .await
            }
        }
    }

    pub async fn read(
        &self,
        model: &str,
        ids: Vec<i64>,
        fields: Option<Vec<String>>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => c.read(model, ids, fields, context).await,
            OdooClient::Legacy(c) => c.read(model, ids, fields, context).await,
        }
    }

    pub async fn create(
        &self,
        model: &str,
        values: Value,
        context: Option<Value>,
    ) -> OdooResult<i64> {
        match self {
            OdooClient::Modern(c) => c.create(model, values, context).await,
            OdooClient::Legacy(c) => c.create(model, values, context).await,
        }
    }

    pub async fn write(
        &self,
        model: &str,
        ids: Vec<i64>,
        values: Value,
        context: Option<Value>,
    ) -> OdooResult<bool> {
        match self {
            OdooClient::Modern(c) => c.write(model, ids, values, context).await,
            OdooClient::Legacy(c) => c.write(model, ids, values, context).await,
        }
    }

    pub async fn unlink(
        &self,
        model: &str,
        ids: Vec<i64>,
        context: Option<Value>,
    ) -> OdooResult<bool> {
        match self {
            OdooClient::Modern(c) => c.unlink(model, ids, context).await,
            OdooClient::Legacy(c) => c.unlink(model, ids, context).await,
        }
    }

    pub async fn search_count(
        &self,
        model: &str,
        domain: Option<Value>,
        context: Option<Value>,
    ) -> OdooResult<i64> {
        match self {
            OdooClient::Modern(c) => c.search_count(model, domain, context).await,
            OdooClient::Legacy(c) => c.search_count(model, domain, context).await,
        }
    }

    pub async fn fields_get(&self, model: &str, context: Option<Value>) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => c.fields_get(model, context).await,
            OdooClient::Legacy(c) => c.fields_get(model, context).await,
        }
    }

    pub async fn call_named(
        &self,
        model: &str,
        method: &str,
        ids: Option<Vec<i64>>,
        params: serde_json::Map<String, Value>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => c.call_named(model, method, ids, params, context).await,
            OdooClient::Legacy(c) => c.call_named(model, method, ids, params, context).await,
        }
    }

    pub async fn download_report_pdf(&self, report_name: &str, ids: &[i64]) -> OdooResult<Vec<u8>> {
        match self {
            OdooClient::Modern(c) => c.download_report_pdf(report_name, ids).await,
            OdooClient::Legacy(c) => c.download_report_pdf(report_name, ids).await,
        }
    }

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
        match self {
            OdooClient::Modern(c) => {
                c.read_group(
                    model, domain, fields, groupby, offset, limit, orderby, lazy, context,
                )
                .await
            }
            OdooClient::Legacy(c) => {
                c.read_group(
                    model, domain, fields, groupby, offset, limit, orderby, lazy, context,
                )
                .await
            }
        }
    }

    pub async fn name_search(
        &self,
        model: &str,
        name: Option<String>,
        args: Option<Value>,
        operator: Option<String>,
        limit: Option<i64>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => {
                c.name_search(model, name, args, operator, limit, context)
                    .await
            }
            OdooClient::Legacy(c) => {
                c.name_search(model, name, args, operator, limit, context)
                    .await
            }
        }
    }

    pub async fn name_get(
        &self,
        model: &str,
        ids: Vec<i64>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => c.name_get(model, ids, context).await,
            OdooClient::Legacy(c) => c.name_get(model, ids, context).await,
        }
    }

    pub async fn default_get(
        &self,
        model: &str,
        fields_list: Vec<String>,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => c.default_get(model, fields_list, context).await,
            OdooClient::Legacy(c) => c.default_get(model, fields_list, context).await,
        }
    }

    pub async fn copy(
        &self,
        model: &str,
        id: i64,
        default: Option<Value>,
        context: Option<Value>,
    ) -> OdooResult<i64> {
        match self {
            OdooClient::Modern(c) => c.copy(model, id, default, context).await,
            OdooClient::Legacy(c) => c.copy(model, id, default, context).await,
        }
    }

    pub async fn onchange(
        &self,
        model: &str,
        ids: Vec<i64>,
        values: Value,
        field_name: Vec<String>,
        field_onchange: Value,
        context: Option<Value>,
    ) -> OdooResult<Value> {
        match self {
            OdooClient::Modern(c) => {
                c.onchange(model, ids, values, field_name, field_onchange, context)
                    .await
            }
            OdooClient::Legacy(c) => {
                c.onchange(model, ids, values, field_name, field_onchange, context)
                    .await
            }
        }
    }
}
