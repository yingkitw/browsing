//! Chrome DevTools Protocol (CDP) client implementation

use crate::error::{BrowsingError, Result};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};
use tokio_tungstenite::{connect_async, tungstenite::Message};

/// CDP client for WebSocket communication with Chrome
pub struct CdpClient {
    url: String,
    sender: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<Value>>>>,
    request_id: Arc<Mutex<u64>>,
    pending_requests: Arc<Mutex<HashMap<u64, mpsc::UnboundedSender<Value>>>>,
}

impl CdpClient {
    /// Create a new CDP client with the given WebSocket URL
    pub fn new(url: String) -> Self {
        Self {
            url,
            sender: Arc::new(Mutex::new(None)),
            receiver: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Start the WebSocket connection to the browser
    pub async fn start(&mut self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| BrowsingError::Cdp(format!("Failed to connect to CDP: {e}")))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (_tx_resp, rx_resp) = mpsc::unbounded_channel();

        *self.sender.lock().await = Some(tx);
        *self.receiver.lock().await = Some(rx_resp);

        let pending_requests = Arc::clone(&self.pending_requests);

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(Message::Close(_)) => break, // Graceful close initiated by us
                            Some(m) => {
                                if let Err(e) = write.send(m).await {
                                    tracing::debug!("WebSocket send error during shutdown: {}", e);
                                    break;
                                }
                            }
                            None => break, // Channel closed, sender dropped during shutdown
                        }
                    }
                    msg = read.next() => {
                        match msg {
                            Some(Ok(Message::Text(text))) => {
                                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                    if let Some(id_val) = value.get("id").and_then(|v| v.as_u64()) {
                                        if let Some(tx) = pending_requests.lock().await.remove(&id_val) {
                                            let _ = tx.send(value);
                                        }
                                    }
                                }
                            }
                            Some(Ok(Message::Close(_))) => break,
                            Some(Err(e)) => {
                                tracing::debug!("WebSocket closed: {}", e);
                                break;
                            }
                            Some(Ok(_)) => {}
                            None => break,
                        }
                    }
                }
            }
        });

        Ok(())
    }

    /// Send a CDP command without a session
    pub async fn send_command(&self, method: &str, params: Value) -> Result<Value> {
        self.send_command_with_session(method, params, None).await
    }

    /// Send a CDP command with an optional session
    pub async fn send_command_with_session(
        &self,
        method: &str,
        params: Value,
        session_id: Option<&str>,
    ) -> Result<Value> {
        let mut request_id = self.request_id.lock().await;
        let id = *request_id;
        *request_id += 1;
        drop(request_id);

        let mut request = serde_json::json!({
            "id": id,
            "method": method,
            "params": params
        });

        // Add sessionId if provided
        if let Some(sid) = session_id {
            request["sessionId"] = serde_json::json!(sid);
        }

        let (tx, mut rx) = mpsc::unbounded_channel();
        self.pending_requests.lock().await.insert(id, tx);

        if let Some(sender) = self.sender.lock().await.as_ref() {
            sender
                .send(Message::Text(request.to_string()))
                .map_err(|e| BrowsingError::Cdp(format!("Failed to send command: {e}")))?;
        }

        // Wait for response
        if let Some(response) = rx.recv().await {
            if let Some(error) = response.get("error") {
                return Err(BrowsingError::Cdp(format!("CDP error: {error}")));
            }
            return Ok(response["result"].clone());
        }

        Err(BrowsingError::Cdp("No response received".to_string()))
    }

    /// Gracefully close the WebSocket connection (works with Arc via &self)
    pub async fn close(&self) {
        if let Some(sender) = self.sender.lock().await.as_ref() {
            let _ = sender.send(Message::Close(None));
        }
    }
}

/// CDP session for a specific target
pub struct CdpSession {
    /// The CDP client instance
    pub client: Arc<CdpClient>,
    /// Target ID for this session
    pub target_id: String,
    /// Session ID
    pub session_id: String,
    /// Title of the target
    pub title: String,
    /// URL of the target
    pub url: String,
}

impl CdpSession {
    /// Creates a new CDP session for a specific target
    pub async fn for_target(
        client: Arc<CdpClient>,
        target_id: String,
        domains: Option<Vec<String>>,
    ) -> Result<Self> {
        // Attach to target
        let params = serde_json::json!({
            "targetId": target_id,
            "flatten": true
        });

        let result = client.send_command("Target.attachToTarget", params).await?;

        let session_id = result["sessionId"]
            .as_str()
            .ok_or_else(|| BrowsingError::Cdp("No sessionId in response".to_string()))?
            .to_string();

        // Enable domains
        let domains = domains.unwrap_or_else(|| {
            vec![
                "Page".to_string(),
                "DOM".to_string(),
                "DOMSnapshot".to_string(),
                "Accessibility".to_string(),
                "Runtime".to_string(),
                "Inspector".to_string(),
            ]
        });

        for domain in &domains {
            let method = format!("{domain}.enable");
            let _ = client.send_command_with_session(&method, serde_json::json!({}), Some(&session_id)).await;
        }

        // Get target info
        let target_info_params = serde_json::json!({"targetId": target_id});
        let target_info = client
            .send_command("Target.getTargetInfo", target_info_params)
            .await?;

        let title = target_info["targetInfo"]["title"]
            .as_str()
            .unwrap_or("Unknown title")
            .to_string();
        let url = target_info["targetInfo"]["url"]
            .as_str()
            .unwrap_or("about:blank")
            .to_string();

        Ok(Self {
            client,
            target_id,
            session_id,
            title,
            url,
        })
    }
}
