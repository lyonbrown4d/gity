//! HTTP client for the Gity REST API.
//!
//! The server answers every endpoint with an envelope (`{code, data, message}`),
//! so the client unwraps `data` and turns non-zero codes into [`Error::Api`].
//! Paths are given relative to the API root, for example `/projects/1/issues`.
use serde_json::Value;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("request to {url} failed")]
  Transport {
    url: String,
    #[source]
    source: reqwest::Error,
  },
  #[error("{url} did not return the expected JSON envelope")]
  Envelope {
    url: String,
    #[source]
    source: reqwest::Error,
  },
  #[error("{message} (HTTP {status})")]
  Api { status: u16, message: String },
  #[error("project '{0}' not found")]
  ProjectNotFound(String),
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone)]
pub struct Client {
  http: reqwest::Client,
  base: String,
  token: String,
}

impl Client {
  /// A client without credentials, used for `auth/login` and `register`.
  pub fn anonymous(host: &str) -> Self {
    Self::new(host, "")
  }

  /// `host` is the server origin, for example `http://127.0.0.1:8080`.
  pub fn new(host: &str, token: &str) -> Self {
    Self {
      http: reqwest::Client::new(),
      base: format!("{}/api/v1", host.trim_end_matches('/')),
      token: token.to_owned(),
    }
  }

  pub fn base_url(&self) -> &str {
    &self.base
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
      .request(method, &url)
      .header("accept", "application/json");
    if !self.token.is_empty() {
      request = request.bearer_auth(&self.token);
    }
    if let Some(body) = body {
      request = request.json(&body);
    }
    let response = request.send().await.map_err(|source| Error::Transport {
      url: url.clone(),
      source,
    })?;
    let status = response.status();
    let payload: Value = response.json().await.map_err(|source| Error::Envelope {
      url: url.clone(),
      source,
    })?;
    let code = payload["code"].as_i64().unwrap_or(-1);
    if !status.is_success() || code != 0 {
      let message = payload["message"]
        .as_str()
        .filter(|value| !value.is_empty())
        .unwrap_or("request failed")
        .to_owned();
      return Err(Error::Api {
        status: status.as_u16(),
        message,
      });
    }
    Ok(payload["data"].clone())
  }

  /// Accepts a numeric project id or a `group/project` (or bare) project path.
  pub async fn project_id(&self, reference: &str) -> Result<String> {
    if reference.parse::<i64>().is_ok() {
      return Ok(reference.to_owned());
    }
    let projects = self.get("/projects").await?;
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
      .ok_or_else(|| Error::ProjectNotFound(reference.to_owned()))
  }
}
