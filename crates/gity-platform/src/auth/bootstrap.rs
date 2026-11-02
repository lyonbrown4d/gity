use super::{hash_password, validate_username};
use crate::AppState;
#[cfg(test)]
use files::SECRET_CLEANUP_AGE;
use files::{
  OwnedSecretFile, cleanup_generated_secret, generated_path, generated_secret, read_input_secret,
  remove_owned_secret, validate_password,
};
use gity_core::{AppError, new_id};
use gity_identity::BootstrapAccountCommand;
mod files;
#[cfg(test)]
mod tests;

pub(crate) async fn initialize(state: &AppState) -> Result<bool, AppError> {
  let config = &state.settings.bootstrap;
  if !config.enabled {
    return Ok(false);
  }
  let Some(mut guard) = state.identity.begin_bootstrap().await? else {
    if config.initial_password_file.is_none() {
      cleanup_generated_secret(&generated_path(state))?;
    }
    return Ok(false);
  };
  let mut owned_secret: Option<OwnedSecretFile> = None;
  let result = async {
    let username = validate_username(&config.username)?.to_owned();
    let password = match &config.initial_password_file {
      Some(path) => read_input_secret(path)?,
      None => {
        let generated = generated_secret(&generated_path(state))?;
        owned_secret = generated.owned_file;
        generated.value
      }
    };
    validate_password(&password)?;
    let password_hash = hash_password(password).await?;
    let organization_id = new_id();
    guard
      .create_account(BootstrapAccountCommand {
        user_id: new_id(),
        organization_id,
        membership_id: new_id(),
        username,
        email: config.email.clone(),
        password_hash,
        timestamp: chrono::Utc::now().naive_utc(),
      })
      .await
      .map_err(repository_error)?;
    guard
      .commit()
      .await
      .map_err(|_| AppError::internal("administrator bootstrap failed"))?;
    Ok::<(), AppError>(())
  }
  .await;
  match result {
    Ok(()) => Ok(true),
    Err(error) => {
      let _ = guard.rollback().await;
      if let Some(owned_file) = owned_secret {
        remove_owned_secret(&owned_file);
      }
      Err(error)
    }
  }
}

fn repository_error(error: sea_orm::DbErr) -> AppError {
  if matches!(&error, sea_orm::DbErr::Custom(message) if message == "bootstrap username already exists")
  {
    AppError::conflict("bootstrap username already exists")
  } else {
    AppError::internal("administrator bootstrap failed")
  }
}
