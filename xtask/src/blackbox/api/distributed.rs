use super::{BlackboxArgs, Reporter, RunConfig, Scenario, data, request_json, require_status};
use anyhow::{Result, bail};
use reqwest::{Method, StatusCode, blocking::Client};
use serde_json::json;
use std::path::Path;

pub fn distributed(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  scenario: &Scenario,
  admin_token: &str,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(60))
    .build()?;
  check_git_only_image(root, args, reporter)?;
  check_registry_outage(root, args, config, scenario, &client, reporter)?;
  let path = format!("/admin/git/repositories/{}/placement", scenario.project_id);
  let (status, body) = request_json(&client, config, Method::GET, &path, Some(admin_token), None)?;
  require_status(status, StatusCode::OK, "Git placement query")?;
  let placement = data(&body);
  let primary = placement["primary_node"]
    .as_str()
    .ok_or_else(|| anyhow::anyhow!("placement omitted the primary node"))?;
  let epoch = placement["epoch"]
    .as_i64()
    .ok_or_else(|| anyhow::anyhow!("placement omitted the fencing epoch"))?;
  let target = placement["replicas"]
    .as_array()
    .ok_or_else(|| anyhow::anyhow!("placement omitted replica state"))?
    .iter()
    .find(|replica| {
      replica["node"].as_str().is_some_and(|node| node != primary)
        && replica["state"].as_str() == Some("ready")
        && replica["generation"].as_i64().unwrap_or(-1)
          >= placement["generation"].as_i64().unwrap_or(i64::MAX)
    })
    .and_then(|replica| replica["node"].as_str())
    .ok_or_else(|| {
      anyhow::anyhow!("placement has no ready replica at the current generation for promotion")
    })?
    .to_owned();
  reporter.pass(
    "distributed-placement-replicas-ready",
    format!(
      "placement reports ready replicas at generation {}",
      placement["generation"]
    ),
  );

  let primary_service = match primary {
    "git-1" | "git-node" => "git-node",
    "git-2" | "git-node-2" => "git-node-2",
    other => bail!("blackbox cannot map placement primary node {other} to a Compose service"),
  };
  super::super::runtime::control_service(root, args, "stop", primary_service)?;
  let expected = format!("blackbox API commit {}\n", scenario.suffix);
  let unavailable_read = request_json(
    &client,
    config,
    Method::GET,
    &format!(
      "/projects/{}/repository/blob?ref_name=main&path=blackbox%2Fapi.txt",
      scenario.project_id
    ),
    Some(&scenario.owner_token),
    None,
  );
  let restart = super::super::runtime::control_service(root, args, "start", primary_service);
  restart?;
  let (unavailable_status, unavailable_body) = unavailable_read?;
  if unavailable_status.is_success() {
    if data(&unavailable_body)["content"].as_str() != Some(expected.as_str()) {
      bail!("Git read during primary outage did not match the committed bytes");
    }
    reporter.pass(
      "distributed-primary-unavailable-read",
      "repository read remained available with exact fixture bytes; no failover durability claim",
    );
  } else if unavailable_status.is_server_error() {
    reporter.pass(
      "distributed-primary-unavailable-fail-closed",
      format!("Git read returned HTTP {unavailable_status} while the primary was stopped"),
    );
  } else {
    bail!("Git read during primary outage returned unexpected HTTP {unavailable_status}");
  }
  let recovery_deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
  let recovered_body = loop {
    if let Ok((status, body)) = request_json(
      &client,
      config,
      Method::GET,
      &format!(
        "/projects/{}/repository/blob?ref_name=main&path=blackbox%2Fapi.txt",
        scenario.project_id
      ),
      Some(&scenario.owner_token),
      None,
    ) && status == StatusCode::OK
    {
      break body;
    }
    if std::time::Instant::now() >= recovery_deadline {
      bail!("repository read did not recover within 60 seconds after primary restart");
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
  };
  if data(&recovered_body)["content"].as_str() != Some(expected.as_str()) {
    bail!("repository recovery read did not match the committed bytes");
  }
  reporter.pass(
    "distributed-primary-restart-read",
    "read returned the existing fixture bytes after the primary restarted",
  );

  let (status, promoted) = request_json(
    &client,
    config,
    Method::POST,
    &format!("/admin/git/repositories/{}/promote", scenario.project_id),
    Some(admin_token),
    Some(json!({"node":target})),
  )?;
  require_status(status, StatusCode::OK, "controlled Git promotion")?;
  let promoted = data(&promoted);
  let promoted_epoch = promoted["epoch"]
    .as_i64()
    .ok_or_else(|| anyhow::anyhow!("promotion response omitted fencing epoch"))?;
  if promoted["primary_node"].as_str() != Some(target.as_str()) || promoted_epoch <= epoch {
    bail!("controlled promotion did not change primary and advance fencing epoch");
  }
  reporter.pass(
    "distributed-controlled-promotion",
    format!("primary changed to a ready replica and epoch advanced to {promoted_epoch}"),
  );

  let credential = scenario
    .git_credential
    .as_ref()
    .ok_or_else(|| anyhow::anyhow!("SSH credential missing for post-promotion checks"))?;
  super::super::git::after_promotion(config, scenario, credential, work, reporter)?;
  Ok(())
}

fn check_registry_outage(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  scenario: &Scenario,
  client: &Client,
  reporter: &mut Reporter,
) -> Result<()> {
  let registry_url = format!(
    "{}/projects/{}/packages/generic/blackbox/1.0.{:x}/payload.bin",
    config.api_url,
    scenario.project_id,
    scenario.suffix.as_bytes()[0]
  );
  super::super::runtime::control_service(root, args, "stop", "registry")?;
  let registry_read = client
    .get(&registry_url)
    .bearer_auth(&scenario.owner_token)
    .send();
  let health = client.get(format!("{}/api/health", config.http_url)).send();
  let restart = super::super::runtime::control_service(root, args, "start", "registry");
  restart?;
  let registry_status = registry_read?.status();
  require_status(
    health?.status(),
    StatusCode::OK,
    "API health while Registry is unavailable",
  )?;
  if !registry_status.is_server_error() {
    bail!("package download during Registry outage expected 5xx, got {registry_status}");
  }

  let expected = fixture_bytes(&scenario.suffix);
  let deadline = std::time::Instant::now() + std::time::Duration::from_secs(60);
  loop {
    if let Ok(response) = client
      .get(&registry_url)
      .bearer_auth(&scenario.owner_token)
      .send()
      && response.status() == StatusCode::OK
    {
      let bytes = response.bytes()?;
      if bytes.as_ref() != expected.as_slice() {
        bail!("Generic package bytes changed after Registry recovery");
      }
      reporter.pass(
          "distributed-registry-outage-recovery",
          format!(
            "Registry route returned {registry_status} during outage; API health stayed ready; exact {}-byte package recovered",
            expected.len()
          ),
        );
      return Ok(());
    }
    if std::time::Instant::now() >= deadline {
      bail!("Generic package download did not recover within 60 seconds");
    }
    std::thread::sleep(std::time::Duration::from_secs(2));
  }
}

fn fixture_bytes(seed: &str) -> Vec<u8> {
  use sha2::{Digest, Sha256};

  let mut bytes = Vec::with_capacity(12 * 1024 * 1024);
  let mut index = 0_u64;
  while bytes.len() < 12 * 1024 * 1024 {
    bytes.extend_from_slice(&Sha256::digest(format!("{seed}:{index}")));
    index += 1;
  }
  bytes.truncate(12 * 1024 * 1024);
  bytes
}

fn check_git_only_image(root: &Path, args: &BlackboxArgs, reporter: &mut Reporter) -> Result<()> {
  let mut command = std::process::Command::new("docker");
  super::super::runtime::configure_compose(&mut command, root, args)?;
  command.args([
    "exec",
    "--no-TTY",
    "git-server",
    "curl",
    "--fail",
    "--silent",
    "http://127.0.0.1:8080/openapi.json",
  ]);
  let output = super::super::process::run_output(&mut command, "inspect Git-only image API")?;
  let document: serde_json::Value = serde_json::from_slice(&output.stdout)?;
  let paths = document["paths"]
    .as_object()
    .ok_or_else(|| anyhow::anyhow!("Git-only image did not return OpenAPI paths"))?;
  anyhow::ensure!(
    !paths.is_empty(),
    "Git-only image returned empty API documentation"
  );
  anyhow::ensure!(
    !paths
      .keys()
      .any(|path| path.contains("/packages") || path.contains("/projects/{id}/registry")),
    "Git-only image unexpectedly includes package or OCI APIs"
  );
  reporter.pass(
    "git-only-image-excludes-registries",
    "Git collaboration image excludes package and OCI API routes",
  );
  Ok(())
}
