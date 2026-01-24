use mcp_rust_sdk::error::{Error, ErrorCode};
use serde_json::{Value, json};

use crate::mcp::tools::OdooClientPool;

fn resource_err(message: impl Into<String>) -> Error {
    Error::protocol(ErrorCode::InvalidRequest, message)
}

/// MCP Resource URI parser and dispatcher
/// Supports odoo:// URI scheme with the following formats:
/// - odoo://instances - List all configured instances
/// - odoo://{instance}/models - List models for an instance
/// - odoo://{instance}/metadata/{model} - Get model metadata
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceUri {
    Instances,
    Models { instance: String },
    Metadata { instance: String, model: String },
}

impl ResourceUri {
    /// Parse an odoo:// URI into a ResourceUri
    pub fn parse(uri: &str) -> Result<Self, String> {
        if !uri.starts_with("odoo://") {
            return Err(format!(
                "Invalid URI scheme: expected 'odoo://', got '{}'",
                uri
            ));
        }

        let path = &uri[7..]; // Remove "odoo://"

        if path == "instances" {
            return Ok(ResourceUri::Instances);
        }

        // Split by first '/'
        let parts: Vec<&str> = path.splitn(2, '/').collect();

        match parts.as_slice() {
            [_instance] => {
                // Just instance name - this is invalid, need something after
                Err(format!("Invalid resource URI: {}", uri))
            }
            [instance, rest] => {
                // Parse the rest part
                if rest.starts_with("models") {
                    if *rest == "models" {
                        Ok(ResourceUri::Models {
                            instance: instance.to_string(),
                        })
                    } else {
                        Err(format!("Invalid models URI: {}", uri))
                    }
                } else if let Some(model) = rest.strip_prefix("metadata/") {
                    if model.is_empty() {
                        Err("Invalid metadata URI: missing model name".to_string())
                    } else {
                        Ok(ResourceUri::Metadata {
                            instance: instance.to_string(),
                            model: model.to_string(),
                        })
                    }
                } else {
                    Err(format!("Invalid resource type in URI: {}", uri))
                }
            }
            _ => Err(format!("Invalid resource URI: {}", uri)),
        }
    }

    /// Get the URI string representation
    pub fn to_uri(&self) -> String {
        match self {
            ResourceUri::Instances => "odoo://instances".to_string(),
            ResourceUri::Models { instance } => format!("odoo://{}/models", instance),
            ResourceUri::Metadata { instance, model } => {
                format!("odoo://{}/metadata/{}", instance, model)
            }
        }
    }
}

/// List all available resources
pub async fn list_resources(pool: &OdooClientPool) -> Result<Value, Error> {
    let mut resources = vec![];

    // Root resource: list of instances
    resources.push(json!({
        "uri": "odoo://instances",
        "name": "Odoo Instances",
        "description": "List of configured Odoo instances",
        "mimeType": "application/json"
    }));

    // Per-instance resources: models
    for instance in pool.instance_names() {
        resources.push(json!({
            "uri": format!("odoo://{}/models", instance),
            "name": format!("Models in {}", instance),
            "description": format!("List of accessible models in Odoo instance '{}'", instance),
            "mimeType": "application/json"
        }));
    }

    Ok(json!({
        "resources": resources
    }))
}

/// Read a specific resource by URI
pub async fn read_resource(pool: &OdooClientPool, uri: &str) -> Result<Value, Error> {
    let resource = ResourceUri::parse(uri).map_err(resource_err)?;

    match resource {
        ResourceUri::Instances => read_instances(pool).await,
        ResourceUri::Models { instance } => read_models(pool, &instance).await,
        ResourceUri::Metadata { instance, model } => read_metadata(pool, &instance, &model).await,
    }
}

/// Read the list of instances
async fn read_instances(pool: &OdooClientPool) -> Result<Value, Error> {
    let instances = pool.instance_names();
    let instance_list: Vec<Value> = instances
        .iter()
        .map(|name| {
            json!({
                "name": name,
            })
        })
        .collect();

    Ok(json!({
        "contents": [{
            "uri": "odoo://instances",
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&instance_list)
                .unwrap_or_else(|_| "[]".to_string())
        }]
    }))
}

/// Read the list of models for an instance
async fn read_models(pool: &OdooClientPool, instance: &str) -> Result<Value, Error> {
    let client = pool
        .get(instance)
        .await
        .map_err(|e| resource_err(e.to_string()))?;

    let models = client
        .search_read(
            "ir.model",
            Some(json!([])),
            Some(vec!["model".to_string(), "name".to_string()]),
            None,
            None,
            None,
            None,
        )
        .await
        .map_err(|e| resource_err(e.to_string()))?;

    let uri = format!("odoo://{}/models", instance);
    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&models)
                .unwrap_or_else(|_| "[]".to_string())
        }]
    }))
}

/// Read metadata for a specific model
async fn read_metadata(pool: &OdooClientPool, instance: &str, model: &str) -> Result<Value, Error> {
    let client = pool
        .get(instance)
        .await
        .map_err(|e| resource_err(e.to_string()))?;

    let fields = client
        .fields_get(model, None)
        .await
        .map_err(|e| resource_err(e.to_string()))?;

    let domain = json!([["model", "=", model]]);
    let info = client
        .search_read(
            "ir.model",
            Some(domain),
            Some(vec!["name".to_string(), "model".to_string()]),
            Some(1),
            None,
            None,
            None,
        )
        .await
        .map_err(|e| resource_err(e.to_string()))?;

    let description = info
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|o: &Value| o.get("name"))
        .and_then(|v: &Value| v.as_str())
        .unwrap_or(model)
        .to_string();

    let metadata = json!({
        "model": {
            "name": model,
            "description": description,
            "fields": fields
        }
    });

    let uri = format!("odoo://{}/metadata/{}", instance, model);
    Ok(json!({
        "contents": [{
            "uri": uri,
            "mimeType": "application/json",
            "text": serde_json::to_string_pretty(&metadata)
                .unwrap_or_else(|_| "{}".to_string())
        }]
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_instances_uri() {
        let uri = ResourceUri::parse("odoo://instances").unwrap();
        assert_eq!(uri, ResourceUri::Instances);
    }

    #[test]
    fn test_parse_models_uri() {
        let uri = ResourceUri::parse("odoo://prod/models").unwrap();
        assert_eq!(
            uri,
            ResourceUri::Models {
                instance: "prod".to_string()
            }
        );
    }

    #[test]
    fn test_parse_metadata_uri() {
        let uri = ResourceUri::parse("odoo://prod/metadata/sale.order").unwrap();
        assert_eq!(
            uri,
            ResourceUri::Metadata {
                instance: "prod".to_string(),
                model: "sale.order".to_string()
            }
        );
    }

    #[test]
    fn test_parse_invalid_scheme() {
        let result = ResourceUri::parse("http://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_resource_type() {
        let result = ResourceUri::parse("odoo://prod");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_resource_type() {
        let result = ResourceUri::parse("odoo://prod/invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_metadata_missing_model() {
        let result = ResourceUri::parse("odoo://prod/metadata/");
        assert!(result.is_err());
    }

    #[test]
    fn test_to_uri_instances() {
        let uri = ResourceUri::Instances;
        assert_eq!(uri.to_uri(), "odoo://instances");
    }

    #[test]
    fn test_to_uri_models() {
        let uri = ResourceUri::Models {
            instance: "prod".to_string(),
        };
        assert_eq!(uri.to_uri(), "odoo://prod/models");
    }

    #[test]
    fn test_to_uri_metadata() {
        let uri = ResourceUri::Metadata {
            instance: "prod".to_string(),
            model: "sale.order".to_string(),
        };
        assert_eq!(uri.to_uri(), "odoo://prod/metadata/sale.order");
    }

    #[test]
    fn test_roundtrip_instances() {
        let original = "odoo://instances";
        let parsed = ResourceUri::parse(original).unwrap();
        assert_eq!(parsed.to_uri(), original);
    }

    #[test]
    fn test_roundtrip_models() {
        let original = "odoo://production/models";
        let parsed = ResourceUri::parse(original).unwrap();
        assert_eq!(parsed.to_uri(), original);
    }

    #[test]
    fn test_roundtrip_metadata() {
        let original = "odoo://staging/metadata/account.invoice";
        let parsed = ResourceUri::parse(original).unwrap();
        assert_eq!(parsed.to_uri(), original);
    }

    #[test]
    fn test_metadata_with_special_chars() {
        let uri = "odoo://prod/metadata/sale.order.line";
        let parsed = ResourceUri::parse(uri).unwrap();
        assert_eq!(parsed.to_uri(), uri);
    }

    #[test]
    fn test_parse_models_hyphenated_instance() {
        let uri = ResourceUri::parse("odoo://prod-db/models").unwrap();
        assert_eq!(
            uri,
            ResourceUri::Models {
                instance: "prod-db".to_string()
            }
        );
    }

    #[test]
    fn test_parse_metadata_underscore_instance() {
        let uri = ResourceUri::parse("odoo://prod_db/metadata/res.partner").unwrap();
        assert_eq!(
            uri,
            ResourceUri::Metadata {
                instance: "prod_db".to_string(),
                model: "res.partner".to_string()
            }
        );
    }

    #[test]
    fn test_parse_metadata_complex_model_name() {
        let uri = ResourceUri::parse("odoo://prod/metadata/account.invoice.line.tax").unwrap();
        assert_eq!(
            uri,
            ResourceUri::Metadata {
                instance: "prod".to_string(),
                model: "account.invoice.line.tax".to_string()
            }
        );
    }

    #[test]
    fn test_parse_empty_string() {
        let result = ResourceUri::parse("");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_just_odoo_prefix() {
        let result = ResourceUri::parse("odoo://");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_case_sensitive_scheme() {
        let result = ResourceUri::parse("ODOO://instances");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_case_sensitive_instances() {
        let result = ResourceUri::parse("odoo://Instances");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_whitespace_in_uri() {
        let result = ResourceUri::parse("odoo://prod /models");
        // This should parse but with spaces in instance name
        match result {
            Ok(ResourceUri::Models { instance }) => {
                assert_eq!(instance, "prod ");
            }
            _ => panic!("Expected Models variant"),
        }
    }

    #[test]
    fn test_parse_double_slash_in_model() {
        let result = ResourceUri::parse("odoo://prod/metadata/model//name");
        // This parses with the full remainder as model name
        match result {
            Ok(ResourceUri::Metadata { model, .. }) => {
                assert_eq!(model, "model//name");
            }
            _ => panic!("Expected Metadata variant"),
        }
    }

    #[test]
    fn test_resource_uri_debug() {
        let uri = ResourceUri::Instances;
        let debug_str = format!("{:?}", uri);
        assert!(debug_str.contains("Instances"));
    }

    #[test]
    fn test_resource_uri_clone() {
        let uri1 = ResourceUri::Metadata {
            instance: "prod".to_string(),
            model: "sale.order".to_string(),
        };
        let uri2 = uri1.clone();
        assert_eq!(uri1, uri2);
    }

    #[test]
    fn test_resource_uri_eq() {
        let uri1 = ResourceUri::Models {
            instance: "prod".to_string(),
        };
        let uri2 = ResourceUri::Models {
            instance: "prod".to_string(),
        };
        assert_eq!(uri1, uri2);
    }

    #[test]
    fn test_resource_uri_ne() {
        let uri1 = ResourceUri::Models {
            instance: "prod".to_string(),
        };
        let uri2 = ResourceUri::Models {
            instance: "staging".to_string(),
        };
        assert_ne!(uri1, uri2);
    }

    #[test]
    fn test_multiple_metadata_calls() {
        let uri1 = "odoo://prod/metadata/sale.order";
        let uri2 = "odoo://staging/metadata/sale.order";

        let parsed1 = ResourceUri::parse(uri1).unwrap();
        let parsed2 = ResourceUri::parse(uri2).unwrap();

        assert_ne!(parsed1, parsed2);
        assert_eq!(parsed1.to_uri(), uri1);
        assert_eq!(parsed2.to_uri(), uri2);
    }

    #[test]
    fn test_numeric_instance_name() {
        let uri = ResourceUri::parse("odoo://123/models").unwrap();
        match uri {
            ResourceUri::Models { instance } => {
                assert_eq!(instance, "123");
            }
            _ => panic!("Expected Models variant"),
        }
    }

    #[test]
    fn test_alphanumeric_model_name() {
        let uri = ResourceUri::parse("odoo://prod/metadata/model123v2").unwrap();
        match uri {
            ResourceUri::Metadata { model, .. } => {
                assert_eq!(model, "model123v2");
            }
            _ => panic!("Expected Metadata variant"),
        }
    }

    #[test]
    fn test_parse_models_extra_slash() {
        let result = ResourceUri::parse("odoo://prod/models/");
        assert!(result.is_err());
    }

    #[test]
    fn test_all_resource_types_to_uri() {
        let uris = vec![
            ("odoo://instances", ResourceUri::Instances),
            (
                "odoo://prod/models",
                ResourceUri::Models {
                    instance: "prod".to_string(),
                },
            ),
            (
                "odoo://prod/metadata/res.partner",
                ResourceUri::Metadata {
                    instance: "prod".to_string(),
                    model: "res.partner".to_string(),
                },
            ),
        ];

        for (original, parsed) in uris {
            assert_eq!(parsed.to_uri(), original);
        }
    }
}
