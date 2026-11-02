use super::*;
use std::{process::Command, time::Instant};

pub(super) fn run(
  root: &Path,
  args: &BlackboxArgs,
  f: &Fixture<'_>,
  work: &Path,
  reporter: &mut Reporter,
) -> Result<()> {
  let mr = f.create_merge("recovery-feature")?;
  let iid = api::id(&mr["iid"], "recovery merge IID")?;
  let mr_id = api::id(&mr["id"], "recovery merge ID")?.parse::<i64>()?;
  let project = f.project.parse::<i64>()?;
  f.approve(&iid)?;
  database::install(root, args, project)?;
  // Always remove the injected fault even if an assertion or restart fails.
  let result = exercise(root, args, f, work, &iid, mr_id, reporter);
  let cleanup = database::remove(root, args);
  result?;
  cleanup?;
  Ok(())
}

fn exercise(
  root: &Path,
  args: &BlackboxArgs,
  f: &Fixture<'_>,
  work: &Path,
  iid: &str,
  mr: i64,
  reporter: &mut Reporter,
) -> Result<()> {
  let project = f.project.parse::<i64>()?;
  let path = format!("/merge-requests/{iid}");
  let previous = f.main_sha()?;
  let failed = f.request(
    Method::POST,
    &format!("{path}/merge"),
    &f.reviewer,
    Some(json!({})),
  )?;
  api::require_status(
    failed.0,
    StatusCode::INTERNAL_SERVER_ERROR,
    "injected post-Git DB failure",
  )?;
  let merge_sha = f.main_sha()?;
  ensure!(
    merge_sha != previous,
    "fault injection occurred before the Git merge committed"
  );
  let pending = f.get(&path)?;
  ensure!(
    pending["state"] == "opened" && !pending["active_merge_operation_id"].is_null(),
    "failed DB transaction did not retain active merge intent"
  );
  let later = f.commit("main", "blackbox/after-merge.txt")?;
  ensure!(
    later != merge_sha,
    "later target commit did not advance the branch"
  );
  super::native::verify(f, work, &merge_sha, &later)?;
  restart(root, args, true)?;
  wait_http(f, &path)?;
  ensure!(
    f.get(&path)?["state"] == "opened",
    "restart ignored the installed DB fault"
  );
  database::remove(root, args)?;
  // Maintenance's configured interval is 300 seconds. A second worker restart
  // triggers its immediate first scan without changing product configuration.
  restart(root, args, false)?;
  wait_http(f, &path)?;
  let deadline = Instant::now() + Duration::from_secs(120);
  loop {
    let row = f.get(&path)?;
    if row["state"] == "merged" {
      ensure!(
        row["merge_commit_sha"] == merge_sha,
        "recovery substituted current branch tip for recorded merge SHA"
      );
      ensure!(
        row["active_merge_operation_id"].is_null(),
        "recovery did not release completed merge intent"
      );
      break;
    }
    ensure!(
      Instant::now() < deadline,
      "merge was not recovered within 120 seconds"
    );
    std::thread::sleep(Duration::from_secs(2));
  }
  ensure!(
    f.main_sha()? == later,
    "read-only recovery changed the newer target commit"
  );
  let delivery_deadline = Instant::now() + Duration::from_secs(30);
  loop {
    match database::completion_counts(root, args, project, mr) {
      Ok(()) => break,
      Err(error) if Instant::now() >= delivery_deadline => return Err(error),
      Err(_) => std::thread::sleep(Duration::from_secs(1)),
    }
  }
  restart(root, args, false)?;
  wait_http(f, &path)?;
  ensure!(
    f.get(&path)?["merge_commit_sha"] == merge_sha,
    "second restart changed the recorded result"
  );
  database::completion_counts(root, args, project, mr)?;
  reporter.pass("merge-git-success-db-failure-restart", format!("project {project} MR !{iid}: temporary DB trigger rejected completion after Git commit {merge_sha}; restart recovered that exact SHA while target remained {later}; one audit, notification and completed journal after repeated restart"));
  Ok(())
}

fn restart(root: &Path, args: &BlackboxArgs, storage: bool) -> Result<()> {
  let mut command = Command::new("docker");
  runtime::configure_compose(&mut command, root, args)?;
  command.arg("restart");
  match args.mode {
    Mode::Standalone => {
      command.arg("gity");
    }
    Mode::Distributed if storage => {
      command.args(["git-node", "git-node-2", "api", "worker"]);
    }
    Mode::Distributed => {
      command.arg("worker");
    }
  }
  process::run_status(&mut command, "restart real merge recovery components")
}

fn wait_http(f: &Fixture<'_>, path: &str) -> Result<()> {
  let deadline = Instant::now() + Duration::from_secs(180);
  loop {
    if f.get(path).is_ok() && f.main_sha().is_ok() {
      return Ok(());
    }
    ensure!(
      Instant::now() < deadline,
      "restarted merge components did not become ready"
    );
    std::thread::sleep(Duration::from_secs(2));
  }
}
