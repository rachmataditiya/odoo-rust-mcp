//! Tests for Odoo configuration types and parsing
#[cfg(test)]
mod tests {
    use rust_mcp::odoo::config::{OdooEnvConfig, OdooInstanceConfig, OdooAuthMode};
    use std::collections::HashMap;

    #[test]
    fn test_odoo_instance_config_creation() {
        let config = OdooInstanceConfig {
            url: "http://localhost:8069".to_string(),
            db: Some("test_db".to_string()),
            api_key: Some("test_key".to_string()),
            username: None,
            password: None,
            version: Some("19".to_string()),
            timeout_ms: Some(30_000),
            max_retries: Some(3),
            extra: HashMap::new(),
        };

        assert_eq!(config.url, "http://localhost:8069");
        assert_eq!(config.db, Some("test_db".to_string()));
        assert_eq!(config.api_key, Some("test_key".to_string()));
        assert_eq!(config.timeout_ms, Some(30_000));
        assert_eq!(config.max_retries, Some(3));
    }

    #[test]
    fn test_odoo_instance_config_optional_fields() {
        let config = OdooInstanceConfig {
            url: "http://localhost:8069".to_string(),
            db: None,
            api_key: None,
            username: None,
            password: None,
            version: None,
            timeout_ms: None,
            max_retries: None,
            extra: HashMap::new(),
        };

        assert_eq!(config.db, None);
        assert_eq!(config.api_key, None);
        assert_eq!(config.timeout_ms, None);
        assert_eq!(config.max_retries, None);
    }

    #[test]
    fn test_odoo_env_config_creation() {
        let mut instances = HashMap::new();
        instances.insert(
            "prod".to_string(),
            OdooInstanceConfig {
                url: "https://prod.example.com".to_string(),
                db: Some("production".to_string()),
                api_key: Some("prod_key".to_string()),
                username: None,
                password: None,
                version: Some("19".to_string()),
                timeout_ms: Some(30_000),
                max_retries: Some(5),
                extra: HashMap::new(),
            },
        );

        let config = OdooEnvConfig { instances };

        assert!(config.instances.contains_key("prod"));
        let prod = config.instances.get("prod").unwrap();
        assert_eq!(prod.url, "https://prod.example.com");
    }

    #[test]
    fn test_odoo_env_config_multiple_instances() {
        let mut instances = HashMap::new();

        instances.insert(
            "dev".to_string(),
            OdooInstanceConfig {
                url: "http://dev.local:8069".to_string(),
                db: Some("dev".to_string()),
                api_key: Some("dev_key".to_string()),
                username: None,
                password: None,
                version: Some("19".to_string()),
                timeout_ms: Some(20_000),
                max_retries: Some(2),
                extra: HashMap::new(),
            },
        );

        instances.insert(
            "staging".to_string(),
            OdooInstanceConfig {
                url: "http://staging.local:8069".to_string(),
                db: Some("staging".to_string()),
                api_key: Some("staging_key".to_string()),
                username: None,
                password: None,
                version: Some("19".to_string()),
                timeout_ms: Some(25_000),
                max_retries: Some(3),
                extra: HashMap::new(),
            },
        );

        let config = OdooEnvConfig { instances };

        assert_eq!(config.instances.len(), 2);
        assert!(config.instances.contains_key("staging"));
    }

    #[test]
    fn test_auth_mode_api_key_default() {
        let config = OdooInstanceConfig {
            url: "https://example.odoo.com".to_string(),
            api_key: Some("token123".to_string()),
            username: None,
            password: None,
            version: Some("19".to_string()),
            db: None,
            timeout_ms: None,
            max_retries: None,
            extra: HashMap::new(),
        };

        assert_eq!(config.auth_mode(), OdooAuthMode::ApiKey);
    }

    #[test]
    fn test_auth_mode_password_with_credentials() {
        let config = OdooInstanceConfig {
            url: "https://example.odoo.com".to_string(),
            username: Some("admin".to_string()),
            password: Some("password".to_string()),
            api_key: None,
            version: Some("17".to_string()),
            db: None,
            timeout_ms: None,
            max_retries: None,
            extra: HashMap::new(),
        };

        assert_eq!(config.auth_mode(), OdooAuthMode::Password);
    }
}
