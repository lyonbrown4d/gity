use super::{read_bounded, safe_remove};
use std::{io::Cursor, path::Path};

#[test]
fn diagnostic_reader_caps_retained_output_but_drains_input() {
  let input = vec![b'x'; 128 * 1024];
  let output = read_bounded(Cursor::new(input), 1024);
  assert!(output.starts_with(&"x".repeat(1024)));
  assert!(output.ends_with("[diagnostic output truncated]"));
  assert!(output.len() < 1100);
}

#[test]
fn cleanup_refuses_to_remove_its_root() {
  let root = std::env::temp_dir();
  assert!(safe_remove(&root, Path::new(&root)).is_err());
}

#[cfg(unix)]
#[test]
fn askpass_delivers_credentials_as_literal_data_and_empty_token_redaction_is_a_noop() {
  let dir = tempfile::tempdir().unwrap();
  let helper = super::Askpass::create(dir.path(), crate::git_stress::Transport::Http).unwrap();
  let token = "literal ' \" $token ; `not a command`";
  let output = std::process::Command::new(&helper.path)
    .arg("Password")
    .env("GITY_GIT_PERF_TOKEN", token)
    .output()
    .unwrap();
  assert!(output.status.success());
  assert_eq!(
    String::from_utf8(output.stdout).unwrap(),
    format!("{token}\n")
  );
  assert_eq!(super::sanitize("SSH denied", ""), "SSH denied");
}

#[cfg(unix)]
#[test]
fn git_timeout_kills_the_shell_descendant_and_reaps_bounded_diagnostic_readers() {
  let dir = tempfile::tempdir().unwrap();
  let helper = super::Askpass::create(dir.path(), crate::git_stress::Transport::Http).unwrap();
  let mut config = crate::git_stress::workload::tests::config(crate::git_stress::Mode::LsRemote);
  config.timeout_ms = 50;
  let started = std::time::Instant::now();
  let result = super::git(
    &config,
    &helper,
    "",
    &["-c", "alias.perf-timeout=!sleep 30", "perf-timeout"],
    dir.path(),
  );
  assert!(result.unwrap_err().to_string().contains("timeout"));
  assert!(started.elapsed() < std::time::Duration::from_secs(3));
}

#[cfg(unix)]
#[test]
fn fixed_ssh_wrapper_overrides_repository_configuration_and_quotes_its_own_path() {
  use std::os::unix::fs::PermissionsExt;
  let dir = tempfile::tempdir().unwrap();
  let origin = dir.path().join("origin.git");
  for (path, args) in [
    (dir.path(), vec!["init", "--quiet"]),
    (origin.as_path(), vec!["init", "--bare", "--quiet"]),
  ] {
    std::fs::create_dir_all(path).unwrap();
    assert!(
      std::process::Command::new("git")
        .args(args)
        .arg(path)
        .output()
        .unwrap()
        .status
        .success()
    );
  }
  assert!(
    std::process::Command::new("git")
      .current_dir(dir.path())
      .args(["config", "core.sshCommand", "false"])
      .output()
      .unwrap()
      .status
      .success()
  );
  let path = dir.path().join("wrapper '; touch marker; #");
  let origin_arg = shlex::try_quote(origin.to_str().unwrap()).unwrap();
  std::fs::write(
    &path,
    format!("#!/bin/sh\nexec git upload-pack {origin_arg}\n"),
  )
  .unwrap();
  std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
  let helper = super::Askpass { path };
  let hosts = dir.path().join("hosts");
  std::fs::write(&hosts, "").unwrap();
  let mut config = crate::git_stress::workload::tests::config(crate::git_stress::Mode::LsRemote);
  config.transport = crate::git_stress::Transport::Ssh;
  config.ssh_key = Some(hosts.clone());
  config.ssh_known_hosts = Some(hosts);
  config.timeout_ms = 10000;
  super::git(
    &config,
    &helper,
    "",
    &["ls-remote", "ssh://git@host.invalid/repo.git"],
    dir.path(),
  )
  .unwrap();
  assert!(!dir.path().join("marker").exists());
}
