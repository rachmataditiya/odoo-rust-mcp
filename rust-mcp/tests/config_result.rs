//! Tests for ConfigResult type and methods
#[cfg(test)]
mod tests {
    use rust_mcp::config_manager::manager::ConfigResult;

    #[test]
    fn test_config_result_ok() {
        let result = ConfigResult::ok("Test message");
        assert!(result.success);
        assert_eq!(result.message, "Test message");
        assert_eq!(result.warning, None);
        assert!(!result.rollback_performed);
    }

    #[test]
    fn test_config_result_error() {
        let result = ConfigResult::error("Error message");
        assert!(!result.success);
        assert_eq!(result.message, "Error message");
        assert_eq!(result.warning, None);
        assert!(!result.rollback_performed);
    }

    #[test]
    fn test_config_result_with_warning() {
        let result = ConfigResult::ok("Success").with_warning("Warning message");
        assert!(result.success);
        assert_eq!(result.message, "Success");
        assert_eq!(result.warning, Some("Warning message".to_string()));
        assert!(!result.rollback_performed);
    }

    #[test]
    fn test_config_result_with_rollback() {
        let result = ConfigResult::error("Failed").with_rollback();
        assert!(!result.success);
        assert_eq!(result.message, "Failed");
        assert!(result.rollback_performed);
    }

    #[test]
    fn test_config_result_chaining() {
        let result = ConfigResult::ok("Saved")
            .with_warning("Deprecated field")
            .with_rollback();

        assert!(result.success);
        assert_eq!(result.message, "Saved");
        assert_eq!(result.warning, Some("Deprecated field".to_string()));
        assert!(result.rollback_performed);
    }

    #[test]
    fn test_config_result_from_string() {
        let result = ConfigResult::ok("Message from String".to_string());
        assert!(result.success);
        assert_eq!(result.message, "Message from String");

        let result_err = ConfigResult::error("Error from String".to_string());
        assert!(!result_err.success);
        assert_eq!(result_err.message, "Error from String");
    }

    #[test]
    fn test_config_result_multiple_warnings() {
        let result = ConfigResult::ok("Saved").with_warning("First warning");
        // Note: Last warning overwrites previous
        let result = result.with_warning("Second warning");
        assert_eq!(result.warning, Some("Second warning".to_string()));
    }
}
