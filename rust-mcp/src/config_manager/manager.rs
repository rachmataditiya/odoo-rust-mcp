use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

#[derive(Clone)]
pub struct ConfigManager {
    config_dir: PathBuf,
    instances_cache: Arc<RwLock<Value>>,
}

impl ConfigManager {
    pub fn new(config_dir: PathBuf) -> Self {
        Self {
            config_dir,
            instances_cache: Arc::new(RwLock::new(json!({}))),
        }
    }

    /// Load instances config from file
    pub async fn load_instances(&self) -> anyhow::Result<Value> {
        let path = self.config_dir.join("instances.json");

        if !path.exists() {
            warn!(
                "instances.json not found at {:?}, returning empty config",
                path
            );
            return Ok(json!({}));
        }

        let content = fs::read_to_string(&path)?;
        let config: Value = serde_json::from_str(&content)?;

        // Update cache
        {
            let mut cache = self.instances_cache.write().await;
            *cache = config.clone();
        }

        info!("Loaded instances config from {:?}", path);
        Ok(config)
    }

    /// Get cached instances config
    pub async fn get_instances(&self) -> Value {
        self.instances_cache.read().await.clone()
    }

    /// Save instances config to file
    pub async fn save_instances(&self, config: Value) -> anyhow::Result<()> {
        let path = self.config_dir.join("instances.json");

        // Create parent directory if not exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Validate JSON structure
        if !config.is_object() {
            return Err(anyhow::anyhow!("Config must be a JSON object"));
        }

        let json_str = serde_json::to_string_pretty(&config)?;
        fs::write(&path, json_str)?;

        // Update cache
        {
            let mut cache = self.instances_cache.write().await;
            *cache = config;
        }

        info!("Saved instances config to {:?}", path);
        Ok(())
    }

    /// Load tools config
    pub async fn load_tools(&self) -> anyhow::Result<Value> {
        let path = self.config_dir.join("tools.json");

        if !path.exists() {
            warn!("tools.json not found at {:?}, returning empty array", path);
            return Ok(json!([]));
        }

        let content = fs::read_to_string(&path)?;
        let config: Value = serde_json::from_str(&content)?;

        // Extract tools array from {"tools": [...]} or return array directly
        let tools = if let Some(tools_array) = config.get("tools").and_then(|v| v.as_array()) {
            json!(tools_array)
        } else if config.is_array() {
            config
        } else {
            return Err(anyhow::anyhow!(
                "Invalid tools.json format: expected object with 'tools' array or array directly"
            ));
        };

        info!("Loaded tools config from {:?}", path);
        Ok(tools)
    }

    /// Save tools config to file
    pub async fn save_tools(&self, config: Value) -> anyhow::Result<()> {
        let path = self.config_dir.join("tools.json");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Accept either array directly or object with tools array
        let tools_array = if config.is_array() {
            config
        } else if let Some(tools) = config.get("tools").and_then(|v| v.as_array()) {
            json!(tools)
        } else {
            return Err(anyhow::anyhow!(
                "Tools config must be a JSON array or object with 'tools' array"
            ));
        };

        // Save as {"tools": [...]} format to match file structure
        let file_content = json!({ "tools": tools_array });

        let json_str = serde_json::to_string_pretty(&file_content)?;
        fs::write(&path, json_str)?;

        info!("Saved tools config to {:?}", path);
        Ok(())
    }

    /// Load prompts config
    pub async fn load_prompts(&self) -> anyhow::Result<Value> {
        let path = self.config_dir.join("prompts.json");

        if !path.exists() {
            warn!(
                "prompts.json not found at {:?}, returning empty array",
                path
            );
            return Ok(json!([]));
        }

        let content = fs::read_to_string(&path)?;
        let config: Value = serde_json::from_str(&content)?;

        // Extract prompts array from {"prompts": [...]} or return array directly
        let prompts = if let Some(prompts_array) = config.get("prompts").and_then(|v| v.as_array())
        {
            json!(prompts_array)
        } else if config.is_array() {
            config
        } else {
            return Err(anyhow::anyhow!(
                "Invalid prompts.json format: expected object with 'prompts' array or array directly"
            ));
        };

        info!("Loaded prompts config from {:?}", path);
        Ok(prompts)
    }

    /// Save prompts config to file
    pub async fn save_prompts(&self, config: Value) -> anyhow::Result<()> {
        let path = self.config_dir.join("prompts.json");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Accept either array directly or object with prompts array
        let prompts_array = if config.is_array() {
            config
        } else if let Some(prompts) = config.get("prompts").and_then(|v| v.as_array()) {
            json!(prompts)
        } else {
            return Err(anyhow::anyhow!(
                "Prompts config must be a JSON array or object with 'prompts' array"
            ));
        };

        // Save as {"prompts": [...]} format to match file structure
        let file_content = json!({ "prompts": prompts_array });

        let json_str = serde_json::to_string_pretty(&file_content)?;
        fs::write(&path, json_str)?;

        info!("Saved prompts config to {:?}", path);
        Ok(())
    }

    /// Load server config
    pub async fn load_server(&self) -> anyhow::Result<Value> {
        let path = self.config_dir.join("server.json");

        if !path.exists() {
            warn!(
                "server.json not found at {:?}, returning empty config",
                path
            );
            return Ok(json!({}));
        }

        let content = fs::read_to_string(&path)?;
        let config: Value = serde_json::from_str(&content)?;

        info!("Loaded server config from {:?}", path);
        Ok(config)
    }

    /// Save server config to file
    pub async fn save_server(&self, config: Value) -> anyhow::Result<()> {
        let path = self.config_dir.join("server.json");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        if !config.is_object() {
            return Err(anyhow::anyhow!("Server config must be a JSON object"));
        }

        let json_str = serde_json::to_string_pretty(&config)?;
        fs::write(&path, json_str)?;

        info!("Saved server config to {:?}", path);
        Ok(())
    }

    pub fn config_dir(&self) -> &PathBuf {
        &self.config_dir
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load_instances() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ConfigManager::new(temp_dir.path().to_path_buf());

        let config = json!({
            "default": {
                "url": "http://localhost:8069",
                "db": "mydb",
                "apiKey": "test_key"
            }
        });

        manager.save_instances(config.clone()).await.unwrap();
        let loaded = manager.load_instances().await.unwrap();

        assert_eq!(loaded, config);
    }
}
