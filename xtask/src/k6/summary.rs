use anyhow::{Context, Result, bail};
use std::{
  fs::{File, OpenOptions},
  path::{Path, PathBuf},
  process::Command,
};

/// Reserve the destination atomically; k6 writes only into an isolated scratch
/// directory, never into an existing report or a directory containing user data.
pub(super) struct SummaryExport {
  destination: PathBuf,
  reserved: File,
  scratch: tempfile::TempDir,
  published: bool,
}

impl SummaryExport {
  pub fn prepare(root: &Path, path: &Path) -> Result<Self> {
    let path = if path.is_absolute() {
      path.to_owned()
    } else {
      root.join(path)
    };
    let parent = path
      .parent()
      .context("summary export parent missing")?
      .canonicalize()
      .context("summary export parent must already exist")?;
    let destination = parent.join(
      path
        .file_name()
        .context("summary export filename missing")?,
    );
    let mut options = OpenOptions::new();
    options.create_new(true).read(true).write(true);
    #[cfg(unix)]
    {
      use std::os::unix::fs::OpenOptionsExt;
      options.mode(0o600);
    }
    let scratch = tempfile::Builder::new()
      .prefix(".gity-k6-summary-")
      .tempdir_in(parent)
      .context("create isolated k6 summary directory")?;
    let reserved = options
      .open(&destination)
      .context("summary export must be a new file")?;
    Ok(Self {
      destination,
      reserved,
      scratch,
      published: false,
    })
  }

  pub fn local(&self, command: &mut Command) {
    command.arg("--summary-export").arg(self.output_path());
  }

  pub fn docker_mount(&self, command: &mut Command) -> Result<()> {
    let source = self
      .scratch
      .path()
      .to_str()
      .context("summary directory must be UTF-8")?;
    if source.contains([',', '\n', '\r']) {
      bail!("summary directory cannot contain commas or line breaks");
    }
    command
      .arg("--mount")
      .arg(format!("type=bind,src={source},dst=/summary"));
    #[cfg(unix)]
    {
      // The official image runs as an unprivileged k6 user; use the invoking
      // user's IDs so a 0700 output mount is writable and host-readable.
      command
        .arg("--user")
        .arg(format!("{}:{}", unsafe { libc::geteuid() }, unsafe {
          libc::getegid()
        }));
    }
    Ok(())
  }

  pub fn docker_argument(&self, command: &mut Command) {
    command.args(["--summary-export", "/summary/summary.json"]);
  }

  pub fn publish(&mut self) -> Result<bool> {
    let path = self.output_path();
    if !path.exists() {
      return Ok(false);
    }
    if !self.owns_destination() {
      bail!("reserved k6 summary path changed during run");
    }
    let mut input = File::open(path).context("open k6 summary")?;
    let value: serde_json::Value =
      serde_json::from_reader(&input).context("k6 summary is not valid JSON")?;
    if !value["metrics"].is_object() {
      bail!("k6 summary has no metrics object");
    }
    use std::io::{Seek, SeekFrom};
    input.seek(SeekFrom::Start(0))?;
    std::io::copy(&mut input, &mut self.reserved).context("publish reserved k6 summary")?;
    self.reserved.sync_all()?;
    self.published = true;
    println!("k6 summary: {}", self.destination.display());
    Ok(true)
  }

  fn owns_destination(&self) -> bool {
    let Ok(path) = std::fs::symlink_metadata(&self.destination) else {
      return false;
    };
    let Ok(file) = self.reserved.metadata() else {
      return false;
    };
    if !path.is_file() || path.file_type().is_symlink() {
      return false;
    }
    #[cfg(unix)]
    {
      use std::os::unix::fs::MetadataExt;
      path.dev() == file.dev() && path.ino() == file.ino()
    }
    #[cfg(not(unix))]
    {
      path.created().ok() == file.created().ok()
    }
  }

  fn output_path(&self) -> PathBuf {
    self.scratch.path().join("summary.json")
  }
}

impl Drop for SummaryExport {
  fn drop(&mut self) {
    if !self.published && self.owns_destination() {
      let _ = std::fs::remove_file(&self.destination);
    }
  }
}

#[cfg(test)]
#[path = "summary_tests.rs"]
mod tests;
