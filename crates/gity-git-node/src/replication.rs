use crate::{
  GitNodeService,
  state::{RepositoryRole, RepositoryState},
};
use anyhow::{Context, Result, bail, ensure};
use gity_git_wire::{
  NodeClient, ReferenceSnapshot, RepositoryHead, RepositoryKey, TransferKind, TransferOpen,
  wire::{TransferFrame, transfer_frame::Payload},
};
use std::time::Duration;
use tokio::{io::AsyncWriteExt, sync::mpsc, time::timeout};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

impl GitNodeService {
  pub(crate) async fn sync_replica(
    &self,
    repo: &str,
    replica_epoch: u64,
    source: gity_git_wire::NodeEndpoint,
    source_epoch: u64,
    stop: CancellationToken,
  ) -> Result<ReferenceSnapshot> {
    let deadline = Duration::from_secs(self.settings.replica_timeout_secs);
    self
      .sync_replica_inner(repo, replica_epoch, source, source_epoch, stop, deadline)
      .await
  }

  async fn sync_replica_inner(
    &self,
    repo: &str,
    replica_epoch: u64,
    source: gity_git_wire::NodeEndpoint,
    source_epoch: u64,
    stop: CancellationToken,
    deadline: Duration,
  ) -> Result<ReferenceSnapshot> {
    ensure!(source_epoch > 0, "source epoch must be positive");
    ensure!(
      source.id != self.settings.id,
      "a node cannot replicate from itself"
    );
    let initializing = match self.load_repository_state(repo)? {
      Some(state) => {
        ensure!(
          matches!(
            state.role,
            RepositoryRole::Replica | RepositoryRole::Fenced | RepositoryRole::InitializingReplica
          ),
          "cannot overwrite a primary or primary initialization"
        );
        ensure!(replica_epoch >= state.epoch, "stale replica epoch");
        if state.role == RepositoryRole::InitializingReplica && replica_epoch > state.epoch {
          self.save_repository_state(
            repo,
            RepositoryState {
              role: RepositoryRole::InitializingReplica,
              epoch: replica_epoch,
              initial_branch: None,
            },
          )?;
        }
        state.role == RepositoryRole::InitializingReplica
      }
      None => {
        let path = self.git.repository_path(repo)?;
        ensure!(
          !path.exists(),
          "unmanaged repository requires explicit activation"
        );
        self.save_repository_state(
          repo,
          RepositoryState {
            role: RepositoryRole::InitializingReplica,
            epoch: replica_epoch,
            initial_branch: None,
          },
        )?;
        true
      }
    };

    let client = self.replication_source(source, source_epoch).await?;
    let key = RepositoryKey::try_from(repo.to_owned())?;
    let receiving = self.receive_bundle(&client, key, source_epoch, stop.child_token());
    let received = tokio::select! {
      _ = stop.cancelled() => bail!("replica work cancelled"),
      result = timeout(deadline, receiving) => result.context("replica synchronization timed out")?,
    }?;
    let repository_path = self.git.repository_path(repo)?;
    if initializing {
      let branch = initial_branch(&received.snapshot);
      self.git.validate_initial_branch(branch)?;
      if repository_path.exists() {
        self.git.resume_bare_initialization(repo, branch).await?;
      } else {
        self.git.init_bare(repo, branch).await?;
      }
      self
        .git
        .ensure_update_hook(repo, &self.settings.hook_executable)
        .await?;
    }

    self
      .git
      .apply_replica_bundle_cancellable(
        repo,
        received.bundle.as_ref().map(|bundle| bundle.path()),
        received.snapshot.clone(),
        stop.clone(),
      )
      .await?;
    ensure!(!stop.is_cancelled(), "replica work cancelled");
    self.save_repository_state(
      repo,
      RepositoryState {
        role: RepositoryRole::Replica,
        epoch: replica_epoch,
        initial_branch: None,
      },
    )?;
    Ok(received.snapshot)
  }

  async fn replication_source(
    &self,
    mut source: gity_git_wire::NodeEndpoint,
    epoch: u64,
  ) -> Result<NodeClient> {
    ensure!(
      source.client_certificate.is_none() && source.client_key.is_none(),
      "replica sources must not delegate caller TLS identities"
    );
    if source.uri.starts_with("https://") {
      let certificate = self
        .settings
        .tls_certificate
        .as_ref()
        .context("replica TLS identity is unavailable")?;
      let key = self
        .settings
        .tls_key
        .as_ref()
        .context("replica TLS identity is unavailable")?;
      source.client_certificate = Some(tokio::fs::read_to_string(certificate).await?);
      source.client_key = Some(tokio::fs::read_to_string(key).await?);
    }
    Ok(NodeClient::new(source)?.with_epoch(epoch))
  }

  async fn receive_bundle(
    &self,
    client: &NodeClient,
    repository: RepositoryKey,
    epoch: u64,
    stop: CancellationToken,
  ) -> Result<ReceivedBundle> {
    let deadline = Duration::from_secs(self.settings.replica_timeout_secs);
    tokio::select! {
      _ = stop.cancelled() => bail!("replica work cancelled"),
      result = timeout(deadline, async {
        let transfer = fetch_transfer(client, repository, epoch).await?;
        read_transfer(transfer, self.settings.replica_max_bytes).await
      }) => {
        result.context("replica transfer timed out")?
      }
    }
  }
}

struct ReceivedBundle {
  snapshot: ReferenceSnapshot,
  bundle: Option<tempfile::NamedTempFile>,
}

async fn fetch_transfer(
  client: &NodeClient,
  repository: RepositoryKey,
  epoch: u64,
) -> Result<tonic::Streaming<TransferFrame>> {
  let (sender, incoming) = mpsc::channel(2);
  sender
    .send(TransferFrame {
      payload: Some(Payload::OpenJson(serde_json::to_vec(&TransferOpen {
        epoch,
        repository,
        kind: TransferKind::Bundle,
      })?)),
    })
    .await
    .context("open source bundle transfer")?;
  sender
    .send(TransferFrame {
      payload: Some(Payload::InputFinished(true)),
    })
    .await
    .context("finish source bundle request")?;
  drop(sender);
  let response = client
    .client()
    .transfer(client.authenticate(ReceiverStream::new(incoming))?)
    .await?;
  Ok(response.into_inner())
}

async fn read_transfer(
  mut stream: tonic::Streaming<TransferFrame>,
  max_bytes: u64,
) -> Result<ReceivedBundle> {
  let mut snapshot = None;
  let mut bundle = None;
  let mut file = None;
  let mut bytes_written = 0u64;
  loop {
    let frame = stream
      .message()
      .await?
      .context("source bundle stream ended early")?;
    match frame.payload.context("source sent an empty frame")? {
      Payload::SnapshotJson(bytes) if snapshot.is_none() && bundle.is_none() => {
        snapshot = Some(serde_json::from_slice::<ReferenceSnapshot>(&bytes)?);
      }
      Payload::Data(bytes) if snapshot.is_some() => {
        ensure!(
          !bytes.is_empty() && bytes.len() <= 64 * 1024,
          "invalid bundle chunk size"
        );
        bytes_written = bytes_written
          .checked_add(bytes.len() as u64)
          .context("bundle size overflow")?;
        ensure!(
          bytes_written <= max_bytes,
          "replica bundle exceeds configured limit"
        );
        if file.is_none() {
          let temporary = tempfile::NamedTempFile::new()?;
          file = Some(tokio::fs::File::from_std(temporary.reopen()?));
          bundle = Some(temporary);
        }
        file
          .as_mut()
          .context("bundle file is unavailable")?
          .write_all(&bytes)
          .await?;
      }
      Payload::Exit(status) => {
        ensure!(status.success, "source bundle transfer failed");
        if let Some(file) = file.as_mut() {
          file.flush().await?;
          file.sync_all().await?;
        }
        drop(file);
        ensure!(
          stream.message().await?.is_none(),
          "frames followed bundle exit"
        );
        return Ok(ReceivedBundle {
          snapshot: snapshot.context("source omitted reference snapshot")?,
          bundle,
        });
      }
      _ => bail!("unexpected source bundle frame"),
    }
  }
}

fn initial_branch(snapshot: &ReferenceSnapshot) -> &str {
  match &snapshot.head {
    RepositoryHead::Symbolic(target) => target.strip_prefix("refs/heads/").unwrap_or("main"),
    RepositoryHead::Detached(_) => "main",
  }
}
