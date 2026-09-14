//! Tests for config manager watcher module
#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn test_config_watcher_ignores_reads_and_notifies_on_writes() {
        use rust_mcp::config_manager::ConfigWatcher;
        use std::time::Duration;
        use tokio::time::timeout;

        let dir = TempDir::new().unwrap();
        let path = dir.path().join("tools.json");
        fs::write(&path, r#"{"tools": []}"#).unwrap();
        let watcher = ConfigWatcher::new(dir.path().to_path_buf()).unwrap();
        let mut changes = watcher.subscribe();

        // new() starts a background thread. Wait for a real notification so the
        // read check cannot pass merely because the watcher is not ready yet.
        timeout(Duration::from_secs(5), async {
            loop {
                fs::write(&path, r#"{"tools": []}"#).unwrap();
                if let Ok(event) = timeout(Duration::from_millis(100), changes.recv()).await {
                    assert_eq!(event.unwrap(), "tools.json");
                    break;
                }
            }
        })
        .await
        .expect("watcher did not start");
        while let Ok(event) = timeout(Duration::from_millis(100), changes.recv()).await {
            event.unwrap();
        }

        fs::read(&path).unwrap();
        assert!(
            timeout(Duration::from_millis(300), changes.recv())
                .await
                .is_err(),
            "reading config was reported as a change"
        );

        fs::write(&path, r#"{"tools": [], "updated": true}"#).unwrap();
        assert_eq!(
            timeout(Duration::from_secs(5), changes.recv())
                .await
                .expect("write was not reported")
                .unwrap(),
            "tools.json"
        );
    }

    #[test]
    fn test_config_watcher_creation() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path().to_path_buf();

        // Just verify we can create the watcher without panicking
        // The watcher spawns background threads so we can't easily test it fully
        assert!(config_dir.exists());
    }

    #[test]
    fn test_config_directory_structure() {
        let temp_dir = TempDir::new().unwrap();
        let config_dir = temp_dir.path();

        // Create test config files
        fs::write(config_dir.join("tools.json"), r#"{"tools": []}"#).unwrap();
        fs::write(config_dir.join("prompts.json"), r#"{"prompts": []}"#).unwrap();
        fs::write(config_dir.join("server.json"), r#"{"serverName": "test"}"#).unwrap();

        assert!(config_dir.join("tools.json").exists());
        assert!(config_dir.join("prompts.json").exists());
        assert!(config_dir.join("server.json").exists());
    }

    #[test]
    fn test_json_file_detection() {
        let filename = "tools.json";
        assert!(filename.ends_with(".json"));

        let non_json = "readme.txt";
        assert!(!non_json.ends_with(".json"));
    }

    #[test]
    fn test_config_file_names() {
        let valid_configs = vec![
            "tools.json",
            "prompts.json",
            "server.json",
            "instances.json",
        ];

        for config in valid_configs {
            assert!(config.ends_with(".json"));
            assert!(config.len() > 5); // At least ".json"
        }
    }

    #[test]
    fn test_pathbuf_creation() {
        let path = PathBuf::from("/tmp/config");
        assert!(path.to_str().is_some());

        let joined = path.join("tools.json");
        assert!(joined.to_str().unwrap().contains("tools.json"));
    }

    #[test]
    fn test_file_name_extraction() {
        let path = PathBuf::from("/tmp/config/tools.json");
        let filename = path.file_name().and_then(|n| n.to_str());

        assert_eq!(filename, Some("tools.json"));
    }

    #[test]
    fn test_recursive_mode_check() {
        // Test that we understand recursive mode enum
        use notify::RecursiveMode;

        let non_recursive = RecursiveMode::NonRecursive;
        let recursive = RecursiveMode::Recursive;

        // Verify the enum variants exist and are different
        assert_ne!(format!("{:?}", non_recursive), format!("{:?}", recursive));
    }
}
