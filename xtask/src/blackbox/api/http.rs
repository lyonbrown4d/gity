use super::super::RunConfig;
use anyhow::{Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::Value;

pub fn request_json(
  client: &Client,
  config: &RunConfig,
  method: Method,
  path: &str,
  token: Option<&str>,
  body: Option<Value>,
) -> Result<(StatusCode, Value)> {
  let mut request = client.request(method, format!("{}{}", config.api_url, path));
  if let Some(token) = token {
    request = request.bearer_auth(token);
  }
  if let Some(body) = body {
    request = request.json(&body);
  }
  let response = request.send()?;
  let status = response.status();
  let bytes = response.bytes()?;
  Ok((
    status,
    serde_json::from_slice(&bytes).unwrap_or(Value::Null),
  ))
}

pub fn data(value: &Value) -> &Value {
  value.get("data").unwrap_or(value)
}

pub fn require_status(actual: StatusCode, expected: StatusCode, action: &str) -> Result<()> {
  if actual != expected {
    bail!("{action} expected HTTP {expected}, received HTTP {actual}");
  }
  Ok(())
}

pub fn require_denied(status: StatusCode, action: &str) -> Result<()> {
  if !matches!(
    status,
    StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN | StatusCode::NOT_FOUND
  ) {
    bail!("{action} expected authorization denial, received HTTP {status}");
  }
  Ok(())
}

pub fn id(value: &Value, description: &str) -> Result<String> {
  match value {
    Value::String(value) => Ok(value.clone()),
    Value::Number(value) => Ok(value.to_string()),
    _ => bail!("{description} response did not include an identifier"),
  }
}

pub fn check_openapi_routes(spec: &Value) -> Result<()> {
  let paths = spec["paths"]
    .as_object()
    .ok_or_else(|| anyhow::anyhow!("OpenAPI document has no route map"))?;
  for route in [
    "/api/v1/auth/register",
    "/api/v1/auth/login",
    "/api/v1/projects",
    "/api/v1/projects/{id}/file-commits",
    "/api/v1/projects/{id}/packages/generic/{name}/{version}/{file}",
  ] {
    if !paths.contains_key(route) {
      anyhow::bail!("OpenAPI document is missing an exercised route");
    }
  }
  Ok(())
}
