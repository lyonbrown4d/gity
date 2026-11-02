use super::Stats;
use super::api::Repository;
use super::process::{self, Askpass};
use crate::git_stress::{Config, Mode};
use anyhow::{Result, bail};
use std::{fs, path::Path, time::Instant};

#[allow(clippy::too_many_arguments)]
pub(super) fn run_operation(
  config: &Config,
  root: &Path,
  run_id: &str,
  repo: &Repository,
  askpass: &Askpass,
  op: &'static str,
  worker: usize,
  iteration: usize,
  cwd: &Path,
) -> Result<()> {
  match op {
    "ls-remote" => process::git(
      config,
      askpass,
      &repo.token,
      &["ls-remote", "--heads", &repo.url],
      root,
    ),
    "clone" => {
      let dir = root.join(format!("clone-{worker}-{iteration}"));
      let target = process::path_string(&dir)?;
      let depth = config.clone_depth.to_string();
      let mut args = vec!["clone", "--quiet"];
      if config.clone_depth > 0 {
        args.extend(["--depth", depth.as_str()]);
      }
      args.extend([repo.url.as_str(), target.as_str()]);
      let result = process::git(config, askpass, &repo.token, &args, root);
      if !config.keep_workdir {
        let _ = process::safe_remove(root, &dir);
      }
      result
    }
    "fetch" => process::git(
      config,
      askpass,
      &repo.token,
      &["fetch", "--prune", "origin"],
      cwd,
    ),
    "push" => push(config, run_id, repo, askpass, (worker, iteration), cwd),
    _ => bail!("unsupported Git operation"),
  }
}

fn push(
  config: &Config,
  run_id: &str,
  repo: &Repository,
  askpass: &Askpass,
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
  process::git(
    config,
    askpass,
    &repo.token,
    &["push", "origin", &format!("HEAD:refs/heads/{branch}")],
    cwd,
  )
}

pub(super) fn choose(config: &Config, iteration: usize) -> &'static str {
  match config.mode {
    Mode::Clone => return "clone",
    Mode::Fetch => return "fetch",
    Mode::Push => return "push",
    Mode::LsRemote => return "ls-remote",
    Mode::Mixed => {}
  }
  let total: u64 = config.weights.iter().map(|&weight| u64::from(weight)).sum();
  let mut cursor = (iteration as u64).wrapping_mul(9973) % total;
  for (name, weight) in [
    ("clone", config.weights[0]),
    ("fetch", config.weights[1]),
    ("push", config.weights[2]),
    ("ls-remote", config.weights[3]),
  ] {
    if cursor < u64::from(weight) {
      return name;
    }
    cursor -= u64::from(weight);
  }
  "fetch"
}

pub(in crate::git_stress) fn timed<T>(
  stats: &Stats,
  name: &str,
  action: impl FnOnce() -> Result<T>,
) -> Result<T> {
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
