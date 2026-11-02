use super::{Reporter, RunConfig, api, process};
use anyhow::{Context, Result, bail};
use reqwest::{StatusCode, blocking::Client};
use std::{path::Path, process::Command};

mod auth;

pub fn run(
  config: &RunConfig,
  scenario: &api::Scenario,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let registry = registry_authority(&config.oci_url)?;
  let docker_config = work.join("docker-config");
  std::fs::create_dir(&docker_config).context("create private Docker client config")?;
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&docker_config, std::fs::Permissions::from_mode(0o700))?;
  }

  let login = process::run_with_stdin(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args([
        "login",
        &registry,
        "--username",
        &scenario.owner_username,
        "--password-stdin",
      ]),
    scenario.owner_token.as_bytes(),
    "Docker OCI login",
  );
  login?;
  reporter.pass(
    "oci-docker-login",
    "Docker authenticated with an opaque Gity access token",
  );

  let client = Client::builder()
    .timeout(std::time::Duration::from_secs(30))
    .build()?;
  let catalog = client
    .get(format!("{}/v2/_catalog", config.oci_url))
    .bearer_auth(&scenario.owner_token)
    .send()
    .context("request disabled OCI catalog")?;
  if catalog.status() != StatusCode::NOT_FOUND {
    bail!(
      "OCI catalog should be disabled, received HTTP {}",
      catalog.status()
    );
  }
  let invalid = client
    .get(format!(
      "{}/v2/{}/blackbox-smoke/manifests/latest",
      config.oci_url, scenario.project_path
    ))
    .bearer_auth("opaque-blackbox-invalid-token")
    .send()
    .context("request OCI manifest with invalid opaque token")?;
  if invalid.status() != StatusCode::UNAUTHORIZED {
    bail!(
      "invalid OCI opaque token should be rejected, received HTTP {}",
      invalid.status()
    );
  }
  reporter.pass(
    "oci-catalog-and-opaque-auth-denials",
    "catalog disabled and invalid token rejected",
  );

  let build = work.join("oci-fixture");
  std::fs::create_dir(&build).context("create OCI fixture build context")?;
  std::fs::write(
    build.join("Dockerfile"),
    format!(
      "FROM scratch\nLABEL org.gity.blackbox={}\nCOPY payload.txt /blackbox.txt\n",
      scenario.suffix
    ),
  )?;
  std::fs::write(
    build.join("payload.txt"),
    format!("Gity blackbox OCI fixture {}\n", scenario.suffix),
  )?;
  let tag = format!(
    "{registry}/{}/blackbox-smoke:bb-{}",
    scenario.project_path, scenario.suffix
  );
  let _cleanup = ImageCleanup(tag.clone());
  command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["build", "--quiet", "--tag", &tag])
      .arg(&build),
    "build task-owned OCI fixture image",
    reporter,
  )?;
  let pushed = Command::new("docker")
    .arg("--config")
    .arg(&docker_config)
    .args(["push", &tag])
    .output()
    .context("push task-owned OCI fixture image")?;
  if !pushed.status.success() {
    bail!(
      "OCI fixture push failed: {}",
      reporter.redact(&String::from_utf8_lossy(&pushed.stderr))
    );
  }
  reporter.pass(
    "oci-real-docker-push",
    "Docker pushed manifest and blobs to the Gity registry",
  );
  auth::check_auth(&client, config, scenario, work, &registry, &tag, reporter)?;

  command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["image", "rm", "--force", &tag]),
    "remove task-owned local OCI image before cold pull",
    reporter,
  )?;
  command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["pull", &tag]),
    "pull OCI image from Gity registry",
    reporter,
  )?;
  let image_id = command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["image", "inspect", "--format", "{{.Id}}", &tag]),
    "inspect pulled OCI fixture image",
    reporter,
  )?;
  let image_id = String::from_utf8(image_id.stdout)
    .context("Docker image ID is not UTF-8")?
    .trim()
    .to_owned();
  if !image_id.starts_with("sha256:") {
    bail!("Docker returned an invalid image ID for the pulled fixture");
  }
  reporter.pass(
    "oci-cold-docker-pull",
    format!("Docker pulled fixture image {image_id}"),
  );

  let (policy_status, _) = api::request_json(
    &client,
    config,
    reqwest::Method::PUT,
    &format!("/projects/{}/registry/settings", scenario.project_id),
    Some(&scenario.owner_token),
    Some(serde_json::json!({
      "mode":"mirror",
      "mirror_alias":"dockerhub",
      "mirror_repository_prefix":"library/alpine"
    })),
  )?;
  api::require_status(
    policy_status,
    StatusCode::OK,
    "configure Docker Hub read-through mirror",
  )?;
  let mirror_tag = format!("{registry}/{}:3.22", scenario.project_path);
  let _mirror_cleanup = ImageCleanup(mirror_tag.clone());
  command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["pull", &mirror_tag]),
    "cold pull through Gity Docker Hub mirror",
    reporter,
  )?;
  let mirror_image_id = inspect_image(&mirror_tag)?;
  reporter.pass(
    "oci-mirror-cold-pull",
    "Gity resolved and cached the configured Docker Hub image",
  );
  command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["image", "rm", "--force", &mirror_tag]),
    "remove local mirror image before repeat pull",
    reporter,
  )?;
  command_output(
    Command::new("docker")
      .arg("--config")
      .arg(&docker_config)
      .args(["pull", &mirror_tag]),
    "repeat pull through Gity Docker Hub mirror",
    reporter,
  )?;
  anyhow::ensure!(
    mirror_image_id == inspect_image(&mirror_tag)?,
    "repeat mirror pull returned a different image ID"
  );
  reporter.pass(
    "oci-mirror-repeat-pull",
    format!("repeat mirror pull returned identical image {mirror_image_id}"),
  );

  let outsider_mirror = client
    .get(format!(
      "{}/v2/{}/manifests/3.22",
      config.oci_url, scenario.project_path
    ))
    .bearer_auth(&scenario.outsider_token)
    .send()
    .context("check outsider access to private mirror project")?;
  api::require_denied(outsider_mirror.status(), "private mirror outsider pull")?;
  let denied_push = Command::new("docker")
    .arg("--config")
    .arg(&docker_config)
    .args([
      "push",
      &format!("{registry}/{}:3.22", scenario.project_path),
    ])
    .output()
    .context("attempt push to read-only mirror")?;
  if denied_push.status.success() {
    bail!("Gity accepted a push to a project configured as a read-only mirror");
  }
  reporter.pass(
    "oci-mirror-private-and-read-only",
    "outsider pull and authenticated push were denied for the mirror project",
  );
  Ok(())
}

fn registry_authority(url: &str) -> Result<String> {
  let parsed = reqwest::Url::parse(url).context("parse OCI endpoint URL")?;
  let host = parsed
    .host_str()
    .ok_or_else(|| anyhow::anyhow!("OCI endpoint has no host"))?;
  Ok(match parsed.port() {
    Some(port) => format!("{host}:{port}"),
    None => host.to_owned(),
  })
}

struct ImageCleanup(String);

impl Drop for ImageCleanup {
  fn drop(&mut self) {
    let _ = Command::new("docker")
      .args(["image", "rm", "--force", &self.0])
      .output();
  }
}

fn inspect_image(tag: &str) -> Result<String> {
  let output = process::run_output(
    Command::new("docker").args(["image", "inspect", "--format", "{{.Id}}", tag]),
    "inspect mirror image ID",
  )?;
  let id = String::from_utf8(output.stdout)?.trim().to_owned();
  anyhow::ensure!(id.starts_with("sha256:"), "invalid mirror image ID");
  Ok(id)
}

fn command_output(
  command: &mut Command,
  name: &str,
  reporter: &Reporter,
) -> Result<std::process::Output> {
  let output = command
    .stdin(std::process::Stdio::null())
    .output()
    .with_context(|| format!("start {name}"))?;
  if !output.status.success() {
    let detail = String::from_utf8_lossy(&output.stderr)
      .chars()
      .take(8192)
      .collect::<String>();
    bail!("{name} failed: {}", reporter.redact(&detail));
  }
  Ok(output)
}
