use crate::{commit, validate_path, GitService};
use anyhow::{bail, Context, Result};
use git2::{Index, IndexEntry, IndexTime, Oid, Reference, Repository, Signature};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Clone, Deserialize)]
pub struct CreateFileCommitInput {
    pub branch_name: String,
    pub file_path: String,
    pub content: String,
    pub message: String,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_email: String,
}

#[derive(Clone, Deserialize)]
pub struct MergeBranchesInput {
    pub target_branch: String,
    pub source_branch: String,
    #[serde(default)]
    pub expected_source_sha: Option<String>,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub author_name: String,
    #[serde(default)]
    pub author_email: String,
}

impl GitService {
    pub async fn init_bare(&self, repo: &str, initial_branch: &str) -> Result<()> {
        validate_ref(initial_branch, "heads")?;
        let path = self.repository_path(repo)?;
        let branch = initial_branch.to_owned();
        tokio::task::spawn_blocking(move || -> Result<()> {
            if path.exists() {
                bail!("repository path already exists");
            }
            let mut opts = git2::RepositoryInitOptions::new();
            opts.bare(true)
                .initial_head(&branch)
                .mkdir(true)
                .mkpath(true);
            Repository::init_opts(path, &opts)?;
            Ok(())
        })
        .await??;
        Ok(())
    }

    pub async fn create_branch(&self, repo: &str, name: &str, source: &str) -> Result<()> {
        let name = validate_ref(name, "heads")?;
        let source = source.to_owned();
        self.read(repo, move |r| {
            let c = commit(&r, &source, "")?;
            if r.find_reference(&name).is_ok() {
                bail!("git branch already exists");
            }
            r.reference(&name, c.id(), false, "create branch")?;
            Ok(Value::Null)
        })
        .await?;
        Ok(())
    }

    pub async fn delete_branch(&self, repo: &str, name: &str) -> Result<()> {
        self.delete_ref(repo, name, "heads").await
    }
    pub async fn delete_tag(&self, repo: &str, name: &str) -> Result<()> {
        self.delete_ref(repo, name, "tags").await
    }
    async fn delete_ref(&self, repo: &str, name: &str, kind: &str) -> Result<()> {
        let reference = validate_ref(name, kind)?;
        self.read(repo, move |r| {
            r.find_reference(&reference)
                .context("git reference not found")?
                .delete()?;
            Ok(Value::Null)
        })
        .await?;
        Ok(())
    }

    pub async fn create_tag(&self, repo: &str, name: &str, source: &str) -> Result<()> {
        let name = validate_ref(name, "tags")?;
        let source = source.to_owned();
        self.read(repo, move |r| {
            if r.find_reference(&name).is_ok() {
                bail!("git tag already exists");
            }
            r.reference(&name, commit(&r, &source, "")?.id(), false, "create tag")?;
            Ok(Value::Null)
        })
        .await?;
        Ok(())
    }

    /// Build objects without a working tree and compare-and-swap the branch tip.
    /// Competing pushes cannot silently overwrite commits made by another writer.
    pub async fn create_file_commit(
        &self,
        repo: &str,
        input: CreateFileCommitInput,
    ) -> Result<String> {
        let reference = validate_ref(&input.branch_name, "heads")?;
        validate_path(&input.file_path, false)?;
        if input.message.trim().is_empty() {
            bail!("commit message is required");
        }
        let result = self
            .read(repo, move |r| {
                let old = r.find_reference(&reference).ok().and_then(|v| v.target());
                let parent = match old {
                    Some(id) => Some(r.find_commit(id)?),
                    None => r.head().ok().and_then(|head| head.peel_to_commit().ok()),
                };
                let mut index = Index::new()?;
                if let Some(c) = &parent {
                    index.read_tree(&c.tree()?)?;
                }
                if index
                    .get_path(std::path::Path::new(&input.file_path), 0)
                    .is_some()
                {
                    bail!("git file already exists");
                }
                // Refuse replacing a subtree or placing a file beneath an existing blob.
                if let Some(c) = &parent {
                    let tree = c.tree()?;
                    if tree
                        .get_path(std::path::Path::new(&input.file_path))
                        .is_ok()
                    {
                        bail!("git file already exists");
                    }
                    let mut ancestor = std::path::Path::new(&input.file_path).parent();
                    while let Some(path) = ancestor.filter(|p| !p.as_os_str().is_empty()) {
                        if let Ok(entry) = tree.get_path(path) {
                            if entry.kind() != Some(git2::ObjectType::Tree) {
                                bail!("file parent is not a directory");
                            }
                        }
                        ancestor = path.parent();
                    }
                }
                let id = r.blob(input.content.as_bytes())?;
                index.add(&IndexEntry {
                    ctime: IndexTime::new(0, 0),
                    mtime: IndexTime::new(0, 0),
                    dev: 0,
                    ino: 0,
                    mode: 0o100644,
                    uid: 0,
                    gid: 0,
                    file_size: input
                        .content
                        .len()
                        .try_into()
                        .context("file is too large")?,
                    id,
                    flags: 0,
                    flags_extended: 0,
                    path: input.file_path.as_bytes().to_vec(),
                })?;
                let tree = r.find_tree(index.write_tree_to(&r)?)?;
                let signature = signature(&input.author_name, &input.author_email)?;
                let parents = parent.iter().collect::<Vec<_>>();
                let id = r.commit(
                    None,
                    &signature,
                    &signature,
                    &input.message,
                    &tree,
                    &parents,
                )?;
                update_tip(&r, &reference, id, old)?;
                Ok(json!({"hash":id.to_string()}))
            })
            .await?;
        Ok(result["hash"]
            .as_str()
            .context("commit result lacks object ID")?
            .to_owned())
    }

    pub async fn merge_branches(&self, repo: &str, input: MergeBranchesInput) -> Result<()> {
        let target = validate_ref(&input.target_branch, "heads")?;
        let source = validate_ref(&input.source_branch, "heads")?;
        self.read(repo, move |r| {
            // Lock both tips across validation and object construction. A native
            // push cannot advance the checked source between CI approval and merge.
            let mut transaction = r.transaction()?;
            let mut references = vec![source.as_str(), target.as_str()];
            references.sort_unstable();
            references.dedup();
            for reference in references {
                transaction.lock_ref(reference)?;
            }
            let target_commit = r.find_reference(&target)?.peel_to_commit()?;
            let source_commit = r.find_reference(&source)?.peel_to_commit()?;
            if let Some(expected) = &input.expected_source_sha {
                if expected != &source_commit.id().to_string() {
                    bail!("merge source changed; refresh required");
                }
            }
            if target_commit.id() == source_commit.id()
                || r.graph_descendant_of(target_commit.id(), source_commit.id())?
            {
                return Ok(Value::Null);
            }
            let mut index = r.merge_commits(&target_commit, &source_commit, None)?;
            if index.has_conflicts() {
                bail!("git merge conflict");
            }
            let tree = r.find_tree(index.write_tree_to(&r)?)?;
            let signature = signature(&input.author_name, &input.author_email)?;
            let message = if input.message.trim().is_empty() {
                format!(
                    "Merge branch '{}' into '{}'",
                    input.source_branch, input.target_branch
                )
            } else {
                input.message
            };
            let id = r.commit(
                None,
                &signature,
                &signature,
                &message,
                &tree,
                &[&target_commit, &source_commit],
            )?;
            transaction.set_target(&target, id, Some(&signature), "Gity merge")?;
            transaction.commit()?;
            Ok(Value::Null)
        })
        .await?;
        Ok(())
    }

    pub async fn diff_branches(&self, repo: &str, target: &str, source: &str) -> Result<String> {
        let target = validate_ref(target, "heads")?;
        let source = validate_ref(source, "heads")?;
        let value = self
            .read(repo, move |r| {
                let t = r.find_reference(&target)?.peel_to_commit()?;
                let s = r.find_reference(&source)?.peel_to_commit()?;
                let base = r.find_commit(r.merge_base(t.id(), s.id())?)?;
                let mut diff = r.diff_tree_to_tree(Some(&base.tree()?), Some(&s.tree()?), None)?;
                diff.find_similar(None)?;
                let mut bytes = Vec::new();
                diff.print(git2::DiffFormat::Patch, |_, _, line| {
                    if matches!(line.origin(), ' ' | '+' | '-') {
                        bytes.push(line.origin() as u8);
                    }
                    bytes.extend_from_slice(line.content());
                    true
                })?;
                Ok(json!(String::from_utf8_lossy(&bytes)))
            })
            .await?;
        Ok(value.as_str().unwrap_or_default().to_owned())
    }
}

pub(crate) fn validate_ref(name: &str, kind: &str) -> Result<String> {
    let reference = format!("refs/{kind}/{name}");
    if name.is_empty() || name.starts_with('-') || !Reference::is_valid_name(&reference) {
        bail!("invalid git {kind} name");
    }
    Ok(reference)
}
fn signature(name: &str, email: &str) -> Result<Signature<'static>> {
    Ok(Signature::now(
        if name.trim().is_empty() { "Gity" } else { name },
        if email.trim().is_empty() {
            "noreply@gity.local"
        } else {
            email
        },
    )?)
}
fn update_tip(repo: &Repository, reference: &str, id: Oid, old: Option<Oid>) -> Result<()> {
    if let Some(old) = old {
        repo.reference_matching(reference, id, true, old, "Gity commit")?;
    } else {
        repo.reference(reference, id, false, "Gity initial commit")?;
    }
    Ok(())
}
