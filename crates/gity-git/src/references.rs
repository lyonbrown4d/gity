use anyhow::{bail, Result};
use git2::Reference;

pub(crate) fn validate_ref(name: &str, kind: &str) -> Result<String> {
  let reference = format!("refs/{kind}/{name}");
  // Keep ownership markers bounded and leave space for lock-file suffixes.
  if name.is_empty()
    || name.len() > 1024
    || name.split('/').any(|part| part.len() > 240)
    || name.starts_with('-')
    || !Reference::is_valid_name(&reference)
  {
    bail!("invalid git {kind} name");
  }
  Ok(reference)
}
