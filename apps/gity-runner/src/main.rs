mod client;
mod command;
mod executor;
mod links;
mod output;
mod resources;
mod source;
use anyhow::{Context, Result};
use clap::Parser;
use client::Client;
use std::path::PathBuf;

/// Execute project jobs claimed from a Gity server. Each runner token is scoped
/// to one project. Container execution is the default isolation boundary.
#[derive(Parser)]
struct Args {
    #[arg(
        long,
        env = "GITY_RUNNER__SERVER_URL",
        default_value = "http://localhost:8080"
    )]
    server: String,
    #[arg(long, env = "GITY_RUNNER__TOKEN", hide_env_values = true)]
    token: String,
    #[arg(long, env = "GITY_RUNNER__EXECUTOR", default_value = "container")]
    executor: executor::Executor,
    #[arg(long, env = "GITY_RUNNER__CONTAINER_BIN", default_value = "docker")]
    container_bin: PathBuf,
    #[arg(long, env = "GITY_RUNNER__IMAGE", default_value = "alpine:3.22")]
    image: String,
    #[arg(long, env = "GITY_RUNNER__WORK_ROOT", default_value = "data/runner")]
    work_root: PathBuf,
    #[arg(long, default_value_t = 3)]
    poll_seconds: u64,
    #[arg(long)]
    once: bool,
    #[command(flatten)]
    resources: resources::Resources,
}
#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let args = Args::parse();
    args.resources.validate()?;
    if args.token.trim().is_empty() {
        anyhow::bail!("GITY_RUNNER__TOKEN is required");
    }
    tokio::fs::create_dir_all(&args.work_root).await?;
    let client = Client::new(&args.server, args.token.clone())?;
    loop {
        match run_one(&client, &args).await {
            Ok(claimed) => {
                if args.once {
                    return Ok(());
                }
                if !claimed {
                    tokio::time::sleep(std::time::Duration::from_secs(
                        args.poll_seconds.clamp(1, 60),
                    ))
                    .await;
                }
            }
            Err(error) => {
                eprintln!("runner: {error:#}");
                if args.once {
                    return Err(error);
                }
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
async fn run_one(client: &Client, args: &Args) -> Result<bool> {
    let response = client
        .post(
            "/runners/jobs/claim",
            serde_json::json!({"lease_seconds":180}),
        )
        .await?;
    if response["claimed"] != true {
        return Ok(false);
    }
    let job = &response["job"];
    let id = job["id"].as_str().context("claimed job ID missing")?;
    let attempt = job["attempts"]
        .as_i64()
        .context("claimed attempt missing")?;
    let data: executor::Job =
        serde_json::from_str(job["payload"].as_str().context("claimed payload missing")?)
            .context("decode claimed job")?;
    let options = executor::Options {
        executor: args.executor,
        container_bin: args.container_bin.clone(),
        image: args.image.clone(),
        work_root: args.work_root.clone(),
        resources: args.resources.clone(),
    };
    let result = executor::execute(client, id, attempt, data, &options).await;
    match result {
        Ok(result) => {
            client
                .post(
                    &format!(
                        "/runners/jobs/{id}/{}",
                        if result.success { "complete" } else { "fail" }
                    ),
                    serde_json::json!({
                                "attempt":attempt,
                                "exit_code":result.exit_code,
                                "result":format!("exit_code={} duration_ms={}",
                                    result.exit_code,
                                    result.duration_millis),
                                "error":if result.success{
                                    ""
                                }
                                else{
                                    "script exited unsuccessfully"
                                }

                            }
                    ),
                )
                .await?;
        }
        Err(error) => {
            let _ = client
                .post(
                    &format!("/runners/jobs/{id}/fail"),
                    serde_json::json!({
                                "attempt":attempt,
                                "error":"runner execution failed",
                                "result":"execution terminated; inspect runner diagnostics"
                            }
                    ),
                )
                .await;
            return Err(error);
        }
    }
    Ok(true)
}
