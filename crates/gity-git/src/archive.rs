use crate::{GitProcess, GitService};
use anyhow::{bail, Result};
use tokio::io::AsyncReadExt;
impl GitService {
    /// Native streaming ZIP archive; dropping the reader cancels native Git.
    pub async fn spawn_archive(&self, repo: &str, revision: &str) -> Result<GitProcess> {
        if revision.is_empty() || revision.starts_with('-') || revision.contains('\0') {
            bail!("git source reference not found");
        }
        let id = self.resolve_revision(repo, revision, "").await?;
        let permit = self.rpc_limit.clone().acquire_owned().await?;
        let mut command = self.command(repo)?;
        command.args(["archive", "--format=zip", &id]);
        crate::protocol::spawn_process(command, permit)
    }
    /// Bounded compatibility API; callers serving large archives should stream.
    pub async fn archive(&self, repo: &str, revision: &str) -> Result<Vec<u8>> {
        let mut reader = self
            .spawn_archive(repo, revision)
            .await?
            .into_reader()?
            .take(256 * 1024 * 1024 + 1);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        if bytes.len() > 256 * 1024 * 1024 {
            bail!("archive exceeds 256 MiB; use streaming archive");
        }
        Ok(bytes)
    }
}
