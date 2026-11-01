use crate::{GitNodeService, state::RepositoryRole};
use anyhow::{Context, Result, ensure};
use gity_git_wire::{
  TransferKind, TransferOpen,
  wire::{ExitStatus, TransferFrame, transfer_frame::Payload},
};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};
use tonic::{Status, Streaming};

impl GitNodeService {
  pub(crate) async fn transfer_stream(
    &self,
    mut incoming: Streaming<TransferFrame>,
  ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<TransferFrame, Status>>, Status> {
    let first = incoming
      .message()
      .await?
      .ok_or_else(|| Status::invalid_argument("Git transfer open frame required"))?;
    let Some(Payload::OpenJson(bytes)) = first.payload else {
      return Err(Status::invalid_argument("Git transfer open frame required"));
    };
    let open: TransferOpen = serde_json::from_slice(&bytes)
      .map_err(|_| Status::invalid_argument("invalid Git transfer options"))?;
    let write = is_write_transfer(&open.kind);
    let bundle = matches!(&open.kind, TransferKind::Bundle);
    let service = self.clone();
    let repository = open.repository.as_str().to_owned();
    let (outgoing, responses) = mpsc::channel(2);
    self
      .spawn_worker(move |stop| async move {
        let run = async {
          let _guard = if write || bundle {
            Some(service.write_lock(&repository).await)
          } else {
            None
          };
          service.check_transfer_access(&repository, open.epoch, write, bundle)?;
          if bundle {
            send_bundle(&service, &repository, outgoing.clone(), stop.clone()).await
          } else {
            relay_git_transfer(
              &service,
              &repository,
              open.kind,
              incoming,
              outgoing.clone(),
              stop.clone(),
            )
            .await
          }
        };
        let result = tokio::select! {
          _ = stop.cancelled() => Err(anyhow::anyhow!("Git transfer cancelled")),
          _ = outgoing.closed() => Err(anyhow::anyhow!("Git transfer consumer disconnected")),
          result = run => result,
        };
        send_exit(&outgoing, result.is_ok()).await;
      })
      .await?;
    Ok(tokio_stream::wrappers::ReceiverStream::new(responses))
  }

  fn check_transfer_access(&self, repo: &str, epoch: u64, write: bool, bundle: bool) -> Result<()> {
    let state = self.require_repository_state(repo, epoch, bundle)?;
    if write {
      self.allow_write().map_err(anyhow::Error::msg)?;
      ensure!(
        state.role == RepositoryRole::Primary,
        "repository is not primary"
      );
    }
    Ok(())
  }
}

fn is_write_transfer(kind: &TransferKind) -> bool {
  matches!(
    kind,
    TransferKind::Http {
      receive: true,
      advertise: false,
      ..
    } | TransferKind::Ssh { receive: true, .. }
  )
}

async fn send_bundle(
  service: &GitNodeService,
  repo: &str,
  outgoing: mpsc::Sender<Result<TransferFrame, Status>>,
  stop: tokio_util::sync::CancellationToken,
) -> Result<()> {
  let snapshot = service.git.reference_snapshot(repo).await?;
  send_frame(
    &outgoing,
    Payload::SnapshotJson(serde_json::to_vec(&snapshot)?),
  )
  .await?;
  if snapshot.refs.is_empty() {
    return Ok(());
  }
  let mut process = service.git.spawn_bundle(repo).await?;
  let mut stdout = process
    .take_stdout()
    .context("bundle process has no stdout")?;
  let mut buffer = vec![0; 64 * 1024];
  loop {
    let count = tokio::select! {
      _ = stop.cancelled() => anyhow::bail!("Git bundle transfer cancelled"),
      _ = outgoing.closed() => anyhow::bail!("Git bundle consumer disconnected"),
      count = stdout.read(&mut buffer) => count?,
    };
    if count == 0 {
      break;
    }
    send_frame(&outgoing, Payload::Data(buffer[..count].to_vec())).await?;
  }
  drop(stdout);
  process.wait().await
}

async fn relay_git_transfer(
  service: &GitNodeService,
  repo: &str,
  kind: TransferKind,
  mut incoming: Streaming<TransferFrame>,
  outgoing: mpsc::Sender<Result<TransferFrame, Status>>,
  stop: tokio_util::sync::CancellationToken,
) -> Result<()> {
  let mut process = service.open_transfer(repo, kind).await?;
  async {
    let mut stdin = process.take_stdin();
    let mut stdout = process.take_stdout().context("Git stdout unavailable")?;
    let input = async {
      loop {
        let frame = tokio::select! {
          _ = stop.cancelled() => anyhow::bail!("Git transfer cancelled"),
          _ = outgoing.closed() => anyhow::bail!("Git transfer consumer disconnected"),
          frame = incoming.message() => frame?,
        };
        match frame.and_then(|frame| frame.payload) {
          Some(Payload::Data(bytes)) => {
            ensure!(bytes.len() <= 64 * 1024, "Git transfer chunk exceeds limit");
            stdin
              .as_mut()
              .context("Git operation does not accept input")?
              .write_all(&bytes)
              .await?;
          }
          Some(Payload::InputFinished(true)) | None => {
            drop(stdin.take());
            return Ok::<_, anyhow::Error>(());
          }
          _ => anyhow::bail!("unexpected Git transfer input frame"),
        }
      }
    };
    let output = async {
      let mut buffer = vec![0; 64 * 1024];
      loop {
        let count = tokio::select! {
          _ = stop.cancelled() => anyhow::bail!("Git transfer cancelled"),
          _ = outgoing.closed() => anyhow::bail!("Git transfer consumer disconnected"),
          count = stdout.read(&mut buffer) => count?,
        };
        if count == 0 {
          return Ok::<_, anyhow::Error>(());
        }
        send_frame(&outgoing, Payload::Data(buffer[..count].to_vec())).await?;
      }
    };
    tokio::pin!(input, output);
    tokio::select! {
      result = &mut output => result?,
      result = &mut input => { result?; output.as_mut().await?; }
    }
    Ok::<_, anyhow::Error>(())
  }
  .await?;
  process.wait().await
}

async fn send_frame(
  outgoing: &mpsc::Sender<Result<TransferFrame, Status>>,
  payload: Payload,
) -> Result<()> {
  outgoing
    .send(Ok(TransferFrame {
      payload: Some(payload),
    }))
    .await
    .context("Git transfer consumer disconnected")
}

async fn send_exit(outgoing: &mpsc::Sender<Result<TransferFrame, Status>>, success: bool) {
  let _ = outgoing
    .send(Ok(TransferFrame {
      payload: Some(Payload::Exit(ExitStatus {
        success,
        message: if success {
          String::new()
        } else {
          "Git storage transfer failed".into()
        },
      })),
    }))
    .await;
}
