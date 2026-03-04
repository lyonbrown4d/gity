use git2::Repository as Git2Repo;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct GitRepository {
  repo: Git2Repo,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
  #[error("failed to open repository: {0}")]
  Open(#[from] git2::Error),
}

impl GitRepository {
  /// 打开一个仓库（bare 或非 bare）。
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, RepositoryError> {
    let repo = Git2Repo::open(path.as_ref())?;
    Ok(Self { repo })
  }

  /// 获取内部的 git2 repository 引用。
  pub fn inner(&self) -> &Git2Repo {
    &self.repo
  }
}
