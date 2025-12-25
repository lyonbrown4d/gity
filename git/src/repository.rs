use gix::Repository as GixRepo;
use std::path::Path;
use thiserror::Error;

#[derive(Debug)]
pub struct GitRepository {
  repo: GixRepo,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
  #[error("failed to open repository: {0}")]
  Open(#[from] gix::open::Error),
}

impl GitRepository {
  /// 打开一个仓库（从当前目录或指定路径向上查找）
  pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, RepositoryError> {
    let repo = gix::open(path.as_ref())?;
    Ok(Self { repo })
  }

  /// 获取内部的 gix repo 引用（以供 object/ref 操作）
  pub fn inner(&self) -> &GixRepo {
    &self.repo
  }
}
