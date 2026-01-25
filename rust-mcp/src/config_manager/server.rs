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
    // If dist doesn't exist, show a helpful error message
    let static_dir = std::path::Path::new("static/dist");
    if !static_dir.exists() || !static_dir.is_dir() {
        warn!(
            "static/dist directory not found. Please build the React UI first with: cd config-ui && npm run build"
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
        // Serve static files (React app)
        .nest_service("/", ServeDir::new("static/dist"))
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
