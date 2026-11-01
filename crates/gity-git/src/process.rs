use anyhow::{bail, Context, Result};
use std::{
  future::Future,
  pin::Pin,
  task::{Context as TaskContext, Poll},
};
use tokio::{
  io::{AsyncRead, AsyncWrite, ReadBuf},
  process::Child,
  sync::OwnedSemaphorePermit,
  task::JoinHandle,
};
use tokio_util::sync::CancellationToken;
pub type GitInput = Pin<Box<dyn AsyncWrite + Send>>;
pub type GitOutput = Pin<Box<dyn AsyncRead + Send>>;

/// Dropping the process, or its streaming reader, terminates native Git.
pub struct GitProcess {
  child: Option<Child>,
  stdin: Option<GitInput>,
  stdout: Option<GitOutput>,
  remote_task: Option<JoinHandle<Result<()>>>,
  remote_cancel: CancellationToken,
  stderr: Option<JoinHandle<std::io::Result<Vec<u8>>>>,
  _permit: OwnedSemaphorePermit,
}
impl GitProcess {
  pub fn take_stdin(&mut self) -> Option<GitInput> {
    self.stdin.take()
  }
  pub fn take_stdout(&mut self) -> Option<GitOutput> {
    self.stdout.take()
  }
  pub async fn wait(mut self) -> Result<()> {
    drop(self.stdin.take());
    if let Some(task) = self.remote_task.take() {
      return task.await?;
    }
    let status = self
      .child
      .as_mut()
      .context("missing Git process")?
      .wait()
      .await?;
    let stderr = self
      .stderr
      .take()
      .context("missing stderr drain")?
      .await??;
    if !status.success() {
      bail!(
        "git process failed ({status}): {}",
        String::from_utf8_lossy(&stderr)
      );
    }
    Ok(())
  }
  /// Use with ReaderStream::new(process.into_reader()?). The reader owns the
  /// process and RPC permit until response completion or client cancellation.
  pub fn into_reader(self) -> Result<GitReader> {
    self.into_reader_with_completion(|| async {})
  }
  /// Run post-receive work only after native Git reports a successful exit.
  pub fn into_reader_with_completion<F, Fut>(mut self, completed: F) -> Result<GitReader>
  where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = ()> + Send + 'static,
  {
    let stdout = self.stdout.take().context("git stdout was already taken")?;
    let cancellation = CancellationToken::new();
    let signal = cancellation.clone();
    let (complete, completion) = tokio::sync::oneshot::channel();
    tokio::spawn(async move {
      let result = tokio::select! {
          result=self.wait()=>result.map_err(|e|std::io::Error::other(e.to_string())),
          _=signal.cancelled()=>Err(std::io::Error::new(std::io::ErrorKind::Interrupted,"Git response cancelled")),
      };
      if result.is_ok() {
        completed().await;
      }
      let _ = complete.send(result);
    });
    Ok(GitReader {
      stdout,
      completion,
      cancellation,
      eof: false,
      finished: false,
    })
  }
}
impl Drop for GitProcess {
  fn drop(&mut self) {
    #[cfg(unix)]
    if let Some(pid) = self.child.as_ref().and_then(Child::id) {
      // Git helpers and hooks inherit this process group.
      unsafe {
        libc::kill(-(pid as i32), libc::SIGKILL);
      }
    }
    if let Some(child) = &mut self.child {
      let _ = child.start_kill();
    }
    self.remote_cancel.cancel();
    if let Some(task) = &self.remote_task {
      task.abort();
    }
  }
}

pub struct GitReader {
  stdout: GitOutput,
  completion: tokio::sync::oneshot::Receiver<std::io::Result<()>>,
  cancellation: CancellationToken,
  eof: bool,
  finished: bool,
}
impl AsyncRead for GitReader {
  fn poll_read(
    mut self: Pin<&mut Self>,
    cx: &mut TaskContext<'_>,
    buf: &mut ReadBuf<'_>,
  ) -> Poll<std::io::Result<()>> {
    if self.finished || buf.remaining() == 0 {
      return Poll::Ready(Ok(()));
    }
    if !self.eof {
      let before = buf.filled().len();
      match Pin::new(&mut self.stdout).poll_read(cx, buf) {
        Poll::Pending => return Poll::Pending,
        Poll::Ready(Err(error)) => return Poll::Ready(Err(error)),
        Poll::Ready(Ok(())) if buf.filled().len() > before => return Poll::Ready(Ok(())),
        Poll::Ready(Ok(())) => self.eof = true,
      }
    }
    match Pin::new(&mut self.completion).poll(cx) {
      Poll::Pending => Poll::Pending,
      Poll::Ready(result) => {
        self.finished = true;
        Poll::Ready(result.unwrap_or_else(|_| Err(std::io::Error::other("Git supervisor stopped"))))
      }
    }
  }
}
impl Drop for GitReader {
  fn drop(&mut self) {
    self.cancellation.cancel();
  }
}

impl GitProcess {
  pub(crate) fn local(
    mut child: Child,
    stderr: JoinHandle<std::io::Result<Vec<u8>>>,
    permit: OwnedSemaphorePermit,
  ) -> Self {
    Self {
      stdin: child.stdin.take().map(|input| Box::pin(input) as GitInput),
      stdout: child
        .stdout
        .take()
        .map(|output| Box::pin(output) as GitOutput),
      child: Some(child),
      stderr: Some(stderr),
      remote_task: None,
      remote_cancel: CancellationToken::new(),
      _permit: permit,
    }
  }
  pub(crate) fn remote(
    input: GitInput,
    output: GitOutput,
    task: JoinHandle<Result<()>>,
    cancel: CancellationToken,
    permit: OwnedSemaphorePermit,
  ) -> Self {
    Self {
      child: None,
      stdin: Some(input),
      stdout: Some(output),
      stderr: None,
      remote_task: Some(task),
      remote_cancel: cancel,
      _permit: permit,
    }
  }
}
