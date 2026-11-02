use super::super::secret;
use crate::AppState;
use gity_core::AppError;
#[cfg(unix)]
use std::os::unix::{
  fs::{MetadataExt, OpenOptionsExt},
  io::AsRawFd,
};
use std::{
  fs::{self, File, OpenOptions},
  io::{Read, Write},
  path::{Path, PathBuf},
  time::{Duration, SystemTime},
};

const MAX_SECRET_BYTES: u64 = 1024;
pub(super) const SECRET_CLEANUP_AGE: Duration = Duration::from_secs(24 * 60 * 60);

pub(super) struct OwnedSecretFile {
  path: PathBuf,
  #[cfg(unix)]
  device: u64,
  #[cfg(unix)]
  inode: u64,
}

pub(super) struct GeneratedSecret {
  pub value: String,
  pub owned_file: Option<OwnedSecretFile>,
}

pub(super) fn generated_path(state: &AppState) -> PathBuf {
  state
    .settings
    .bootstrap
    .generated_password_file
    .clone()
    .unwrap_or_else(|| state.settings.storage_root.join("bootstrap/admin-password"))
}

pub(super) fn generated_secret(path: &Path) -> Result<GeneratedSecret, AppError> {
  ensure_private_parent(path)?;
  let value = secret();
  match create_private_file(path, value.as_bytes()) {
    Ok((device, inode)) => Ok(GeneratedSecret {
      value,
      owned_file: Some(OwnedSecretFile {
        path: path.to_owned(),
        #[cfg(unix)]
        device,
        #[cfg(unix)]
        inode,
      }),
    }),
    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(GeneratedSecret {
      value: read_generated_secret(path)?,
      owned_file: None,
    }),
    Err(_) => Err(AppError::internal(
      "could not write administrator password file",
    )),
  }
}

pub(super) fn read_input_secret(path: &Path) -> Result<String, AppError> {
  if fs::metadata(path)
    .map_err(|_| AppError::internal("could not read administrator password file"))?
    .len()
    > MAX_SECRET_BYTES + 2
  {
    return Err(AppError::bad_request(
      "administrator password file is too large",
    ));
  }
  let value = fs::read_to_string(path)
    .map_err(|_| AppError::internal("could not read administrator password file"))?;
  let value = value.trim_end_matches(['\r', '\n']).to_owned();
  validate_password(&value)?;
  Ok(value)
}

fn read_generated_secret(path: &Path) -> Result<String, AppError> {
  let mut file = open_existing_private_file(path)?;
  let mut bytes = Vec::new();
  Read::take(&mut file, MAX_SECRET_BYTES + 3)
    .read_to_end(&mut bytes)
    .map_err(|_| AppError::internal("could not read administrator password file"))?;
  if bytes.len() as u64 > MAX_SECRET_BYTES + 2 {
    return Err(AppError::internal(
      "administrator password file is too large",
    ));
  }
  let value = String::from_utf8(bytes)
    .map_err(|_| AppError::internal("administrator password file is invalid"))?;
  let value = value.trim_end_matches(['\r', '\n']).to_owned();
  validate_password(&value)?;
  Ok(value)
}

pub(super) fn validate_password(password: &str) -> Result<(), AppError> {
  if !(8..=1024).contains(&password.len()) {
    return Err(AppError::bad_request(
      "administrator password must contain 8–1024 bytes",
    ));
  }
  Ok(())
}

fn create_private_file(path: &Path, value: &[u8]) -> std::io::Result<(u64, u64)> {
  let mut options = OpenOptions::new();
  options.write(true).create_new(true);
  #[cfg(unix)]
  options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
  let mut file = options.open(path)?;
  #[cfg(unix)]
  let opened_metadata = file.metadata()?;
  let result = (|| {
    file.write_all(value)?;
    file.sync_all()?;
    #[cfg(unix)]
    {
      let metadata = file.metadata()?;
      if metadata.uid() != unsafe { libc::geteuid() } || metadata.mode() & 0o7777 != 0o600 {
        return Err(std::io::Error::new(
          std::io::ErrorKind::PermissionDenied,
          "generated password file is not private",
        ));
      }
      Ok((metadata.dev(), metadata.ino()))
    }
    #[cfg(not(unix))]
    Ok((0, 0))
  })();
  if result.is_err() {
    #[cfg(unix)]
    if let Ok(path_metadata) = fs::symlink_metadata(path)
      && path_metadata.file_type().is_file()
      && path_metadata.dev() == opened_metadata.dev()
      && path_metadata.ino() == opened_metadata.ino()
    {
      drop(file);
      let _ = fs::remove_file(path);
    }
    #[cfg(not(unix))]
    {
      drop(file);
      let _ = fs::remove_file(path);
    }
  }
  result
}

fn ensure_private_parent(path: &Path) -> Result<(), AppError> {
  let parent = path
    .parent()
    .filter(|parent| !parent.as_os_str().is_empty())
    .unwrap_or_else(|| Path::new("."));
  let existed = fs::symlink_metadata(parent).is_ok();
  if !existed {
    fs::create_dir_all(parent)
      .map_err(|_| AppError::internal("could not create administrator password directory"))?;
  }
  #[cfg(unix)]
  {
    let metadata = fs::symlink_metadata(parent)
      .map_err(|_| AppError::internal("administrator password directory is unavailable"))?;
    if !metadata.file_type().is_dir() || metadata.file_type().is_symlink() {
      return Err(AppError::internal(
        "administrator password directory is unsafe",
      ));
    }
    let directory = open_directory(parent)?;
    let opened = directory
      .metadata()
      .map_err(|_| AppError::internal("administrator password directory is unavailable"))?;
    if opened.uid() != unsafe { libc::geteuid() }
      || opened.dev() != metadata.dev()
      || opened.ino() != metadata.ino()
    {
      return Err(AppError::internal(
        "administrator password directory is not owned by this user",
      ));
    }
    if existed && opened.mode() & 0o7777 != 0o700 {
      return Err(AppError::internal(
        "administrator password directory must have mode 0700",
      ));
    }
    if !existed && unsafe { libc::fchmod(directory.as_raw_fd(), 0o700) } != 0 {
      return Err(AppError::internal(
        "could not secure administrator password directory",
      ));
    }
  }
  Ok(())
}

fn open_existing_private_file(path: &Path) -> Result<File, AppError> {
  #[cfg(unix)]
  {
    let before = fs::symlink_metadata(path)
      .map_err(|_| AppError::internal("administrator password file is unavailable"))?;
    if !before.file_type().is_file() || before.file_type().is_symlink() {
      return Err(AppError::internal("administrator password file is unsafe"));
    }
    if before.len() > MAX_SECRET_BYTES + 2
      || before.uid() != unsafe { libc::geteuid() }
      || before.mode() & 0o7777 != 0o600
    {
      return Err(AppError::internal(
        "administrator password file is not private",
      ));
    }
    let mut options = OpenOptions::new();
    options.read(true).custom_flags(libc::O_NOFOLLOW);
    let file = options
      .open(path)
      .map_err(|_| AppError::internal("administrator password file is unsafe"))?;
    let opened = file
      .metadata()
      .map_err(|_| AppError::internal("administrator password file is unavailable"))?;
    if !opened.is_file()
      || opened.dev() != before.dev()
      || opened.ino() != before.ino()
      || opened.uid() != before.uid()
      || opened.mode() & 0o7777 != 0o600
    {
      return Err(AppError::internal(
        "administrator password file changed during access",
      ));
    }
    Ok(file)
  }
  #[cfg(not(unix))]
  {
    File::open(path).map_err(|_| AppError::internal("administrator password file is unavailable"))
  }
}

pub(super) fn cleanup_generated_secret(path: &Path) -> Result<(), AppError> {
  if !path.exists() {
    return Ok(());
  }
  ensure_private_parent(path)?;
  let file = open_existing_private_file(path)?;
  let metadata = file
    .metadata()
    .map_err(|_| AppError::internal("administrator password file is unavailable"))?;
  let modified = metadata
    .modified()
    .map_err(|_| AppError::internal("administrator password file timestamp is unavailable"))?;
  if SystemTime::now()
    .duration_since(modified)
    .is_ok_and(|age| age >= SECRET_CLEANUP_AGE)
  {
    drop(file);
    fs::remove_file(path)
      .map_err(|_| AppError::internal("could not remove expired bootstrap password file"))?;
  }
  Ok(())
}

pub(super) fn remove_owned_secret(secret: &OwnedSecretFile) {
  #[cfg(unix)]
  if let Ok(metadata) = fs::symlink_metadata(&secret.path)
    && metadata.file_type().is_file()
    && metadata.dev() == secret.device
    && metadata.ino() == secret.inode
    && metadata.uid() == unsafe { libc::geteuid() }
  {
    let _ = fs::remove_file(&secret.path);
  }
  #[cfg(not(unix))]
  let _ = fs::remove_file(&secret.path);
}

#[cfg(unix)]
fn open_directory(path: &Path) -> Result<File, AppError> {
  let mut options = OpenOptions::new();
  options
    .read(true)
    .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW);
  options
    .open(path)
    .map_err(|_| AppError::internal("administrator password directory is unsafe"))
}
