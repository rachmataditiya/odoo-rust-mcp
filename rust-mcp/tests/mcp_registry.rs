//! Tests for MCP Registry and tool definitions
#[cfg(test)]
mod tests {
    use rust_mcp::mcp::registry::{OpSpec, ToolDef};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn test_tool_def_creation() {
        let tool = ToolDef {
            name: "search_partners".to_string(),
            description: "Search for partners in Odoo".to_string(),
            op: OpSpec {
                op_type: "search".to_string(),
                map: Default::default(),
            },
            input_schema: json!({
                "type": "object",
                "properties": {
                    "search_term": { "type": "string" }
                }
            }),
            guards: None,
        };

        assert_eq!(tool.name, "search_partners");
        assert_eq!(tool.op.op_type, "search");
        assert!(tool.op.map.is_empty());
    }

    #[test]
    fn test_op_spec_with_map_entries() {
        let mut map = HashMap::new();
        map.insert("model".to_string(), "res.partner".to_string());
        map.insert("domain".to_string(), "[['name', 'ilike', 'acme']]".to_string());

        let op = OpSpec {
            op_type: "search".to_string(),
            map,
        };

        assert_eq!(op.op_type, "search");
        assert_eq!(op.map.len(), 2);
        assert_eq!(op.map.get("model"), Some(&"res.partner".to_string()));
        assert_eq!(
            op.map.get("domain"),
            Some(&"[['name', 'ilike', 'acme']]".to_string())
        );
    }

    #[test]
    fn test_op_spec_create_operation() {
        let op = OpSpec {
            op_type: "create".to_string(),
            map: Default::default(),
        };

        assert_eq!(op.op_type, "create");
        assert!(op.map.is_empty());
    }

    #[test]
    fn test_op_spec_read_operation() {
        let mut map = HashMap::new();
        map.insert("model".to_string(), "account.move".to_string());
        map.insert("record_id".to_string(), "123".to_string());

        let op = OpSpec {
            op_type: "read".to_string(),
            map,
        };

        assert_eq!(op.op_type, "read");
        assert_eq!(op.map.len(), 2);
        assert_eq!(op.map.get("record_id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_tool_def_with_guards() {
        use rust_mcp::mcp::registry::ToolGuards;

        let tool = ToolDef {
            name: "admin_tool".to_string(),
            description: "Admin-only tool".to_string(),
            op: OpSpec {
                op_type: "admin_action".to_string(),
                map: Default::default(),
            },
            input_schema: json!({"type": "object"}),
            guards: Some(ToolGuards {
                requires_env_true: Some("ADMIN_MODE".to_string()),
            }),
        };

        assert_eq!(tool.name, "admin_tool");
        assert!(tool.guards.is_some());
        let guards = tool.guards.unwrap();
        assert_eq!(guards.requires_env_true, Some("ADMIN_MODE".to_string()));
    }

    #[test]
    fn test_op_spec_deserialization() {
        let json_str = r#"{"type": "create", "map": {"model": "sale.order"}}"#;
        let op: OpSpec = serde_json::from_str(json_str).expect("Failed to deserialize");

        assert_eq!(op.op_type, "create");
        assert_eq!(op.map.get("model"), Some(&"sale.order".to_string()));
    }

    #[test]
    fn test_tool_def_deserialization() {
        let json_str = r#"{
            "name": "list_products",
            "description": "List all products",
            "inputSchema": {"type": "object"},
            "op": {"type": "search", "map": {"model": "product.product"}}
        }"#;
        let tool: ToolDef = serde_json::from_str(json_str).expect("Failed to deserialize");

        assert_eq!(tool.name, "list_products");
        assert_eq!(tool.op.op_type, "search");
        assert_eq!(tool.op.map.get("model"), Some(&"product.product".to_string()));
    }
}
