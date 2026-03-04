use chrono::{DateTime, Utc};
use entity::{
  organization_invitations, organization_members, organizations, repositories, repository_branches,
  repository_commits, users,
};
use sea_orm::sea_query::Expr;
use sea_orm::{
  ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DbErr, EntityTrait, IntoActiveModel,
  QueryFilter, QueryOrder, QuerySelect, Set,
};

pub struct AppRepository;

impl AppRepository {
  pub async fn find_user_by_username_or_email<C: ConnectionTrait>(
    conn: &C,
    identity: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(
        Condition::any()
          .add(users::Column::Username.eq(identity.to_string()))
          .add(users::Column::Email.eq(identity.to_string())),
      )
      .one(conn)
      .await
  }

  pub async fn find_duplicate_user_by_username_or_email<C: ConnectionTrait>(
    conn: &C,
    username: &str,
    email: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find()
      .filter(
        Condition::any()
          .add(users::Column::Username.eq(username.to_string()))
          .add(users::Column::Email.eq(email.to_string())),
      )
      .one(conn)
      .await
  }

  pub async fn find_active_user_by_id<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
  ) -> Result<Option<users::Model>, DbErr> {
    users::Entity::find_by_id(user_id.to_string())
      .filter(users::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn list_active_users<C: ConnectionTrait>(
    conn: &C,
    limit: Option<u64>,
  ) -> Result<Vec<users::Model>, DbErr> {
    let mut query = users::Entity::find()
      .filter(users::Column::DeletedAt.is_null())
      .order_by_desc(users::Column::CreatedAt);

    if let Some(limit) = limit {
      query = query.limit(limit);
    }

    query.all(conn).await
  }

  pub async fn insert_user<C: ConnectionTrait>(
    conn: &C,
    active: users::ActiveModel,
  ) -> Result<users::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_user_profile<C: ConnectionTrait>(
    conn: &C,
    model: users::Model,
    username: Option<String>,
    email: Option<String>,
    password_hash: Option<String>,
    password_salt: Option<String>,
  ) -> Result<users::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(username) = username {
      active.username = Set(username);
    }
    if let Some(email) = email {
      active.email = Set(email);
    }
    if let Some(password_hash) = password_hash {
      active.password = Set(password_hash);
    }
    if let Some(password_salt) = password_salt {
      active.password_salt = Set(password_salt);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn find_active_organization_by_id<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find_by_id(organization_id.to_string())
      .filter(organizations::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn find_active_organization_by_key<C: ConnectionTrait>(
    conn: &C,
    key: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(
        Condition::all()
          .add(organizations::Column::Key.eq(key.to_string()))
          .add(organizations::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn list_active_organizations_by_ids<C: ConnectionTrait>(
    conn: &C,
    organization_ids: Vec<String>,
  ) -> Result<Vec<organizations::Model>, DbErr> {
    if organization_ids.is_empty() {
      return Ok(vec![]);
    }

    organizations::Entity::find()
      .filter(
        Condition::all()
          .add(organizations::Column::Id.is_in(organization_ids))
          .add(organizations::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_active_organizations<C: ConnectionTrait>(
    conn: &C,
  ) -> Result<Vec<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(organizations::Column::DeletedAt.is_null())
      .order_by_desc(organizations::Column::CreatedAt)
      .all(conn)
      .await
  }

  pub async fn find_organization_by_key<C: ConnectionTrait>(
    conn: &C,
    key: &str,
  ) -> Result<Option<organizations::Model>, DbErr> {
    organizations::Entity::find()
      .filter(organizations::Column::Key.eq(key.to_string()))
      .one(conn)
      .await
  }

  pub async fn insert_organization<C: ConnectionTrait>(
    conn: &C,
    active: organizations::ActiveModel,
  ) -> Result<organizations::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_organization<C: ConnectionTrait>(
    conn: &C,
    model: organizations::Model,
    key: Option<String>,
    name: Option<String>,
    status: Option<organizations::OrgStatus>,
    deleted_at: Option<Option<sea_orm::prelude::DateTimeWithTimeZone>>,
  ) -> Result<organizations::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(key) = key {
      active.key = Set(key);
    }
    if let Some(name) = name {
      active.name = Set(name);
    }
    if let Some(status) = status {
      active.status = Set(status);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn find_active_membership<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
    organization_id: &str,
  ) -> Result<Option<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn list_active_memberships_by_user<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
  ) -> Result<Vec<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_active_memberships_by_organization<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
  ) -> Result<Vec<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn find_first_active_membership_by_user<C: ConnectionTrait>(
    conn: &C,
    user_id: &str,
  ) -> Result<Option<organization_members::Model>, DbErr> {
    organization_members::Entity::find()
      .filter(
        Condition::all()
          .add(organization_members::Column::UserId.eq(user_id.to_string()))
          .add(organization_members::Column::DeletedAt.is_null()),
      )
      .order_by_asc(organization_members::Column::CreatedAt)
      .one(conn)
      .await
  }

  pub async fn exists_active_membership<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    user_id: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      organization_members::Entity::find()
        .filter(
          Condition::all()
            .add(organization_members::Column::OrganizationId.eq(organization_id.to_string()))
            .add(organization_members::Column::UserId.eq(user_id.to_string()))
            .add(organization_members::Column::DeletedAt.is_null()),
        )
        .one(conn)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_organization_membership<C: ConnectionTrait>(
    conn: &C,
    active: organization_members::ActiveModel,
  ) -> Result<organization_members::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_organization_membership<C: ConnectionTrait>(
    conn: &C,
    model: organization_members::Model,
    role: Option<organization_members::MemberRole>,
    deleted_at: Option<Option<sea_orm::prelude::DateTimeWithTimeZone>>,
  ) -> Result<organization_members::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(role) = role {
      active.role = Set(role);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn find_pending_invitation_by_org_and_email<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    email: &str,
  ) -> Result<Option<organization_invitations::Model>, DbErr> {
    organization_invitations::Entity::find()
      .filter(
        Condition::all()
          .add(organization_invitations::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_invitations::Column::Email.eq(email.to_string()))
          .add(
            organization_invitations::Column::Status
              .eq(organization_invitations::InvitationStatus::Pending),
          )
          .add(organization_invitations::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn insert_invitation<C: ConnectionTrait>(
    conn: &C,
    active: organization_invitations::ActiveModel,
  ) -> Result<organization_invitations::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn find_active_invitation_by_id<C: ConnectionTrait>(
    conn: &C,
    invitation_id: &str,
  ) -> Result<Option<organization_invitations::Model>, DbErr> {
    organization_invitations::Entity::find_by_id(invitation_id.to_string())
      .filter(organization_invitations::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn find_active_invitation_by_id_and_org<C: ConnectionTrait>(
    conn: &C,
    invitation_id: &str,
    organization_id: &str,
  ) -> Result<Option<organization_invitations::Model>, DbErr> {
    organization_invitations::Entity::find_by_id(invitation_id.to_string())
      .filter(
        Condition::all()
          .add(organization_invitations::Column::OrganizationId.eq(organization_id.to_string()))
          .add(organization_invitations::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn update_invitation<C: ConnectionTrait>(
    conn: &C,
    model: organization_invitations::Model,
    status: organization_invitations::InvitationStatus,
    accepted_by_user_id: Option<String>,
  ) -> Result<organization_invitations::Model, DbErr> {
    let mut active = model.into_active_model();
    active.status = Set(status);
    active.accepted_by_user_id = Set(accepted_by_user_id);
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn expire_pending_invitations_before<C: ConnectionTrait>(
    conn: &C,
    now: DateTime<Utc>,
  ) -> Result<u64, DbErr> {
    let result = organization_invitations::Entity::update_many()
      .col_expr(
        organization_invitations::Column::Status,
        Expr::value(organization_invitations::InvitationStatus::Expired),
      )
      .col_expr(
        organization_invitations::Column::UpdatedAt,
        Expr::value(now),
      )
      .filter(
        organization_invitations::Column::Status
          .eq(organization_invitations::InvitationStatus::Pending),
      )
      .filter(organization_invitations::Column::DeletedAt.is_null())
      .filter(organization_invitations::Column::ExpiresAt.lt(now))
      .exec(conn)
      .await?;

    Ok(result.rows_affected)
  }

  pub async fn find_active_repository_by_id<C: ConnectionTrait>(
    conn: &C,
    repo_id: &str,
  ) -> Result<Option<repositories::Model>, DbErr> {
    repositories::Entity::find_by_id(repo_id.to_string())
      .filter(repositories::Column::DeletedAt.is_null())
      .one(conn)
      .await
  }

  pub async fn find_active_repository_by_org_and_key<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    repo_key: &str,
  ) -> Result<Option<repositories::Model>, DbErr> {
    repositories::Entity::find()
      .filter(
        Condition::all()
          .add(repositories::Column::OrganizationId.eq(organization_id.to_string()))
          .add(repositories::Column::Key.eq(repo_key.to_string()))
          .add(repositories::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn exists_active_repository_by_org_and_key<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
    repo_key: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      Self::find_active_repository_by_org_and_key(conn, organization_id, repo_key)
        .await?
        .is_some(),
    )
  }

  pub async fn list_active_repositories_by_org<C: ConnectionTrait>(
    conn: &C,
    organization_id: &str,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    repositories::Entity::find()
      .filter(
        Condition::all()
          .add(repositories::Column::OrganizationId.eq(organization_id.to_string()))
          .add(repositories::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_active_repositories<C: ConnectionTrait>(
    conn: &C,
    organization_id: Option<&str>,
  ) -> Result<Vec<repositories::Model>, DbErr> {
    let mut query = repositories::Entity::find().filter(repositories::Column::DeletedAt.is_null());
    if let Some(organization_id) = organization_id {
      query = query.filter(repositories::Column::OrganizationId.eq(organization_id.to_string()));
    }

    query
      .order_by_desc(repositories::Column::CreatedAt)
      .all(conn)
      .await
  }

  pub async fn insert_repository<C: ConnectionTrait>(
    conn: &C,
    active: repositories::ActiveModel,
  ) -> Result<repositories::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_repository<C: ConnectionTrait>(
    conn: &C,
    model: repositories::Model,
    description: Option<Option<String>>,
    visibility: Option<repositories::RepositoryVisibility>,
    default_branch: Option<String>,
    deleted_at: Option<Option<sea_orm::prelude::DateTimeWithTimeZone>>,
  ) -> Result<repositories::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(description) = description {
      active.description = Set(description);
    }
    if let Some(visibility) = visibility {
      active.visibility = Set(visibility);
    }
    if let Some(default_branch) = default_branch {
      active.default_branch = Set(default_branch);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn list_active_users_by_ids<C: ConnectionTrait>(
    conn: &C,
    user_ids: Vec<String>,
  ) -> Result<Vec<users::Model>, DbErr> {
    if user_ids.is_empty() {
      return Ok(vec![]);
    }

    users::Entity::find()
      .filter(
        Condition::all()
          .add(users::Column::Id.is_in(user_ids))
          .add(users::Column::DeletedAt.is_null()),
      )
      .all(conn)
      .await
  }

  pub async fn list_repository_branches_by_repo_id<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    include_deleted: bool,
  ) -> Result<Vec<repository_branches::Model>, DbErr> {
    let mut query = repository_branches::Entity::find()
      .filter(repository_branches::Column::RepositoryId.eq(repository_id.to_string()));

    if !include_deleted {
      query = query.filter(repository_branches::Column::DeletedAt.is_null());
    }

    query
      .order_by_asc(repository_branches::Column::Name)
      .all(conn)
      .await
  }

  pub async fn find_active_branch_by_repo_and_name<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<Option<repository_branches::Model>, DbErr> {
    repository_branches::Entity::find()
      .filter(
        Condition::all()
          .add(repository_branches::Column::RepositoryId.eq(repository_id.to_string()))
          .add(repository_branches::Column::Name.eq(branch_name.to_string()))
          .add(repository_branches::Column::DeletedAt.is_null()),
      )
      .one(conn)
      .await
  }

  pub async fn exists_active_branch_by_repo_and_name<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    branch_name: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      Self::find_active_branch_by_repo_and_name(conn, repository_id, branch_name)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_branch<C: ConnectionTrait>(
    conn: &C,
    active: repository_branches::ActiveModel,
  ) -> Result<repository_branches::Model, DbErr> {
    active.insert(conn).await
  }

  pub async fn update_branch<C: ConnectionTrait>(
    conn: &C,
    model: repository_branches::Model,
    is_protected: Option<bool>,
    last_commit_sha: Option<Option<String>>,
    deleted_at: Option<Option<sea_orm::prelude::DateTimeWithTimeZone>>,
  ) -> Result<repository_branches::Model, DbErr> {
    let mut active = model.into_active_model();
    if let Some(is_protected) = is_protected {
      active.is_protected = Set(is_protected);
    }
    if let Some(last_commit_sha) = last_commit_sha {
      active.last_commit_sha = Set(last_commit_sha);
    }
    if let Some(deleted_at) = deleted_at {
      active.deleted_at = Set(deleted_at);
    }
    active.updated_at = Set(Utc::now().into());
    active.update(conn).await
  }

  pub async fn list_commits_by_repo<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    branch_name: Option<String>,
    limit: u64,
  ) -> Result<Vec<repository_commits::Model>, DbErr> {
    let mut finder = repository_commits::Entity::find()
      .filter(repository_commits::Column::RepositoryId.eq(repository_id.to_string()));
    if let Some(branch_name) = branch_name {
      finder = finder.filter(repository_commits::Column::BranchName.eq(branch_name));
    }

    finder
      .order_by_desc(repository_commits::Column::CreatedAt)
      .limit(limit)
      .all(conn)
      .await
  }

  pub async fn exists_commit_by_repo_and_sha<C: ConnectionTrait>(
    conn: &C,
    repository_id: &str,
    commit_sha: &str,
  ) -> Result<bool, DbErr> {
    Ok(
      repository_commits::Entity::find()
        .filter(
          Condition::all()
            .add(repository_commits::Column::RepositoryId.eq(repository_id.to_string()))
            .add(repository_commits::Column::CommitSha.eq(commit_sha.to_string())),
        )
        .one(conn)
        .await?
        .is_some(),
    )
  }

  pub async fn insert_commit<C: ConnectionTrait>(
    conn: &C,
    active: repository_commits::ActiveModel,
  ) -> Result<repository_commits::Model, DbErr> {
    active.insert(conn).await
  }
}
