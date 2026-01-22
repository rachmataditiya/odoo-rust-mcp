use async_trait::async_trait;
use futures::Stream;
use std::{
    io::Write,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::io::{AsyncBufReadExt, BufReader as TokioBufReader};
use tokio::sync::broadcast;

use mcp_rust_sdk::{
    error::{Error, ErrorCode},
    protocol::{Notification, Request, Response},
    transport::{Message, Transport},
};

/// Stdio transport compatible with Cursor's MCP client.
///
/// Cursor speaks "plain" JSON-RPC objects over stdio (no outer `{ "type": ... }` tag).
/// The `mcp_rust_sdk` transport `Message` enum is tagged with `"type"`, so the default
/// SDK `StdioTransport` rejects Cursor messages with "missing field `type`".
///
/// This transport accepts BOTH formats:
/// - SDK tagged messages: `{ "type": "request|response|notification", ... }`
/// - Plain JSON-RPC objects: `{ "jsonrpc": "2.0", "method": "...", ... }`
///
/// When sending messages, we always emit plain JSON-RPC objects (Cursor-friendly).
pub struct CursorStdioTransport {
    stdout: Arc<Mutex<std::io::Stdout>>,
    receiver: broadcast::Receiver<Result<Message, Error>>,
}

impl CursorStdioTransport {
    pub fn new() -> (Self, broadcast::Sender<Result<Message, Error>>) {
        let (sender, receiver) = broadcast::channel(100);
        let transport = Self {
            stdout: Arc::new(Mutex::new(std::io::stdout())),
            receiver,
        };

        let stdin = tokio::io::stdin();
        let mut reader = TokioBufReader::new(stdin);
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line).await {
                    Ok(0) => break,
                    Ok(_) => {
                        let raw = line.trim();
                        if raw.is_empty() {
                            continue;
                        }

                        // First, try SDK tagged message format.
                        if let Ok(msg) = serde_json::from_str::<Message>(raw) {
                            if sender_clone.send(Ok(msg)).is_err() {
                                break;
                            }
                            continue;
                        }

                        // Fallback: plain JSON-RPC object.
                        let msg = match serde_json::from_str::<serde_json::Value>(raw) {
                            Ok(v) => parse_jsonrpc_value(v),
                            Err(err) => Err(Error::Serialization(err.to_string())),
                        };

                        if sender_clone.send(msg).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = sender_clone.send(Err(Error::Io(err.to_string())));
                        break;
                    }
                }
            }
        });

        (transport, sender)
    }
}

fn parse_jsonrpc_value(v: serde_json::Value) -> Result<Message, Error> {
    let obj = v.as_object().ok_or_else(|| {
        Error::protocol(ErrorCode::InvalidRequest, "Expected JSON object message")
    })?;

    // We classify based on "method"/"id" presence.
    if obj.contains_key("method") {
        if obj.contains_key("id") {
            let req: Request = serde_json::from_value(v)?;
            return Ok(Message::Request(req));
        }
        let notif: Notification = serde_json::from_value(serde_json::Value::Object(obj.clone()))?;
        return Ok(Message::Notification(notif));
    }

    if obj.contains_key("id") {
        let resp: Response = serde_json::from_value(serde_json::Value::Object(obj.clone()))?;
        return Ok(Message::Response(resp));
    }

    Err(Error::protocol(
        ErrorCode::InvalidRequest,
        "Unable to classify JSON-RPC message",
    ))
}

#[async_trait]
impl Transport for CursorStdioTransport {
    async fn send(&self, message: Message) -> Result<(), Error> {
        let mut stdout = self.stdout.lock().map_err(|_| {
            Error::protocol(ErrorCode::InternalError, "Failed to acquire stdout lock")
        })?;

        // Emit plain JSON-RPC (Cursor-friendly).
        let json = match message {
            Message::Request(r) => serde_json::to_string(&r)?,
            Message::Response(r) => serde_json::to_string(&r)?,
            Message::Notification(n) => serde_json::to_string(&n)?,
        };
        writeln!(stdout, "{json}").map_err(|e| Error::Io(e.to_string()))?;
        stdout.flush().map_err(|e| Error::Io(e.to_string()))?;
        Ok(())
    }

    fn receive(&self) -> Pin<Box<dyn Stream<Item = Result<Message, Error>> + Send>> {
        let rx = self.receiver.resubscribe();
        Box::pin(futures::stream::unfold(rx, |mut rx| async move {
            match rx.recv().await {
                Ok(msg) => Some((msg, rx)),
                Err(_) => None,
            }
        }))
    }

    async fn close(&self) -> Result<(), Error> {
        Ok(())
    }
}
