use axum::Json;
use gity_api_contracts::{ApiError, ApiResult};
use gity_core::AppError;
use serde::Serialize;
use serde_json::Value;

pub(crate) trait IntoApiOutput {
  fn into_api_output(self) -> ApiResult<Value>;
}
impl<T: Serialize> IntoApiOutput for Result<Json<T>, AppError> {
  fn into_api_output(self) -> ApiResult<Value> {
    self.map_err(convert)?.into_api_output()
  }
}
impl<T: Serialize> IntoApiOutput for Json<T> {
  fn into_api_output(self) -> ApiResult<Value> {
    let value = serde_json::to_value(self.0)
      .map_err(|_| ApiError::new(500, "INTERNAL", "internal server error"))?;
    Ok(gity_core::externalize(
      if value.get("code").is_some() && value.get("data").is_some() {
        value["data"].clone()
      } else {
        value
      },
    ))
  }
}
pub(crate) fn convert(error: AppError) -> ApiError {
  let status = error.status.as_u16();
  let code = match status {
    400 | 422 => "INVALID_ARGUMENT",
    401 => "UNAUTHENTICATED",
    403 => "FORBIDDEN",
    404 => "NOT_FOUND",
    409 => "CONFLICT",
    429 => "RESOURCE_EXHAUSTED",
    _ => "INTERNAL",
  };
  let message = if status >= 500 {
    "internal server error".into()
  } else {
    error.message
  };
  ApiError::new(
    status,
    code,
    gity_i18n::translate_message(gity_core::locale::current_language(), &message),
  )
}

pub(crate) fn error(status: u16, code: &str, message: impl Into<String>) -> ApiError {
  ApiError::new(
    status,
    code,
    gity_i18n::translate_message(gity_core::locale::current_language(), &message.into()),
  )
}
