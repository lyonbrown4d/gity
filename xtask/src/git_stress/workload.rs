use self::operations::{choose, run_operation, timed};
use self::process::{Askpass, git, path_string, safe_remove, sanitize};
use super::Config;
use super::api::{self, Repository};
use super::{seed::seed, summary::Summary};
use anyhow::{Context, Result, bail};
use std::{
  collections::BTreeMap,
  fs,
  path::{Path, PathBuf},
  sync::{
    Arc, Barrier, Mutex, OnceLock,
    atomic::{AtomicUsize, Ordering},
  },
  thread,
  time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

#[path = "operations.rs"]
pub(super) mod operations;
#[path = "process.rs"]
pub(super) mod process;
#[cfg(test)]
#[path = "workload_tests.rs"]
pub(super) mod tests;

pub(super) type Stats = Arc<Mutex<BTreeMap<String, Vec<(Duration, bool)>>>>;

pub fn run(root: &Path, config: Config) -> Result<()> {
  let setup_started = Instant::now();
  let run_id = format!("gitperf-{}", uuid::Uuid::now_v7().simple());
  let base = PathBuf::from(&config.workdir);
  let base = if base.is_absolute() {
    base
  } else {
    root.join(base)
  };
  let work_root = base.join(&run_id);
  fs::create_dir_all(&work_root).context("create temporary Git work directory")?;
  let repository = match api::prepare(root, &config, &run_id) {
    Ok(repository) => repository,
    Err(error) => {
      if !config.keep_workdir {
        safe_remove(&base, &work_root)?;
      }
      return Err(error);
    }
  };
  let prepared = setup_started.elapsed();
  let (summary, result) = match execute(
    &config,
    &work_root,
    &run_id,
    &repository,
    setup_started,
    prepared,
  ) {
    Ok((summary, 0)) => (Some(summary), Ok(())),
    Ok((summary, failures)) => (
      Some(summary),
      Err(anyhow::anyhow!(
        "{failures} Git workload operation(s) failed"
      )),
    ),
    Err(error) => (None, Err(error)),
  };
  if let Some(summary) = &summary {
    summary.print();
  }
  let resource_cleanup = api::cleanup(&repository);
  let work_cleanup = if config.keep_workdir {
    Ok(())
  } else {
    safe_remove(&base, &work_root)
  };
  // Write after cleanup so an explicitly requested summary survives even when
  // its parent was inside the generated temporary work directory.
  let summary_result = match (summary, config.summary.as_deref()) {
    (Some(summary), Some(path)) => summary.write(root, path),
    _ => Ok(()),
  };
  let failures = [
    ("run", result),
    ("API cleanup", resource_cleanup),
    ("workdir cleanup", work_cleanup),
    ("summary", summary_result),
  ]
  .into_iter()
  .filter_map(|(name, result)| result.err().map(|e| format!("{name}: {e:#}")))
  .collect::<Vec<_>>();
  if !failures.is_empty() {
    bail!("Git stress failed: {}", failures.join("; "));
  }
  Ok(())
}

fn execute(
  config: &Config,
  work_root: &Path,
  run_id: &str,
  repo: &Repository,
  setup_started: Instant,
  prepared: Duration,
) -> Result<(Summary, usize)> {
  let askpass = Askpass::create(work_root, config.transport)?;
  let stats: Stats = Arc::new(Mutex::new(BTreeMap::new()));
  stats
    .lock()
    .unwrap()
    .insert("setup.api".into(), vec![(prepared, true)]);
  if config.project_url.is_empty() {
    seed(config, work_root, run_id, repo, &askpass, &stats)?;
  }
  let mut worker_dirs = Vec::new();
  for worker in 0..config.workers {
    let dir = work_root.join(format!("worker-{worker}"));
    let target = path_string(&dir)?;
    timed(&stats, "setup.clone", || {
      git(
        config,
        &askpass,
        &repo.token,
        &[
          "clone",
          "--quiet",
          "--branch",
          "main",
          repo.url.as_str(),
          &target,
        ],
        work_root,
      )
    })?;
    git(
      config,
      &askpass,
      &repo.token,
      &["config", "user.name", &format!("Gity Perf {worker}")],
      &dir,
    )?;
    git(
      config,
      &askpass,
      &repo.token,
      &["config", "user.email", &format!("perf-{worker}@gity.local")],
      &dir,
    )?;
    worker_dirs.push(dir);
  }
  println!("Git client stress target: {}", repo.url);
  println!(
    "workload mode={:?} transport={:?} workers={} iterations={} duration_seconds={:?} clone_depth={}",
    config.mode,
    config.transport,
    config.workers,
    config.iterations,
    config.duration_seconds,
    config.clone_depth
  );
  let next = AtomicUsize::new(0);
  let errors = AtomicUsize::new(0);
  let gate = Barrier::new(config.workers + 1);
  let started = OnceLock::<Instant>::new();
  let started_unix = OnceLock::<f64>::new();
  let setup_elapsed = setup_started.elapsed();
  thread::scope(|scope| {
    for (worker, dir) in worker_dirs.iter().enumerate() {
      let stats = Arc::clone(&stats);
      let (askpass, next, errors, gate, started) = (&askpass, &next, &errors, &gate, &started);
      scope.spawn(move || {
        gate.wait();
        loop {
          if config
            .duration_seconds
            .is_some_and(|seconds| started.get().unwrap().elapsed() >= Duration::from_secs(seconds))
          {
            break;
          }
          let iteration = next.fetch_add(1, Ordering::Relaxed);
          if config.duration_seconds.is_none() && iteration >= config.iterations {
            break;
          }
          let operation = choose(config, iteration);
          if let Err(error) = timed(&stats, operation, || {
            run_operation(
              config, work_root, run_id, repo, askpass, operation, worker, iteration, dir,
            )
          }) {
            let safe = sanitize(&error.to_string(), &repo.token);
            eprintln!("[{operation}] worker={worker} iteration={iteration} failed: {safe}");
            errors.fetch_add(1, Ordering::Relaxed);
          }
        }
      });
    }
    started_unix.set(unix_seconds()).unwrap();
    started.set(Instant::now()).unwrap();
    gate.wait();
  });
  let finished_unix = unix_seconds();
  let elapsed = started.get().unwrap().elapsed();
  Ok((
    Summary::new(
      config,
      run_id,
      &stats,
      setup_elapsed,
      elapsed,
      (*started_unix.get().unwrap(), finished_unix),
    ),
    errors.load(Ordering::Relaxed),
  ))
}

/// UTC wall-clock anchors are independent of monotonic latency/throughput time.
fn unix_seconds() -> f64 {
  match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(duration) => duration.as_secs_f64(),
    Err(before_epoch) => -before_epoch.duration().as_secs_f64(),
  }
}
