//! Primary-ack Git sharding, current-replica reads and controlled fenced promotion.
mod domain;
mod module;
mod promotion;
mod replication;
mod repository;
mod router;
pub use domain::{Placement, Replica, ReplicaStatus};
pub use module::GitClusterModule;
pub use router::GitCluster;
