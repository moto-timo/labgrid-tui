use std::collections::BTreeMap;

use anyhow::{Context, Result};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;
use tracing::{debug, error, info, warn};

use super::proto::{
    self, coordinator_client::CoordinatorClient as TonicClient, client_in_message, subscribe,
    update_response, ClientInMessage, StartupDone, Subscribe, Sync as ProtoSync,
};
use crate::model::{PlaceInfo, MatchInfo, ResourceInfo, ResourcePath};

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

        let channel = Channel::from_shared(self.url.clone())
            .context("invalid coordinator URL")?
            .connect()
            .await
            .context("failed to connect to coordinator")?;

        let mut client = TonicClient::new(channel);

        // Create the outbound message stream
        let (out_tx, out_rx) = mpsc::channel::<ClientInMessage>(32);
        let out_stream = ReceiverStream::new(out_rx);

        // Open bidirectional stream
        let response = client
            .client_stream(out_stream)
            .await
            .context("failed to open client stream")?;

        let mut in_stream = response.into_inner();

        // Send StartupDone
        out_tx
            .send(ClientInMessage {
                kind: Some(client_in_message::Kind::Startup(StartupDone {
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    name: "labgrid-tui".to_string(),
                })),
            })
            .await
            .context("failed to send StartupDone")?;

        // Subscribe to all places
        out_tx
            .send(ClientInMessage {
                kind: Some(client_in_message::Kind::Subscribe(Subscribe {
                    is_unsubscribe: None,
                    kind: Some(subscribe::Kind::AllPlaces(true)),
                })),
            })
            .await
            .context("failed to send subscribe places")?;

        // Subscribe to all resources
        out_tx
            .send(ClientInMessage {
                kind: Some(client_in_message::Kind::Subscribe(Subscribe {
                    is_unsubscribe: None,
                    kind: Some(subscribe::Kind::AllResources(true)),
                })),
            })
            .await
            .context("failed to send subscribe resources")?;

        // Send initial sync request
        out_tx
            .send(ClientInMessage {
                kind: Some(client_in_message::Kind::Sync(ProtoSync { id: 1 })),
            })
            .await
            .context("failed to send sync")?;

        let _ = event_tx.send(CoordinatorEvent::Connected);

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
}

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
    // exporter_name is `optional string` in proto → Option<String>
    // group_name and resource_name are plain `string` → String
    let exporter = path.exporter_name.clone()?;
    Some(ResourcePath {
        exporter,
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

    // acquired_resources is `repeated string` in the proto
    let acquired_resources = place.acquired_resources.clone();

    PlaceInfo {
        name: place.name.clone(),
        aliases: place.aliases.clone(),
        comment: place.comment.clone(),
        tags: place.tags.iter().map(|(k, v)| (k.clone(), v.clone())).collect(),
        matches,
        acquired: place.acquired.clone(),
        acquired_resources,
        allowed: place.allowed.clone(),
        created: place.created,
        changed: place.changed,
        reservation: place.reservation.clone(),
    }
}
