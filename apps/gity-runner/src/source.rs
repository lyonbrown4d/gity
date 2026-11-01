use anyhow::{Context, Result, bail};
use std::{
  io::{Read, Seek, Write},
  path::Path,
};

/// Source archives are treated as untrusted, including Git symlink entries.
pub fn extract_reader(reader: impl Read + Seek, destination: &Path) -> Result<()> {
  let mut archive = zip::ZipArchive::new(reader)?;
  if archive.len() > 100_000 {
    bail!("source archive contains too many entries");
  }
  let mut total = 0u64;
  let mut links = Vec::new();
  for index in 0..archive.len() {
    let mut entry = archive.by_index(index)?;
    let enclosed = entry
      .enclosed_name()
      .context("source archive contains an unsafe path")?;
    if enclosed
      .components()
      .any(|v| v.as_os_str().eq_ignore_ascii_case(".git"))
    {
      bail!("source archive contains .git metadata");
    }
    let mode = entry.unix_mode().unwrap_or(0o100644);
    total = total
      .checked_add(entry.size())
      .context("source archive size overflow")?;
    if total > 512 * 1024 * 1024 {
      bail!("source archive exceeds 512 MiB expanded limit");
    }
    if mode & 0o170000 == 0o120000 {
      let mut bytes = Vec::new();
      entry.by_ref().take(4097).read_to_end(&mut bytes)?;
      if bytes.len() > 4096 || bytes.contains(&0) {
        bail!("invalid source symlink target");
      }
      let target = String::from_utf8(bytes).context("source symlink target is not UTF-8")?;
      links.push((enclosed, std::path::PathBuf::from(target)));
      continue;
    }
    let path = destination.join(enclosed);
    if entry.is_dir() {
      std::fs::create_dir_all(path)?;
      continue;
    }
    if let Some(parent) = path.parent() {
      std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::OpenOptions::new()
      .write(true)
      .create_new(true)
      .open(&path)?;
    let copied = std::io::copy(&mut entry.by_ref().take(512 * 1024 * 1024 + 1), &mut file)?;
    if copied != entry.size() {
      bail!("source archive entry size mismatch");
    }
    file.flush()?;
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      std::fs::set_permissions(
        path,
        std::fs::Permissions::from_mode(if mode & 0o111 != 0 { 0o755 } else { 0o644 }),
      )?;
    }
  }
  crate::links::install(destination, links)?;
  Ok(())
}
#[cfg(test)]
#[path = "source_tests.rs"]
mod tests;
