use crate::client::Client;
use anyhow::{Context, Result, bail};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::json;
use std::{
    path::Path,
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt},
    sync::mpsc,
};
pub(crate) async fn capture(
    mut source: impl AsyncRead + Unpin,
    send: mpsc::Sender<Vec<u8>>,
) -> Result<()> {
    let mut buffer = vec![0u8; 8192];
    loop {
        let count = source.read(&mut buffer).await?;
        if count == 0 {
            break;
        }
        if send.send(buffer[..count].to_vec()).await.is_err() {
            break;
        }
    }
    Ok(())
}
pub(crate) async fn upload_logs(
    client: Client,
    id: String,
    attempt: i64,
    mut receive: mpsc::Receiver<Vec<u8>>,
) -> Result<()> {
    let started = Instant::now();
    let mut total = 0;
    let mut truncated = false;
    let max = 4 * 1024 * 1024;
    let mut pending = Vec::new();
    let mut tick = tokio::time::interval(Duration::from_secs(2));
    loop {
        let flush = tokio::select! {
            chunk=receive.recv()=>match chunk{
                Some(chunk)=>{
                    let size=chunk.len().min(max-total);
                    pending.extend_from_slice(&chunk[..size]);
                    total+=size;
                    truncated|=size<chunk.len();
                    pending.len()>=32768
                }
                ,
                None=>{
                    client.post(&format!("/runners/jobs/{id}/trace"),
                        json!({
                                "attempt":attempt,
                                "output":String::from_utf8_lossy(&pending),
                                "output_truncated":truncated,
                                "duration_millis":started.elapsed().as_millis() as u64,
                                "final":true
                            }
                    )).await?;
                    return Ok(());
            }},
            _=tick.tick()=>!pending.is_empty(),
        };
        if flush {
            // Preserve a possibly incomplete UTF-8 sequence for the next chunk.
            let end = match std::str::from_utf8(&pending) {
                Ok(_) => pending.len(),
                Err(error) if error.error_len().is_none() => error.valid_up_to(),
                Err(_) => pending.len(),
            };
            client
                .post(
                    &format!("/runners/jobs/{id}/trace"),
                    json!({
                                "attempt":attempt,
                                "output":String::from_utf8_lossy(&pending[..end]),
                                "output_truncated":truncated,
                                "duration_millis":started.elapsed().as_millis() as u64
                            }
                    ),
                )
                .await?;
            pending.drain(..end);
        }
    }
}
pub(crate) async fn upload_artifacts(
    client: &Client,
    id: &str,
    attempt: i64,
    workspace: &Path,
    paths: &[String],
) -> Result<()> {
    let root = workspace.canonicalize()?;
    for relative in paths {
        if Path::new(relative)
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)))
        {
            bail!("invalid artifact path");
        }
        let candidate = root.join(relative);
        let metadata = std::fs::symlink_metadata(&candidate).context("artifact file is missing")?;
        if metadata.file_type().is_symlink()
            || !metadata.is_file()
            || metadata.len() > 64 * 1024 * 1024
        {
            bail!("artifact must be a regular file at most 64 MiB");
        }
        let path = candidate.canonicalize()?;
        if !path.starts_with(&root) {
            bail!("artifact path escapes workspace");
        }
        let bytes = tokio::fs::read(&path).await?;
        client.post(&format!("/runners/jobs/{id}/artifacts"),
            json!({
                    "attempt":attempt,
                    "name":relative,
                    "file_name":path.file_name().context("artifact filename missing")?.to_string_lossy(),
                    "file_path":relative,
                    "content_type":"application/octet-stream",
                    "content_base64":STANDARD.encode(bytes)
                }
        )).await?;
    }
    Ok(())
}
