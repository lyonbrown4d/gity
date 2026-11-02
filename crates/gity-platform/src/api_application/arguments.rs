use gity_api_contracts::{ApiError, ApiResult};
use serde::de::DeserializeOwned;
use serde_json::Value;

/// Arguments are already checked against the operation's local JSON Schema.
#[derive(Clone)]
pub(crate) struct CallArguments {
  pub path: Vec<Value>,
  pub query: Value,
  pub body: Value,
}
impl CallArguments {
  pub fn path<T: DeserializeOwned>(&self) -> ApiResult<T> {
    let single = self.path.first().cloned().unwrap_or(Value::Null);
    serde_json::from_value(single)
      .or_else(|_| serde_json::from_value(Value::Array(self.path.clone())))
      .map_err(|_| invalid("path"))
  }
  pub fn query<T: DeserializeOwned>(&self) -> ApiResult<T> {
    serde_json::from_value(self.query.clone())
      .or_else(|_| {
        let mut query = self.query.clone();
        if let Some(map) = query.as_object_mut() {
          for value in map.values_mut() {
            if !value.is_string() {
              *value = Value::String(value.to_string());
            }
          }
        }
        serde_json::from_value(query)
      })
      .map_err(|_| invalid("query"))
  }
  pub fn body<T: DeserializeOwned>(&self) -> ApiResult<T> {
    serde_json::from_value(self.body.clone()).map_err(|_| invalid("body"))
  }
}
fn invalid(part: &str) -> ApiError {
  super::output::error(400, "INVALID_ARGUMENT", format!("invalid {part} arguments"))
}
