use crate::blackbox::{Reporter, RunConfig, api};
use anyhow::{Context, Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::json;

pub(super) fn check(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  reporter: &mut Reporter,
) -> Result<()> {
  let (status, response) = api::request_json(
    client,
    config,
    Method::POST,
    &format!("/projects/{}/deploy-tokens", scenario.project_id),
    Some(&scenario.owner_token),
    Some(
      json!({"name":"blackbox-mount-registry-writer","scopes":["read_registry","write_registry"]}),
    ),
  )?;
  api::require_status(
    status,
    StatusCode::OK,
    "create project-scoped registry writer token",
  )?;
  let data = api::data(&response);
  let record = &data["project_token"];
  let id = api::id(&record["id"], "registry mount writer token")?;
  let token = data["token"]
    .as_str()
    .context("registry mount writer token missing")?;
  reporter.remember_secret(token);
  let probes = (|| -> Result<()> {
    // A creator's super-admin role must never be used as the test identity:
    // this deploy token binds both read and write scopes to exactly one project.
    let scopes = record["scopes"]
      .as_array()
      .context("registry writer scopes missing")?;
    if api::id(&record["project_id"], "registry writer project")? != scenario.project_id
      || !["read_registry", "write_registry"]
        .iter()
        .all(|scope| scopes.iter().any(|value| value.as_str() == Some(*scope)))
    {
      bail!("registry mount token lacks the intended project/write scope");
    }
    for (target, source, action) in [
      (
        &scenario.project_path,
        &scenario.outsider_project_path,
        "inaccessible mount source falls back to a new upload",
      ),
      (
        &scenario.outsider_project_path,
        &scenario.project_path,
        "mount requires target project write ACL",
      ),
    ] {
      let response = client
        .post(format!(
          "{}/v2/{target}/blobs/uploads/?mount=sha256:{}&from={source}/source",
          config.oci_url,
          "a".repeat(64)
        ))
        .bearer_auth(token)
        .send()
        .with_context(|| action.to_owned())?;
      if target == &scenario.project_path {
        api::require_status(response.status(), StatusCode::ACCEPTED, action)?;
        let upload_id = response
          .headers()
          .get("docker-upload-uuid")
          .and_then(|value| value.to_str().ok())
          .filter(|value| !value.is_empty())
          .context("mount fallback did not create an upload UUID")?;
        let location = response
          .headers()
          .get("location")
          .and_then(|value| value.to_str().ok())
          .context("mount fallback upload location missing")?;
        let base = reqwest::Url::parse(&config.oci_url)?;
        let upload = base.join(location)?;
        let prefix = format!("/v2/{target}/blobs/uploads/");
        if upload.origin() != base.origin()
          || upload
            .path()
            .strip_prefix(&prefix)
            .is_none_or(str::is_empty)
          || !upload.path().ends_with(upload_id)
          || upload
            .query_pairs()
            .any(|(key, _)| key == "mount" || key == "from")
        {
          bail!("mount fallback did not return a new upload in the authorized target repository");
        }
      } else if !matches!(
        response.status(),
        StatusCode::FORBIDDEN | StatusCode::NOT_FOUND
      ) {
        bail!(
          "target-write mount ACL returned unexpected status {}",
          response.status()
        );
      }
    }
    Ok(())
  })();
  // Revoke explicitly even when a negative assertion fails. The caller owns the
  // whole temporary project; no other project's credential is modified.
  let revoked = api::request_json(
    client,
    config,
    Method::DELETE,
    &format!("/projects/{}/deploy-tokens/{id}", scenario.project_id),
    Some(&scenario.owner_token),
    None,
  )
  .and_then(|(status, _)| {
    api::require_status(status, StatusCode::OK, "revoke registry mount writer token")
  });
  revoked?;
  probes?;
  reporter.pass("oci-project-scoped-mount-acl", "inaccessible source fell back to 202 with a new target upload UUID; unauthorized target remained denied; project token explicitly revoked");
  Ok(())
}
