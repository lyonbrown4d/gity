use super::super::{Reporter, RunConfig, api, process};
use anyhow::{Context, Result, bail};
use reqwest::{StatusCode, blocking::Client};
use std::{path::Path, process::Command};

pub(super) fn check_auth(
  client: &Client,
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  registry: &str,
  tag: &str,
  reporter: &mut Reporter,
) -> Result<()> {
  let outsider_pull = client
    .get(format!(
      "{}/v2/{}/manifests/bb-{}",
      config.oci_url, scenario.project_path, scenario.suffix
    ))
    .bearer_auth(&scenario.outsider_token)
    .send()
    .context("check outsider access to private hosted image")?;
  api::require_denied(outsider_pull.status(), "private hosted OCI outsider pull")?;

  let (read_status, read_token_response) = api::request_json(
    client,
    config,
    reqwest::Method::POST,
    &format!("/projects/{}/deploy-tokens", scenario.project_id),
    Some(&scenario.owner_token),
    Some(serde_json::json!({
      "name":"blackbox-read-registry",
      "scopes":["read_registry"]
    })),
  )?;
  api::require_status(
    read_status,
    StatusCode::OK,
    "create registry read-only project token",
  )?;
  let read_record = api::data(&read_token_response)["project_token"].clone();
  let read_token = api::data(&read_token_response)["token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("registry token was not returned"))?
    .to_owned();
  reporter.remember_secret(&read_token);
  let read_config = work.join("docker-read-only-config");
  std::fs::create_dir(&read_config)?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&read_config, std::fs::Permissions::from_mode(0o700))?;
  }
  process::run_with_stdin(
    Command::new("docker")
      .arg("--config")
      .arg(&read_config)
      .args([
        "login",
        registry,
        "--username",
        read_record["username"].as_str().unwrap_or("token"),
        "--password-stdin",
      ]),
    read_token.as_bytes(),
    "Docker login with registry-only project token",
  )?;
  process::run_output(
    Command::new("docker")
      .arg("--config")
      .arg(&read_config)
      .args(["pull", tag]),
    "pull hosted OCI image with read_registry-only project token",
  )?;
  let denied_tag = format!(
    "{registry}/{}/read-only-write-denied:bb-{}",
    scenario.project_path, scenario.suffix
  );
  let _denied_cleanup = super::ImageCleanup(denied_tag.clone());
  let tagged = Command::new("docker")
    .arg("--config")
    .arg(&read_config)
    .args(["tag", tag, &denied_tag])
    .output()
    .context("tag read-only OCI fixture")?;
  if !tagged.status.success() {
    bail!("Docker could not tag the read-only push fixture");
  }
  let denied_push = Command::new("docker")
    .arg("--config")
    .arg(&read_config)
    .args(["push", &denied_tag])
    .output()
    .context("attempt push with read-only registry token")?;
  if denied_push.status.success() {
    bail!("read_registry-only token was allowed to push an OCI image");
  }

  let (package_status, package_token_response) = api::request_json(
    client,
    config,
    reqwest::Method::POST,
    &format!("/projects/{}/deploy-tokens", scenario.project_id),
    Some(&scenario.owner_token),
    Some(serde_json::json!({
      "name":"blackbox-read-package",
      "scopes":["read_package"]
    })),
  )?;
  api::require_status(
    package_status,
    StatusCode::OK,
    "create package-only project token",
  )?;
  let package_token = api::data(&package_token_response)["token"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("package token was not returned"))?;
  reporter.remember_secret(package_token);
  let package_pull = client
    .get(format!(
      "{}/v2/{}/manifests/bb-{}",
      config.oci_url, scenario.project_path, scenario.suffix
    ))
    .bearer_auth(package_token)
    .send()
    .context("attempt OCI pull with package-only token")?;
  api::require_denied(package_pull.status(), "package-only OCI pull")?;

  let token_id = api::id(&read_record["id"], "registry token")?;
  let (revoke_status, _) = api::request_json(
    client,
    config,
    reqwest::Method::DELETE,
    &format!("/projects/{}/deploy-tokens/{token_id}", scenario.project_id),
    Some(&scenario.owner_token),
    None,
  )?;
  api::require_status(revoke_status, StatusCode::OK, "revoke registry-only token")?;
  let revoked_pull = client
    .get(format!(
      "{}/v2/{}/manifests/bb-{}",
      config.oci_url, scenario.project_path, scenario.suffix
    ))
    .bearer_auth(&read_token)
    .send()
    .context("attempt OCI pull with revoked token")?;
  api::require_denied(revoked_pull.status(), "revoked OCI token pull")?;

  let mount = client
    .post(format!(
      "{}/v2/{}/blobs/uploads/?mount=sha256:{}&from={}/source",
      config.oci_url,
      scenario.project_path,
      "a".repeat(64),
      scenario.outsider_project_path
    ))
    .bearer_auth(&scenario.owner_token)
    .send()
    .context("attempt cross-project OCI blob mount")?;
  api::require_denied(
    mount.status(),
    "cross-project OCI blob mount without source ACL",
  )?;
  let source_denied_mount = client
    .post(format!(
      "{}/v2/{}/blobs/uploads/?mount=sha256:{}&from={}/source",
      config.oci_url,
      scenario.outsider_project_path,
      "a".repeat(64),
      scenario.project_path
    ))
    .bearer_auth(&scenario.owner_token)
    .send()
    .context("attempt cross-project OCI blob mount without source read ACL")?;
  api::require_denied(
    source_denied_mount.status(),
    "cross-project OCI mount with target ACL but no source ACL",
  )?;
  reporter.pass(
    "oci-scopes-revocation-private-mount-acl",
    "private project denied outsider; registry-only token pulled but could not push; package-only and revoked tokens were denied; blob mounts were denied when either source-read or target-write ACL was missing",
  );
  Ok(())
}
