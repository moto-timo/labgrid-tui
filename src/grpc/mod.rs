pub mod client;

// Re-exported for external consumers of the library crate.
#[allow(unused_imports)]
pub use client::CoordinatorClient;

/// Generated protobuf types from labgrid-coordinator.proto.
pub mod proto {
    tonic::include_proto!("labgrid");
}
