use super::{Credential, field, request};
use crate::git_stress::{Config, Transport, transport};
use anyhow::{Context, Result, bail};
use reqwest::blocking::Client;
use serde_json::json;

pub(super) fn issue(
  client: &Client,
  config: &Config,
  project: &str,
  api_token: &str,
  user: i64,
  run_id: &str,
) -> Result<(String, Credential)> {
  if config.transport == Transport::Ssh {
    let public = transport::public_key(config.ssh_key.as_deref().context("SSH key missing")?)?;
    let row = request(
      client,
      &config.base_url,
      "POST",
      &format!("/projects/{project}/deploy-keys"),
      Some(api_token),
      Some(json!({
        "title":format!("{run_id}-ssh"),"public_key":public,"can_push":true
      })),
    )?;
    let id = field(&row, &["id"]);
    if id.is_empty() {
      bail!("deploy-key response omitted its ID");
    }
    return Ok((String::new(), Credential::DeployKey(id)));
  }
  let row = request(
    client,
    &config.base_url,
    "POST",
    &format!("/projects/{project}/access-tokens"),
    Some(api_token),
    Some(json!({
      "name":format!("{run_id}-git-token"),"username":config.token_username,
      "scopes":["read_repository","write_repository"],"created_by_user_id":user,"expires_at":""
    })),
  )?;
  let token = field(&row, &["token"]);
  let id = row
    .get("project_token")
    .map(|v| field(v, &["id"]))
    .unwrap_or_default();
  if token.is_empty() || id.is_empty() {
    bail!("project access-token response omitted token or token ID");
  }
  Ok((token, Credential::AccessToken(id)))
}
