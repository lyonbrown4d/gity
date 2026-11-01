use crate::{replication::parse_oid, GitProcess, GitService};
use anyhow::{ensure, Context, Result};
use git2::{Oid, Reference, Repository};
use gity_git_wire::{ReferenceSnapshot, RepositoryHead};
use std::{
  collections::{BTreeMap, BTreeSet},
  path::{Path, PathBuf},
};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio_util::sync::CancellationToken;

const MAX_BUNDLE_REFS: usize = 100_000;
const MAX_BUNDLE_REF_LINE: usize = 4096;

impl GitService {
  /// Import verified objects, check repository integrity, then mirror refs and HEAD.
  /// The caller must hold the repository's exclusive write lock for this operation.
  pub async fn apply_replica_bundle(
    &self,
    repo: &str,
    bundle: Option<&Path>,
    snapshot: ReferenceSnapshot,
  ) -> Result<()> {
    self
      .apply_replica_bundle_cancellable(repo, bundle, snapshot, CancellationToken::new())
      .await
  }

  pub async fn apply_replica_bundle_cancellable(
    &self,
    repo: &str,
    bundle: Option<&Path>,
    snapshot: ReferenceSnapshot,
    cancellation: CancellationToken,
  ) -> Result<()> {
    let path = self.repository_path(repo)?;
    let refs = validate_snapshot(&snapshot)?;
    let bundle = bundle.map(canonical_bundle_path).transpose()?;
    ensure!(
      snapshot.refs.is_empty() == bundle.is_none(),
      "a bundle is required exactly when the snapshot has refs"
    );

    if let Some(bundle) = &bundle {
      verify_bundle(self, repo, bundle, &snapshot).await?;
      run_git(self, repo, ["bundle", "unbundle"], Some(bundle)).await?;
      run_git(
        self,
        repo,
        ["fsck", "--full", "--strict", "--no-reflogs"],
        None,
      )
      .await?;
    }

    tokio::task::spawn_blocking(move || apply_references(path, snapshot, refs, cancellation))
      .await
      .context("reference transaction worker failed")??;
    Ok(())
  }
}

fn canonical_bundle_path(path: &Path) -> Result<PathBuf> {
  let path = path.canonicalize().context("resolve replication bundle")?;
  ensure!(path.is_file(), "replication bundle must be a file");
  Ok(path)
}

fn validate_snapshot(snapshot: &ReferenceSnapshot) -> Result<BTreeMap<String, Oid>> {
  ensure!(
    snapshot.refs.len() <= MAX_BUNDLE_REFS,
    "too many snapshot refs"
  );
  let mut refs = BTreeMap::new();
  for (name, value) in &snapshot.refs {
    ensure!(
      name.starts_with("refs/") && name != "HEAD" && Reference::is_valid_name(name),
      "invalid snapshot reference name"
    );
    refs.insert(name.clone(), parse_oid(value)?);
  }
  match &snapshot.head {
    RepositoryHead::Symbolic(target) => ensure!(
      target.starts_with("refs/") && Reference::is_valid_name(target),
      "invalid symbolic HEAD target"
    ),
    RepositoryHead::Detached(value) => {
      parse_oid(value)?;
    }
  }
  Ok(refs)
}

async fn verify_bundle(
  service: &GitService,
  repo: &str,
  bundle: &Path,
  snapshot: &ReferenceSnapshot,
) -> Result<()> {
  let actual = list_bundle_heads(service, repo, bundle).await?;
  let expected = expected_bundle_heads(snapshot)?;
  ensure!(
    actual == expected,
    "bundle heads do not match the reference snapshot"
  );
  run_git(service, repo, ["bundle", "verify"], Some(bundle)).await?;
  Ok(())
}

fn expected_bundle_heads(snapshot: &ReferenceSnapshot) -> Result<BTreeMap<String, String>> {
  let mut expected = snapshot.refs.clone();
  let head_target = match &snapshot.head {
    RepositoryHead::Symbolic(target) => snapshot.refs.get(target).cloned(),
    RepositoryHead::Detached(value) => Some(value.clone()),
  };
  if let Some(target) = head_target {
    expected.insert("HEAD".to_owned(), target);
  }
  Ok(expected)
}

async fn list_bundle_heads(
  service: &GitService,
  repo: &str,
  bundle: &Path,
) -> Result<BTreeMap<String, String>> {
  let process = spawn_git(service, repo, ["bundle", "list-heads"], Some(bundle)).await?;
  let reader = BufReader::new(process.into_reader()?);
  let mut reader = reader;
  let mut heads = BTreeMap::new();
  while let Some(line) = read_bundle_head_line(&mut reader).await? {
    let (oid, name) = line.split_once(' ').context("malformed bundle head")?;
    ensure!(
      !name.is_empty() && !name.contains(' '),
      "malformed bundle ref name"
    );
    ensure!(heads.len() < MAX_BUNDLE_REFS + 1, "too many bundle refs");
    let oid = parse_oid(oid)?.to_string();
    ensure!(
      heads.insert(name.to_owned(), oid).is_none(),
      "duplicate bundle ref"
    );
  }
  Ok(heads)
}

async fn read_bundle_head_line<R>(reader: &mut R) -> Result<Option<String>>
where
  R: AsyncBufRead + Unpin,
{
  let mut line = Vec::with_capacity(128);
  loop {
    let buffer = reader.fill_buf().await?;
    if buffer.is_empty() {
      ensure!(line.is_empty(), "unterminated bundle head line");
      return Ok(None);
    }
    let count = buffer
      .iter()
      .position(|byte| *byte == b'\n')
      .map_or(buffer.len(), |index| index + 1);
    ensure!(
      line.len() + count <= MAX_BUNDLE_REF_LINE,
      "bundle ref line is too long"
    );
    let complete = buffer.get(count - 1) == Some(&b'\n');
    line.extend_from_slice(&buffer[..count]);
    reader.consume(count);
    if complete {
      line.pop();
      if line.last() == Some(&b'\r') {
        line.pop();
      }
      return Ok(Some(
        String::from_utf8(line).context("bundle ref line is not UTF-8")?,
      ));
    }
  }
}

async fn run_git<const N: usize>(
  service: &GitService,
  repo: &str,
  args: [&str; N],
  bundle: Option<&Path>,
) -> Result<()> {
  let mut process = spawn_git(service, repo, args, bundle).await?;
  let mut stdout = process.take_stdout().context("Git process has no stdout")?;
  let mut buffer = [0u8; 64 * 1024];
  while stdout.read(&mut buffer).await? != 0 {}
  drop(stdout);
  process.wait().await
}

async fn spawn_git<const N: usize>(
  service: &GitService,
  repo: &str,
  args: [&str; N],
  bundle: Option<&Path>,
) -> Result<GitProcess> {
  let permit = service.rpc_limit.clone().acquire_owned().await?;
  let mut command = service.command(repo)?;
  command.args(args);
  if let Some(bundle) = bundle {
    command.arg(bundle);
  }
  crate::protocol::spawn_process(command, permit)
}

fn apply_references(
  path: PathBuf,
  snapshot: ReferenceSnapshot,
  refs: BTreeMap<String, Oid>,
  cancellation: CancellationToken,
) -> Result<()> {
  let repository = Repository::open_bare(path).context("open replica repository")?;
  for oid in refs.values() {
    ensure!(!cancellation.is_cancelled(), "replica apply cancelled");
    repository
      .find_object(*oid, None)
      .context("snapshot ref object is missing")?;
  }
  if let RepositoryHead::Detached(value) = &snapshot.head {
    repository
      .find_object(parse_oid(value)?, None)
      .context("detached HEAD object is missing")?;
  }

  let desired: BTreeSet<_> = refs.keys().cloned().collect();
  let current = repository
    .references()?
    .map(|reference| {
      reference?
        .name()
        .map(str::to_owned)
        .context("repository contains a non-UTF-8 reference name")
    })
    .collect::<Result<BTreeSet<_>>>()?;
  let mut locks: BTreeSet<_> = current.union(&desired).cloned().collect();
  locks.insert("HEAD".to_owned());

  let mut transaction = repository.transaction()?;
  for name in &locks {
    ensure!(!cancellation.is_cancelled(), "replica apply cancelled");
    transaction.lock_ref(name)?;
  }
  for name in current.difference(&desired) {
    ensure!(!cancellation.is_cancelled(), "replica apply cancelled");
    transaction.remove(name)?;
  }
  for (name, oid) in &refs {
    ensure!(!cancellation.is_cancelled(), "replica apply cancelled");
    transaction.set_target(name, *oid, None, "Gity replica update")?;
  }
  ensure!(!cancellation.is_cancelled(), "replica apply cancelled");
  match &snapshot.head {
    RepositoryHead::Symbolic(target) => {
      transaction.set_symbolic_target("HEAD", target, None, "Gity replica HEAD update")?
    }
    RepositoryHead::Detached(value) => {
      transaction.set_target("HEAD", parse_oid(value)?, None, "Gity replica HEAD update")?
    }
  }
  transaction.commit()?;
  Ok(())
}
