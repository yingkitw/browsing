//! Chrome DevTools Protocol (CDP) client implementation

use crate::error::{BrowserUseError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

/// CDP client for WebSocket communication with Chrome
pub struct CdpClient {
    url: String,
    sender: Arc<Mutex<Option<mpsc::UnboundedSender<Message>>>>,
    receiver: Arc<Mutex<Option<mpsc::UnboundedReceiver<Value>>>>,
    request_id: Arc<Mutex<u64>>,
    pending_requests: Arc<Mutex<HashMap<u64, mpsc::UnboundedSender<Value>>>>,
}

impl CdpClient {
    pub fn new(url: String) -> Self {
        Self {
            url,
            sender: Arc::new(Mutex::new(None)),
            receiver: Arc::new(Mutex::new(None)),
            request_id: Arc::new(Mutex::new(0)),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn start(&mut self) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.url)
            .await
            .map_err(|e| BrowserUseError::Cdp(format!("Failed to connect to CDP: {}", e)))?;

        let (mut write, mut read) = ws_stream.split();
        let (tx, mut rx) = mpsc::unbounded_channel();
        let (tx_resp, mut rx_resp) = mpsc::unbounded_channel();

        *self.sender.lock().await = Some(tx);
        *self.receiver.lock().await = Some(rx_resp);

        let pending_requests = Arc::clone(&self.pending_requests);

        // Spawn task to handle incoming messages
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                        if let Err(e) = write.send(msg).await {
                            eprintln!("Error sending message: {}", e);
                            break;
                        }
                    }
                    Some(msg) = read.next() => {
                        match msg {
                            Ok(Message::Text(text)) => {
                                if let Ok(value) = serde_json::from_str::<Value>(&text) {
                                    if let Some(id_val) = value.get("id").and_then(|v| v.as_u64()) {
                                        if let Some(tx) = pending_requests.lock().await.remove(&id_val) {
                                            let _ = tx.send(value);
                                        }
                                    }
                                }
                            }
                            Ok(Message::Close(_)) => break,
                            Err(e) => {
                                eprintln!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {}
                        }
                    }
                }
            }
        });

        Ok(())
    }

    pub async fn send_command(&self, method: &str, params: Value) -> Result<Value> {
        self.send_command_with_session(method, params, None).await
    }

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
                .map_err(|e| BrowserUseError::Cdp(format!("Failed to send command: {}", e)))?;
        }

        // Wait for response
        if let Some(response) = rx.recv().await {
            if let Some(error) = response.get("error") {
                return Err(BrowserUseError::Cdp(format!("CDP error: {}", error)));
            }
            return Ok(response["result"].clone());
        }

        Err(BrowserUseError::Cdp("No response received".to_string()))
    }

    pub async fn stop(&mut self) -> Result<()> {
        // Close WebSocket connection
        if let Some(sender) = self.sender.lock().await.as_ref() {
            let _ = sender.send(Message::Close(None));
        }
        Ok(())
    }
}

/// CDP session for a specific target
pub struct CdpSession {
    pub client: Arc<CdpClient>,
    pub target_id: String,
    pub session_id: String,
    pub title: String,
    pub url: String,
}

impl CdpSession {
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

        let result = client
            .send_command("Target.attachToTarget", params)
            .await?;

        let session_id = result["sessionId"]
            .as_str()
            .ok_or_else(|| BrowserUseError::Cdp("No sessionId in response".to_string()))?
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
            let method = format!("{}.enable", domain);
            let _ = client.send_command(&method, serde_json::json!({})).await;
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

