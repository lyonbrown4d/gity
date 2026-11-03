use crate::{mutations::signature, validate_path};
use anyhow::{ensure, Context, Result};
use git2::{Oid, Repository};
use gity_git_wire::InitialCommit;
use std::collections::BTreeMap;

const MAX_FILES: usize = 100;
const MAX_PATH_BYTES: usize = 4096;
const MAX_PATH_DEPTH: usize = 64;
const MAX_SIGNATURE_FIELD_BYTES: usize = 256;
const MAX_CONTENT_BYTES: usize = 1_048_576;
const MAX_TOTAL_BYTES: usize = 8_388_608;

#[derive(Default)]
struct Directory {
  files: BTreeMap<String, String>,
  directories: BTreeMap<String, Directory>,
}

pub(crate) fn validate(branch: &str, commit: Option<&InitialCommit>) -> Result<()> {
  crate::references::validate_ref(branch, "heads")?;
  if let Some(commit) = commit {
    ensure!(commit.files.len() <= MAX_FILES, "too many initial files");
    let mut paths = std::collections::BTreeSet::new();
    let mut total = 0usize;
    for file in &commit.files {
      validate_path(&file.path, false)?;
      ensure!(
        file.path.len() <= MAX_PATH_BYTES,
        "initial file path is too long"
      );
      ensure!(
        file.path.split('/').count() <= MAX_PATH_DEPTH,
        "initial file path is too deep"
      );
      ensure!(
        paths.insert(file.path.as_str()),
        "duplicate initial file path"
      );
      ensure!(
        file.content.len() <= MAX_CONTENT_BYTES,
        "initial file is too large"
      );
      total = total.saturating_add(file.content.len());
      ensure!(total <= MAX_TOTAL_BYTES, "initial files are too large");
    }
    validate_signature_field(&commit.author_name)?;
    validate_signature_field(&commit.author_email)?;
    signature(&commit.author_name, &commit.author_email)?;
    for path in &paths {
      let mut parent = *path;
      while let Some((prefix, _)) = parent.rsplit_once('/') {
        ensure!(
          !paths.contains(prefix),
          "initial file path conflicts with a directory"
        );
        parent = prefix;
      }
    }
  }
  Ok(())
}

fn validate_signature_field(value: &str) -> Result<()> {
  ensure!(
    value.len() <= MAX_SIGNATURE_FIELD_BYTES,
    "initial author field is too long"
  );
  ensure!(
    !value.chars().any(char::is_control),
    "initial author field contains control characters"
  );
  Ok(())
}

pub(crate) fn initialize(
  repo: &Repository,
  branch: &str,
  initial: Option<&InitialCommit>,
) -> Result<()> {
  let reference_name = format!("refs/heads/{branch}");
  let Some(initial) = initial else {
    for reference in repo.references()? {
      let reference = reference?;
      ensure!(
        !reference.is_branch() || reference.target().is_none(),
        "repository already contains a branch commit"
      );
    }
    return Ok(());
  };
  for reference in repo.references()? {
    let reference = reference?;
    if reference.is_branch() {
      ensure!(
        reference.name().ok() == Some(reference_name.as_str()),
        "repository has an unexpected branch"
      );
    }
  }
  let tree_id = write_tree(repo, initial)?;
  match repo.find_reference(&reference_name) {
    Ok(reference) => {
      let id = reference.target().context("initial branch has no target")?;
      let commit = repo.find_commit(id)?;
      ensure!(
        commit.parent_count() == 0,
        "initial branch has a different commit"
      );
      ensure!(
        commit.tree_id() == tree_id,
        "initial branch has different initial files"
      );
      Ok(())
    }
    Err(error) if error.code() == git2::ErrorCode::NotFound => {
      let tree = repo.find_tree(tree_id)?;
      let signature = signature(&initial.author_name, &initial.author_email)?;
      repo.commit(
        Some(&reference_name),
        &signature,
        &signature,
        "Initial commit",
        &tree,
        &[],
      )?;
      Ok(())
    }
    Err(error) => Err(error.into()),
  }
}

fn write_tree(repo: &Repository, initial: &InitialCommit) -> Result<Oid> {
  let mut root = Directory::default();
  for file in &initial.files {
    let mut parts = file.path.split('/').peekable();
    let mut directory = &mut root;
    while let Some(part) = parts.next() {
      if parts.peek().is_some() {
        ensure!(
          !directory.files.contains_key(part),
          "initial file path conflicts with a directory"
        );
        directory = directory.directories.entry(part.to_owned()).or_default();
      } else {
        ensure!(
          !directory.directories.contains_key(part),
          "initial file path conflicts with a directory"
        );
        ensure!(
          directory
            .files
            .insert(part.to_owned(), file.content.clone())
            .is_none(),
          "duplicate initial file path"
        );
      }
    }
  }
  write_directory(repo, &root)
}

fn write_directory(repo: &Repository, directory: &Directory) -> Result<Oid> {
  let mut builder = repo.treebuilder(None)?;
  for (name, content) in &directory.files {
    let blob = repo.blob(content.as_bytes())?;
    builder.insert(name, blob, 0o100644)?;
  }
  for (name, child) in &directory.directories {
    let tree = write_directory(repo, child)?;
    builder.insert(name, tree, 0o040000)?;
  }
  builder.write().context("write initial repository tree")
}
