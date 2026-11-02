use super::*;
use crate::git_stress::Mode;

fn config() -> Config {
  crate::git_stress::workload::tests::config(Mode::Mixed)
}

#[test]
fn ssh_repository_url_keeps_git_identity_and_uses_default_or_explicit_port() {
  let mut config = config();
  config.transport = Transport::Ssh;
  assert_eq!(
    repository_url("ssh://git@example.test/team/repo.git", &config).unwrap(),
    "ssh://git@example.test:2222/team/repo.git"
  );
  assert_eq!(
    repository_url("ssh://git@example.test:2229/team/repo.git", &config).unwrap(),
    "ssh://git@example.test:2229/team/repo.git"
  );
  assert!(repository_url("http://example.test/repo.git", &config).is_err());
  assert!(repository_url("ssh://wrong@example.test/repo.git", &config).is_err());
  assert!(repository_url("ssh://git:secret@example.test/repo.git", &config).is_err());
}
#[test]
fn known_hosts_quoting_rejects_line_injection_and_preserves_shell_special_characters_as_data() {
  assert!(known_hosts_option(Path::new("key\nProxyCommand=bad")).is_err());
  let option = known_hosts_option(Path::new("/tmp/host's $file;`touch bad` % \\\" name")).unwrap();
  assert!(option.starts_with("UserKnownHostsFile=\""));
  assert!(option.ends_with('"'));
  assert!(option.contains("%%"));
  assert!(SSH_WRAPPER.contains("GlobalKnownHostsFile=/dev/null"));
  assert!(SSH_WRAPPER.contains("StrictHostKeyChecking=yes"));
  assert!(!SSH_WRAPPER.contains("eval"));
}
#[test]
fn automatic_ssh_url_uses_configured_host_port_and_project_path() {
  let mut config = config();
  config.base_url = "http://127.0.0.1:18080".into();
  config.transport = Transport::Ssh;
  assert_eq!(
    project_url(&config, "team/repo", "http://unused/repo.git").unwrap(),
    "ssh://git@127.0.0.1:2222/team/repo.git"
  );
  config.ssh_host = Some("ssh.example.test".into());
  assert!(
    project_url(&config, "team/repo", "")
      .unwrap()
      .contains("ssh.example.test:2222")
  );
}

#[test]
fn openssh_accepts_quoted_known_hosts_paths_without_interpreting_shell_metacharacters() {
  let option =
    known_hosts_option(Path::new("/tmp/host's $file;`touch marker` % \\\" name")).unwrap();
  let output = Command::new("ssh")
    .args(["-G", "-F", "/dev/null", "-o", &option, "git@localhost"])
    .output()
    .unwrap();
  assert!(
    output.status.success(),
    "{}",
    String::from_utf8_lossy(&output.stderr)
  );
}

#[test]
fn existing_ssh_repository_needs_neither_http_token_nor_api_connection() {
  let mut config = config();
  config.transport = Transport::Ssh;
  config.base_url = "http://127.0.0.1:1".into();
  config.project_url = "ssh://git@example.test/team/project.git".into();
  config.token_env = String::new();
  let repository = crate::git_stress::api::prepare(Path::new("."), &config, "unused").unwrap();
  assert!(repository.token.is_empty());
  assert!(repository.cleanup.is_none());
  assert_eq!(
    repository.url,
    "ssh://git@example.test:2222/team/project.git"
  );
}
