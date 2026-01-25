use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde_json::{Value, json};
use std::sync::Arc;
use tower_http::{cors::CorsLayer, services::ServeDir};
use tracing::{error, info, warn};

use super::{ConfigManager, ConfigWatcher};

#[derive(Clone)]
struct AppState {
    config_manager: ConfigManager,
    config_watcher: Arc<ConfigWatcher>,
}

pub async fn start_config_server(port: u16, config_dir: std::path::PathBuf) -> anyhow::Result<()> {
    let config_manager = ConfigManager::new(config_dir.clone());
    let config_watcher = Arc::new(ConfigWatcher::new(config_dir)?);

    let state = AppState {
        config_manager,
        config_watcher,
    };

    // Serve static files from dist directory (React app)
    // Try multiple paths: relative to current dir, relative to executable, and absolute
    let mut static_dir_abs = None;

    // Try 1: Relative to current working directory
    let static_dir = std::path::Path::new("static/dist");
    if static_dir.exists()
        && static_dir.is_dir()
        && let Ok(canonical) = static_dir.canonicalize()
    {
        static_dir_abs = Some(canonical);
    }

    // Try 2: Relative to executable location (for installed binaries)
    if static_dir_abs.is_none()
        && let Ok(exe_path) = std::env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        let candidate = exe_dir.join("static/dist");
        if candidate.exists()
            && candidate.is_dir()
            && let Ok(canonical) = candidate.canonicalize()
        {
            static_dir_abs = Some(canonical);
        }
    }

    // Try 3: Homebrew share directory
    if static_dir_abs.is_none() {
        let candidates = [
            "/opt/homebrew/share/rust-mcp/static/dist",
            "/usr/local/share/rust-mcp/static/dist",
            "/usr/share/rust-mcp/static/dist",
        ];
        for candidate_str in &candidates {
            let candidate = std::path::Path::new(candidate_str);
            if candidate.exists() && candidate.is_dir() {
                static_dir_abs = Some(candidate.to_path_buf());
                break;
            }
        }
    }

    // Try 4: Relative to project root (for development)
    if static_dir_abs.is_none()
        && let Ok(current_dir) = std::env::current_dir()
    {
        let candidate = current_dir.join("rust-mcp/static/dist");
        if candidate.exists()
            && candidate.is_dir()
            && let Ok(canonical) = candidate.canonicalize()
        {
            static_dir_abs = Some(canonical);
        }
    }

    let static_dir_final = static_dir_abs.unwrap_or_else(|| {
        warn!("static/dist directory not found in any location. Please build the React UI first with: cd config-ui && npm run build");
        std::path::PathBuf::from("static/dist")
    });

    if static_dir_final.exists() && static_dir_final.is_dir() {
        info!("Serving static files from: {:?}", static_dir_final);
    } else {
        warn!(
            "static/dist directory not found at {:?}. Please build the React UI first with: cd config-ui && npm run build",
            static_dir_final
        );
    }

    let app = Router::new()
        // Health check
        .route("/health", get(health_check))
        // Instances endpoints
        .route("/api/config/instances", get(get_instances))
        .route("/api/config/instances", post(update_instances))
        // Tools endpoints
        .route("/api/config/tools", get(get_tools))
        .route("/api/config/tools", post(update_tools))
        // Prompts endpoints
        .route("/api/config/prompts", get(get_prompts))
        .route("/api/config/prompts", post(update_prompts))
        // Server endpoints
        .route("/api/config/server", get(get_server))
        .route("/api/config/server", post(update_server))
        // Serve static files (React app) - use fallback_service for root path
        .fallback_service(ServeDir::new(&static_dir_final))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("Config server listening on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "odoo-rust-mcp-config"
    }))
}

async fn get_instances(State(state): State<AppState>) -> impl IntoResponse {
    match state.config_manager.load_instances().await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            error!("Failed to load instances: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn update_instances(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match state.config_manager.save_instances(payload).await {
        Ok(_) => {
            state.config_watcher.notify("instances.json");
            (StatusCode::OK, Json(json!({ "status": "saved" }))).into_response()
        }
        Err(e) => {
            error!("Failed to save instances: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn get_tools(State(state): State<AppState>) -> impl IntoResponse {
    match state.config_manager.load_tools().await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            error!("Failed to load tools: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn update_tools(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match state.config_manager.save_tools(payload).await {
        Ok(_) => {
            state.config_watcher.notify("tools.json");
            (StatusCode::OK, Json(json!({ "status": "saved" }))).into_response()
        }
        Err(e) => {
            error!("Failed to save tools: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn get_prompts(State(state): State<AppState>) -> impl IntoResponse {
    match state.config_manager.load_prompts().await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            error!("Failed to load prompts: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn update_prompts(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match state.config_manager.save_prompts(payload).await {
        Ok(_) => {
            state.config_watcher.notify("prompts.json");
            (StatusCode::OK, Json(json!({ "status": "saved" }))).into_response()
        }
        Err(e) => {
            error!("Failed to save prompts: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn get_server(State(state): State<AppState>) -> impl IntoResponse {
    match state.config_manager.load_server().await {
        Ok(config) => (StatusCode::OK, Json(config)).into_response(),
        Err(e) => {
            error!("Failed to load server: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}

async fn update_server(
    State(state): State<AppState>,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    match state.config_manager.save_server(payload).await {
        Ok(_) => {
            state.config_watcher.notify("server.json");
            (StatusCode::OK, Json(json!({ "status": "saved" }))).into_response()
        }
        Err(e) => {
            error!("Failed to save server: {}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(json!({ "error": e.to_string() })),
            )
                .into_response()
        }
    }
}
