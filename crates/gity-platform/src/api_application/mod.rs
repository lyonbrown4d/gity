//! Shared, authenticated application operations for the three API transports.
mod arguments;
pub(crate) mod catalog;
mod credentials;
mod ids;
pub(crate) mod module;
mod output;
#[cfg(feature = "packages")]
mod package_operations;
pub(crate) mod schemas;
mod service;
pub(crate) use crate::AppState;
pub(crate) use arguments::CallArguments;
pub(crate) use credentials::{context, headers};
pub(crate) use gity_api_contracts::ApiTransport;
use gity_api_contracts::{ApiContext, ApiResult};
pub(crate) use output::IntoApiOutput;
use serde_json::Value;
pub(crate) use service::PlatformApi;
use std::{future::Future, pin::Pin};

pub(crate) type InvocationFuture = Pin<Box<dyn Future<Output = ApiResult<Value>> + Send>>;
pub(crate) struct OperationRegistration {
  pub prefix: &'static str,
  pub method: &'static str,
  pub path: &'static str,
  pub handler: &'static str,
  pub invoke: fn(AppState, ApiContext, CallArguments) -> InvocationFuture,
}
inventory::collect!(OperationRegistration);

#[cfg(test)]
mod tests;

#[cfg(test)]
mod behavior_tests;

#[cfg(test)]
mod protocol_tests;
