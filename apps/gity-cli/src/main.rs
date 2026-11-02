mod client;
mod commands;
mod config;
mod format;

use clap::{Parser, Subcommand};
use commands::{api, auth, issue, mr, pipeline, project, token};
use std::path::PathBuf;

/// Command line client for a Gity server, modelled on `glab`.
#[derive(Parser)]
#[command(name = "gity", version, about, long_about = None)]
struct Cli {
  /// Gity server base URL, for example http://127.0.0.1:8080
  #[arg(long, global = true, env = "GITY_HOST")]
  host: Option<String>,
  /// Personal access token; overrides the profile stored in the config file
  #[arg(long, global = true, env = "GITY_TOKEN", hide_env_values = true)]
  token: Option<String>,
  /// Print raw JSON instead of a table
  #[arg(long, global = true)]
  json: bool,
  /// Config file path; defaults to ~/.config/gity/config.toml
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
  /// Manage personal access tokens
  Token {
    #[command(subcommand)]
    command: token::TokenCommand,
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
  /// Inspect pipelines
  Pipeline {
    #[command(subcommand)]
    command: pipeline::PipelineCommand,
  },
  /// Call the REST API directly, like `gh api`
  Api(api::ApiArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
  let cli = Cli::parse();
  let file = config::resolve_path(cli.config.clone());
  let options = client::Options {
    host: cli.host,
    token: cli.token,
    json: cli.json,
    config_file: file,
  };
  match cli.command {
    Command::Auth { command } => auth::run(command, &options).await,
    Command::Token { command } => token::run(command, &options).await,
    Command::Project { command } => project::run(command, &options).await,
    Command::Issue { command } => issue::run(command, &options).await,
    Command::Mr { command } => mr::run(command, &options).await,
    Command::Pipeline { command } => pipeline::run(command, &options).await,
    Command::Api(args) => api::run(args, &options).await,
  }
}
