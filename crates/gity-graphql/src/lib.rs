//! Schema-driven GraphQL adapter for application operations.
mod config;
mod input;
mod output;
mod refs;
mod schema;
mod service;

pub use config::GraphqlConfig;
pub use service::{GraphqlModule, GraphqlService};

#[cfg(test)]
mod tests;
