use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ApiResponse<T> {
  pub code: i32,
  pub message: String,
  pub data: T,
}

impl<T> ApiResponse<T> {
  pub fn success(data: T) -> Self {
    Self {
      code: 0,
      message: "ok".to_string(),
      data,
    }
  }

  pub fn with_message(code: i32, message: impl Into<String>, data: T) -> Self {
    Self {
      code,
      message: message.into(),
      data,
    }
  }
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, ToSchema)]
pub struct EmptyData {}
