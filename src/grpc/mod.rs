pub mod client;

pub use client::CoordinatorClient;

/// Generated protobuf types from labgrid-coordinator.proto.
pub mod proto {
    tonic::include_proto!("labgrid");
}
