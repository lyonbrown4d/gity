use crate::{
  compose,
  lifecycle::{PlatformModule, PlatformOptions, shutdown_signal},
};
use clap::{Parser, Subcommand};
use gity_core::Settings;
use std::path::Path;

#[derive(Parser)]
#[command(version, about = "Gity Rust Git hosting platform")]
struct Cli {
  #[arg(long, global = true)]
  config: Option<std::path::PathBuf>,
  #[command(subcommand)]
  command: Option<Command>,
}
#[derive(Subcommand)]
enum Command {
  /// Serve HTTP. Run migrations separately before starting.
  Serve,
  /// Apply migrations and run the HTTP service and scheduler.
  Standalone,
  /// Initialize or upgrade the current Rust database schema.
  Migrate,
  /// Read the replacement password from stdin; revoke existing sessions.
  SetPassword { username: String },
  /// Grant administrator access to an existing local account.
  GrantAdmin { username: String },
  /// Run the background pipeline scheduler.
  Worker,
  /// Git update-hook adapter, invoked only by native receive-pack.
  Update {
    reference: String,
    old: String,
    new: String,
  },
  /// Print the OpenAPI document without starting the service.
  Openapi {
    #[arg(long)]
    output: Option<std::path::PathBuf>,
  },
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ApplicationRole {
  Server,
  Standalone,
  Api,
  Git,
  Worker,
}
pub async fn run(role: ApplicationRole) -> anyhow::Result<()> {
  let cli = Cli::parse();
  if let Some(Command::Openapi { output }) = &cli.command {
    return export_openapi(output.as_deref());
  }
  if let Some(Command::Update {
    reference,
    old,
    new,
  }) = &cli.command
  {
    let binary = std::env::var("GITY_GIT__BIN").unwrap_or_else(|_| "git".into());
    return gity_git::check_update(Path::new("."), Path::new(&binary), reference, old, new);
  }
  let mut settings = Settings::try_load(cli.config.as_deref())?;
  if role == ApplicationRole::Standalone {
    settings.components.standalone = true;
  }
  if matches!(role, ApplicationRole::Git | ApplicationRole::Worker) {
    settings.bootstrap.enabled = false;
  }
  if settings.git_hook.is_none() {
    settings.git_hook = Some(std::env::current_exe()?);
  }
  if let Some(
    command @ (Command::Migrate | Command::SetPassword { .. } | Command::GrantAdmin { .. }),
  ) = &cli.command
  {
    return administer(&settings, command).await;
  }
  if settings.search.global.enabled
    && settings.search.global.backend == "tantivy"
    && (matches!(
      role,
      ApplicationRole::Api | ApplicationRole::Git | ApplicationRole::Worker
    ) || matches!(cli.command, Some(Command::Worker)))
  {
    anyhow::bail!(
      "component deployments require search.global.backend=opensearch (or disable global search); Tantivy requires one standalone process and its persistent index volume"
    );
  }
  let telemetry = crate::observability::init(&settings.observability)?;
  let (_injector, state) = compose(settings).await?;
  match cli.command.unwrap_or(match role {
    ApplicationRole::Standalone => Command::Standalone,
    ApplicationRole::Worker => Command::Worker,
    _ => Command::Serve,
  }) {
    Command::Openapi { .. }
    | Command::Migrate
    | Command::SetPassword { .. }
    | Command::GrantAdmin { .. } => {
      unreachable!("handled before application composition")
    }
    Command::Worker => {
      run_components(
        state,
        PlatformOptions {
          http: false,
          ssh: false,
          ..PlatformOptions::default()
        },
      )
      .await?;
    }
    command => {
      crate::standalone::run(
        state,
        PlatformOptions {
          ssh: role != ApplicationRole::Api,
          scheduler: role != ApplicationRole::Api,
          migrate: matches!(command, Command::Standalone),
          ..PlatformOptions::default()
        },
        role == ApplicationRole::Standalone,
      )
      .await?;
    }
  }
  telemetry.shutdown();
  Ok(())
}

/// `openapi` needs neither configuration nor a database connection.
fn export_openapi(output: Option<&std::path::Path>) -> anyhow::Result<()> {
  if let Some(path) = output {
    return crate::openapi::write_document(path);
  }
  let json = serde_json::to_string_pretty(&crate::openapi::document())?;
  use std::io::Write as _;
  let mut stdout = std::io::stdout().lock();
  stdout.write_all(json.as_bytes())?;
  stdout.write_all(b"\n")?;
  Ok(())
}
async fn run_components(state: crate::AppState, options: PlatformOptions) -> anyhow::Result<()> {
  let module = PlatformModule::new(state, options);
  let mut failure = module.subscribe();
  let mut app = fluxdi::Application::new(module);
  app.bootstrap().await?;
  let failed = tokio::select! {
      _ = shutdown_signal() => None,
      result = failure.changed() => {
          result?;
          failure.borrow().clone()
      }
  };
  let stopped = app.shutdown().await;
  stopped?;
  if let Some(error) = failed {
    anyhow::bail!("application component failed: {error}");
  }
  Ok(())
}

async fn administer(settings: &Settings, command: &Command) -> anyhow::Result<()> {
  let db = gity_core::Db::connect(&settings.database_url).await?;
  if matches!(command, Command::Migrate) {
    db.migrate().await?;
    return Ok(());
  }
  let module = gity_identity::IdentityModule::new(fluxdi::Shared::new(db.connection));
  let identity = module.repository()?;
  let username = match command {
    Command::SetPassword { username } | Command::GrantAdmin { username } => username,
    _ => unreachable!(),
  };
  let user = identity
    .user_by_username(username)
    .await?
    .ok_or_else(|| anyhow::anyhow!("user not found"))?;
  match command {
    Command::GrantAdmin { .. } => {
      identity
        .update_user_profile(
          user.id,
          gity_identity::UserProfileChanges {
            is_super_admin: Some(true),
            ..Default::default()
          },
        )
        .await?;
    }
    Command::SetPassword { .. } => {
      let mut password = String::new();
      std::io::stdin().read_line(&mut password)?;
      let password = password.trim_end_matches(['\r', '\n']);
      anyhow::ensure!(
        (8..=1024).contains(&password.len()),
        "password must contain 8–1024 bytes"
      );
      let hash = crate::auth::hash_password(password.to_owned()).await?;
      anyhow::ensure!(
        identity
          .set_password_and_revoke_sessions(user.id, hash)
          .await?,
        "user not found"
      );
    }
    _ => unreachable!(),
  }
  Ok(())
}
