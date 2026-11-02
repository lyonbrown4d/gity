use crate::features::test_support::fixture;
use gity_identity::{PersonalSshKeyCommand, UserProfileChanges};
use sea_orm::DbErr;
use std::sync::Arc;

fn command(user_id: i64) -> PersonalSshKeyCommand {
  let private =
    russh::keys::PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519).unwrap();
  let (public_key, fingerprint) =
    crate::ssh::canonical_key(&private.public_key().to_openssh().unwrap()).unwrap();
  PersonalSshKeyCommand {
    id: gity_core::new_id(),
    user_id,
    title: "lifecycle".into(),
    public_key,
    fingerprint,
  }
}

#[tokio::test]
async fn stale_authenticated_ssh_registration_is_denied_after_user_deletion_commits() {
  let (_directory, state) = fixture().await;
  // Model a request that authenticated while active, then yielded before its
  // application command reached the database. No test-only production hooks.
  assert!(
    state
      .identity
      .user_by_id(3)
      .await
      .unwrap()
      .unwrap()
      .is_active()
  );
  let command = command(3);
  let fingerprint = command.fingerprint.clone();
  let identity = state.identity.clone();
  let (resume, waiting) = tokio::sync::oneshot::channel();
  let creation = tokio::spawn(async move {
    waiting.await.unwrap();
    identity.add_personal_ssh_key(command).await
  });
  assert!(state.identity.delete_user_and_credentials(3).await.unwrap());
  resume.send(()).unwrap();
  assert!(matches!(
    creation.await.unwrap(),
    Err(DbErr::RecordNotFound(_))
  ));
  assert!(
    state
      .identity
      .personal_ssh_key(&fingerprint)
      .await
      .unwrap()
      .is_none()
  );
  assert!(
    state
      .identity
      .personal_ssh_keys(3)
      .await
      .unwrap()
      .is_empty()
  );
}

#[tokio::test]
async fn overlapping_ssh_registration_and_account_deletion_leave_no_credentials_or_owner() {
  let (_directory, state) = fixture().await;
  let gate = Arc::new(tokio::sync::Barrier::new(3));
  let identity = state.identity.clone();
  let create_gate = gate.clone();
  let command = command(3);
  let creation = tokio::spawn(async move {
    create_gate.wait().await;
    identity.add_personal_ssh_key(command).await
  });
  let identity = state.identity.clone();
  let delete_gate = gate.clone();
  let deletion = tokio::spawn(async move {
    delete_gate.wait().await;
    identity.delete_user_and_credentials(3).await
  });
  gate.wait().await;
  let (created, deleted) = tokio::join!(creation, deletion);
  assert!(deleted.unwrap().unwrap());
  if let Err(error) = created.unwrap() {
    assert!(matches!(error, DbErr::RecordNotFound(_)), "{error}");
  }
  assert!(state.identity.user_by_id(3).await.unwrap().is_none());
  assert!(
    state
      .identity
      .personal_ssh_keys(3)
      .await
      .unwrap()
      .is_empty()
  );
  assert!(state.identity.personal_tokens(3).await.unwrap().is_empty());
}

#[tokio::test]
async fn stale_authenticated_ssh_registration_is_denied_after_owner_is_disabled() {
  let (_directory, state) = fixture().await;
  let command = command(3);
  let fingerprint = command.fingerprint.clone();
  state
    .identity
    .update_user_profile(
      3,
      UserProfileChanges {
        status: Some("disabled".into()),
        ..Default::default()
      },
    )
    .await
    .unwrap();
  assert!(matches!(
    state.identity.add_personal_ssh_key(command).await,
    Err(DbErr::RecordNotFound(_))
  ));
  assert!(
    state
      .identity
      .personal_ssh_key(&fingerprint)
      .await
      .unwrap()
      .is_none()
  );
}
