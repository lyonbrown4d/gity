//! Shared transport contracts; implementations execute existing application use cases.
mod context;
mod operation;
use async_trait::async_trait;
pub use context::{ApiContext, ApiTransport};
pub use operation::{ApiError, ApiResult, OperationContract, OperationKind};
use serde_json::Value;
use std::sync::Arc;

/// Object-safe application port used by protocol adapters, without HTTP or ORM types.
#[async_trait]
pub trait ApplicationApi: Send + Sync {
  fn operations(&self) -> Arc<[OperationContract]>;
  async fn authenticate(&self, context: &ApiContext) -> ApiResult<()>;
  async fn invoke(
    &self,
    context: &ApiContext,
    operation: &str,
    arguments: Value,
  ) -> ApiResult<Value>;
}

#[derive(Clone)]
pub struct ApplicationApiHandle(pub Arc<dyn ApplicationApi>);
