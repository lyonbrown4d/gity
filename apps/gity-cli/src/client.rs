//! Thin HTTP client over the Gity REST API, unwrapping its `{code,data}` envelope.
use crate::config;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::path::PathBuf;

#[derive(Clone, Debug, Default)]
pub struct Options {
  pub host: Option<String>,
  pub token: Option<String>,
  pub json: bool,
  pub config_file: PathBuf,
}

#[derive(Clone)]
pub struct Client {
  http: reqwest::Client,
  base: String,
  token: String,
}

impl Client {
  pub fn new(host: &str, token: &str) -> Self {
    Self {
      http: reqwest::Client::new(),
      base: format!("{}/api/v1", host.trim_end_matches('/')),
      token: token.to_owned(),
    }
  }

  /// Builds a client from resolved credentials. `required` allows commands such
  /// as `auth login` to run without a stored token.
  pub fn from_options(options: &Options) -> Result<Client> {
    let resolved = config::resolve(options)?;
    Ok(Self::new(&resolved.host, &resolved.token))
  }

  pub async fn get(&self, path: &str) -> Result<Value> {
    self.send(reqwest::Method::GET, path, None).await
  }

  pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
    self.send(reqwest::Method::POST, path, Some(body)).await
  }

  pub async fn put(&self, path: &str, body: Value) -> Result<Value> {
    self.send(reqwest::Method::PUT, path, Some(body)).await
  }

  pub async fn patch(&self, path: &str, body: Value) -> Result<Value> {
    self.send(reqwest::Method::PATCH, path, Some(body)).await
  }

  pub async fn delete(&self, path: &str) -> Result<Value> {
    self.send(reqwest::Method::DELETE, path, None).await
  }

  pub async fn send(
    &self,
    method: reqwest::Method,
    path: &str,
    body: Option<Value>,
  ) -> Result<Value> {
    let url = format!("{}{}", self.base, path);
    let mut request = self
      .http
      .request(method.clone(), &url)
      .bearer_auth(&self.token)
      .header("accept", "application/json");
    if let Some(body) = body {
      request = request.json(&body);
    }
    let response = request
      .send()
      .await
      .with_context(|| format!("{method} {url} failed"))?;
    let status = response.status();
    let payload: Value = response
      .json()
      .await
      .with_context(|| format!("{method} {url} returned a non-JSON body"))?;
    let code = payload["code"].as_i64().unwrap_or(-1);
    if !status.is_success() || code != 0 {
      let message = payload["message"]
        .as_str()
        .filter(|value| !value.is_empty())
        .unwrap_or("request failed");
      bail!("{message} (HTTP {})", status.as_u16());
    }
    Ok(payload["data"].clone())
  }
}

/// Accepts either a numeric project id or a `group/project` path.
pub async fn project_id(client: &Client, reference: &str) -> Result<String> {
  if reference.parse::<i64>().is_ok() {
    return Ok(reference.to_owned());
  }
  let projects = client.get("/projects").await?;
  let wanted = reference.trim_matches('/');
  projects
    .as_array()
    .into_iter()
    .flatten()
    .find(|project| {
      ["full_path", "path", "name"]
        .iter()
        .filter_map(|key| project[*key].as_str())
        .any(|value| value == wanted || value.rsplit('/').next() == Some(wanted))
    })
    .and_then(|project| project["id"].as_str().map(str::to_owned))
    .ok_or_else(|| anyhow::anyhow!("project '{reference}' not found"))
}
