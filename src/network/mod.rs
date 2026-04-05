pub mod session;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::protocol::{ClientMessage, ServerMessage};

pub struct NetworkClient {
    /// Send commands to the server (game loop -> network thread)
    cmd_tx: mpsc::Sender<ClientMessage>,
    /// Receive events from the server (network thread -> game loop)
    event_rx: mpsc::Receiver<ServerMessage>,
    /// Whether currently connected
    pub connected: Arc<AtomicBool>,
}

impl NetworkClient {
    /// Connect to the server. Spawns a background thread with a tokio runtime.
    pub fn connect(server_url: &str) -> Self {
        let (cmd_tx, mut cmd_rx) = mpsc::channel::<ClientMessage>(64);
        let (event_tx, event_rx) = mpsc::channel::<ServerMessage>(64);
        let connected = Arc::new(AtomicBool::new(false));
        let connected_clone = connected.clone();
        let url = server_url.to_string();

        // Spawn a background thread (same pattern as AIController in engine/integration.rs)
        std::thread::Builder::new()
            .name("network".into())
            .spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .unwrap();

                rt.block_on(async move {
                    // Connect to WebSocket
                    let ws_result = tokio_tungstenite::connect_async(&url).await;
                    let ws_stream = match ws_result {
                        Ok((stream, _)) => stream,
                        Err(e) => {
                            let _ = event_tx
                                .send(ServerMessage::Error {
                                    message: format!("Connection failed: {}", e),
                                })
                                .await;
                            return;
                        }
                    };

                    connected_clone.store(true, Ordering::Relaxed);

                    use futures_util::{SinkExt, StreamExt};
                    let (mut ws_sink, mut ws_stream) = ws_stream.split();

                    // Two concurrent tasks:
                    // 1. cmd_rx -> ws_sink (send commands to server)
                    // 2. ws_stream -> event_tx (receive events from server)

                    let send_task = tokio::spawn(async move {
                        while let Some(cmd) = cmd_rx.recv().await {
                            if let Ok(json) = serde_json::to_string(&cmd) {
                                use tokio_tungstenite::tungstenite::Message;
                                if ws_sink.send(Message::Text(json.into())).await.is_err() {
                                    break;
                                }
                            }
                        }
                    });

                    let event_tx_clone = event_tx.clone();
                    let recv_task = tokio::spawn(async move {
                        while let Some(Ok(msg)) = ws_stream.next().await {
                            use tokio_tungstenite::tungstenite::Message;
                            match msg {
                                Message::Text(text) => {
                                    if let Ok(server_msg) =
                                        serde_json::from_str::<ServerMessage>(&text)
                                    {
                                        if event_tx_clone.send(server_msg).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                                Message::Close(_) => break,
                                _ => {}
                            }
                        }
                    });

                    // Wait for either task to finish (means disconnection)
                    tokio::select! {
                        _ = send_task => {}
                        _ = recv_task => {}
                    }

                    connected_clone.store(false, Ordering::Relaxed);
                });
            })
            .expect("Failed to spawn network thread");

        Self {
            cmd_tx,
            event_rx,
            connected,
        }
    }

    /// Send a command to the server. Non-blocking (uses try_send).
    pub fn send(&self, msg: ClientMessage) {
        let _ = self.cmd_tx.try_send(msg);
    }

    /// Try to receive a server message. Non-blocking.
    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        self.event_rx.try_recv().ok()
    }

    /// Check if connected.
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Relaxed)
    }
}
