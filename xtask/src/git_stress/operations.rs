use super::Stats;
use super::api::Repository;
use super::process::{self, Askpass};
use crate::git_stress::{Config, Mode};
use anyhow::{Result, bail};
use std::{fs, path::Path, time::Instant};

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
  process::git(
    config,
    askpass,
    &repo.token,
    &["config", "user.name", "Gity Perf"],
    &dir,
  )?;
  process::git(
    config,
    askpass,
    &repo.token,
    &["config", "user.email", "perf@gity.local"],
    &dir,
  )?;
  fs::write(
    dir.join("README.md"),
    format!("# {run_id}\n\nGit client stress seed repository.\n"),
  )?;
  for index in 0..config.seed_files {
    fs::write(
      dir.join("src").join(format!("file-{index}.txt")),
      format!("seed {run_id} {index}\n"),
    )?;
  }
  process::git(config, askpass, &repo.token, &["add", "."], &dir)?;
  timed(stats, "setup.commit", || {
    process::git(
      config,
      askpass,
      &repo.token,
      &["commit", "-m", "Seed git client stress repository"],
      &dir,
    )
  })?;
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

#[allow(clippy::too_many_arguments)]
pub(super) fn run_operation(
  config: &Config,
  root: &Path,
  run_id: &str,
  repo: &Repository,
  askpass: &Askpass,
  stats: &Stats,
  op: &'static str,
  worker: usize,
  iteration: usize,
  cwd: &Path,
) -> Result<()> {
  match op {
    "ls-remote" => timed(stats, op, || {
      process::git(
        config,
        askpass,
        &repo.token,
        &["ls-remote", "--heads", &repo.url],
        root,
      )
    }),
    "clone" => {
      let dir = root.join(format!("clone-{worker}-{iteration}"));
      let target = process::path_string(&dir)?;
      let result = timed(stats, op, || {
        process::git(
          config,
          askpass,
          &repo.token,
          &["clone", "--quiet", "--depth", "1", &repo.url, &target],
          root,
        )
      });
      if !config.keep_workdir {
        let _ = process::safe_remove(root, &dir);
      }
      result
    }
    "fetch" => timed(stats, op, || {
      process::git(
        config,
        askpass,
        &repo.token,
        &["fetch", "--prune", "origin"],
        cwd,
      )
    }),
    "push" => push(
      config,
      run_id,
      repo,
      askpass,
      stats,
      (worker, iteration),
      cwd,
    ),
    _ => bail!("unsupported Git operation"),
  }
}

fn push(
  config: &Config,
  run_id: &str,
  repo: &Repository,
  askpass: &Askpass,
  stats: &Stats,
  position: (usize, usize),
  cwd: &Path,
) -> Result<()> {
  let (worker, iteration) = position;
  let branch = format!("perf/{run_id}/{worker}-{iteration}");
  process::git(
    config,
    askpass,
    &repo.token,
    &["checkout", "--quiet", "-B", &branch, "origin/main"],
    cwd,
  )?;
  let dir = cwd.join("perf-output").join(worker.to_string());
  fs::create_dir_all(&dir)?;
  fs::write(
    dir.join(format!("{iteration}.txt")),
    format!("worker={worker}\niteration={iteration}\nrun={run_id}\n"),
  )?;
  process::git(config, askpass, &repo.token, &["add", "."], cwd)?;
  process::git(
    config,
    askpass,
    &repo.token,
    &["commit", "-m", &format!("perf push {worker}-{iteration}")],
    cwd,
  )?;
  timed(stats, "push", || {
    process::git(
      config,
      askpass,
      &repo.token,
      &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
      cwd,
    )
  })
}

pub(super) fn choose(config: &Config, iteration: usize) -> &'static str {
  match config.mode {
    Mode::Clone => return "clone",
    Mode::Fetch => return "fetch",
    Mode::Push => return "push",
    Mode::LsRemote => return "ls-remote",
    Mode::Mixed => {}
  }
  let total: u32 = config.weights.iter().sum();
  let mut cursor = ((iteration as u64 * 9973) % total as u64) as u32;
  for (name, weight) in [
    ("clone", config.weights[0]),
    ("fetch", config.weights[1]),
    ("push", config.weights[2]),
    ("ls-remote", config.weights[3]),
  ] {
    if cursor < weight {
      return name;
    }
    cursor -= weight;
  }
  "fetch"
}

pub(super) fn timed<T>(stats: &Stats, name: &str, action: impl FnOnce() -> Result<T>) -> Result<T> {
  let started = Instant::now();
  let result = action();
  stats
    .lock()
    .unwrap()
    .entry(name.to_owned())
    .or_default()
    .push((started.elapsed(), result.is_ok()));
  result
}
