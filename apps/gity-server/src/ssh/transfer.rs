use super::keys;
use crate::{AppState, projects, repository};
use gity_git::{GitRpc, SshPolicy};
use russh::{
    Channel,
    server::{Handle, Msg},
};
use serde_json::Value;
use std::time::Duration;
use tokio::io::AsyncWriteExt;

pub(super) struct Authorized {
    project: Value,
    service: GitRpc,
    policy: Option<SshPolicy>,
    pub protocol: Option<String>,
}
pub(super) async fn authorize(
    state: &AppState,
    fingerprint: &str,
    path: &str,
    service: GitRpc,
) -> anyhow::Result<Authorized> {
    let project = state
        .db
        .one(
            "SELECT * FROM projects WHERE full_path=? AND status='active' AND deleted_at IS NULL",
            vec![path.into()],
        )
        .await?
        .ok_or_else(|| anyhow::anyhow!("repository unavailable"))?;
    let id = projects::id(&project["id"])?;
    let key = keys::authorized(state, fingerprint)
        .await?
        .into_iter()
        .find(|key| key["project_id"] == project["id"])
        .ok_or_else(|| anyhow::anyhow!("key is not authorized for this repository"))?;
    let policy = if service == GitRpc::ReceivePack {
        anyhow::ensure!(
            key["can_push"]
                .as_bool()
                .unwrap_or_else(|| key["can_push"].as_i64() == Some(1)),
            "deploy key is read-only"
        );
        let mut rules = state
            .db
            .all(
                "SELECT * FROM project_branch_protections WHERE project_id=?",
                vec![id.into()],
            )
            .await?;
        for rule in &mut rules {
            for field in ["require_merge_request", "allow_force_push", "allow_delete"] {
                rule[field] = Value::Bool(
                    rule[field]
                        .as_bool()
                        .unwrap_or_else(|| rule[field].as_i64() == Some(1)),
                );
            }
        }
        state
            .git
            .ensure_update_hook(
                &repository::repo(&project),
                state
                    .settings
                    .git_hook
                    .clone()
                    .unwrap_or(std::env::current_exe()?),
            )
            .await?;
        Some(SshPolicy {
            default_branch: repository::default_branch(&project).into(),
            rules: serde_json::from_value(Value::Array(rules))?,
        })
    } else {
        None
    };
    state
        .db
        .execute(
            "UPDATE project_deploy_keys SET last_used_at=? WHERE id=?",
            vec![gity_core::now().into(), projects::id(&key["id"])?.into()],
        )
        .await?;
    Ok(Authorized {
        project,
        service,
        policy,
        protocol: None,
    })
}
#[tracing::instrument(name="git.ssh", skip_all, fields(project_id=authorized.project["id"].as_i64().unwrap_or_default(), service=?authorized.service))]
pub(super) async fn run(
    state: AppState,
    channel: Channel<Msg>,
    handle: Handle,
    authorized: Authorized,
) {
    let id = channel.id();
    let timeout = Duration::from_secs(state.settings.ssh.operation_timeout_seconds);
    let project_id = authorized.project["id"].as_i64().unwrap_or_default();
    let operation = if authorized.service == GitRpc::ReceivePack {
        "push"
    } else {
        "fetch"
    };
    let started = std::time::Instant::now();
    let result = tokio::time::timeout(timeout, exchange(&state, channel, authorized)).await;
    let status = match result {
        Ok(Ok(())) => 0,
        Ok(Err(error)) => {
            tracing::warn!(error=%error,"SSH Git transfer failed");
            1
        }
        Err(_) => {
            tracing::warn!("SSH Git operation timed out");
            1
        }
    };
    let result = if status == 0 { "success" } else { "failure" };
    metrics::counter!("gity_ssh_operations_total", "operation"=>operation, "result"=>result)
        .increment(1);
    metrics::histogram!("gity_ssh_operation_seconds", "operation"=>operation)
        .record(started.elapsed().as_secs_f64());
    tracing::info!(
        project_id,
        operation,
        result,
        elapsed_ms = started.elapsed().as_millis() as u64,
        "SSH Git operation completed"
    );
    let _ = handle.exit_status_request(id, status).await;
    let _ = handle.eof(id).await;
    let _ = handle.close(id).await;
}
async fn exchange(
    state: &AppState,
    channel: Channel<Msg>,
    authorized: Authorized,
) -> anyhow::Result<()> {
    let path = repository::repo(&authorized.project);
    let mut process = state
        .git
        .spawn_ssh(
            &path,
            authorized.service,
            authorized.protocol.as_deref(),
            authorized.policy.as_ref(),
        )
        .await?;
    let mut stdin = process
        .take_stdin()
        .ok_or_else(|| anyhow::anyhow!("Git input unavailable"))?;
    let (mut reader, mut writer) = tokio::io::split(channel.into_stream());
    // Dropping the scoped task on cancellation closes stdin and releases the channel.
    let (sent, updates) = tokio::sync::oneshot::channel::<Vec<(String, String)>>();
    let writing = authorized.service == GitRpc::ReceivePack;
    let completion_state = state.clone();
    let project = authorized.project.clone();
    let mut output = process.into_reader_with_completion(move || async move {
        if !writing {
            return;
        }
        let Ok(updates) = updates.await else {
            return;
        };
        for (reference, sha) in updates {
            if sha.bytes().all(|b| b == b'0') {
                continue;
            }
            let current = completion_state
                .git
                .resolve_revision(&path, &reference, repository::default_branch(&project))
                .await;
            if current.as_ref().is_ok_and(|current| current == &sha) {
                crate::events::repository_changed(&completion_state, &project, &reference, &sha)
                    .await;
            }
        }
    })?;
    let input = tokio::spawn(async move {
        if writing {
            let (prefix, updates) = super::updates::read(&mut reader)
                .await
                .map_err(std::io::Error::other)?;
            let _ = sent.send(updates);
            stdin.write_all(&prefix).await?;
        }
        tokio::io::copy(&mut reader, &mut stdin).await?;
        stdin.shutdown().await
    });
    let input = InputTask(input);
    let copied = tokio::io::copy(&mut output, &mut writer).await;
    drop(input);
    copied?;
    writer.flush().await?;
    Ok(())
}
struct InputTask(tokio::task::JoinHandle<std::io::Result<()>>);
impl Drop for InputTask {
    fn drop(&mut self) {
        self.0.abort();
    }
}
