use axum::{
  Json,
  http::StatusCode,
  response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::{Value, json};

#[derive(Debug)]
pub struct AppError {
  pub status: StatusCode,
  pub message: String,
}
impl AppError {
  pub fn bad_request(message: impl Into<String>) -> Self {
    Self::new(StatusCode::BAD_REQUEST, message)
  }
  pub fn unauthorized(message: impl Into<String>) -> Self {
    Self::new(StatusCode::UNAUTHORIZED, message)
  }
  pub fn forbidden(message: impl Into<String>) -> Self {
    Self::new(StatusCode::FORBIDDEN, message)
  }
  pub fn not_found(message: impl Into<String>) -> Self {
    Self::new(StatusCode::NOT_FOUND, message)
  }
  pub fn conflict(message: impl Into<String>) -> Self {
    Self::new(StatusCode::CONFLICT, message)
  }
  pub fn internal(message: impl Into<String>) -> Self {
    Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
  }
  pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
    Self {
      status,
      message: message.into(),
    }
  }
}
impl std::fmt::Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.message)
  }
}
impl std::error::Error for AppError {}
impl From<sea_orm::DbErr> for AppError {
  fn from(error: sea_orm::DbErr) -> Self {
    tracing::error!(error = %error, "database operation failed");
    match error.sql_err() {
      Some(sea_orm::SqlErr::UniqueConstraintViolation(_)) => {
        Self::conflict("resource already exists")
      }
      Some(sea_orm::SqlErr::ForeignKeyConstraintViolation(_)) => {
        Self::bad_request("referenced resource does not exist")
      }
      _ => Self::internal("database operation failed"),
    }
  }
}
impl From<std::io::Error> for AppError {
  fn from(e: std::io::Error) -> Self {
    tracing::error!(error=%e,"storage operation failed");
    Self::internal("storage operation failed")
  }
}
impl IntoResponse for AppError {
  fn into_response(self) -> Response {
    let message = if self.status.is_server_error() {
      tracing::error!(message=%self.message,"request failed");
      "internal server error".to_owned()
    } else {
      self.message
    };
    let message = gity_i18n::translate_message(crate::locale::current_language(), &message);
    let mut response = (
      self.status,
      Json(json!({"code":self.status.as_u16(),"message":message,"data":null})),
    )
      .into_response();
    if self.status == StatusCode::UNAUTHORIZED {
      response.headers_mut().insert(
        axum::http::header::WWW_AUTHENTICATE,
        axum::http::HeaderValue::from_static("Bearer realm=\"Gity\""),
      );
    }
    response
  }
}
pub fn externalize(mut value: Value) -> Value {
  normalize_timestamps(&mut value);
  match &mut value {
    Value::Object(object) => {
      for (key, item) in object.iter_mut() {
        if (key == "id" || key.ends_with("_id")) && item.is_number() {
          *item = Value::String(item.to_string());
        } else {
          *item = externalize(item.take());
        }
      }
    }
    Value::Array(items) => {
      for item in items {
        *item = externalize(item.take());
      }
    }
    _ => {}
  }
  value
}
pub(crate) fn normalize_timestamps(value: &mut Value) {
  match value {
    Value::Object(map) => {
      for (key, value) in map {
        if key.ends_with("_at") || ["run_after", "locked_until"].contains(&key.as_str()) {
          if let Some(text) = value.as_str() {
            let parsed = chrono::DateTime::parse_from_rfc3339(text)
              .map(|v| v.with_timezone(&chrono::Utc))
              .ok()
              .or_else(|| {
                ["%Y-%m-%dT%H:%M:%S%.f", "%Y-%m-%d %H:%M:%S%.f"]
                  .into_iter()
                  .find_map(|format| {
                    chrono::NaiveDateTime::parse_from_str(text, format)
                      .ok()
                      .map(|v| v.and_utc())
                  })
              });
            if let Some(date) = parsed {
              *value = Value::String(date.to_rfc3339_opts(chrono::SecondsFormat::Millis, true));
            }
          }
        } else {
          normalize_timestamps(value);
        }
      }
    }
    Value::Array(values) => {
      for value in values {
        normalize_timestamps(value);
      }
    }
    _ => {}
  }
}
pub fn ok(data: impl Serialize) -> Json<Value> {
  Json(
    json!({"code":0,"message":"OK","data":externalize(serde_json::to_value(data).expect("serializable response"))}),
  )
}
