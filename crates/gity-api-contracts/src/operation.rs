use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
  Query,
  Mutation,
}

/// Contracts are produced from existing annotated business operations, not database tables.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OperationContract {
  pub name: String,
  pub description: String,
  pub group: String,
  pub kind: OperationKind,
  pub input_schema: Value,
  pub output_schema: Value,
  pub destructive: bool,
  pub idempotent: bool,
}
#[derive(Clone, Debug, thiserror::Error, Serialize, Deserialize)]
#[error("{message}")]
pub struct ApiError {
  pub code: String,
  pub message: String,
  pub status: u16,
}
impl ApiError {
  pub fn new(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self {
    Self {
      status,
      code: code.into(),
      message: message.into(),
    }
  }
}
pub type ApiResult<T> = Result<T, ApiError>;
