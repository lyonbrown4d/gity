use super::*;
use gity_identity::UserProfileChanges;

pub(super) async fn update_profile(s: &AppState, id: i64, body: &Value, admin: bool) -> Reply {
  let input: spec::UpdateUser =
    serde_json::from_value(body.clone()).map_err(|_| AppError::bad_request("invalid profile"))?;
  if !admin && input.status.is_some() {
    return Err(AppError::forbidden("administrator required"));
  }
  let password = input
    .password
    .filter(|password| !password.trim().is_empty());
  if password
    .as_ref()
    .is_some_and(|password| !(8..=1024).contains(&password.len()))
  {
    return Err(AppError::bad_request("password must contain 8–1024 bytes"));
  }
  let username = if body.get("username").is_some() {
    Some(auth::validate_username(input.username.as_deref().unwrap_or_default())?.to_owned())
  } else {
    None
  };
  if let Some(username) = &username
    && s
      .identity
      .user_model_by_username(username)
      .await?
      .is_some_and(|user| user.id != id)
  {
    return Err(AppError::conflict("resource already exists"));
  }
  let display_name = if body.get("display_name").is_some() {
    Some(
      input
        .display_name
        .clone()
        .ok_or_else(|| AppError::bad_request("invalid profile"))?,
    )
  } else {
    None
  };
  // Hashing and validation finish before the repository opens its transaction.
  let password_hash = if let Some(password) = password {
    Some(auth::hash_password(password).await?)
  } else {
    None
  };
  let changes = UserProfileChanges {
    username,
    display_name,
    email: body.get("email").map(|_| input.email),
    is_super_admin: admin.then_some(input.is_super_admin).flatten(),
    status: if admin {
      input.status.map(|value| value.as_str().to_owned())
    } else {
      None
    },
    password_hash,
  };
  let updated = s
    .identity
    .update_user_profile(id, changes)
    .await?
    .ok_or_else(|| AppError::not_found("user not found"))?;
  Ok(ok(view_model(updated)))
}
