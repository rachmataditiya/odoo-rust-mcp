use std::{pin::Pin, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::Stream;
use tokio::sync::{broadcast, mpsc};

use mcp_rust_sdk::{
    error::{Error, ErrorCode},
    protocol::{Notification, Request, RequestId},
    transport::{Message, Transport},
};

use rust_mcp::mcp::{
    McpOdooHandler, registry::Registry, runtime::ServerCompat, tools::OdooClientPool,
};
use uuid::Uuid;

struct MockTransport {
    client_to_server: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Result<Message, Error>>>>,
    server_to_client: broadcast::Sender<Result<Message, Error>>,
}

impl MockTransport {
    fn new() -> (
        Self,
        mpsc::UnboundedSender<Result<Message, Error>>,
        broadcast::Receiver<Result<Message, Error>>,
    ) {
        let (tx1, rx1) = mpsc::unbounded_channel();
        let (tx2, rx2) = broadcast::channel(100);
        (
            Self {
                client_to_server: Arc::new(tokio::sync::Mutex::new(rx1)),
                server_to_client: tx2.clone(),
            },
            tx1,
            rx2,
        )
    }
}

#[async_trait]
impl Transport for MockTransport {
    async fn send(&self, message: Message) -> Result<(), Error> {
        self.server_to_client
            .send(Ok(message))
            .map(|_| ())
            .map_err(|_| Error::protocol(ErrorCode::InternalError, "Failed to send message"))
    }

    fn receive(&self) -> Pin<Box<dyn Stream<Item = Result<Message, Error>> + Send>> {
        let rx = self.client_to_server.clone();
        Box::pin(async_stream::stream! {
            let mut rx = rx.lock().await;
            while let Some(msg) = rx.recv().await {
                yield msg;
            }
        })
    }

    async fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}

#[tokio::test]
async fn mcp_initialize_and_list_tools_prompts() {
    // Minimal env so OdooClientPool can load.
    unsafe {
        std::env::set_var("ODOO_URL", "http://localhost:8069");
        std::env::set_var("ODOO_DB", "v19_pos");
        std::env::set_var("ODOO_API_KEY", "dummy");
    }

    // Use temp config paths so tests don't depend on repo files.
    let tmp = std::env::temp_dir().join(format!("odoo-rust-mcp-test-{}", Uuid::new_v4()));
    unsafe {
        std::env::set_var(
            "MCP_TOOLS_JSON",
            tmp.join("tools.json").to_string_lossy().to_string(),
        );
        std::env::set_var(
            "MCP_PROMPTS_JSON",
            tmp.join("prompts.json").to_string_lossy().to_string(),
        );
        std::env::set_var(
            "MCP_SERVER_JSON",
            tmp.join("server.json").to_string_lossy().to_string(),
        );
    }

    let pool = OdooClientPool::from_env().unwrap();
    let registry = Arc::new(Registry::from_env());
    registry.initial_load().await.unwrap();
    let handler = Arc::new(McpOdooHandler::new(pool, registry));

    let (transport, client_tx, mut client_rx) = MockTransport::new();
    let server = ServerCompat::new(Arc::new(transport), handler);

    let server_handle = tokio::spawn(async move {
        if let Err(e) = server.start().await {
            eprintln!("server error: {}", e);
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;

    // initialize (SDK legacy uses "implementation", and we accept it)
    let init_request = Request::new(
        "initialize",
        Some(serde_json::json!({
            "implementation": { "name": "test-client", "version": "0.1.0" },
            "capabilities": {},
            "protocolVersion": "2025-11-05"
        })),
        RequestId::Number(1),
    );
    client_tx.send(Ok(Message::Request(init_request))).unwrap();

    // receive init response
    let init_resp = tokio::time::timeout(Duration::from_secs(2), client_rx.recv())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match init_resp {
        Message::Response(resp) => assert!(resp.error.is_none()),
        _ => panic!("expected response"),
    }

    // send initialized notification
    client_tx
        .send(Ok(Message::Notification(Notification::new(
            "initialized",
            None,
        ))))
        .unwrap();
    tokio::time::sleep(Duration::from_millis(20)).await;

    // tools/list
    let list_tools = Request::new(
        "tools/list",
        Some(serde_json::json!({})),
        RequestId::Number(2),
    );
    client_tx.send(Ok(Message::Request(list_tools))).unwrap();
    let resp = tokio::time::timeout(Duration::from_secs(2), client_rx.recv())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match resp {
        Message::Response(resp) => {
            assert!(resp.error.is_none());
            let v = resp.result.unwrap();
            assert!(v.get("tools").is_some());
        }
        _ => panic!("expected response"),
    }

    // prompts/list
    let list_prompts = Request::new(
        "prompts/list",
        Some(serde_json::json!({})),
        RequestId::Number(3),
    );
    client_tx.send(Ok(Message::Request(list_prompts))).unwrap();
    let resp = tokio::time::timeout(Duration::from_secs(2), client_rx.recv())
        .await
        .unwrap()
        .unwrap()
        .unwrap();
    match resp {
        Message::Response(resp) => {
            assert!(resp.error.is_none());
            let v = resp.result.unwrap();
            assert!(v.get("prompts").is_some());
        }
        _ => panic!("expected response"),
    }

    // exit
    client_tx
        .send(Ok(Message::Notification(Notification::new("exit", None))))
        .unwrap();
    let _ = tokio::time::timeout(Duration::from_secs(2), server_handle).await;
}
