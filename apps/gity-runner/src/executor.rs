use crate::{client::Client, source};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    process::Stdio,
    time::{Duration, Instant},
};
use tokio::{process::Command, sync::mpsc};

#[derive(Clone, Copy, clap::ValueEnum)]
pub enum Executor {
    Shell,
    Container,
}
pub struct Options {
    pub executor: Executor,
    pub container_bin: PathBuf,
    pub image: String,
    pub work_root: PathBuf,
    pub resources: crate::resources::Resources,
}
#[derive(Clone, Deserialize)]
pub struct Job {
    pub script: Vec<String>,
    #[serde(default)]
    pub commit_sha: String,
    #[serde(default)]
    pub image: String,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default = "timeout")]
    pub timeout_seconds: u64,
}
fn timeout() -> u64 {
    3600
}
pub struct Outcome {
    pub success: bool,
    pub exit_code: i32,
    pub duration_millis: u128,
}

pub async fn execute(
    client: &Client,
    id: &str,
    attempt: i64,
    job: Job,
    options: &Options,
) -> Result<Outcome> {
    if job.script.is_empty() || job.timeout_seconds == 0 || job.timeout_seconds > 86400 {
        bail!("invalid claimed script or timeout");
    }
    let temporary = tempfile::Builder::new()
        .prefix("gity-job-")
        .tempdir_in(&options.work_root)?;
    let workspace = temporary.path().join("source");
    tokio::fs::create_dir(&workspace).await?;
    if !job.commit_sha.is_empty() {
        let archive = temporary.path().join("source.zip");
        client
            .download(
                &format!("/runners/jobs/{id}/source"),
                json!({"attempt":attempt}),
                &archive,
                256 * 1024 * 1024,
            )
            .await?;
        let destination = workspace.clone();
        tokio::task::spawn_blocking(move || {
            source::extract_reader(std::fs::File::open(archive)?, &destination)
        })
        .await??;
    }
    let script = temporary.path().join("script.sh");
    let mut body = String::from("#!/bin/sh\nset -eu\n");
    for command in &job.script {
        body.push_str(command);
        body.push('\n');
    }
    tokio::fs::write(&script, body).await?;
    let name = format!("gity-job-{id}-{attempt}");
    let mut command =
        super::command::build_command(&job, options, &workspace, &script, &name, temporary.path())?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    command.process_group(0);
    let mut child = command.spawn().context("start CI executor")?;
    let child_id = child.id();
    let started = Instant::now();
    let (send, receive) = mpsc::channel::<Vec<u8>>(16);
    let out = tokio::spawn(super::output::capture(
        child.stdout.take().context("capture script stdout")?,
        send.clone(),
    ));
    let err = tokio::spawn(super::output::capture(
        child.stderr.take().context("capture script stderr")?,
        send,
    ));
    let log_client = client.clone();
    let log_id = id.to_owned();
    let logs = tokio::spawn(async move {
        super::output::upload_logs(log_client, log_id, attempt, receive).await
    });
    let deadline = tokio::time::sleep(Duration::from_secs(job.timeout_seconds));
    tokio::pin!(deadline);
    let pulse_client = client.clone();
    let pulse_id = id.to_owned();
    let mut pulse = tokio::spawn(async move {
        let mut heartbeat = tokio::time::interval(Duration::from_secs(15));
        heartbeat.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        loop {
            heartbeat.tick().await;
            let response = tokio::time::timeout(
                Duration::from_secs(15),
                pulse_client.post("/runners/heartbeat", json!({})),
            )
            .await
            .context("runner heartbeat timed out")??;
            if response["running_job_attempts"][&pulse_id].as_i64() != Some(attempt) {
                return Err::<(), anyhow::Error>(anyhow::anyhow!(
                    "CI job was canceled or its lease was lost"
                ));
            }
        }
    });
    let status: Result<std::process::ExitStatus> = tokio::select! {
        status=child.wait()=>status.context("wait for CI script"),
        _=&mut deadline=>Err(anyhow::anyhow!("CI job exceeded its timeout")),
        result=&mut pulse=>match result {Ok(Err(error))=>Err(error),Ok(Ok(()))=>Err(anyhow::anyhow!("CI job heartbeat stopped")),Err(error)=>Err(error.into())},
    };
    pulse.abort();
    // Scripts may fork background children that inherit the trace pipes. Tear
    // down the whole group after either success or failure before collecting IO.
    terminate_group(child_id);
    if status.is_err() {
        let _ = child.kill().await;
        let _ = child.wait().await;
    }
    if matches!(options.executor, Executor::Container) {
        let _ = Command::new(&options.container_bin)
            .args(["rm", "--force", &name])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;
    }
    out.await??;
    err.await??;
    logs.await??;
    let status = status?;
    let success = status.success();
    let code = status.code().unwrap_or(-1);
    if success {
        super::output::upload_artifacts(client, id, attempt, &workspace, &job.artifacts).await?;
    }
    Ok(Outcome {
        success,
        exit_code: code,
        duration_millis: started.elapsed().as_millis(),
    })
}
fn terminate_group(pid: Option<u32>) {
    #[cfg(unix)]
    if let Some(pid) = pid {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
}
