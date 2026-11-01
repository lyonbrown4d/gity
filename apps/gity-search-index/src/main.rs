mod composition;
mod refresh;
use anyhow::Result;
use clap::Parser;
use gity_core::Settings;
#[derive(Parser)]
#[command(about = "Build commit-pinned Gity repository search indexes")]
struct Args {
  #[arg(long)]
  config: Option<std::path::PathBuf>,
  /// Rebuild selected projects; omission rebuilds all non-deleted projects.
  #[arg(long = "project-id")]
  project_ids: Vec<i64>,
  /// Keep refreshing according to search.refresh_interval_seconds.
  #[arg(long)]
  watch: bool,
}
#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();
  let settings = Settings::try_load(args.config.as_deref())?;
  let (_module, runtime) = composition::compose(settings).await?;
  loop {
    let result = refresh::run(&runtime, &args.project_ids).await;
    if !args.watch {
      return result;
    }
    if let Err(error) = result {
      eprintln!("search refresh failed: {error:#}");
    }
    tokio::select! {
        _=tokio::time::sleep(std::time::Duration::from_secs(runtime.settings.search.refresh_interval_seconds.max(1)))=>{},
        _=tokio::signal::ctrl_c()=>return Ok(()),
    }
  }
}
