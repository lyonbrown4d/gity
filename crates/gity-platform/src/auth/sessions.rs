use super::*;

pub(super) async fn create_account(
  state: &AppState,
  body: Credentials,
) -> Result<Json<Value>, AppError> {
  let hash = hash_password(body.password).await?;
  let id = new_id();
  let org_id = new_id();
  let membership_id = new_id();
  let username = body.username.trim().to_owned();
  let user = state
    .identity
    .register_account(
      id,
      org_id,
      membership_id,
      username,
      hash,
      chrono::Utc::now().naive_utc(),
    )
    .await?;
  session(
    state,
    &json!({
        "id": user.id,
        "username": user.username,
        "organization_id": org_id,
        "status": user.status,
        "is_super_admin": user.is_super_admin,
    }),
    None,
  )
  .await
}
pub(super) async fn session(
  state: &AppState,
  user: &Value,
  family: Option<String>,
) -> Result<Json<Value>, AppError> {
  from_user(user)?;
  let id = user["id"]
    .as_i64()
    .ok_or_else(|| AppError::internal("invalid user"))?;
  let access = secret();
  let refresh = secret();
  let family = family.unwrap_or_else(secret);
  let now = chrono::Utc::now();
  state
    .identity
    .create_session_pair(
      id,
      token_hash(&access),
      token_hash(&refresh),
      family.clone(),
      now + chrono::Duration::hours(1),
      now + chrono::Duration::hours(24 * 30),
    )
    .await?;
  Ok(ok(
    json!({"user_id":id.to_string(),"username":user["username"],"organization_id":user.get("organization_id"),"token":access,"refresh_token":refresh}),
  ))
}
