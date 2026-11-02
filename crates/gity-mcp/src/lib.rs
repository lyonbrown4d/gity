//! MCP Streamable HTTP adapter for the shared application API.
mod config;
mod handler;
mod http_adapter;
mod module;

pub use config::{McpConfig, McpConfigError};
pub use http_adapter::McpService;
pub use module::McpModule;

#[cfg(test)]
mod tests;
