use super::workload::{
  Stats,
  operations::timed,
  process::{self, Askpass},
};
use super::{Config, api::Repository};
use anyhow::Result;
use std::{
  fs::{self, File},
  io::{Read, Write},
  path::Path,
};

pub(super) fn seed(
  config: &Config,
  root: &Path,
  run_id: &str,
  repo: &Repository,
  askpass: &Askpass,
  stats: &Stats,
) -> Result<()> {
  let dir = root.join("seed");
  fs::create_dir_all(dir.join("src"))?;
  timed(stats, "setup.init", || {
    process::git(config, askpass, &repo.token, &["init", "-b", "main"], &dir)
  })?;
  for args in [
    ["config", "user.name", "Gity Perf"],
    ["config", "user.email", "perf@gity.local"],
  ] {
    process::git(config, askpass, &repo.token, &args, &dir)?;
  }
  fs::write(
    dir.join("README.md"),
    format!("# {run_id}\n\nGit client stress seed repository.\n"),
  )?;
  for commit in 0..config.seed_commits {
    timed(stats, "setup.seed", || {
      for index in 0..config.seed_files {
        let path = dir.join("src").join(format!("file-{index}.bin"));
        if let Some(bytes) = config.bytes_per_file {
          random_file(&path, bytes)?;
        } else {
          fs::write(path, format!("seed {run_id} {index} commit {commit}\n"))?;
        }
      }
      Ok(())
    })?;
    process::git(config, askpass, &repo.token, &["add", "."], &dir)?;
    timed(stats, "setup.commit", || {
      process::git(
        config,
        askpass,
        &repo.token,
        &[
          "commit",
          "--allow-empty",
          "-m",
          &format!("Seed git client stress {commit}"),
        ],
        &dir,
      )
    })?;
  }
  process::git(
    config,
    askpass,
    &repo.token,
    &["remote", "add", "origin", &repo.url],
    &dir,
  )?;
  timed(stats, "setup.push", || {
    process::git(
      config,
      askpass,
      &repo.token,
      &["push", "origin", "HEAD:refs/heads/main"],
      &dir,
    )
  })
}

/// OS entropy provides data that Git cannot reduce to repeated filler text.
/// Bounded I/O avoids retaining an entire multi-MiB seed file in memory.
pub(super) fn random_file(path: &Path, bytes: u64) -> Result<()> {
  let mut random = File::open("/dev/urandom")?;
  let mut output = File::create(path)?;
  let mut remaining = bytes;
  let mut buffer = [0_u8; 128 * 1024];
  while remaining > 0 {
    let count = remaining.min(buffer.len() as u64) as usize;
    random.read_exact(&mut buffer[..count])?;
    output.write_all(&buffer[..count])?;
    remaining -= count as u64;
  }
  Ok(())
}

#[cfg(test)]
#[path = "seed_tests.rs"]
mod tests;
