use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::convert::Infallible;

use axum::extract::{Query, State};
use axum::http::{HeaderMap, HeaderName, HeaderValue, StatusCode};
use axum::response::sse::{Event, Sse};
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use mcp_rust_sdk::error::{Error as McpError, ErrorCode};
use mcp_rust_sdk::protocol::{RequestId, Response};
use mcp_rust_sdk::server::ServerHandler;
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::sync::{broadcast, Mutex};
use tokio_stream::wrappers::{BroadcastStream, IntervalStream};
use tokio_stream::{iter, StreamExt};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

use crate::mcp::McpOdooHandler;

static MCP_SESSION_ID: HeaderName = HeaderName::from_static("mcp-session-id");

#[derive(Clone, Default)]
struct SessionState {
    initialized: bool,
}

#[derive(Clone)]
struct AppState {
    handler: Arc<McpOdooHandler>,
    sessions: Arc<Mutex<HashMap<String, SessionState>>>,
    sse_channels: Arc<Mutex<HashMap<String, broadcast::Sender<Value>>>>,
}

pub async fn serve(handler: Arc<McpOdooHandler>, listen: &str) -> anyhow::Result<()> {
    let state = AppState {
        handler,
        sessions: Arc::new(Mutex::new(HashMap::new())),
        sse_channels: Arc::new(Mutex::new(HashMap::new())),
    };

    let app = Router::new()
        // Streamable HTTP (recommended)
        .route("/mcp", post(mcp_post).get(mcp_get))
        // Legacy SSE transport (Cursor supports `SSE` transport option)
        .route("/sse", get(legacy_sse))
        .route("/messages", post(legacy_messages))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr: SocketAddr = listen.parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

fn jsonrpc_err(id: RequestId, code: ErrorCode, message: impl Into<String>) -> Response {
    Response::error(
        id,
        mcp_rust_sdk::protocol::ResponseError {
            code: code.into(),
            message: message.into(),
            data: None,
        },
    )
}

fn cursor_initialize_result(
    params: &Value,
    odoo_instances: Vec<String>,
    protocol_default: String,
    server_name: String,
    instructions: String,
) -> Result<Value, McpError> {
    let protocol_version = params
        .get("protocolVersion")
        .and_then(|v| v.as_str())
        .unwrap_or(&protocol_default)
        .to_string();

    Ok(json!({
        "protocolVersion": protocol_version,
        "capabilities": {
            "tools": { "listChanged": true },
            "prompts": { "listChanged": true },
            "resources": {},
            "experimental": {
                "odooInstances": { "available": odoo_instances }
            }
        },
        "serverInfo": {
            "name": server_name,
            "version": env!("CARGO_PKG_VERSION")
        },
        "instructions": instructions
    }))
}

async fn handle_jsonrpc(
    state: &AppState,
    session_id: Option<String>,
    v: Value,
) -> Result<(Option<String>, Option<Value>, StatusCode), (StatusCode, Value)> {
    let obj = v.as_object().ok_or((StatusCode::BAD_REQUEST, json!({"error":"expected object"})))?;
    let method = obj
        .get("method")
        .and_then(|m| m.as_str())
        .ok_or((StatusCode::BAD_REQUEST, json!({"error":"missing method"})))?
        .to_string();

    // Notifications have no id.
    let id_val = obj.get("id").cloned();
    let params = obj.get("params").cloned();

    // Streamable HTTP sessions: create on initialize, otherwise accept without requiring.
    let effective_session = session_id;
    if method == "initialize" {
        let id_val = id_val.ok_or((StatusCode::BAD_REQUEST, json!({"error":"initialize requires id"})))?;
        let id: RequestId = serde_json::from_value(id_val)
            .map_err(|e| (StatusCode::BAD_REQUEST, json!({"error": e.to_string()})))?;
        let params = params.unwrap_or_else(|| json!({}));
        let protocol_default = state.handler.protocol_version_default().await;
        let server_name = state.handler.server_name().await;
        let instructions = state.handler.instructions().await;
        let result = cursor_initialize_result(
            &params,
            state.handler.instance_names(),
            protocol_default,
            server_name,
            instructions,
        )
            .map_err(|e| (StatusCode::BAD_REQUEST, json!({"error": e.to_string()})))?;

        let sess = Uuid::new_v4().to_string();
        state
            .sessions
            .lock()
            .await
            .insert(sess.clone(), SessionState { initialized: true });
        state
            .sse_channels
            .lock()
            .await
            .entry(sess.clone())
            .or_insert_with(|| broadcast::channel(256).0);

        let resp = Response::success(id, Some(result));
        return Ok((Some(sess), Some(serde_json::to_value(resp).unwrap()), StatusCode::OK));
    }

    // initialized notification toggles gating for the session (if provided)
    if method == "initialized" {
        if let Some(sess) = &effective_session {
            if let Some(st) = state.sessions.lock().await.get_mut(sess) {
                st.initialized = true;
            }
        }
        return Ok((None, None, StatusCode::ACCEPTED));
    }

    // For other methods, if we have a known session and it's not initialized, reject.
    if let Some(sess) = &effective_session {
        if let Some(st) = state.sessions.lock().await.get(sess) {
            if !st.initialized {
                // Cursor typically sends initialized quickly; if not, still allow read-only ops?
                // We'll follow MCP gating to match stdio behavior.
                let id = id_val
                    .clone()
                    .and_then(|x| serde_json::from_value::<RequestId>(x).ok())
                    .unwrap_or(RequestId::String("unknown".to_string()));
                let resp = jsonrpc_err(id, ErrorCode::ServerNotInitialized, "Server not initialized");
                return Ok((None, Some(serde_json::to_value(resp).unwrap()), StatusCode::OK));
            }
        }
    }

    // Notifications: best-effort handle_method, return 202.
    if id_val.is_none() {
        let _ = state.handler.handle_method(&method, params).await;
        return Ok((None, None, StatusCode::ACCEPTED));
    }

    let id: RequestId = serde_json::from_value(id_val.unwrap())
        .map_err(|e| (StatusCode::BAD_REQUEST, json!({"error": e.to_string()})))?;

    let result = state
        .handler
        .handle_method(&method, params)
        .await
        .map_err(|e| (StatusCode::OK, jsonrpc_err(id.clone(), ErrorCode::InternalError, e.to_string()).to_value()))?;
    let resp = Response::success(id, Some(result));
    Ok((None, Some(serde_json::to_value(resp).unwrap()), StatusCode::OK))
}

async fn mcp_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let session = headers
        .get(&MCP_SESSION_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    // We only implement single-message requests for now (Cursor uses single).
    let (new_sess, maybe_resp, status) = match handle_jsonrpc(&state, session, body).await {
        Ok(v) => v,
        Err((sc, v)) => return (sc, Json(v)).into_response(),
    };

    let mut out_headers = HeaderMap::new();
    if let Some(sess) = new_sess {
        let _ = out_headers.insert(&MCP_SESSION_ID, HeaderValue::from_str(&sess).unwrap());
    }

    match maybe_resp {
        Some(v) => (status, out_headers, Json(v)).into_response(),
        None => (status, out_headers).into_response(),
    }
}

async fn mcp_get(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    // Optional server->client channel (notifications).
    let session = headers
        .get(&MCP_SESSION_ID)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| "default".to_string());

    let tx = {
        let mut chans = state.sse_channels.lock().await;
        chans
            .entry(session)
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    };

    let keepalive = IntervalStream::new(tokio::time::interval(Duration::from_secs(15)))
        .map(|_| Ok::<Event, Infallible>(Event::default().comment("keepalive")));

    let stream = BroadcastStream::new(tx.subscribe()).filter_map(|msg| match msg {
        Ok(v) => Some(Ok(Event::default().event("message").data(v.to_string()))),
        Err(_) => None,
    });

    Sse::new(keepalive.merge(stream)).keep_alive(axum::response::sse::KeepAlive::default())
}

#[derive(Deserialize)]
struct LegacyQuery {
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

async fn legacy_sse(State(state): State<AppState>) -> impl IntoResponse {
    let session_id = Uuid::new_v4().to_string();
    let tx = {
        let mut chans = state.sse_channels.lock().await;
        chans
            .entry(session_id.clone())
            .or_insert_with(|| broadcast::channel(256).0)
            .clone()
    };

    // First event tells the client where to POST messages (legacy spec).
    let endpoint_event = iter(vec![Ok::<Event, Infallible>(
        Event::default()
            .event("endpoint")
            .data(format!("/messages?sessionId={session_id}")),
    )]);

    let stream = BroadcastStream::new(tx.subscribe()).filter_map(|msg| match msg {
        Ok(v) => Some(Ok(Event::default().event("message").data(v.to_string()))),
        Err(_) => None,
    });

    Sse::new(endpoint_event.chain(stream))
        .keep_alive(axum::response::sse::KeepAlive::default())
}

async fn legacy_messages(
    State(state): State<AppState>,
    Query(q): Query<LegacyQuery>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    let session = q.session_id.or_else(|| {
        headers
            .get(&MCP_SESSION_ID)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
    });

    // Legacy transport: responses are delivered on SSE stream, not in HTTP response.
    let (_new_sess, maybe_resp, _status) = match handle_jsonrpc(&state, session.clone(), body).await
    {
        Ok(v) => v,
        Err((_sc, _v)) => return StatusCode::BAD_REQUEST,
    };

    if let (Some(sess), Some(resp)) = (session, maybe_resp) {
        if let Some(tx) = state.sse_channels.lock().await.get(&sess).cloned() {
            let _ = tx.send(resp);
        }
    }

    StatusCode::ACCEPTED
}

trait ResponseExt {
    fn to_value(self) -> Value;
}

impl ResponseExt for Response {
    fn to_value(self) -> Value {
        serde_json::to_value(self).unwrap_or_else(|_| json!({}))
    }
}

