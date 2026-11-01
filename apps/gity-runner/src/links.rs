use anyhow::{Context, Result, bail};
use std::{
  collections::{BTreeMap, VecDeque},
  ffi::OsString,
  path::{Component, Path, PathBuf},
};

enum Part {
  Parent,
  Normal(OsString),
}
fn parts(path: &Path) -> Result<VecDeque<Part>> {
  let mut result = VecDeque::new();
  for part in path.components() {
    match part {
      Component::Normal(value) => result.push_back(Part::Normal(value.to_owned())),
      Component::ParentDir => result.push_back(Part::Parent),
      Component::CurDir => {}
      _ => bail!("source symlink has an absolute target"),
    }
  }
  Ok(result)
}
// Resolve '..' after expanding each symlink, as the operating system does.
// Lexically normalizing first would let `alias/..` escape through another link.
fn validate(name: &Path, target: &Path, links: &BTreeMap<PathBuf, PathBuf>) -> Result<()> {
  let mut queue = parts(name.parent().unwrap_or(Path::new("")))?;
  queue.extend(parts(target)?);
  let mut resolved = PathBuf::new();
  let mut hops = 0;
  while let Some(part) = queue.pop_front() {
    match part {
      Part::Parent => {
        if !resolved.pop() {
          bail!("source symlink escapes its workspace");
        }
      }
      Part::Normal(value) => {
        resolved.push(value);
        if let Some(target) = links.get(&resolved) {
          hops += 1;
          if hops > 40 {
            bail!("source symlink cycle or excessive link depth");
          }
          resolved.pop();
          for part in parts(target)?.into_iter().rev() {
            queue.push_front(part);
          }
        }
      }
    }
  }
  Ok(())
}
pub fn install(root: &Path, entries: Vec<(PathBuf, PathBuf)>) -> Result<()> {
  let mut links = BTreeMap::new();
  for (name, target) in entries {
    if links.insert(name, target).is_some() {
      bail!("source archive contains duplicate symlink entries");
    }
  }
  for (name, target) in &links {
    validate(name, target, &links)?;
  }
  let root = root.canonicalize()?;
  for (name, target) in links {
    let link = root.join(name);
    let parent = link.parent().context("invalid source link path")?;
    std::fs::create_dir_all(parent)?;
    if !parent.canonicalize()?.starts_with(&root) {
      bail!("source link parent escapes workspace");
    }
    if std::fs::symlink_metadata(&link).is_ok() {
      bail!("source link collides with another archive entry");
    }
    #[cfg(unix)]
    std::os::unix::fs::symlink(target, link)?;
    #[cfg(not(unix))]
    bail!("source symlinks require a Unix runner host");
  }
  Ok(())
}
