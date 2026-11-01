use crate::{Args, client::Client};
use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, parser::ValueSource};
use serde_json::{Value, json};

#[cfg(test)]
#[path = "settings_tests.rs"]
mod tests;

pub fn load() -> Result<Args> {
  let matches = Args::command().get_matches();
  let cli = Args::from_arg_matches(&matches)?;
  let defaults = defaults();
  let mut builder = config::Config::builder().add_source(config::File::from_str(
    &defaults.to_string(),
    config::FileFormat::Json,
  ));
  if let Some(path) = &cli.config {
    builder = builder.add_source(config::File::from(path.as_path()).required(true));
  }
  builder = builder.add_source(
    config::Environment::with_prefix("GITY_RUNNER")
      .prefix_separator("__")
      .separator("__")
      .try_parsing(true),
  );
  let serialized = serde_json::to_value(&cli)?;
  let mut overrides = serde_json::Map::new();
  for (argument, key) in [
    ("server", "server_url"),
    ("token", "token"),
    ("token_directory", "token_directory"),
    ("executor", "executor"),
    ("container_bin", "container_bin"),
    ("image", "image"),
    ("work_root", "work_root"),
    ("poll_seconds", "poll_seconds"),
    ("once", "once"),
    ("memory_mb", "memory_mb"),
    ("cpus", "cpus"),
    ("pids_limit", "pids_limit"),
  ] {
    if matches.value_source(argument) == Some(ValueSource::CommandLine) {
      overrides.insert(key.into(), serialized[key].clone());
    }
  }
  builder
    .add_source(config::File::from_str(
      &serde_json::Value::Object(overrides).to_string(),
      config::FileFormat::Json,
    ))
    .build()
    .and_then(|value| value.try_deserialize())
    .map_err(|_| {
      anyhow::anyhow!("invalid Runner configuration; check field types and required settings")
    })
}

fn defaults() -> Value {
  let mut observability = gity_observability::ObservabilityConfig::for_service("gity-runner");
  observability.log_level = "gity=info,tower_http=info".into();
  observability.metrics_enabled = false;
  observability.metrics_address = "127.0.0.1:9091".into();
  json!({"server_url":"http://localhost:8080","token":"","executor":"container","container_bin":"docker",
    "image":"alpine:3.22","work_root":"data/runner","poll_seconds":3,"once":false,"memory_mb":1024,"cpus":2,"pids_limit":256,
    "observability":observability})
}

pub async fn clients(args: &Args) -> Result<Vec<Client>> {
  let mut tokens = Vec::new();
  if !args.token.is_empty() {
    tokens.push(args.token.clone());
  }
  if let Some(directory) = &args.token_directory {
    let mut entries = tokio::fs::read_dir(directory)
      .await
      .context("open Runner credential directory")?;
    while let Some(entry) = entries.next_entry().await? {
      if entry
        .path()
        .extension()
        .is_none_or(|value| value != "token")
      {
        continue;
      }
      let metadata = tokio::fs::symlink_metadata(entry.path()).await?;
      anyhow::ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink() && metadata.len() <= 4096,
        "invalid Runner credential file"
      );
      let token = tokio::fs::read_to_string(entry.path()).await?;
      if !token.trim().is_empty() {
        tokens.push(token.trim().to_owned());
      }
      anyhow::ensure!(tokens.len() <= 1024, "Runner credential limit exceeded");
    }
  }
  tokens
    .into_iter()
    .map(|token| Client::new(&args.server, token))
    .collect()
}
