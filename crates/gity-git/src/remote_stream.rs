use crate::{GitProcess, GitService};
use anyhow::{bail, Context, Result};
use gity_git_wire::{
  wire::{transfer_frame::Payload, TransferFrame},
  RepositoryKey, TransferKind, TransferOpen,
};
use tokio::{
  io::{AsyncReadExt, AsyncWriteExt},
  sync::mpsc,
};
use tokio_stream::wrappers::ReceiverStream;
use tokio_util::sync::CancellationToken;

impl GitService {
  pub(crate) async fn stream_remote(
    &self,
    repo: &str,
    kind: TransferKind,
    write: bool,
  ) -> Result<GitProcess> {
    let permit = self.rpc_limit.clone().acquire_owned().await?;
    let router = self
      .remote
      .as_ref()
      .context("remote backend required")?
      .clone();
    let node = if write {
      router.begin_write(repo).await?
    } else {
      router.node_for(repo, false).await?
    };
    let open = TransferOpen {
      epoch: node.epoch(),
      repository: RepositoryKey::try_from(repo.to_owned())?,
      kind,
    };
    let (send, incoming) = mpsc::channel(2);
    send
      .send(TransferFrame {
        payload: Some(Payload::OpenJson(serde_json::to_vec(&open)?)),
      })
      .await?;
    let response = node
      .client()
      .transfer(node.authenticate(ReceiverStream::new(incoming))?)
      .await;
    let mut response = match response {
      Ok(response) => response.into_inner(),
      Err(error) => {
        if write {
          router
            .write_finished(
              repo,
              &node,
              gity_git_wire::definitive_rejection(&anyhow::anyhow!(error.clone())),
            )
            .await?;
        }
        return Err(error.into());
      }
    };
    let repository = repo.to_owned();
    let (input, mut upload_input) = tokio::io::duplex(64 * 1024);
    let (mut output, reader) = tokio::io::duplex(64 * 1024);
    let cancellation = CancellationToken::new();
    let signal = cancellation.clone();
    let task = tokio::spawn(async move {
      let upload = async move {
        let mut buffer = vec![0; 64 * 1024];
        loop {
          let count = upload_input.read(&mut buffer).await?;
          if count == 0 {
            send
              .send(TransferFrame {
                payload: Some(Payload::InputFinished(true)),
              })
              .await?;
            return Ok::<_, anyhow::Error>(());
          }
          send
            .send(TransferFrame {
              payload: Some(Payload::Data(buffer[..count].to_vec())),
            })
            .await?;
        }
      };
      let download = async move {
        while let Some(frame) = response.message().await? {
          match frame.payload {
            Some(Payload::Data(bytes)) => output.write_all(&bytes).await?,
            Some(Payload::Exit(status)) => {
              if write {
                router.write_finished(&repository, &node, true).await?;
              }
              output.shutdown().await?;
              if !status.success {
                bail!("Git storage process failed: {}", status.message);
              }
              return Ok(());
            }
            _ => bail!("unexpected Git storage response frame"),
          }
        }
        bail!("Git storage stream ended without exit status")
      };
      let exchange = async {
        tokio::pin!(upload, download);
        tokio::select! {
            result = &mut download => result,
            result = &mut upload => { result?; download.await }
        }
      };
      tokio::select! {
          result = exchange => result,
          _ = signal.cancelled() => bail!("Git storage transfer cancelled"),
      }
    });
    Ok(GitProcess::remote(
      Box::pin(input),
      Box::pin(reader),
      task,
      cancellation,
      permit,
    ))
  }
}
