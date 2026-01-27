//! Tests for Odoo error types and handling
#[cfg(test)]
mod tests {
    use rust_mcp::odoo::types::{OdooError, OdooErrorBody};
    use serde_json::json;

    #[test]
    fn test_odoo_error_invalid_response() {
        let error = OdooError::InvalidResponse("Bad response".to_string());
        match error {
            OdooError::InvalidResponse(msg) => assert_eq!(msg, "Bad response"),
            _ => panic!("Expected InvalidResponse"),
        }
    }

    #[test]
    fn test_odoo_error_api_with_body() {
        let body = OdooErrorBody {
            name: Some("AccessDenied".to_string()),
            message: Some("Access denied".to_string()),
            arguments: vec![],
            context: json!({}),
            debug: None,
        };

        let error = OdooError::Api {
            status: 403,
            message: "Forbidden".to_string(),
            body: Some(body),
        };

        let display = format!("{}", error);
        assert!(display.contains("403"));
        assert!(display.contains("Forbidden"));
    }

    #[test]
    fn test_odoo_error_api_without_body() {
        let error = OdooError::Api {
            status: 401,
            message: "Unauthorized".to_string(),
            body: None,
        };

        let display = format!("{}", error);
        assert!(display.contains("401"));
        assert!(display.contains("Unauthorized"));
    }

    #[test]
    fn test_odoo_error_display() {
        let error = OdooError::InvalidResponse("Test error".to_string());
        let display_string = format!("{}", error);
        assert!(display_string.contains("Test error"));
    }

    #[test]
    fn test_odoo_error_debug() {
        let error = OdooError::InvalidResponse("Debug test".to_string());
        let debug_string = format!("{:?}", error);
        assert!(debug_string.contains("InvalidResponse"));
    }

    #[test]
    fn test_odoo_error_body_creation() {
        let body = OdooErrorBody {
            name: Some("ServerError".to_string()),
            message: Some("Internal Server Error".to_string()),
            arguments: vec![json!("arg1"), json!(123)],
            context: json!({}),
            debug: Some("Traceback info".to_string()),
        };

        assert_eq!(body.name, Some("ServerError".to_string()));
        assert_eq!(body.message, Some("Internal Server Error".to_string()));
        assert_eq!(body.arguments.len(), 2);
        assert_eq!(body.debug, Some("Traceback info".to_string()));
    }

    #[test]
    fn test_odoo_error_body_with_none_fields() {
        let body = OdooErrorBody {
            name: None,
            message: None,
            arguments: vec![],
            context: json!({}),
            debug: None,
        };

        assert_eq!(body.name, None);
        assert_eq!(body.message, None);
        assert!(body.arguments.is_empty());
        assert_eq!(body.debug, None);
    }

    #[test]
    fn test_odoo_error_body_deserialization() {
        let json_str = r#"{
            "name": "odoo.exceptions.AccessDenied",
            "message": "Access Denied",
            "arguments": ["arg1", 123],
            "debug": "traceback here"
        }"#;

        let body: OdooErrorBody = serde_json::from_str(json_str).unwrap();
        assert_eq!(
            body.name,
            Some("odoo.exceptions.AccessDenied".to_string())
        );
        assert_eq!(body.message, Some("Access Denied".to_string()));
        assert_eq!(body.arguments.len(), 2);
        assert_eq!(body.debug, Some("traceback here".to_string()));
    }

    #[test]
    fn test_odoo_error_chain_message() {
        let error = OdooError::InvalidResponse("First error".to_string());
        let error_message = format!("{}", error);
        assert!(error_message.contains("First error"));
    }

    #[test]
    fn test_multiple_error_types() {
        let error1 = OdooError::InvalidResponse("Invalid".to_string());
        let error2 = OdooError::Api {
            status: 404,
            message: "Not found".to_string(),
            body: None,
        };
        let error3 = OdooError::Api {
            status: 401,
            message: "Unauthorized".to_string(),
            body: None,
        };
        let error4 = OdooError::Api {
            status: 403,
            message: "Forbidden".to_string(),
            body: None,
        };

        let errors = vec![error1, error2, error3, error4];

        assert_eq!(errors.len(), 4);
        assert!(matches!(errors[0], OdooError::InvalidResponse(_)));
        assert!(matches!(errors[1], OdooError::Api { status: 404, .. }));
        assert!(matches!(errors[2], OdooError::Api { status: 401, .. }));
        assert!(matches!(errors[3], OdooError::Api { status: 403, .. }));
    }

    #[test]
    fn test_odoo_error_body_minimal_deserialization() {
        let json_str = r#"{}"#;
        let body: OdooErrorBody = serde_json::from_str(json_str).unwrap();
        assert!(body.name.is_none());
        assert!(body.message.is_none());
        assert!(body.arguments.is_empty());
    }
}
