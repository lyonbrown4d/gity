use gity_git_node_app::configuration::{CliOverrides, load_settings};
use std::sync::{Mutex, OnceLock};

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[test]
fn environment_and_cli_override_file_without_default_cli_values_winning() {
  let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap();
  let directory = tempfile::tempdir().unwrap();
  let path = directory.path().join("node.yaml");
  std::fs::write(
    &path,
    "id: file-node\naddress: 127.0.0.1:9200\nrepo_root: file/repos\ntoken: abcdefghijklmnopqrstuvwxyz123456\nread_only: false\n",
  )
  .unwrap();
  unsafe { std::env::set_var("GITY_GIT_NODE_ID", "environment-node") };
  let settings = load_settings(Some(path.clone()), CliOverrides::default()).unwrap();
  assert_eq!(settings.id, "environment-node");
  assert_eq!(settings.address.port(), 9200);
  assert_eq!(settings.repo_root, std::path::Path::new("file/repos"));

  let cli = CliOverrides {
    id: Some("explicit-node".into()),
    ..CliOverrides::default()
  };
  let settings = load_settings(Some(path), cli).unwrap();
  assert_eq!(settings.id, "explicit-node");
  unsafe { std::env::remove_var("GITY_GIT_NODE_ID") };
}
