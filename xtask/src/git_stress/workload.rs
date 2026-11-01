use self::operations::{choose, run_operation, seed, timed};
use self::process::{Askpass, git, path_string, print_summary, safe_remove, sanitize};
use super::Config;
use super::api::{self, Repository};
use anyhow::{Context, Result, bail};
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[path = "operations.rs"]
mod operations;
#[path = "process.rs"]
mod process;
#[cfg(test)]
#[path = "workload_tests.rs"]
mod tests;

pub(super) type Stats = Arc<Mutex<BTreeMap<String, Vec<(Duration, bool)>>>>;

pub fn run(root: &Path, config: Config) -> Result<()> {
    let run_id = format!(
        "gitperf-{}",
        SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()
    );
    let repository = api::prepare(root, &config, &run_id)?;
    let base = PathBuf::from(&config.workdir);
    let base = if base.is_absolute() {
        base
    } else {
        root.join(base)
    };
    let work_root = base.join(&run_id);
    fs::create_dir_all(&work_root).context("create temporary Git work directory")?;
    let result = execute(&config, &work_root, &run_id, &repository);
    let resource_cleanup = api::cleanup(&repository);
    let work_cleanup = if config.keep_workdir {
        Ok(())
    } else {
        safe_remove(&base, &work_root)
    };
    match (result, resource_cleanup, work_cleanup) {
        (Ok(()), Ok(()), Ok(())) => Ok(()),
        (Err(run), Ok(()), Ok(())) => Err(run),
        (Ok(()), Err(resources), Ok(())) => Err(resources),
        (Ok(()), Ok(()), Err(work)) => Err(work).context("clean temporary Git work directory"),
        (run, resources, work) => bail!(
            "Git stress failed: run={}; API cleanup={}; workdir cleanup={}",
            display_result(run),
            display_result(resources),
            display_result(work)
        ),
    }
}

fn display_result(result: Result<()>) -> String {
    result
        .err()
        .map(|error| format!("{error:#}"))
        .unwrap_or_else(|| "ok".to_owned())
}

fn execute(config: &Config, work_root: &Path, run_id: &str, repo: &Repository) -> Result<()> {
    let askpass = Askpass::create(work_root)?;
    let stats: Stats = Arc::new(Mutex::new(BTreeMap::new()));
    let errors = Arc::new(Mutex::new(Vec::new()));
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
        "workload mode={:?} workers={} iterations={}",
        config.mode, config.workers, config.iterations
    );
    let next = AtomicUsize::new(0);
    thread::scope(|scope| {
        for (worker, dir) in worker_dirs.iter().enumerate() {
            let stats = Arc::clone(&stats);
            let errors = Arc::clone(&errors);
            let askpass = &askpass;
            let next = &next;
            scope.spawn(move || {
                loop {
                    let iteration = next.fetch_add(1, Ordering::Relaxed);
                    if iteration >= config.iterations {
                        break;
                    }
                    let operation = choose(config, iteration);
                    if let Err(error) = run_operation(
                        config, work_root, run_id, repo, askpass, &stats, operation, worker,
                        iteration, dir,
                    ) {
                        let safe = sanitize(&error.to_string(), &repo.token);
                        eprintln!(
                            "[{operation}] worker={worker} iteration={iteration} failed: {safe}"
                        );
                        errors
                            .lock()
                            .unwrap()
                            .push((operation, worker, iteration, safe));
                    }
                }
            });
        }
    });
    print_summary(&stats, errors.lock().unwrap().len());
    if !errors.lock().unwrap().is_empty() {
        bail!(
            "{} Git workload operation(s) failed",
            errors.lock().unwrap().len()
        );
    }
    Ok(())
}
