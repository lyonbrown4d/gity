//! Independently deployable owner of Git repository storage.
mod control;
mod dispatch;
mod module;
mod replication;
mod service;
mod state;
mod transfer;
pub use module::{GitNodeModule, GitNodeSettings};
pub use service::GitNodeService;
