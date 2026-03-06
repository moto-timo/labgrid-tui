use std::collections::BTreeMap;

use anyhow::{Context, Result};
use futures_util::StreamExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

use super::proto::{
    self, coordinator_client::CoordinatorClient as TonicClient, client_in_message, subscribe,
    update_response, ClientInMessage, StartupDone, Subscribe, Sync as ProtoSync,
};
use crate::model::{MatchInfo, PlaceInfo, ResourceInfo, ResourcePath};

/// Messages sent from the gRPC client to the app state.
#[derive(Debug)]
pub enum CoordinatorEvent {
    Connected,
    Disconnected(String),
    ResourceUpdate(ResourceInfo),
    ResourceRemoved(ResourcePath),
    PlaceUpdate(PlaceInfo),
    PlaceRemoved(String),
    SyncComplete(u64),
}

/// Wraps the gRPC connection to a labgrid coordinator.
pub struct CoordinatorClient {
    url: String,
}

impl CoordinatorClient {
    pub fn new(url: String) -> Self {
        Self { url }
    }

    /// Connect to the coordinator and begin streaming updates.
    /// Sends CoordinatorEvents through the provided channel.
    pub async fn run(&self, event_tx: mpsc::UnboundedSender<CoordinatorEvent>) -> Result<()> {
        info!(url = %self.url, "connecting to coordinator");

        let channel = if self.url.starts_with("ws://") || self.url.starts_with("wss://") {
            self.connect_websocket().await?
        } else {
            self.connect_http2().await?
        };

        let mut client = TonicClient::new(channel);

        // Build the initial handshake messages that must be sent immediately.
        // The coordinator (grpclib) won't send response headers until it
        // receives StartupDone, so we use stream::iter to guarantee these
        // are yielded before tonic waits for the response.
        let initial_messages = vec![
            ClientInMessage {
                kind: Some(client_in_message::Kind::Startup(StartupDone {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    name: "labgrid-tui".to_string(),
                })),
            },
            ClientInMessage {
                kind: Some(client_in_message::Kind::Subscribe(Subscribe {
                    is_unsubscribe: None,
                    kind: Some(subscribe::Kind::AllPlaces(true)),
                })),
            },
            ClientInMessage {
                kind: Some(client_in_message::Kind::Subscribe(Subscribe {
                    is_unsubscribe: None,
                    kind: Some(subscribe::Kind::AllResources(true)),
                })),
            },
            ClientInMessage {
                kind: Some(client_in_message::Kind::Sync(ProtoSync { id: 1 })),
            },
        ];
        debug!("opening stream with {} initial messages", initial_messages.len());

        // Chain: yield initial messages immediately, then listen for future
        // messages via the mpsc channel (for future sync requests, etc.)
        let (out_tx, out_rx) = mpsc::channel::<ClientInMessage>(32);
        let out_stream =
            futures_util::stream::iter(initial_messages).chain(ReceiverStream::new(out_rx));

        // Open bidirectional stream with a timeout to avoid hanging forever.
        let response = match tokio::time::timeout(
            std::time::Duration::from_secs(15),
            client.client_stream(out_stream),
        )
        .await
        {
            Ok(Ok(r)) => {
                info!("gRPC ClientStream opened");
                r
            }
            Ok(Err(e)) => {
                error!(error = ?e, "gRPC ClientStream failed");
                return Err(e).with_context(|| {
                    format!("failed to open ClientStream on {}", self.url)
                });
            }
            Err(_) => {
                error!("timed out waiting for ClientStream response headers (15s)");
                anyhow::bail!(
                    "timed out opening ClientStream on {} — coordinator may not \
                     support this gRPC transport",
                    self.url
                );
            }
        };

        let mut in_stream = response.into_inner();
        let _ = event_tx.send(CoordinatorEvent::Connected);
        // Keep out_tx alive for sending future messages (periodic sync, etc.)
        let _out_tx = out_tx;

        // Process incoming messages
        loop {
            match in_stream.message().await {
                Ok(Some(msg)) => {
                    // Handle sync response
                    if let Some(ref sync) = msg.sync {
                        debug!(id = sync.id, "sync complete");
                        let _ = event_tx.send(CoordinatorEvent::SyncComplete(sync.id));
                    }

                    // Handle updates
                    for update in msg.updates {
                        match update.kind {
                            Some(update_response::Kind::Resource(res)) => {
                                if let Some(info) = convert_resource(&res) {
                                    let _ =
                                        event_tx.send(CoordinatorEvent::ResourceUpdate(info));
                                }
                            }
                            Some(update_response::Kind::DelResource(path)) => {
                                if let Some(p) = convert_resource_path(&path) {
                                    let _ =
                                        event_tx.send(CoordinatorEvent::ResourceRemoved(p));
                                }
                            }
                            Some(update_response::Kind::Place(place)) => {
                                let info = convert_place(&place);
                                let _ = event_tx.send(CoordinatorEvent::PlaceUpdate(info));
                            }
                            Some(update_response::Kind::DelPlace(name)) => {
                                let _ = event_tx.send(CoordinatorEvent::PlaceRemoved(name));
                            }
                            None => {
                                warn!("received update with no kind");
                            }
                        }
                    }
                }
                Ok(None) => {
                    info!("coordinator stream ended");
                    let _ = event_tx
                        .send(CoordinatorEvent::Disconnected("stream ended".into()));
                    break;
                }
                Err(e) => {
                    error!(error = %e, "coordinator stream error");
                    let _ = event_tx
                        .send(CoordinatorEvent::Disconnected(e.to_string()));
                    break;
                }
            }
        }

        Ok(())
    }

    /// Standard HTTP/2 gRPC connection (for http:// URLs).
    async fn connect_http2(&self) -> Result<Channel> {
        let endpoint = Channel::from_shared(self.url.clone())
            .context("invalid coordinator URL")?
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30));

        match endpoint.connect().await {
            Ok(ch) => {
                info!("HTTP/2 connection established");
                Ok(ch)
            }
            Err(e) => {
                error!(error = ?e, "HTTP/2 connect failed");
                Err(e).with_context(|| {
                    format!("failed to connect to coordinator at {}", self.url)
                })
            }
        }
    }

    /// WebSocket gRPC connection (for ws:// URLs).
    ///
    /// labgrid's coordinator serves gRPC over WebSocket, not standard HTTP/2.
    /// We create a local TCP proxy that bridges between tonic (HTTP/2) and the
    /// coordinator's WebSocket endpoint.
    async fn connect_websocket(&self) -> Result<Channel> {
        use tokio_tungstenite::connect_async;

        // 1. Establish WebSocket connection to coordinator first
        info!(url = %self.url, "opening WebSocket connection");
        let (ws_stream, _response) = connect_async(&self.url)
            .await
            .with_context(|| format!("WebSocket connection failed to {}", self.url))?;
        info!("WebSocket connection established");

        // 2. Create a local TCP listener for tonic to connect to
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .context("failed to bind local proxy listener")?;
        let local_addr = listener.local_addr()?;
        debug!(addr = %local_addr, "local WebSocket proxy listening");

        // 3. Spawn bridge: accept one TCP connection from tonic, bridge to WS
        tokio::spawn(async move {
            match listener.accept().await {
                Ok((tcp_stream, _)) => {
                    bridge_tcp_ws(tcp_stream, ws_stream).await;
                }
                Err(e) => {
                    error!(error = ?e, "local proxy accept failed");
                }
            }
        });

        // 4. Connect tonic to the local proxy
        let proxy_url = format!("http://127.0.0.1:{}", local_addr.port());
        let endpoint = Channel::from_shared(proxy_url)
            .context("invalid proxy endpoint")?
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(30));

        let channel = endpoint
            .connect()
            .await
            .context("failed to connect to local WebSocket proxy")?;

        info!("gRPC-over-WebSocket channel ready");
        Ok(channel)
    }
}

/// Bridge raw bytes between a TCP stream (tonic/h2) and a WebSocket stream
/// (labgrid coordinator). HTTP/2 frames flow through WebSocket binary messages.
async fn bridge_tcp_ws(
    tcp: tokio::net::TcpStream,
    ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
) {
    use futures_util::{SinkExt, StreamExt};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio_tungstenite::tungstenite::Message;

    let (mut tcp_read, mut tcp_write) = tokio::io::split(tcp);
    let (mut ws_sink, mut ws_source) = ws.split();

    // tonic → WebSocket: read bytes from TCP, send as WS binary frames
    let tcp_to_ws = async {
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            match tcp_read.read(&mut buf).await {
                Ok(0) => {
                    debug!("TCP read EOF, closing WS");
                    let _ = ws_sink.send(Message::Close(None)).await;
                    break;
                }
                Ok(n) => {
                    if ws_sink
                        .send(Message::Binary(buf[..n].to_vec()))
                        .await
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    debug!(error = ?e, "TCP read error");
                    break;
                }
            }
        }
    };

    // WebSocket → tonic: receive WS binary frames, write bytes to TCP
    let ws_to_tcp = async {
        while let Some(msg) = ws_source.next().await {
            match msg {
                Ok(Message::Binary(data)) => {
                    if tcp_write.write_all(&data).await.is_err() {
                        break;
                    }
                    if tcp_write.flush().await.is_err() {
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    debug!("WebSocket close received");
                    break;
                }
                Ok(Message::Ping(payload)) => {
                    // Pong is handled automatically by tungstenite
                    debug!(len = payload.len(), "WS ping");
                }
                Ok(_) => {} // Ignore text frames, pong, etc.
                Err(e) => {
                    debug!(error = ?e, "WebSocket read error");
                    break;
                }
            }
        }
    };

    tokio::select! {
        _ = tcp_to_ws => { debug!("bridge ended: tcp→ws finished"); }
        _ = ws_to_tcp => { debug!("bridge ended: ws→tcp finished"); }
    }
}

// --- Proto conversion helpers (unchanged) ---

fn convert_map_value(val: &proto::MapValue) -> String {
    match &val.kind {
        Some(proto::map_value::Kind::BoolValue(b)) => b.to_string(),
        Some(proto::map_value::Kind::IntValue(i)) => i.to_string(),
        Some(proto::map_value::Kind::UintValue(u)) => u.to_string(),
        Some(proto::map_value::Kind::FloatValue(f)) => f.to_string(),
        Some(proto::map_value::Kind::StringValue(s)) => s.clone(),
        None => String::new(),
    }
}

fn convert_resource_path(path: &proto::resource::Path) -> Option<ResourcePath> {
    // exporter_name is optional; group_name and resource_name are plain strings
    Some(ResourcePath {
        exporter: path.exporter_name.clone()?,
        group: path.group_name.clone(),
        name: path.resource_name.clone(),
    })
}

fn convert_resource(res: &proto::Resource) -> Option<ResourceInfo> {
    let path = res.path.as_ref().and_then(convert_resource_path)?;

    let params: BTreeMap<String, String> = res
        .params
        .iter()
        .map(|(k, v)| (k.clone(), convert_map_value(v)))
        .collect();

    let extra: BTreeMap<String, String> = res
        .extra
        .iter()
        .map(|(k, v)| (k.clone(), convert_map_value(v)))
        .collect();

    let acquired = if res.acquired.is_empty() {
        None
    } else {
        Some(res.acquired.clone())
    };

    Some(ResourceInfo {
        path,
        cls: res.cls.clone(),
        params,
        extra,
        acquired,
        available: res.avail,
    })
}

fn convert_place(place: &proto::Place) -> PlaceInfo {
    let matches = place
        .matches
        .iter()
        .map(|m| MatchInfo {
            exporter: m.exporter.clone(),
            group: m.group.clone(),
            cls: m.cls.clone(),
            name: m.name.clone(),
            rename: m.rename.clone(),
        })
        .collect();

    // acquired_resources is `repeated string` in the upstream proto
    let acquired_resources = place.acquired_resources.clone();

    PlaceInfo {
        name: place.name.clone(),
        aliases: place.aliases.clone(),
        comment: place.comment.clone(),
        tags: place
            .tags
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect(),
        matches,
        acquired: place.acquired.clone(),
        acquired_resources,
        allowed: place.allowed.clone(),
        created: place.created,
        changed: place.changed,
        reservation: place.reservation.clone(),
    }
}
