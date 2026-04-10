#[derive(Debug, Clone, toasty::Model)]
pub struct User {
  #[key]
  #[auto]
  pub id: i64,

  #[unique]
  pub username: String,

  #[unique]
  pub email: String,

  pub display_name: String,
  pub state: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Namespace {
  #[key]
  #[auto]
  pub id: i64,

  #[unique]
  pub full_path: String,

  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub kind: String,
  pub visibility: String,

  #[index]
  pub parent_namespace_id: Option<i64>,

  #[belongs_to(key = parent_namespace_id, references = id)]
  pub parent_namespace: toasty::BelongsTo<Option<Namespace>>,

  #[index]
  pub owner_user_id: Option<String>,

  #[has_many]
  pub projects: toasty::HasMany<Project>,

  #[has_many]
  pub members: toasty::HasMany<NamespaceMember>,

  #[has_many]
  pub invitations: toasty::HasMany<NamespaceInvitation>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct NamespaceMember {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub namespace_id: i64,

  #[belongs_to(key = namespace_id, references = id)]
  pub namespace: toasty::BelongsTo<Namespace>,

  #[index]
  pub user_id: String,

  pub role: String,
  pub state: String,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct NamespaceInvitation {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub namespace_id: i64,

  #[belongs_to(key = namespace_id, references = id)]
  pub namespace: toasty::BelongsTo<Namespace>,

  pub email: String,
  pub role: String,
  pub state: String,

  #[index]
  pub invited_by_user_id: String,

  #[index]
  pub accepted_by_user_id: Option<String>,

  pub expires_at_unix: Option<i64>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct Project {
  #[key]
  #[auto]
  pub id: i64,

  #[unique]
  pub full_path: String,

  #[index]
  pub namespace_id: i64,

  #[belongs_to(key = namespace_id, references = id)]
  pub namespace: toasty::BelongsTo<Namespace>,

  pub path_key: String,
  pub name: String,
  pub description: Option<String>,
  pub visibility: String,
  pub default_branch: String,
  pub archived: bool,

  #[index]
  pub created_by_user_id: String,

  #[has_many]
  pub branches: toasty::HasMany<ProjectBranch>,

  #[has_many]
  pub language_snapshots: toasty::HasMany<ProjectLanguageSnapshot>,

  #[has_many]
  pub issues: toasty::HasMany<ProjectIssue>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProjectBranch {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub project_id: i64,

  #[belongs_to(key = project_id, references = id)]
  pub project: toasty::BelongsTo<Project>,

  #[index]
  pub name: String,

  pub is_protected: bool,
  pub last_commit_sha: Option<String>,
  pub created_at_unix: i64,
  pub updated_at_unix: i64,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProjectLanguageSnapshot {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub project_id: i64,

  #[belongs_to(key = project_id, references = id)]
  pub project: toasty::BelongsTo<Project>,

  #[index]
  pub branch_name: String,

  pub revision: String,
  pub total_bytes: i64,
  pub analyzed_at_unix: i64,
  pub created_at_unix: i64,

  #[has_many]
  pub items: toasty::HasMany<ProjectLanguageSnapshotItem>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProjectLanguageSnapshotItem {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub snapshot_id: i64,

  #[belongs_to(key = snapshot_id, references = id)]
  pub project_language_snapshot: toasty::BelongsTo<ProjectLanguageSnapshot>,

  pub language: String,
  pub bytes: i64,
  pub created_at_unix: i64,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProjectIssue {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub project_id: i64,

  #[belongs_to(key = project_id, references = id)]
  pub project: toasty::BelongsTo<Project>,

  #[index]
  pub iid: i64,

  pub title: String,
  pub description: Option<String>,
  pub state: String,

  #[index]
  pub author_user_id: String,

  #[index]
  pub assignee_user_id: Option<String>,

  pub created_at_unix: i64,
  pub updated_at_unix: i64,
  pub closed_at_unix: Option<i64>,

  #[has_many]
  pub comments: toasty::HasMany<ProjectIssueComment>,
}

#[derive(Debug, Clone, toasty::Model)]
pub struct ProjectIssueComment {
  #[key]
  #[auto]
  pub id: i64,

  #[index]
  pub project_issue_id: i64,

  #[belongs_to(key = project_issue_id, references = id)]
  pub project_issue: toasty::BelongsTo<ProjectIssue>,

  #[index]
  pub author_user_id: String,

  pub body: String,
  pub created_at_unix: i64,
  pub updated_at_unix: i64,
}
