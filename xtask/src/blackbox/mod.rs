use anyhow::{Context, Result, bail};
use clap::{Args, ValueEnum};
use serde::Serialize;
use std::path::{Path, PathBuf};

mod api;
mod ci;
mod git;
mod native;
mod oci;
mod process;
mod proxy;
mod runtime;
pub(super) use runtime::grant_admin;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
  Standalone,
  Distributed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Base {
  Debian,
  Alpine,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, ValueEnum)]
enum ComposeAction {
  Up,
  #[default]
  UpDown,
}

#[derive(Clone, Args)]
pub struct BlackboxArgs {
  #[arg(long, value_enum)]
  mode: Mode,
  #[arg(long, value_enum)]
  base: Base,
  /// A unique name for this black-box Compose project.
  #[arg(long)]
  project: String,
  #[arg(long, default_value = "http://127.0.0.1:8080")]
  http_url: String,
  #[arg(long, default_value = "http://127.0.0.1:8080/api/v1")]
  api_url: String,
  #[arg(long, default_value = "http://127.0.0.1:5000")]
  oci_url: String,
  #[arg(long, default_value = "127.0.0.1")]
  ssh_host: String,
  #[arg(long, default_value_t = 2222)]
  ssh_port: u16,
  #[arg(long, value_enum, default_value_t = ComposeAction::UpDown)]
  compose_action: ComposeAction,
  #[arg(long)]
  env_file: Option<PathBuf>,
  #[arg(long)]
  evidence: Option<PathBuf>,
}

#[derive(Serialize)]
struct Evidence {
  mode: Mode,
  base: Base,
  project: String,
  checks: Vec<Check>,
  failure: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  failure_logs: Option<PathBuf>,
  #[serde(skip_serializing_if = "Option::is_none")]
  log_capture_error: Option<String>,
}

#[derive(Serialize)]
struct Check {
  name: String,
  status: &'static str,
  detail: String,
}

struct Reporter {
  evidence: Evidence,
  secrets: Vec<String>,
}

impl Reporter {
  fn new(mode: Mode, base: Base, project: String) -> Self {
    Self {
      evidence: Evidence {
        mode,
        base,
        project,
        checks: Vec::new(),
        failure: None,
        failure_logs: None,
        log_capture_error: None,
      },
      secrets: Vec::new(),
    }
  }

  fn remember_secret(&mut self, secret: &str) {
    if !secret.is_empty() {
      self.secrets.push(secret.to_owned());
    }
  }

  fn pass(&mut self, name: &str, detail: impl Into<String>) {
    let detail = self.redact(&detail.into());
    println!("PASS {name}");
    self.evidence.checks.push(Check {
      name: name.into(),
      status: "pass",
      detail,
    });
  }

  fn failure(&mut self, error: &anyhow::Error) {
    let message = self.redact(&format!("{error:#}"));
    eprintln!("FAIL {message}");
    self.evidence.failure = Some(message);
  }

  fn note(&self, message: &str) {
    println!("{message}");
  }

  fn failure_logs(&mut self, path: PathBuf) {
    self.evidence.failure_logs = Some(path);
  }

  fn log_capture_error(&mut self, error: &anyhow::Error) {
    self.evidence.log_capture_error = Some(self.redact(&format!("{error:#}")));
  }

  fn redact(&self, value: &str) -> String {
    self
      .secrets
      .iter()
      .filter(|secret| !secret.is_empty())
      .fold(value.to_owned(), |text, secret| {
        text.replace(secret, "[REDACTED]")
      })
  }

  fn write(&self, path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent).context("create evidence directory")?;
    }
    let json = serde_json::to_vec_pretty(&self.evidence)?;
    std::fs::write(path, json).context("write black-box evidence")?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o644))?;
    }
    println!("Evidence: {}", path.display());
    Ok(())
  }
}

struct RunConfig {
  http_url: String,
  api_url: String,
  oci_url: String,
  ssh_host: String,
  ssh_port: u16,
}

pub fn run(root: &Path, mut args: BlackboxArgs) -> Result<()> {
  validate(&args)?;
  runtime::ensure_unclaimed_project(&args)?;
  let evidence_path = args
    .evidence
    .clone()
    .unwrap_or_else(|| std::env::temp_dir().join(format!("gity-blackbox-{}.json", args.project)));
  let mut reporter = Reporter::new(args.mode, args.base, args.project.clone());
  let temp = tempfile::Builder::new()
    .prefix(&format!("gity-blackbox-{}-", args.project))
    .tempdir()
    .context("create isolated black-box work directory")?;
  if let Err(error) = runtime::prepare_environment(root, &mut args, temp.path()) {
    reporter.failure(&error);
    runtime::finalize(root, &args, temp, false, &evidence_path, &mut reporter)?;
    return Err(error);
  }
  if let Err(error) = runtime::remember_environment_secrets(args.env_file.as_deref(), &mut reporter)
  {
    reporter.failure(&error);
    runtime::finalize(root, &args, temp, false, &evidence_path, &mut reporter)?;
    return Err(error);
  }
  let config = RunConfig {
    http_url: args.http_url.trim_end_matches('/').into(),
    api_url: args.api_url.trim_end_matches('/').into(),
    oci_url: args.oci_url.trim_end_matches('/').into(),
    ssh_host: args.ssh_host.clone(),
    ssh_port: args.ssh_port,
  };

  let compose_attempted = true;
  let started = runtime::compose(root, &args, "up");
  let result = match started {
    Ok(()) => {
      let suites = runtime::wait_for_health(&config, &mut reporter)
        .and_then(|_| run_suites(root, &args, &config, temp.path(), &mut reporter));
      let images = runtime::capture_images(root, &args, &mut reporter);
      suites.and(images)
    }
    Err(error) => Err(error),
  };
  if let Err(error) = &result {
    reporter.failure(error);
    if let Err(log_error) =
      runtime::capture_failure_logs(root, &args, &evidence_path, &mut reporter)
    {
      reporter.log_capture_error(&log_error);
    }
  }
  let finalization = runtime::finalize(
    root,
    &args,
    temp,
    compose_attempted,
    &evidence_path,
    &mut reporter,
  );
  result?;
  finalization?;
  Ok(())
}

fn run_suites(
  root: &Path,
  args: &BlackboxArgs,
  config: &RunConfig,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let mut scenario = api::run(root, args, config, reporter)?;
  ci::run(root, args, config, &scenario, reporter)?;
  scenario.git_credential = Some(git::run(root, args, config, &scenario, work, reporter)?);
  oci::run(config, &scenario, work, reporter)?;
  native::run(args, &scenario, work, reporter)?;
  if args.mode == Mode::Distributed {
    api::distributed(
      root,
      args,
      config,
      &scenario,
      &scenario.owner_token,
      work,
      reporter,
    )?;
  }
  Ok(())
}

fn validate(args: &BlackboxArgs) -> Result<()> {
  let project = args.project.as_str();
  if project.len() < 3
    || project.len() > 48
    || !project
      .bytes()
      .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    || !project.as_bytes()[0].is_ascii_alphanumeric()
  {
    bail!("project must be a 3–48 character identifier using letters, digits, '.', '_' or '-'");
  }
  if !project.starts_with("gity-bb-") {
    bail!("black-box Compose project must start with gity-bb-");
  }
  if project.starts_with("gity-rust-blackbox-20261101") {
    bail!("the reserved prior acceptance project cannot be used");
  }
  for (name, value) in [
    ("http-url", &args.http_url),
    ("api-url", &args.api_url),
    ("oci-url", &args.oci_url),
  ] {
    let url = reqwest::Url::parse(value).with_context(|| format!("invalid {name}"))?;
    if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
      bail!("{name} must be an absolute HTTP(S) URL");
    }
  }
  Ok(())
}
