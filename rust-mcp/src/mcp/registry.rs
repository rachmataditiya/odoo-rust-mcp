use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use serde::Deserialize;
use serde_json::Value;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::mcp::prompts::Prompt;

// Embedded seed defaults (used when target files are missing).
const DEFAULT_TOOLS_JSON: &str = include_str!("../../config-defaults/tools.json");
const DEFAULT_PROMPTS_JSON: &str = include_str!("../../config-defaults/prompts.json");
const DEFAULT_SERVER_JSON: &str = include_str!("../../config-defaults/server.json");

#[derive(Debug, Clone, Deserialize)]
struct ToolsConfigFile {
    tools: Vec<ToolDef>,
}

#[derive(Debug, Clone, Deserialize)]
struct PromptsConfigFile {
    prompts: Vec<Prompt>,
}

#[derive(Debug, Clone, Deserialize)]
struct ServerConfigFile {
    #[serde(rename = "serverName")]
    server_name: String,
    instructions: String,
    #[serde(rename = "protocolVersionDefault")]
    protocol_version_default: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
    pub op: OpSpec,
    #[serde(default)]
    pub guards: Option<ToolGuards>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OpSpec {
    #[serde(rename = "type")]
    pub op_type: String,
    #[serde(default)]
    pub map: HashMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ToolGuards {
    /// If set, tool is only listed/callable when env var exists and is truthy.
    #[serde(rename = "requiresEnvTrue")]
    pub requires_env_true: Option<String>,
}

#[derive(Debug, Clone)]
struct RegistryState {
    tools: Vec<ToolDef>,
    tool_by_name: HashMap<String, ToolDef>,
    prompts_by_name: HashMap<String, Prompt>,
    prompt_order: Vec<String>,
    server: ServerConfigFile,
}

impl RegistryState {
    fn empty() -> Self {
        Self {
            tools: Vec::new(),
            tool_by_name: HashMap::new(),
            prompts_by_name: HashMap::new(),
            prompt_order: Vec::new(),
            server: ServerConfigFile {
                server_name: "odoo-rust-mcp".to_string(),
                instructions: "Odoo MCP server".to_string(),
                protocol_version_default: Some("2025-11-05".to_string()),
            },
        }
    }
}

pub struct Registry {
    tools_path: PathBuf,
    prompts_path: PathBuf,
    server_path: PathBuf,
    state: RwLock<RegistryState>,
    watchers: Mutex<Option<WatchGuards>>,
}

struct WatchGuards {
    _watcher: RecommendedWatcher,
}

impl Registry {
    pub fn from_env() -> Self {
        let tools_path =
            std::env::var("MCP_TOOLS_JSON").unwrap_or_else(|_| "config/tools.json".to_string());
        let prompts_path =
            std::env::var("MCP_PROMPTS_JSON").unwrap_or_else(|_| "config/prompts.json".to_string());
        let server_path =
            std::env::var("MCP_SERVER_JSON").unwrap_or_else(|_| "config/server.json".to_string());

        Self {
            tools_path: PathBuf::from(tools_path),
            prompts_path: PathBuf::from(prompts_path),
            server_path: PathBuf::from(server_path),
            state: RwLock::new(RegistryState::empty()),
            watchers: Mutex::new(None),
        }
    }

    /// Ensure JSON files exist (seed defaults on first start), then load into memory.
    pub async fn initial_load(&self) -> anyhow::Result<()> {
        self.ensure_default_files_exist()?;
        self.reload().await
    }

    /// Start file watcher(s) that reload config automatically.
    ///
    /// Safety: call once; subsequent calls are no-ops.
    pub fn start_watchers(self: &Arc<Self>) {
        let mut guard = self
            .watchers
            .lock()
            .expect("registry watcher mutex poisoned");
        if guard.is_some() {
            return;
        }

        // Debounced reload trigger: multiple fs events collapse into one reload.
        let (reload_tx, mut reload_rx) = tokio::sync::mpsc::unbounded_channel::<()>();
        let registry = Arc::clone(self);

        tokio::spawn(async move {
            loop {
                if reload_rx.recv().await.is_none() {
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                while reload_rx.try_recv().is_ok() {}
                if let Err(e) = registry.reload().await {
                    warn!(error = %e, "config reload failed; keeping last good");
                }
            }
        });

        let tools_dir = parent_dir_or_current(&self.tools_path);
        let prompts_dir = parent_dir_or_current(&self.prompts_path);
        let server_dir = parent_dir_or_current(&self.server_path);

        let mut watch_dirs = vec![tools_dir, prompts_dir, server_dir];
        watch_dirs.sort();
        watch_dirs.dedup();

        let mut watcher = match notify::recommended_watcher(move |res| match res {
            Ok(event) => {
                debug!(?event, "config fs event");
                let _ = reload_tx.send(());
            }
            Err(err) => {
                warn!(error = %err, "config watcher error");
            }
        }) {
            Ok(w) => w,
            Err(e) => {
                warn!(error = %e, "failed to create config watcher; auto-reload disabled");
                return;
            }
        };

        for dir in watch_dirs {
            if let Err(e) = watcher.watch(&dir, RecursiveMode::NonRecursive) {
                warn!(dir = %dir.display(), error = %e, "failed to watch config directory");
            } else {
                info!(dir = %dir.display(), "watching config directory");
            }
        }

        *guard = Some(WatchGuards { _watcher: watcher });
    }

    pub async fn server_name(&self) -> String {
        self.state.read().await.server.server_name.clone()
    }

    pub async fn instructions(&self) -> String {
        self.state.read().await.server.instructions.clone()
    }

    pub async fn protocol_version_default(&self) -> String {
        self.state
            .read()
            .await
            .server
            .protocol_version_default
            .clone()
            .unwrap_or_else(|| "2025-11-05".to_string())
    }

    pub async fn list_tools(&self) -> Vec<Value> {
        let st = self.state.read().await;
        st.tools
            .iter()
            .filter(|t| guards_allow(t.guards.as_ref()))
            .map(|t| {
                serde_json::json!({
                    "name": t.name,
                    "description": t.description,
                    "inputSchema": t.input_schema
                })
            })
            .collect()
    }

    pub async fn get_tool(&self, name: &str) -> Option<ToolDef> {
        let st = self.state.read().await;
        let t = st.tool_by_name.get(name)?.clone();
        guards_allow(t.guards.as_ref()).then_some(t)
    }

    pub async fn list_prompts(&self) -> Vec<(String, String)> {
        let st = self.state.read().await;
        st.prompt_order
            .iter()
            .filter_map(|name| {
                st.prompts_by_name
                    .get(name)
                    .map(|p| (p.name.clone(), p.description.clone()))
            })
            .collect()
    }

    pub async fn get_prompt(&self, name: &str) -> Option<Prompt> {
        let st = self.state.read().await;
        st.prompts_by_name.get(name).cloned()
    }

    pub async fn reload(&self) -> anyhow::Result<()> {
        self.ensure_default_files_exist()?;

        let tools = load_tools_file(&self.tools_path)?;
        let prompts = load_prompts_file(&self.prompts_path)?;
        let server = load_server_file(&self.server_path)?;

        // Validate and build maps.
        let mut tool_by_name = HashMap::new();
        for t in &tools {
            validate_cursor_schema(&t.input_schema).map_err(|e| {
                anyhow::anyhow!("tools.json tool '{}' has invalid inputSchema: {e}", t.name)
            })?;
            if tool_by_name.insert(t.name.clone(), t.clone()).is_some() {
                return Err(anyhow::anyhow!(
                    "Duplicate tool name in tools.json: {}",
                    t.name
                ));
            }
        }

        let mut prompts_by_name = HashMap::new();
        let mut prompt_order = Vec::new();
        for p in prompts {
            if prompts_by_name.insert(p.name.clone(), p.clone()).is_some() {
                return Err(anyhow::anyhow!(
                    "Duplicate prompt name in prompts.json: {}",
                    p.name
                ));
            }
            prompt_order.push(p.name.clone());
        }

        let mut st = self.state.write().await;
        st.tools = tools;
        st.tool_by_name = tool_by_name;
        st.prompts_by_name = prompts_by_name;
        st.prompt_order = prompt_order;
        st.server = server;

        info!(path = %self.tools_path.display(), "tools config loaded");
        info!(path = %self.prompts_path.display(), "prompts config loaded");
        info!(path = %self.server_path.display(), "server config loaded");
        Ok(())
    }

    fn ensure_default_files_exist(&self) -> anyhow::Result<()> {
        ensure_file_exists_with_seed(&self.tools_path, DEFAULT_TOOLS_JSON)?;
        ensure_file_exists_with_seed(&self.prompts_path, DEFAULT_PROMPTS_JSON)?;
        ensure_file_exists_with_seed(&self.server_path, DEFAULT_SERVER_JSON)?;
        Ok(())
    }
}

fn guards_allow(guards: Option<&ToolGuards>) -> bool {
    let Some(g) = guards else { return true };
    if let Some(var) = &g.requires_env_true {
        return env_truthy(var);
    }
    true
}

fn env_truthy(var: &str) -> bool {
    match std::env::var(var) {
        Ok(v) => {
            let s = v.trim().to_ascii_lowercase();
            matches!(s.as_str(), "1" | "true" | "yes" | "y" | "on")
        }
        Err(_) => false,
    }
}

fn parent_dir_or_current(path: &Path) -> PathBuf {
    path.parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn ensure_file_exists_with_seed(path: &Path, seed_contents: &str) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, seed_contents)?;
    info!(path = %path.display(), "created default config file");
    Ok(())
}

fn load_tools_file(path: &Path) -> anyhow::Result<Vec<ToolDef>> {
    let raw = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!(e))?;
    let parsed: ToolsConfigFile =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("Invalid tools.json: {e}"))?;
    Ok(parsed.tools)
}

fn load_prompts_file(path: &Path) -> anyhow::Result<Vec<Prompt>> {
    let raw = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!(e))?;
    let parsed: PromptsConfigFile =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("Invalid prompts.json: {e}"))?;
    Ok(parsed.prompts)
}

fn load_server_file(path: &Path) -> anyhow::Result<ServerConfigFile> {
    let raw = std::fs::read_to_string(path).map_err(|e| anyhow::anyhow!(e))?;
    let parsed: ServerConfigFile =
        serde_json::from_str(&raw).map_err(|e| anyhow::anyhow!("Invalid server.json: {e}"))?;
    Ok(parsed)
}

/// Cursor can be picky about JSON Schema features.
/// Reject schemas that likely break Cursor parsing.
fn validate_cursor_schema(schema: &Value) -> anyhow::Result<()> {
    fn walk(v: &Value) -> anyhow::Result<()> {
        match v {
            Value::Object(map) => {
                for (k, vv) in map {
                    if matches!(
                        k.as_str(),
                        "anyOf" | "oneOf" | "allOf" | "$ref" | "definitions"
                    ) {
                        return Err(anyhow::anyhow!("schema contains forbidden key '{k}'"));
                    }
                    if k == "type" && vv.is_array() {
                        return Err(anyhow::anyhow!("schema contains type array"));
                    }
                    walk(vv)?;
                }
            }
            Value::Array(arr) => {
                for vv in arr {
                    walk(vv)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    walk(schema)
}
