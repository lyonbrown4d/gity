use crate::GitService;
use anyhow::{bail, Context, Result};
use std::{
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    process::Stdio,
    task::{Context as TaskContext, Poll},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, ReadBuf},
    process::{Child, ChildStdin, ChildStdout, Command},
    sync::OwnedSemaphorePermit,
    task::JoinHandle,
};
use tokio_util::sync::CancellationToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitMaintenance {
    GarbageCollect,
    Repack,
    UpdateServerInfo,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GitRpc {
    UploadPack,
    ReceivePack,
}
impl GitRpc {
    pub(crate) fn command(self) -> &'static str {
        match self {
            Self::UploadPack => "upload-pack",
            Self::ReceivePack => "receive-pack",
        }
    }
}

/// Dropping the process, or its streaming reader, terminates native Git.
pub struct GitProcess {
    child: Child,
    stderr: JoinHandle<std::io::Result<Vec<u8>>>,
    _permit: OwnedSemaphorePermit,
}
impl GitProcess {
    pub fn take_stdin(&mut self) -> Option<ChildStdin> {
        self.child.stdin.take()
    }
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout.take()
    }
    pub async fn wait(mut self) -> Result<()> {
        drop(self.child.stdin.take());
        let status = self.child.wait().await?;
        let stderr = (&mut self.stderr).await??;
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
        let stdout = self
            .child
            .stdout
            .take()
            .context("git stdout was already taken")?;
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
        if let Some(pid) = self.child.id() {
            // Git helpers and hooks inherit this process group.
            unsafe {
                libc::kill(-(pid as i32), libc::SIGKILL);
            }
        }
        let _ = self.child.start_kill();
    }
}

pub struct GitReader {
    stdout: ChildStdout,
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
                Poll::Ready(
                    result.unwrap_or_else(|_| Err(std::io::Error::other("Git supervisor stopped"))),
                )
            }
        }
    }
}
impl Drop for GitReader {
    fn drop(&mut self) {
        self.cancellation.cancel();
    }
}

impl GitService {
    pub async fn maintenance(&self, repo: &str, operation: GitMaintenance) -> Result<()> {
        let args: &[&str] = match operation {
            GitMaintenance::GarbageCollect => &["gc", "--auto"],
            GitMaintenance::Repack => &["repack", "-a", "-d"],
            GitMaintenance::UpdateServerInfo => &["update-server-info"],
        };
        let output = self.command(repo)?.args(args).output().await?;
        if !output.status.success() {
            bail!(
                "Git maintenance failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    pub(crate) fn command(&self, repo: &str) -> Result<Command> {
        let mut command = Command::new(self.git_bin.as_ref());
        #[cfg(unix)]
        command.process_group(0);
        command
            .current_dir(self.repository_path(repo)?)
            .kill_on_drop(true)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // Process-level ambient Git overrides must not select a different repository.
        for key in [
            "GIT_DIR",
            "GIT_WORK_TREE",
            "GIT_COMMON_DIR",
            "GIT_INDEX_FILE",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
            "GIT_CONFIG_COUNT",
        ] {
            command.env_remove(key);
        }
        command
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_CONFIG_NOSYSTEM", "1");
        Ok(command)
    }

    pub async fn spawn_rpc(
        &self,
        repo: &str,
        service: GitRpc,
        advertise: bool,
        env: BTreeMap<String, String>,
    ) -> Result<GitProcess> {
        let permit = self.rpc_limit.clone().acquire_owned().await?;
        if service == GitRpc::ReceivePack && !advertise {
            // Never silently bypass branch protection because the adapter is absent.
            let hook = self.repository_path(repo)?.join("hooks/update");
            let metadata = std::fs::metadata(&hook)
                .context("receive-pack requires an installed update hook")?;
            if !metadata.is_file() {
                bail!("receive-pack update hook is not a file");
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if metadata.permissions().mode() & 0o111 == 0 {
                    bail!("receive-pack update hook is not executable");
                }
            }
        }
        let mut command = self.command(repo)?;
        command.arg(service.command()).arg("--stateless-rpc");
        if advertise {
            command.arg("--advertise-refs");
        } else {
            command.stdin(Stdio::piped());
        }
        command.arg(".");
        for (key, value) in env {
            if !matches!(
                key.as_str(),
                "GIT_PROTOCOL"
                    | "GITY_DENY_FORCE_PUSH_REFS"
                    | "GITY_DENY_DELETE_REFS"
                    | "GITY_DENY_UPDATE_REFS"
            ) {
                bail!("unsupported Git request environment variable");
            }
            command.env(key, value);
        }
        spawn_process(command, permit)
    }
}

pub(crate) fn spawn_process(
    mut command: Command,
    permit: OwnedSemaphorePermit,
) -> Result<GitProcess> {
    let mut child = command.spawn().context("start Git protocol process")?;
    let mut stderr = child.stderr.take().context("capture Git stderr")?;
    let task = tokio::spawn(async move {
        let mut kept = Vec::new();
        let mut buffer = [0u8; 8192];
        loop {
            let count = stderr.read(&mut buffer).await?;
            if count == 0 {
                break;
            }
            let retain = count.min(65536usize.saturating_sub(kept.len()));
            kept.extend_from_slice(&buffer[..retain]);
        }
        Ok(kept)
    });
    Ok(GitProcess {
        child,
        stderr: task,
        _permit: permit,
    })
}
