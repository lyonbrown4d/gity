use gity_core::AppError;

pub(crate) fn name(value: &str) -> Result<(), AppError> {
  if value.trim().is_empty() || value.len() >= 512 {
    return Err(AppError::bad_request("name required"));
  }
  Ok(())
}
pub(crate) fn visibility(value: &str) -> Result<(), AppError> {
  if !["private", "internal", "public"].contains(&value) {
    return Err(AppError::bad_request("invalid visibility"));
  }
  Ok(())
}
pub(crate) fn slug(value: &str) -> Result<(), AppError> {
  if value.is_empty()
    || value.len() > 100
    || [".", ".."].contains(&value)
    || value.ends_with(".git")
    || !value
      .bytes()
      .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
  {
    return Err(AppError::bad_request("invalid path_key"));
  }
  Ok(())
}
pub(crate) fn lock_path(value: &str) -> Result<String, AppError> {
  let value = value.trim().replace('\\', "/");
  let value = value.trim_matches('/');
  if value.is_empty() || value.split('/').any(|p| ["", ".", ".."].contains(&p)) {
    return Err(AppError::bad_request("invalid lock path"));
  }
  Ok(value.to_owned())
}
pub(crate) fn variable(command: &crate::VariableCommand) -> Result<(), AppError> {
  let key = &command.key;
  if key.is_empty()
    || key.len() > 128
    || !key
      .bytes()
      .enumerate()
      .all(|(i, b)| b == b'_' || b.is_ascii_uppercase() || (i > 0 && b.is_ascii_digit()))
  {
    return Err(AppError::bad_request("ci variable key is invalid"));
  }
  if command.value.len() > 64 * 1024 {
    return Err(AppError::bad_request("ci variable value is too large"));
  }
  if command.masked && (command.value.len() < 8 || command.value.contains(['\n', '\r'])) {
    return Err(AppError::bad_request(
      "masked ci variable value must be at least 8 characters",
    ));
  }
  Ok(())
}
