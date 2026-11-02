//! `gity config` — read and write the glab-style YAML settings.
use crate::client::Options;
use crate::config::{self, Config};
use crate::format;
use anyhow::{Result, bail};
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SettingsCommand {
  /// Print one key: host, git_protocol or editor
  Get { key: String },
  /// Set one key
  Set { key: String, value: String },
  /// Show the whole configuration, including stored hosts
  List,
  /// Manage per-host tokens
  Host {
    #[command(subcommand)]
    command: HostCommand,
  },
}

#[derive(Subcommand)]
pub enum HostCommand {
  /// Store a token for a host and make it current
  Add {
    host: String,
    #[arg(long, env = "GITY_TOKEN", hide_env_values = true)]
    token: String,
  },
  /// Forget a host and its token
  Remove { host: String },
  /// List configured hosts
  List,
}

pub fn run(command: SettingsCommand, options: &Options) -> Result<()> {
  let mut config = config::load(&options.config_file);
  match command {
    SettingsCommand::Get { key } => println!("{}", config::get_key(&config, &key)?),
    SettingsCommand::Set { key, value } => {
      config::set_key(&mut config, &key, &value)?;
      config::save(&options.config_file, &config)?;
      println!("{key} = {value}");
    }
    SettingsCommand::List => print(&config, options),
    SettingsCommand::Host { command } => host(command, &mut config, options)?,
  }
  Ok(())
}

fn host(command: HostCommand, config: &mut Config, options: &Options) -> Result<()> {
  match command {
    HostCommand::Add { host, token } => {
      let host = host.trim_end_matches('/').to_owned();
      config.hosts.insert(
        host.clone(),
        config::HostConfig {
          token: Some(token),
          user: None,
        },
      );
      config.host = Some(host.clone());
      config::save(&options.config_file, config)?;
      println!("configured {host}");
    }
    HostCommand::Remove { host } => {
      let host = host.trim_end_matches('/').to_owned();
      if !config.hosts.remove(&host).is_some() {
        bail!("host {host} is not configured");
      }
      if config.host.as_deref() == Some(host.as_str()) {
        config.host = config.hosts.keys().next().cloned();
      }
      config::save(&options.config_file, config)?;
      println!("removed {host}");
    }
    HostCommand::List => {
      let rows = config
        .hosts
        .iter()
        .map(|(host, entry)| {
          vec![
            host.clone(),
            entry.user.clone().unwrap_or_default(),
            if entry.token.is_some() { "token" } else { "-" }.to_owned(),
            if config.host.as_deref() == Some(host.as_str()) {
              "*".to_owned()
            } else {
              String::new()
            },
          ]
        })
        .collect::<Vec<_>>();
      format::table(&["HOST", "USER", "AUTH", "CURRENT"], &rows);
    }
  }
  Ok(())
}

fn print(config: &Config, options: &Options) {
  println!("file:         {}", options.config_file.display());
  println!("host:         {}", config.host.clone().unwrap_or_default());
  println!("git_protocol: {}", config.git_protocol);
  println!(
    "editor:       {}",
    config.editor.clone().unwrap_or_default()
  );
  if !config.hosts.is_empty() {
    println!(
      "hosts:        {}",
      config.hosts.keys().cloned().collect::<Vec<_>>().join(", ")
    );
  }
}
