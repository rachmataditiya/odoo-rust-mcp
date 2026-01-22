use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OdooErrorBody {
    pub name: Option<String>,
    pub message: Option<String>,
    #[serde(default)]
    pub arguments: Vec<Value>,
    #[serde(default)]
    pub context: Value,
    pub debug: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum OdooError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Odoo API error (status {status}): {message}")]
    Api {
        status: u16,
        message: String,
        body: Option<OdooErrorBody>,
    },

    #[error("Invalid response: {0}")]
    InvalidResponse(String),
}

pub type OdooResult<T> = Result<T, OdooError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_odoo_error_display_api_contains_status() {
        let err = OdooError::Api {
            status: 500,
            message: "Internal Server Error".to_string(),
            body: Some(OdooErrorBody {
                name: Some("ServerError".to_string()),
                message: Some("Something went wrong".to_string()),
                arguments: vec![],
                context: serde_json::Value::Null,
                debug: None,
            }),
        };
        let display = err.to_string();
        assert!(display.contains("500"));
        assert!(display.contains("Internal Server Error"));
    }

    #[test]
    fn test_odoo_error_display_api() {
        let err = OdooError::Api {
            status: 401,
            message: "Unauthorized".to_string(),
            body: None,
        };
        assert!(err.to_string().contains("401"));
        assert!(err.to_string().contains("Unauthorized"));
    }

    #[test]
    fn test_odoo_error_display_invalid_response() {
        let err = OdooError::InvalidResponse("missing field".to_string());
        assert!(err.to_string().contains("missing field"));
    }

    #[test]
    fn test_odoo_error_body_deserialize() {
        let json = r#"{
            "name": "odoo.exceptions.AccessDenied",
            "message": "Access Denied",
            "arguments": ["arg1", 123],
            "debug": "traceback here"
        }"#;
        let body: OdooErrorBody = serde_json::from_str(json).unwrap();
        assert_eq!(body.name, Some("odoo.exceptions.AccessDenied".to_string()));
        assert_eq!(body.message, Some("Access Denied".to_string()));
        assert_eq!(body.arguments.len(), 2);
        assert_eq!(body.debug, Some("traceback here".to_string()));
    }

    #[test]
    fn test_odoo_error_body_deserialize_minimal() {
        let json = r#"{}"#;
        let body: OdooErrorBody = serde_json::from_str(json).unwrap();
        assert!(body.name.is_none());
        assert!(body.message.is_none());
        assert!(body.arguments.is_empty());
    }
}
