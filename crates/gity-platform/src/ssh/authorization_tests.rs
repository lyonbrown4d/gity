use super::*;
use crate::features::test_support::fixture;
use gity_identity::PersonalSshKeyCommand;

async fn personal(state: &AppState, user: i64) -> (i64, String) {
  let key =
    russh::keys::PrivateKey::random(&mut rand::rng(), russh::keys::Algorithm::Ed25519).unwrap();
  let (public_key, fingerprint) =
    keys::canonical_key(&key.public_key().to_openssh().unwrap()).unwrap();
  let id = gity_core::new_id();
  state
    .identity
    .add_personal_ssh_key(PersonalSshKeyCommand {
      id,
      user_id: user,
      title: "test".into(),
      public_key,
      fingerprint: fingerprint.clone(),
    })
    .await
    .unwrap();
  (id, fingerprint)
}

#[tokio::test]
async fn personal_command_authorization_uses_current_roles_project_and_creator_state() {
  let (_dir, state) = fixture().await;
  state.git.init_bare("20.git", "main").await.unwrap();
  let (id, fingerprint) = personal(&state, 1).await;
  let access = authorize(&state, &fingerprint, "team/p20", GitRpc::ReceivePack)
    .await
    .unwrap();
  assert_eq!(access.policy.unwrap().actor_level, 40);
  assert_eq!(access.actor_id, Some(1));
  assert!(
    state
      .identity
      .personal_ssh_key(&fingerprint)
      .await
      .unwrap()
      .unwrap()
      .last_used_at
      .is_some()
  );
  state
    .db
    .execute(
      "UPDATE project_members SET role='reporter' WHERE user_id=1",
      vec![],
    )
    .await
    .unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::ReceivePack)
      .await
      .is_err()
  );
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_ok()
  );
  state
    .db
    .execute("DELETE FROM project_members WHERE user_id=1", vec![])
    .await
    .unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
  state.db.execute("INSERT INTO organization_members (id,organization_id,user_id,role,created_at,updated_at) VALUES (999,10,1,'owner',CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)", vec![]).await.unwrap();
  assert_eq!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::ReceivePack)
      .await
      .unwrap()
      .policy
      .unwrap()
      .actor_level,
    50
  );
  state
    .db
    .execute("UPDATE users SET status='disabled' WHERE id=1", vec![])
    .await
    .unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
  state
    .db
    .execute("UPDATE users SET status='active' WHERE id=1", vec![])
    .await
    .unwrap();
  state
    .db
    .execute("UPDATE projects SET status='inactive' WHERE id=20", vec![])
    .await
    .unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
  state
    .db
    .execute(
      "UPDATE projects SET status='active',deleted_at=CURRENT_TIMESTAMP WHERE id=20",
      vec![],
    )
    .await
    .unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
  state
    .db
    .execute("UPDATE projects SET deleted_at=NULL WHERE id=20", vec![])
    .await
    .unwrap();
  state.identity.delete_personal_ssh_key(1, id).await.unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
}

#[tokio::test]
async fn key_modes_fail_closed_on_ambiguous_fingerprint_even_for_inactive_deploy_creator() {
  let (_dir, state) = fixture().await;
  let (_id, fingerprint) = personal(&state, 1).await;
  let public = state
    .identity
    .personal_ssh_key(&fingerprint)
    .await
    .unwrap()
    .unwrap()
    .public_key;
  state.db.execute("INSERT INTO project_deploy_keys (id,project_id,title,fingerprint,public_key,can_push,created_by_user_id,created_at,updated_at) VALUES (999,20,'collision',?,?,0,2,CURRENT_TIMESTAMP,CURRENT_TIMESTAMP)", vec![fingerprint.clone().into(),public.into()]).await.unwrap();
  state
    .db
    .execute("UPDATE users SET status='disabled' WHERE id=2", vec![])
    .await
    .unwrap();
  assert!(
    keys::authorized(&state, &fingerprint)
      .await
      .unwrap()
      .is_none()
  );
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
}

#[tokio::test]
async fn outsiders_are_project_scoped_until_their_current_account_becomes_administrator() {
  let (_dir, state) = fixture().await;
  state.git.init_bare("20.git", "main").await.unwrap();
  let (_, fingerprint) = personal(&state, 3).await;
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::UploadPack)
      .await
      .is_err()
  );
  assert!(
    authorize(&state, &fingerprint, "team/p30", GitRpc::UploadPack)
      .await
      .is_err()
  );
  state
    .db
    .execute("UPDATE users SET is_super_admin=1 WHERE id=3", vec![])
    .await
    .unwrap();
  let access = authorize(&state, &fingerprint, "team/p20", GitRpc::ReceivePack)
    .await
    .unwrap();
  assert_eq!(access.policy.unwrap().actor_level, 100);
  state
    .db
    .execute("UPDATE users SET is_super_admin=0 WHERE id=3", vec![])
    .await
    .unwrap();
  assert!(
    authorize(&state, &fingerprint, "team/p20", GitRpc::ReceivePack)
      .await
      .is_err()
  );
  assert!(state.identity.delete_user_and_credentials(3).await.unwrap());
  assert!(
    keys::authorized(&state, &fingerprint)
      .await
      .unwrap()
      .is_none()
  );
}
