use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use serde_json::{Map, Value, json};
use tokio::sync::Mutex;

use crate::cleanup;
use crate::mcp::cache::MetadataCache;
use crate::mcp::registry::{OpSpec, ToolDef};
use crate::odoo::config::{OdooEnvConfig, load_odoo_env};
use crate::odoo::types::OdooError;
use crate::odoo::unified_client::OdooClient;

/// Shared state: parsed env + instantiated clients per instance.
/// Supports both Odoo 19+ (JSON-2 API) and Odoo < 19 (JSON-RPC).
#[derive(Clone)]
pub struct OdooClientPool {
    env: Arc<OdooEnvConfig>,
    clients: Arc<Mutex<HashMap<String, OdooClient>>>,
    pub metadata_cache: MetadataCache,
}

impl OdooClientPool {
    pub fn from_env() -> anyhow::Result<Self> {
        let env = load_odoo_env()?;
        Ok(Self {
            env: Arc::new(env),
            clients: Arc::new(Mutex::new(HashMap::new())),
            metadata_cache: MetadataCache::new(),
        })
    }

    pub async fn get(&self, instance: &str) -> anyhow::Result<OdooClient> {
        {
            let guard = self.clients.lock().await;
            if let Some(c) = guard.get(instance) {
                return Ok(c.clone());
            }
        }

        let cfg = self.env.instances.get(instance).ok_or_else(|| {
            let available = self
                .env
                .instances
                .keys()
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            anyhow::anyhow!("Unknown Odoo instance '{instance}'. Available: {available}")
        })?;

        let client = OdooClient::new(cfg)?;
        let mut guard = self.clients.lock().await;
        guard.insert(instance.to_string(), client.clone());
        Ok(client)
    }

    pub fn instance_names(&self) -> Vec<String> {
        self.env.instances.keys().cloned().collect()
    }
}

pub async fn call_tool(
    pool: &OdooClientPool,
    tool: &ToolDef,
    args: Value,
) -> Result<Value, OdooError> {
    execute_op(pool, &tool.op, args).await
}

pub async fn execute_op(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    match op.op_type.as_str() {
        "search" => op_search(pool, op, args).await,
        "search_read" => op_search_read(pool, op, args).await,
        "read" => op_read(pool, op, args).await,
        "create" => op_create(pool, op, args).await,
        "write" => op_write(pool, op, args).await,
        "unlink" => op_unlink(pool, op, args).await,
        "search_count" => op_search_count(pool, op, args).await,
        "workflow_action" => op_workflow_action(pool, op, args).await,
        "execute" => op_execute(pool, op, args).await,
        "generate_report" => op_generate_report(pool, op, args).await,
        "get_model_metadata" => op_get_model_metadata(pool, op, args).await,
        "database_cleanup" => op_database_cleanup(pool, op, args).await,
        "deep_cleanup" => op_deep_cleanup(pool, op, args).await,
        "read_group" => op_read_group(pool, op, args).await,
        "name_search" => op_name_search(pool, op, args).await,
        "name_get" => op_name_get(pool, op, args).await,
        "default_get" => op_default_get(pool, op, args).await,
        "copy" => op_copy(pool, op, args).await,
        "onchange" => op_onchange(pool, op, args).await,
        "list_models" => op_list_models(pool, op, args).await,
        "check_access" => op_check_access(pool, op, args).await,
        "create_batch" => op_create_batch(pool, op, args).await,
        other => Err(OdooError::InvalidResponse(format!(
            "Unknown op.type: {other}"
        ))),
    }
}

fn ptr<'a>(args: &'a Value, op: &'a OpSpec, key: &str) -> Option<&'a Value> {
    op.map.get(key).and_then(|p| args.pointer(p))
}

fn req_str(args: &Value, op: &OpSpec, key: &str) -> Result<String, OdooError> {
    let v = ptr(args, op, key).ok_or_else(|| {
        OdooError::InvalidResponse(format!("Missing required argument '{key}' (map)"))
    })?;
    v.as_str()
        .map(|s| s.to_string())
        .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be string")))
}

fn opt_str(args: &Value, op: &OpSpec, key: &str) -> Result<Option<String>, OdooError> {
    match ptr(args, op, key) {
        None => Ok(None),
        Some(v) if v.is_null() => Ok(None),
        Some(v) => v
            .as_str()
            .map(|s| Some(s.to_string()))
            .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be string"))),
    }
}

fn opt_i64(args: &Value, op: &OpSpec, key: &str) -> Result<Option<i64>, OdooError> {
    match ptr(args, op, key) {
        None => Ok(None),
        Some(v) if v.is_null() => Ok(None),
        Some(v) => v
            .as_i64()
            .map(Some)
            .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be integer"))),
    }
}

fn opt_bool(args: &Value, op: &OpSpec, key: &str) -> Result<Option<bool>, OdooError> {
    match ptr(args, op, key) {
        None => Ok(None),
        Some(v) if v.is_null() => Ok(None),
        Some(v) => v
            .as_bool()
            .map(Some)
            .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be boolean"))),
    }
}

fn opt_value(args: &Value, op: &OpSpec, key: &str) -> Option<Value> {
    ptr(args, op, key).cloned().filter(|v| !v.is_null())
}

fn req_value(args: &Value, op: &OpSpec, key: &str) -> Result<Value, OdooError> {
    ptr(args, op, key).cloned().ok_or_else(|| {
        OdooError::InvalidResponse(format!("Missing required argument '{key}' (map)"))
    })
}

fn opt_vec_string(args: &Value, op: &OpSpec, key: &str) -> Result<Option<Vec<String>>, OdooError> {
    let Some(v) = ptr(args, op, key) else {
        return Ok(None);
    };
    if v.is_null() {
        return Ok(None);
    }
    let arr = v
        .as_array()
        .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be array")))?;
    let mut out = Vec::new();
    for x in arr {
        let s = x.as_str().ok_or_else(|| {
            OdooError::InvalidResponse(format!("Argument '{key}' items must be string"))
        })?;
        out.push(s.to_string());
    }
    Ok(Some(out))
}

fn opt_vec_i64(args: &Value, op: &OpSpec, key: &str) -> Result<Option<Vec<i64>>, OdooError> {
    let Some(v) = ptr(args, op, key) else {
        return Ok(None);
    };
    if v.is_null() {
        return Ok(None);
    }
    let arr = v
        .as_array()
        .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be array")))?;
    let mut out = Vec::new();
    for x in arr {
        let n = x.as_i64().ok_or_else(|| {
            OdooError::InvalidResponse(format!("Argument '{key}' items must be integer"))
        })?;
        out.push(n);
    }
    Ok(Some(out))
}

fn req_vec_i64(args: &Value, op: &OpSpec, key: &str) -> Result<Vec<i64>, OdooError> {
    let v = ptr(args, op, key).ok_or_else(|| {
        OdooError::InvalidResponse(format!("Missing required argument '{key}' (map)"))
    })?;
    let arr = v
        .as_array()
        .ok_or_else(|| OdooError::InvalidResponse(format!("Argument '{key}' must be array")))?;
    let mut out = Vec::new();
    for x in arr {
        let n = x.as_i64().ok_or_else(|| {
            OdooError::InvalidResponse(format!("Argument '{key}' items must be integer"))
        })?;
        out.push(n);
    }
    Ok(out)
}

fn ok_text(payload: Value) -> Value {
    json!({
        "content": [{
            "type": "text",
            "text": serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
        }]
    })
}

async fn op_search(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    let domain = opt_value(&args, op, "domain");
    let limit = opt_i64(&args, op, "limit")?;
    let offset = opt_i64(&args, op, "offset")?;
    let order = opt_str(&args, op, "order")?;
    let context = opt_value(&args, op, "context");

    let ids = client
        .search(&model, domain, limit, offset, order, context)
        .await?;
    Ok(ok_text(json!({ "ids": ids, "count": ids.len() })))
}

async fn op_search_read(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    let domain = opt_value(&args, op, "domain");
    let fields = opt_vec_string(&args, op, "fields")?;
    let limit = opt_i64(&args, op, "limit")?;
    let offset = opt_i64(&args, op, "offset")?;
    let order = opt_str(&args, op, "order")?;
    let context = opt_value(&args, op, "context");

    let records = client
        .search_read(&model, domain, fields, limit, offset, order, context)
        .await?;
    let count = records.as_array().map(|a| a.len()).unwrap_or(0);
    Ok(ok_text(json!({ "records": records, "count": count })))
}

async fn op_read(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let fields = opt_vec_string(&args, op, "fields")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let records = client.read(&model, ids, fields, context).await?;
    Ok(ok_text(json!({ "records": records })))
}

async fn op_create(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let values = req_value(&args, op, "values")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let id = client.create(&model, values, context).await?;
    Ok(ok_text(json!({ "id": id, "success": true })))
}

async fn op_write(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let values = req_value(&args, op, "values")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let ok = client.write(&model, ids.clone(), values, context).await?;
    Ok(ok_text(
        json!({ "success": ok, "updated_count": ids.len() }),
    ))
}

async fn op_unlink(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let ok = client.unlink(&model, ids.clone(), context).await?;
    Ok(ok_text(
        json!({ "success": ok, "deleted_count": ids.len() }),
    ))
}

async fn op_search_count(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let domain = opt_value(&args, op, "domain");
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let count = client.search_count(&model, domain, context).await?;
    Ok(ok_text(json!({ "count": count })))
}

async fn op_workflow_action(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let action = req_str(&args, op, "action")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let params = Map::new();
    let result = client
        .call_named(&model, &action, Some(ids.clone()), params, context)
        .await?;
    Ok(ok_text(json!({ "result": result, "executed_on": ids })))
}

async fn op_execute(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let method = req_str(&args, op, "method")?;
    let args_val = ptr(&args, op, "args").cloned().unwrap_or(Value::Null);
    let kwargs_val = ptr(&args, op, "kwargs").cloned().unwrap_or(Value::Null);
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    let mut params = Map::new();
    let mut ids: Option<Vec<i64>> = None;

    if !args_val.is_null() {
        match args_val {
            Value::Array(arr) => {
                if arr.len() == 1 {
                    if let Some(Value::Array(inner)) = arr.first() {
                        let maybe_ids: Option<Vec<i64>> =
                            inner.iter().map(|x| x.as_i64()).collect::<Option<Vec<_>>>();
                        if maybe_ids.is_some() {
                            ids = maybe_ids;
                        } else {
                            params.insert("args".to_string(), Value::Array(arr));
                        }
                    } else {
                        params.insert("args".to_string(), Value::Array(arr));
                    }
                } else {
                    params.insert("args".to_string(), Value::Array(arr));
                }
            }
            Value::Object(map) => {
                for (k, v) in map {
                    params.insert(k, v);
                }
            }
            other => {
                params.insert("arg".to_string(), other);
            }
        }
    }

    if let Value::Object(map) = kwargs_val {
        for (k, v) in map {
            params.insert(k, v);
        }
    } else if !kwargs_val.is_null() {
        params.insert("kwargs".to_string(), kwargs_val);
    }

    let result = client
        .call_named(&model, &method, ids, params, context)
        .await?;
    Ok(ok_text(json!({ "result": result })))
}

async fn op_generate_report(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let report_name = req_str(&args, op, "reportName")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    let pdf_bytes = client.download_report_pdf(&report_name, &ids).await?;
    let pdf_base64 = base64::engine::general_purpose::STANDARD.encode(pdf_bytes);
    Ok(ok_text(json!({
        "pdf_base64": pdf_base64,
        "report_name": report_name,
        "record_ids": ids
    })))
}

async fn op_get_model_metadata(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let context = opt_value(&args, op, "context");

    // Get cache TTL from environment (default: 300 seconds, 0 disables cache)
    let cache_ttl_secs: u64 = std::env::var("ODOO_METADATA_CACHE_TTL_SECS")
        .unwrap_or_else(|_| "300".to_string())
        .parse()
        .unwrap_or(300);

    // Check cache if TTL > 0
    if cache_ttl_secs > 0 {
        if let Some(cached) = pool.metadata_cache.get(&instance, &model).await {
            return Ok(ok_text(cached));
        }
    }

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let fields = client.fields_get(&model, context.clone()).await?;

    let domain = json!([["model", "=", model]]);
    let info = client
        .search_read(
            "ir.model",
            Some(domain),
            Some(vec!["name".to_string(), "model".to_string()]),
            Some(1),
            None,
            None,
            context,
        )
        .await?;

    let description = info
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|o| o.get("name"))
        .and_then(|v| v.as_str())
        .unwrap_or(&model)
        .to_string();

    let metadata = json!({
        "model": {
            "name": model,
            "description": description,
            "fields": fields
        }
    });

    // Insert into cache if TTL > 0
    if cache_ttl_secs > 0 {
        pool.metadata_cache.insert(&instance, &model, metadata.clone(), cache_ttl_secs).await;
    }

    Ok(ok_text(metadata))
}


async fn op_database_cleanup(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let report = cleanup::database::execute_full_cleanup(
        &client,
        cleanup::database::CleanupOptions {
            remove_test_data: opt_bool(&args, op, "removeTestData")?,
            remove_inactive_records: opt_bool(&args, op, "removeInactivRecords")?,
            cleanup_drafts: opt_bool(&args, op, "cleanupDrafts")?,
            archive_old_records: opt_bool(&args, op, "archiveOldRecords")?,
            optimize_database: opt_bool(&args, op, "optimizeDatabase")?,
            days_threshold: opt_i64(&args, op, "daysThreshold")?,
            dry_run: opt_bool(&args, op, "dryRun")?,
        },
    )
    .await?;
    let v = serde_json::to_value(&report).unwrap_or_else(|_| json!({}));
    Ok(ok_text(v))
}

async fn op_deep_cleanup(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let report = cleanup::deep::execute_deep_cleanup(
        &client,
        cleanup::deep::DeepCleanupOptions {
            dry_run: Some(opt_bool(&args, op, "dryRun")?.unwrap_or(true)),
            keep_company_defaults: opt_bool(&args, op, "keepCompanyDefaults")?,
            keep_user_accounts: opt_bool(&args, op, "keepUserAccounts")?,
            keep_menus: opt_bool(&args, op, "keepMenus")?,
            keep_groups: opt_bool(&args, op, "keepGroups")?,
        },
    )
    .await?;
    let v = serde_json::to_value(&report).unwrap_or_else(|_| json!({}));
    Ok(ok_text(v))
}

async fn op_read_group(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let fields = opt_vec_string(&args, op, "fields")?.unwrap_or_default();
    let groupby = opt_vec_string(&args, op, "groupby")?.unwrap_or_default();
    let domain = opt_value(&args, op, "domain");
    let offset = opt_i64(&args, op, "offset")?;
    let limit = opt_i64(&args, op, "limit")?;
    let orderby = opt_str(&args, op, "orderby")?;
    let lazy = opt_bool(&args, op, "lazy")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let result = client
        .read_group(
            &model, domain, fields, groupby, offset, limit, orderby, lazy, context,
        )
        .await?;
    Ok(ok_text(json!({ "groups": result })))
}

async fn op_name_search(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let name = opt_str(&args, op, "name")?;
    let domain = opt_value(&args, op, "args");
    let operator = opt_str(&args, op, "operator")?;
    let limit = opt_i64(&args, op, "limit")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let result = client
        .name_search(&model, name, domain, operator, limit, context)
        .await?;
    Ok(ok_text(json!({ "results": result })))
}

async fn op_name_get(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let result = client.name_get(&model, ids, context).await?;
    Ok(ok_text(json!({ "names": result })))
}

async fn op_default_get(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let fields_list = opt_vec_string(&args, op, "fields")?.unwrap_or_default();
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let result = client.default_get(&model, fields_list, context).await?;
    Ok(ok_text(json!({ "defaults": result })))
}

async fn op_copy(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let id = opt_i64(&args, op, "id")?
        .ok_or_else(|| OdooError::InvalidResponse("Missing required argument 'id'".to_string()))?;
    let default = opt_value(&args, op, "default");
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let new_id = client.copy(&model, id, default, context).await?;
    Ok(ok_text(json!({ "id": new_id, "success": true })))
}

async fn op_onchange(pool: &OdooClientPool, op: &OpSpec, args: Value) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let ids = req_vec_i64(&args, op, "ids")?;
    let values = req_value(&args, op, "values")?;
    let field_name = opt_vec_string(&args, op, "fieldName")?.unwrap_or_default();
    let field_onchange = opt_value(&args, op, "fieldOnchange").unwrap_or(json!({}));
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;
    let result = client
        .onchange(&model, ids, values, field_name, field_onchange, context)
        .await?;
    Ok(ok_text(json!({ "result": result })))
}

async fn op_list_models(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let domain = opt_value(&args, op, "domain")
        .unwrap_or_else(|| json!([["transient", "=", false]]));
    let limit = opt_i64(&args, op, "limit")?;
    let offset = opt_i64(&args, op, "offset")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    let models = client
        .search_read(
            "ir.model",
            Some(domain),
            Some(vec!["model".to_string(), "name".to_string()]),
            limit,
            offset,
            None,
            context,
        )
        .await?;

    Ok(ok_text(json!({ "models": models })))
}

async fn op_check_access(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let operation = req_str(&args, op, "operation")?;
    let ids = opt_vec_i64(&args, op, "ids")?;
    let context = opt_value(&args, op, "context");

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    // Check model-level access rights
    let mut params = serde_json::Map::new();
    params.insert("operation".to_string(), json!(operation));

    let access_result = client
        .call_named(&model, "check_access_rights", None, params.clone(), context.clone())
        .await?;

    // If IDs provided, also check record-level access rules
    let record_result = if let Some(record_ids) = ids {
        let ids_array: Vec<i64> = record_ids.clone();
        client
            .call_named(
                &model,
                "check_access_rule",
                Some(ids_array),
                params,
                context,
            )
            .await
            .ok()
    } else {
        None
    };

    let result = json!({
        "has_access": true,
        "model": model,
        "operation": operation,
        "model_level": access_result,
        "record_level": record_result
    });

    Ok(ok_text(result))
}

async fn op_create_batch(
    pool: &OdooClientPool,
    op: &OpSpec,
    args: Value,
) -> Result<Value, OdooError> {
    let instance = req_str(&args, op, "instance")?;
    let model = req_str(&args, op, "model")?;
    let values_array = req_value(&args, op, "values")?;
    let context = opt_value(&args, op, "context");

    // Validate values is an array
    let values_list = values_array.as_array().ok_or_else(|| {
        OdooError::InvalidResponse("'values' must be an array".to_string())
    })?;

    // Limit batch size to 100 to prevent abuse
    if values_list.len() > 100 {
        return Err(OdooError::InvalidResponse(
            "Batch size limited to 100 records".to_string(),
        ));
    }

    let client = pool
        .get(&instance)
        .await
        .map_err(|e| OdooError::InvalidResponse(e.to_string()))?;

    let mut created_ids = Vec::new();
    for values in values_list {
        let id = client.create(&model, values.clone(), context.clone()).await?;
        created_ids.push(id);
    }

    Ok(ok_text(json!({
        "ids": created_ids,
        "count": created_ids.len()
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn make_op(map: HashMap<String, String>) -> OpSpec {
        OpSpec {
            op_type: "test".to_string(),
            map,
        }
    }

    #[test]
    fn test_ptr_finds_value_by_json_pointer() {
        let args = json!({
            "data": {
                "name": "test"
            }
        });
        let mut map = HashMap::new();
        map.insert("key".to_string(), "/data/name".to_string());
        let op = make_op(map);

        let result = ptr(&args, &op, "key");
        assert_eq!(result, Some(&json!("test")));
    }

    #[test]
    fn test_ptr_returns_none_for_missing_key() {
        let args = json!({"data": "value"});
        let op = make_op(HashMap::new());

        let result = ptr(&args, &op, "nonexistent");
        assert!(result.is_none());
    }

    #[test]
    fn test_ptr_returns_none_for_invalid_pointer() {
        let args = json!({"data": "value"});
        let mut map = HashMap::new();
        map.insert("key".to_string(), "/nonexistent/path".to_string());
        let op = make_op(map);

        let result = ptr(&args, &op, "key");
        assert!(result.is_none());
    }

    #[test]
    fn test_req_str_success() {
        let args = json!({"name": "test"});
        let mut map = HashMap::new();
        map.insert("name".to_string(), "/name".to_string());
        let op = make_op(map);

        let result = req_str(&args, &op, "name").unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_req_str_missing_returns_error() {
        let args = json!({});
        let op = make_op(HashMap::new());

        let result = req_str(&args, &op, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_req_str_non_string_returns_error() {
        let args = json!({"name": 123});
        let mut map = HashMap::new();
        map.insert("name".to_string(), "/name".to_string());
        let op = make_op(map);

        let result = req_str(&args, &op, "name");
        assert!(result.is_err());
    }

    #[test]
    fn test_opt_str_success() {
        let args = json!({"name": "test"});
        let mut map = HashMap::new();
        map.insert("name".to_string(), "/name".to_string());
        let op = make_op(map);

        let result = opt_str(&args, &op, "name").unwrap();
        assert_eq!(result, Some("test".to_string()));
    }

    #[test]
    fn test_opt_str_missing_returns_none() {
        let args = json!({});
        let op = make_op(HashMap::new());

        let result = opt_str(&args, &op, "name").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_opt_str_null_returns_none() {
        let args = json!({"name": null});
        let mut map = HashMap::new();
        map.insert("name".to_string(), "/name".to_string());
        let op = make_op(map);

        let result = opt_str(&args, &op, "name").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_opt_i64_success() {
        let args = json!({"count": 42});
        let mut map = HashMap::new();
        map.insert("count".to_string(), "/count".to_string());
        let op = make_op(map);

        let result = opt_i64(&args, &op, "count").unwrap();
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_opt_i64_missing_returns_none() {
        let args = json!({});
        let op = make_op(HashMap::new());

        let result = opt_i64(&args, &op, "count").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_opt_i64_null_returns_none() {
        let args = json!({"count": null});
        let mut map = HashMap::new();
        map.insert("count".to_string(), "/count".to_string());
        let op = make_op(map);

        let result = opt_i64(&args, &op, "count").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_opt_bool_success() {
        let args = json!({"active": true});
        let mut map = HashMap::new();
        map.insert("active".to_string(), "/active".to_string());
        let op = make_op(map);

        let result = opt_bool(&args, &op, "active").unwrap();
        assert_eq!(result, Some(true));
    }

    #[test]
    fn test_opt_bool_missing_returns_none() {
        let args = json!({});
        let op = make_op(HashMap::new());

        let result = opt_bool(&args, &op, "active").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_ok_text_format() {
        let result = ok_text(json!({"data": "test"}));
        assert!(result.is_object());
        assert!(result.get("content").is_some());
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1);
        assert_eq!(content[0]["type"], "text");
    }

    #[test]
    fn test_ok_text_contains_json() {
        let data = json!({"key": "value"});
        let result = ok_text(data);
        let text = result["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("key"));
        assert!(text.contains("value"));
    }

    #[test]
    fn test_opt_vec_i64_success() {
        let args = json!({"ids": [1, 2, 3]});
        let mut map = HashMap::new();
        map.insert("ids".to_string(), "/ids".to_string());
        let op = make_op(map);

        let result = opt_vec_i64(&args, &op, "ids").unwrap();
        assert_eq!(result, Some(vec![1, 2, 3]));
    }

    #[test]
    fn test_opt_vec_i64_missing_returns_none() {
        let args = json!({});
        let op = make_op(HashMap::new());

        let result = opt_vec_i64(&args, &op, "ids").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_opt_vec_i64_null_returns_none() {
        let args = json!({"ids": null});
        let mut map = HashMap::new();
        map.insert("ids".to_string(), "/ids".to_string());
        let op = make_op(map);

        let result = opt_vec_i64(&args, &op, "ids").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_opt_vec_i64_invalid_item_returns_error() {
        let args = json!({"ids": [1, "two", 3]});
        let mut map = HashMap::new();
        map.insert("ids".to_string(), "/ids".to_string());
        let op = make_op(map);

        let result = opt_vec_i64(&args, &op, "ids");
        assert!(result.is_err());
    }

    #[test]
    fn test_opt_vec_i64_empty_array() {
        let args = json!({"ids": []});
        let mut map = HashMap::new();
        map.insert("ids".to_string(), "/ids".to_string());
        let op = make_op(map);

        let result = opt_vec_i64(&args, &op, "ids").unwrap();
        assert_eq!(result, Some(vec![]));
    }
}
