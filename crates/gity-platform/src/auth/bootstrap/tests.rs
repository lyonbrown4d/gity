use super::*;
use std::{
  path::PathBuf,
  sync::Arc,
  time::{Duration, SystemTime},
};

fn configure(state: &mut AppState, path: PathBuf) {
  let settings = Arc::make_mut(&mut state.settings);
  settings.bootstrap.initial_password_file = None;
  settings.bootstrap.generated_password_file = Some(path);
}

#[cfg(unix)]
#[tokio::test]
async fn concurrent_instances_with_distinct_secret_paths_have_one_winner_file() {
  use std::os::unix::fs::PermissionsExt;
  let (dir, mut first) = crate::features::test_support::fixture().await;
  let db_path = dir.path().join("bootstrap.sqlite");
  let url = format!("sqlite://{}?mode=rwc", db_path.display());
  let first_db = gity_core::Db::connect(&url).await.unwrap();
  first_db.migrate().await.unwrap();
  let second_db = gity_core::Db::connect(&url).await.unwrap();
  first.db = first_db.clone();
  first.identity =
    gity_identity::IdentityModule::new(fluxdi::Shared::new(first_db.connection.clone()))
      .repository()
      .unwrap();
  let second_dir = tempfile::tempdir().unwrap();
  let mut second = first.clone();
  second.db = second_db.clone();
  second.identity =
    gity_identity::IdentityModule::new(fluxdi::Shared::new(second_db.connection.clone()))
      .repository()
      .unwrap();
  configure(&mut first, dir.path().join("first/admin-password"));
  configure(&mut second, second_dir.path().join("second/admin-password"));
  let (first_result, second_result) = tokio::join!(initialize(&first), initialize(&second));
  let first_won = first_result.unwrap();
  let second_won = second_result.unwrap();
  assert_ne!(first_won, second_won);
  let first_path = first
    .settings
    .bootstrap
    .generated_password_file
    .as_ref()
    .unwrap();
  let second_path = second
    .settings
    .bootstrap
    .generated_password_file
    .as_ref()
    .unwrap();
  assert_eq!(first_path.exists(), first_won);
  assert_eq!(second_path.exists(), second_won);
  let winning_path = if first_won { first_path } else { second_path };
  assert_eq!(
    std::fs::metadata(winning_path)
      .unwrap()
      .permissions()
      .mode()
      & 0o777,
    0o600
  );
  let parent = winning_path.parent().unwrap();
  assert_eq!(
    std::fs::metadata(parent).unwrap().permissions().mode() & 0o777,
    0o700
  );
}

#[cfg(unix)]
#[tokio::test]
async fn generated_secret_rejects_symlinks_and_rolls_back_without_orphan_files() {
  use std::os::unix::fs::{PermissionsExt, symlink};
  let (dir, mut state) = crate::features::test_support::fixture().await;
  let parent = dir.path().join("private");
  std::fs::create_dir(&parent).unwrap();
  std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
  let target = dir.path().join("do-not-change");
  std::fs::write(&target, "target-secret").unwrap();
  let link = parent.join("admin-password");
  symlink(&target, &link).unwrap();
  configure(&mut state, link.clone());
  assert!(initialize(&state).await.is_err());
  assert!(!state.identity.bootstrap_claimed().await.unwrap());
  assert!(
    state
      .identity
      .user_model_by_username("root")
      .await
      .unwrap()
      .is_none()
  );
  assert_eq!(std::fs::read_to_string(&target).unwrap(), "target-secret");
  assert!(
    std::fs::symlink_metadata(&link)
      .unwrap()
      .file_type()
      .is_symlink()
  );

  let (rollback_dir, mut rollback_state) = crate::features::test_support::fixture().await;
  let output = rollback_dir.path().join("secrets/admin-password");
  configure(&mut rollback_state, output.clone());
  rollback_state
    .identity
    .create_user(71, "root".into(), "Existing root".into(), None, false)
    .await
    .unwrap();
  assert_eq!(
    initialize(&rollback_state).await.unwrap_err().status,
    axum::http::StatusCode::CONFLICT
  );
  assert!(!rollback_state.identity.bootstrap_claimed().await.unwrap());
  assert!(
    rollback_state
      .identity
      .password_hash(71)
      .await
      .unwrap()
      .is_none()
  );
  assert!(!output.exists());
}

#[cfg(unix)]
#[tokio::test]
async fn existing_generated_secret_must_be_regular_private_and_bounded() {
  use std::os::unix::fs::PermissionsExt;
  let (dir, mut state) = crate::features::test_support::fixture().await;
  let parent = dir.path().join("existing");
  std::fs::create_dir(&parent).unwrap();
  std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700)).unwrap();
  let directory = parent.join("directory-secret");
  std::fs::create_dir(&directory).unwrap();
  std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).unwrap();
  let oversized = parent.join("oversized-secret");
  std::fs::write(&oversized, vec![b'x'; 1027]).unwrap();
  std::fs::set_permissions(&oversized, std::fs::Permissions::from_mode(0o600)).unwrap();
  let permissive = parent.join("permissive-secret");
  std::fs::write(&permissive, "existing-password").unwrap();
  std::fs::set_permissions(&permissive, std::fs::Permissions::from_mode(0o644)).unwrap();
  for path in [&directory, &oversized, &permissive] {
    configure(&mut state, path.to_owned());
    assert!(initialize(&state).await.is_err());
    assert!(!state.identity.bootstrap_claimed().await.unwrap());
    assert!(
      state
        .identity
        .user_model_by_username("root")
        .await
        .unwrap()
        .is_none()
    );
    assert!(path.exists());
  }
}

#[tokio::test]
async fn initialized_restart_skips_input_and_preserves_password_then_cleans_old_generated_file() {
  let (dir, mut state) = crate::features::test_support::fixture().await;
  let generated = dir.path().join("state/admin-password");
  configure(&mut state, generated.clone());
  assert!(initialize(&state).await.unwrap());
  let user = state
    .identity
    .user_model_by_username("root")
    .await
    .unwrap()
    .unwrap();
  let original_hash = state
    .identity
    .password_hash(user.id)
    .await
    .unwrap()
    .unwrap();
  let old = SystemTime::now() - SECRET_CLEANUP_AGE - Duration::from_secs(60);
  let file = std::fs::File::open(&generated).unwrap();
  file
    .set_times(std::fs::FileTimes::new().set_modified(old))
    .unwrap();
  assert!(!initialize(&state).await.unwrap());
  assert!(!generated.exists());
  assert_eq!(
    state
      .identity
      .password_hash(user.id)
      .await
      .unwrap()
      .as_deref(),
    Some(original_hash.as_str())
  );

  let input = dir.path().join("supplied-password");
  std::fs::write(&input, "input-secret-password").unwrap();
  let (input_dir, mut input_state) = crate::features::test_support::fixture().await;
  let mut settings = (*input_state.settings).clone();
  settings.bootstrap.initial_password_file = Some(input.clone());
  settings.bootstrap.generated_password_file =
    Some(input_dir.path().join("generated/admin-password"));
  input_state.settings = Arc::new(settings);
  assert!(initialize(&input_state).await.unwrap());
  let input_file = std::fs::File::open(&input).unwrap();
  input_file
    .set_times(std::fs::FileTimes::new().set_modified(old))
    .unwrap();
  assert!(!initialize(&input_state).await.unwrap());
  assert!(input.exists());
  std::fs::remove_file(&input).unwrap();
  assert!(!initialize(&input_state).await.unwrap());
  assert!(!input.exists());
}
