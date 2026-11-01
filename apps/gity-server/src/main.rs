use clap::{Parser, Subcommand};
use gity_core::Settings;
use gity_server::{compose, router};
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
    /// Run the background pipeline scheduler.
    Worker,
    /// Git update-hook adapter, invoked only by native receive-pack.
    Update {
        reference: String,
        old: String,
        new: String,
    },
}
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
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
    if settings.git_hook.is_none() {
        settings.git_hook = Some(std::env::current_exe()?);
    }
    if matches!(
        &cli.command,
        Some(Command::Migrate | Command::SetPassword { .. })
    ) {
        // Administrative commands can run while HTTP owns the metrics port.
        settings.observability.metrics_enabled = false;
    }
    let address = settings.address.clone();
    let telemetry = gity_server::observability::init(&settings.observability)?;
    let (_injector, state) = compose(settings).await?;
    match cli.command.unwrap_or(Command::Serve) {
        Command::Migrate => {
            state.db.migrate().await?;
            tracing::info!("database migrations complete");
        }
        Command::SetPassword { username } => {
            let user = state
                .db
                .one(
                    "SELECT id FROM users WHERE username=?",
                    vec![username.into()],
                )
                .await?
                .ok_or_else(|| anyhow::anyhow!("user not found"))?;
            let mut password = String::new();
            std::io::stdin().read_line(&mut password)?;
            gity_server::users::set_password(
                &state,
                user["id"]
                    .as_i64()
                    .ok_or_else(|| anyhow::anyhow!("invalid user id"))?,
                password.trim_end_matches(['\r', '\n']).to_owned(),
            )
            .await?;
            tracing::info!("password changed; sessions revoked");
        }
        Command::Worker => {
            tokio::select! {
            result=async{tokio::try_join!(
                scheduler(state.clone()),
                gity_server::maintenance::run(state)
            )}=>{result?;},
            _=shutdown()=>{},
            }
        }
        command => {
            if matches!(command, Command::Standalone) {
                state.db.migrate().await?;
            }
            let ssh_shutdown = tokio_util::sync::CancellationToken::new();
            let ssh = if state.settings.ssh.enabled {
                let listener = tokio::net::TcpListener::bind(&state.settings.ssh.address).await?;
                let config = gity_server::ssh::configuration(&state)?;
                let state = state.clone();
                let cancel = ssh_shutdown.clone();
                Some(tokio::spawn(async move {
                    gity_server::ssh::run(state, listener, cancel, config).await
                }))
            } else {
                None
            };
            let background = tokio::spawn(scheduler(state.clone()));
            let maintenance = tokio::spawn(gity_server::maintenance::run(state.clone()));
            let listener = tokio::net::TcpListener::bind(&address).await?;
            tracing::info!(address=%listener.local_addr()?,"Gity HTTP server started");
            let http = axum::serve(listener, router(state)).with_graceful_shutdown(shutdown());
            if let Some(mut ssh) = ssh {
                tokio::select! {
                    result=http=>{ssh_shutdown.cancel(); let stopped=ssh.await; result?; stopped??;},
                    result=&mut ssh=>{result??; anyhow::bail!("SSH listener stopped unexpectedly");}
                }
            } else {
                http.await?;
            }
            background.abort();
            maintenance.abort();
            let _ = background.await;
            let _ = maintenance.await;
        }
    }
    telemetry.shutdown();
    Ok(())
}
async fn scheduler(state: gity_server::AppState) -> anyhow::Result<()> {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
    loop {
        interval.tick().await;
        match gity_server::ci::worker_tick(&state).await {
            Err(error) => {
                metrics::counter!("gity_scheduler_ticks_total","result"=>"failure").increment(1);
                tracing::error!(error=%error,"pipeline scheduler tick failed");
            }
            Ok(_) => {
                metrics::counter!("gity_scheduler_ticks_total","result"=>"success").increment(1)
            }
        }
    }
}
async fn shutdown() {
    #[cfg(unix)]
    {
        let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler");
        tokio::select! {_=tokio::signal::ctrl_c()=>{},_=term.recv()=>{}}
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
