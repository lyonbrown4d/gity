#[path = "git_ssh/cancellation.rs"]
mod cancellation;
#[path = "git_ssh/support.rs"]
mod support;
use serde_json::{Value, json};
use support::Fixture;

#[tokio::test]
async fn ssh_clone_push_pull_tags_and_branch_protection_use_real_native_clients() {
  let f = Fixture::new().await;
  let (key, _) = f.key("writer", true).await;
  let root = f.root.path();
  f.ok(root, &key, &["clone", &f.url, "a"]).await;
  f.ok(root, &key, &["clone", &f.url, "b"]).await;
  let a = root.join("a");
  let b = root.join("b");
  for dir in [&a, &b] {
    f.ok(dir, &key, &["config", "user.name", "SSH Owner"]).await;
    f.ok(dir, &key, &["config", "user.email", "ssh@example.test"])
      .await;
  }
  let unmanaged = root.join("unmanaged-hooks");
  std::fs::create_dir_all(&unmanaged).unwrap();
  let bare = root.join("repos/20.git");
  f.ok(
    root,
    &key,
    &[
      "--git-dir",
      bare.to_str().unwrap(),
      "config",
      "core.hooksPath",
      unmanaged.to_str().unwrap(),
    ],
  )
  .await;
  let base = f.ok(&a, &key, &["rev-parse", "HEAD"]).await;
  std::fs::write(a.join("next.txt"), "pushed over SSH\n").unwrap();
  f.ok(&a, &key, &["add", "."]).await;
  f.ok(&a, &key, &["commit", "-m", "SSH push"]).await;
  let pushed = f.ok(&a, &key, &["rev-parse", "HEAD"]).await;
  f.ok(&a, &key, &["push", "origin", "main"]).await;
  f.ok(&b, &key, &["pull", "--ff-only", "origin", "main"])
    .await;
  assert_eq!(
    std::fs::read_to_string(b.join("next.txt")).unwrap(),
    "pushed over SSH\n"
  );
  assert_eq!(f.ok(&b, &key, &["rev-parse", "HEAD"]).await, pushed);
  assert_eq!(
    f.state
      .git
      .resolve_revision("20.git", "main", "")
      .await
      .unwrap(),
    pushed
  );
  f.ok(&a, &key, &["push", "origin", "HEAD:refs/heads/feature/ssh"])
    .await;
  f.ok(&a, &key, &["tag", "v1.0.0"]).await;
  f.ok(&a, &key, &["push", "origin", "v1.0.0"]).await;
  assert!(
    f.ok(&b, &key, &["fetch", "origin", "--tags"])
      .await
      .is_empty()
  );
  assert_eq!(f.ok(&b, &key, &["rev-parse", "v1.0.0"]).await, pushed);
  assert!(
    !f.git(&a, &key, &["push", "origin", ":main"])
      .await
      .status
      .success()
  );
  let rule = json!({"push_access_level":"developer","merge_access_level":"maintainer","require_merge_request":false,"allow_force_push":false,"allow_delete":false});
  assert_eq!(
    f.api(
      "POST",
      "/projects/20/repository/branches/%5B/protect",
      rule.clone()
    )
    .await
    .0,
    400
  );
  assert_eq!(
    f.api(
      "POST",
      "/projects/20/repository/branches/main/protect",
      rule
    )
    .await
    .0,
    200
  );
  // A protected fast-forward must accept brand-new quarantine objects.
  std::fs::write(a.join("protected.txt"), "protected fast-forward\n").unwrap();
  f.ok(&a, &key, &["add", "."]).await;
  f.ok(&a, &key, &["commit", "-m", "protected SSH fast-forward"])
    .await;
  f.ok(&a, &key, &["push", "origin", "main"]).await;
  let pushed = f.ok(&a, &key, &["rev-parse", "HEAD"]).await;
  f.ok(&b, &key, &["pull", "--ff-only", "origin", "main"])
    .await;
  assert_eq!(
    std::fs::read_to_string(b.join("protected.txt")).unwrap(),
    "protected fast-forward\n"
  );
  f.ok(&a, &key, &["reset", "--hard", &base]).await;
  let forced = f
    .git(&a, &key, &["push", "--force", "origin", "main"])
    .await;
  assert!(!forced.status.success(), "force push unexpectedly accepted");
  assert_eq!(
    f.state
      .git
      .resolve_revision("20.git", "main", "")
      .await
      .unwrap(),
    pushed
  );
  assert_eq!(
    f.api(
      "POST",
      "/projects/20/repository/branches/release%2F*/protect",
      json!({"push_access_level":"maintainer","require_merge_request":false})
    )
    .await
    .0,
    200
  );
  // Key creator is a super administrator; the key itself remains developer-scoped.
  assert!(
    !f.git(&a, &key, &["push", "origin", "HEAD:refs/heads/release/new"])
      .await
      .status
      .success()
  );
  assert!(
    !f.git(&a, &key, &["push", "origin", "HEAD:refs/notes/private"])
      .await
      .status
      .success()
  );
  f.stop().await;
}

#[tokio::test]
async fn ssh_readonly_unknown_revoked_and_cross_project_keys_cannot_escalate() {
  let f = Fixture::new().await;
  let (read, read_id) = f.key("reader", false).await;
  let (write, _) = f.key("writer", true).await;
  let public = russh::keys::load_secret_key(&read, None)
    .unwrap()
    .public_key()
    .to_openssh()
    .unwrap();
  assert_eq!(f.api("POST","/projects/20/deploy-keys",json!({"title":"duplicate","public_key":format!("{public} different-comment"),"can_push":true})).await.0,409);
  assert_eq!(
    f.api(
      "POST",
      "/projects/20/deploy-keys",
      json!({"title":"invalid","public_key":"ssh-ed25519 YWJj","can_push":true})
    )
    .await
    .0,
    400
  );

  let root = f.root.path();
  f.ok(root, &read, &["clone", &f.url, "readonly"]).await;
  let a = root.join("readonly");
  assert!(
    !f.git(&a, &read, &["push", "origin", "HEAD:refs/heads/no-write"])
      .await
      .status
      .success()
  );
  let other = f.url.replace("demo.git", "other.git");
  assert!(
    !f.git(root, &write, &["ls-remote", &other])
      .await
      .status
      .success()
  );
  assert_eq!(
    f.api(
      "DELETE",
      &format!("/projects/20/deploy-keys/{read_id}"),
      Value::Null
    )
    .await
    .0,
    200
  );
  assert!(
    !f.git(root, &read, &["ls-remote", &f.url])
      .await
      .status
      .success()
  );
  assert!(
    !f.ssh(&write, &["echo arbitrary-command"])
      .await
      .status
      .success()
  );
  assert!(!f.ssh(&write, &[]).await.status.success());
  assert!(!f.ssh(&write, &["-s", "sftp"]).await.status.success());
  // Disabled key creators revoke their key access at the next SSH authorization.
  f.state
    .db
    .execute("UPDATE users SET status='disabled' WHERE id=1", vec![])
    .await
    .unwrap();
  assert!(
    !f.git(root, &write, &["ls-remote", &f.url])
      .await
      .status
      .success()
  );
  f.stop().await;
}
