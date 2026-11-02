use super::*;
use std::process::Command;

pub(super) fn verify(
  f: &Fixture<'_>,
  work: &Path,
  merge_sha: &str,
  target_sha: &str,
) -> Result<()> {
  let checkout = work.join("merge-recovery-clone");
  let url = format!("{}/{}.git", f.config.http_url, f.project_path);
  let mut clone = command(work, f.owner);
  clone.arg("clone").arg(&url).arg(&checkout);
  process::run_output(&mut clone, "native HTTP clone after failed DB completion")?;
  let mut head = command(&checkout, f.owner);
  head.args(["rev-parse", "HEAD"]);
  let actual =
    String::from_utf8(process::run_output(&mut head, "read actual target commit")?.stdout)?;
  ensure!(
    actual.trim() == target_sha,
    "native clone target differs from API branch"
  );
  let mut object = command(&checkout, f.owner);
  object.args([
    "show",
    &format!("{merge_sha}:blackbox/recovery-feature.txt"),
  ]);
  let actual = process::run_output(&mut object, "read exact merged object with native Git")?.stdout;
  ensure!(
    actual == b"blackbox/recovery-feature.txt",
    "recorded merge commit lacks source content"
  );
  let mut refs = command(&checkout, f.owner);
  refs.args(["ls-remote", "origin"]);
  let refs = String::from_utf8(
    process::run_output(&mut refs, "inspect client-visible merge references")?.stdout,
  )?;
  ensure!(
    !refs.contains("refs/gity/"),
    "private merge receipts leaked into Git advertisement"
  );
  Ok(())
}

fn command(cwd: &Path, token: &str) -> Command {
  let mut command = Command::new("git");
  command
    .current_dir(cwd)
    .env("GIT_TERMINAL_PROMPT", "0")
    .env("GIT_CONFIG_NOSYSTEM", "1")
    .env("GIT_CONFIG_GLOBAL", "/dev/null")
    .env("GIT_CONFIG_COUNT", "2")
    .env("GIT_CONFIG_KEY_0", "http.extraHeader")
    .env(
      "GIT_CONFIG_VALUE_0",
      format!("Authorization: Bearer {token}"),
    )
    .env("GIT_CONFIG_KEY_1", "credential.helper")
    .env("GIT_CONFIG_VALUE_1", "");
  command
}
