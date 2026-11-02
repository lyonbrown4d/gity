use super::{Credential, Repository, request};
use anyhow::{Result, bail};
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Duration;

pub fn cleanup(repository: &Repository) -> Result<()> {
  let Some(resources) = &repository.cleanup else {
    return Ok(());
  };
  let client = Client::builder().timeout(Duration::from_secs(20)).build()?;
  let credential_path = resources
    .credential
    .as_ref()
    .map(|credential| match credential {
      Credential::AccessToken(id) => {
        format!("/projects/{}/access-tokens/{id}", resources.project_id)
      }
      Credential::DeployKey(id) => format!("/projects/{}/deploy-keys/{id}", resources.project_id),
    });
  let token_result = credential_path.map_or(Ok(serde_json::Value::Null), |path| {
    request(
      &client,
      &resources.base_url,
      "DELETE",
      &path,
      Some(&resources.api_token),
      None,
    )
  });
  let project_result = request(
    &client,
    &resources.base_url,
    "DELETE",
    &format!("/projects/{}", resources.project_id),
    Some(&resources.api_token),
    Some(json!({"confirmation": resources.project_path})),
  );
  let session_result = request(
    &client,
    &resources.base_url,
    "POST",
    "/auth/logout",
    Some(&resources.api_token),
    Some(json!({"refresh_token": ""})),
  );
  let failures = [
    token_result
      .err()
      .map(|error| format!("credential revocation: {error}")),
    project_result
      .err()
      .map(|error| format!("project deletion request: {error}")),
    session_result
      .err()
      .map(|error| format!("session logout: {error}")),
  ]
  .into_iter()
  .flatten()
  .collect::<Vec<_>>();
  if failures.is_empty() {
    println!(
      "Marked stress project for deletion and revoked its Git credential: user_id={} organization_id={} project_id={}; the account and organization remain for test-database lifecycle cleanup.",
      resources.user_id, resources.organization_id, resources.project_id
    );
    Ok(())
  } else {
    bail!(
      "stress resource cleanup incomplete: {}",
      failures.join("; ")
    )
  }
}
