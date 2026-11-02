mod client;
mod commands;
mod config;
mod format;

use clap::{CommandFactory, Parser, Subcommand};
use commands::{api, auth, completion, issue, mr, pipeline, project, repo, settings, token};
use std::path::PathBuf;

/// Command line client for a Gity server, modelled on `glab`.
#[derive(Parser)]
#[command(name = "gity", version, about, long_about = None)]
pub struct Cli {
  /// Gity server base URL, for example http://127.0.0.1:8080
  #[arg(long, global = true, env = "GITY_HOST")]
  host: Option<String>,
  /// Personal access token; overrides the host entry in the config file
  #[arg(long, global = true, env = "GITY_TOKEN", hide_env_values = true)]
  token: Option<String>,
  /// Print raw JSON instead of a table
  #[arg(long, global = true)]
  json: bool,
  /// Config file path; defaults to ~/.config/gity-cli/config.yaml
  #[arg(long, global = true, env = "GITY_CONFIG")]
  config: Option<PathBuf>,
  #[command(subcommand)]
  command: Command,
}

#[derive(Subcommand)]
enum Command {
  /// Sign in, inspect or clear the stored credentials
  Auth {
    #[command(subcommand)]
    command: auth::AuthCommand,
  },
  /// Read and change CLI settings, including per-host tokens
  Config {
    #[command(subcommand)]
    command: settings::SettingsCommand,
  },
  /// Manage personal access tokens
  Token {
    #[command(subcommand)]
    command: token::TokenCommand,
  },
  /// Clone a project
  Repo {
    #[command(subcommand)]
    command: repo::RepoCommand,
  },
  /// Browse projects
  Project {
    #[command(subcommand)]
    command: project::ProjectCommand,
  },
  /// Work with issues
  Issue {
    #[command(subcommand)]
    command: issue::IssueCommand,
  },
  /// Work with merge requests
  Mr {
    #[command(subcommand)]
    command: mr::MrCommand,
  },
  /// Inspect and run pipelines
  Pipeline {
    #[command(subcommand)]
    command: pipeline::PipelineCommand,
  },
  /// Print a shell completion script
  Completion {
    #[arg(value_enum)]
    shell: clap_complete::Shell,
  },
  /// Call the REST API directly, like `gh api`
  Api(api::ApiArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let options = client::Options {
    host: cli.host,
    token: cli.token,
    json: cli.json,
    config_file: config::resolve_path(cli.config),
  };
  match cli.command {
    Command::Auth { command } => auth::run(command, &options).await,
    Command::Config { command } => settings::run(command, &options),
    Command::Token { command } => token::run(command, &options).await,
    Command::Repo { command } => repo::run(command, &options).await,
    Command::Project { command } => project::run(command, &options).await,
    Command::Issue { command } => issue::run(command, &options).await,
    Command::Mr { command } => mr::run(command, &options).await,
    Command::Pipeline { command } => pipeline::run(command, &options).await,
    Command::Completion { shell } => completion::run(shell),
    Command::Api(args) => api::run(args, &options).await,
  }
}

/// Exposed for the completion generator.
pub fn command() -> clap::Command {
  Cli::command()
}
