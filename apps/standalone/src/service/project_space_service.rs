use crate::service::git_backend_service::{GitBackendError, GitBackendService};
use application::namespace::{
  CreateNamespaceCommand, NamespaceInvitationView, NamespaceMemberView, NamespaceView,
  UpdateNamespaceCommand,
};
use application::project::{CreateProjectCommand, ProjectView};
use chrono::{Duration, Utc};
use models::constants::{
  invitation_state, member_role, namespace_kind, record_state, visibility_level,
};
use models::gitlab::{Namespace, NamespaceInvitation, NamespaceMember, Project};
use toasty::Db;
#[derive(Clone)]
pub struct ProjectSpaceService {
  db: Db,
  git_backend: GitBackendService,
}

#[derive(Debug, Clone)]
pub enum ProjectSpaceError {
  BadRequest(String),
  NotFound(String),
  Conflict(String),
  Forbidden(String),
  Internal(String),
}

impl ProjectSpaceService {
  pub fn new(db: Db, git_backend: GitBackendService) -> Self {
    Self { db, git_backend }
  }

  pub async fn create_namespace(
    &self,
    mut command: CreateNamespaceCommand,
  ) -> Result<NamespaceView, ProjectSpaceError> {
    command.path_key = normalize_path_key(command.path_key.as_str())?;
    command.name = normalize_non_empty(command.name.as_str(), "name")?;
    command.kind = normalize_namespace_kind(command.kind.as_str())?;
    command.visibility = normalize_visibility(command.visibility.as_str())?;
    command.description = normalize_optional(command.description);
    command.owner_user_id = normalize_optional(command.owner_user_id);

    let parent_full_path = match command.parent_namespace_id {
      Some(parent_id) => {
        let parent = self
          .get_namespace_model(parent_id)
          .await?
          .ok_or_else(|| ProjectSpaceError::NotFound("parent namespace not found".to_string()))?;
        Some(parent.full_path)
      }
      None => None,
    };
    let full_path = build_full_path(parent_full_path.as_deref(), command.path_key.as_str());

    if self
      .find_namespace_by_full_path(full_path.as_str())
      .await?
      .is_some()
    {
      return Err(ProjectSpaceError::Conflict(
        "namespace path already exists".to_string(),
      ));
    }

    let mut db = self.db.clone();
    let namespace = toasty::create!(Namespace {
      full_path: full_path,
      parent_namespace_id: command.parent_namespace_id,
      owner_user_id: command.owner_user_id.clone(),
      path_key: command.path_key,
      name: command.name,
      description: command.description,
      kind: command.kind,
      visibility: command.visibility,
    })
    .exec(&mut db)
    .await
    .map_err(map_db_error)?;

    if let Some(owner_user_id) = command.owner_user_id {
      let _ = self
        .add_namespace_member(
          namespace.id,
          owner_user_id.as_str(),
          member_role::OWNER,
          Some(
            namespace
              .owner_user_id
              .as_deref()
              .unwrap_or(owner_user_id.as_str()),
          ),
        )
        .await;
    }

    Ok(namespace_to_view(namespace))
  }

  pub async fn update_namespace(
    &self,
    command: UpdateNamespaceCommand,
  ) -> Result<NamespaceView, ProjectSpaceError> {
    let mut namespace = self
      .get_namespace_model(command.namespace_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("namespace not found".to_string()))?;

    let name = command
      .name
      .map(|value| normalize_non_empty(value.as_str(), "name"))
      .transpose()?;
    let path_key = command
      .path_key
      .map(|value| normalize_path_key(value.as_str()))
      .transpose()?;
    let description = command
      .description
      .map(|value| value.and_then(normalize_optional_value));
    let visibility = command
      .visibility
      .map(|value| normalize_visibility(value.as_str()))
      .transpose()?;

    if path_key
      .as_ref()
      .is_some_and(|value| value != &namespace.path_key)
    {
      let has_children = self.has_child_namespaces(namespace.id).await?;
      let has_projects = self.has_projects(namespace.id).await?;
      if has_children || has_projects {
        return Err(ProjectSpaceError::BadRequest(
          "cannot change namespace path while child namespaces or projects exist".to_string(),
        ));
      }
    }

    let resolved_path_key = path_key.unwrap_or_else(|| namespace.path_key.clone());
    let resolved_name = name.unwrap_or_else(|| namespace.name.clone());
    let resolved_description = description.unwrap_or(namespace.description.clone());
    let resolved_visibility = visibility.unwrap_or_else(|| namespace.visibility.clone());
    let resolved_full_path = match namespace.parent_namespace_id {
      Some(parent_id) => {
        let parent = self
          .get_namespace_model(parent_id)
          .await?
          .ok_or_else(|| ProjectSpaceError::NotFound("parent namespace not found".to_string()))?;
        build_full_path(Some(parent.full_path.as_str()), resolved_path_key.as_str())
      }
      None => build_full_path(None, resolved_path_key.as_str()),
    };

    if resolved_full_path != namespace.full_path
      && self
        .find_namespace_by_full_path(resolved_full_path.as_str())
        .await?
        .is_some()
    {
      return Err(ProjectSpaceError::Conflict(
        "namespace path already exists".to_string(),
      ));
    }

    let mut db = self.db.clone();
    namespace
      .update()
      .full_path(resolved_full_path)
      .path_key(resolved_path_key)
      .name(resolved_name)
      .description(resolved_description)
      .visibility(resolved_visibility)
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;

    let updated = self
      .get_namespace_model(namespace.id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("namespace not found".to_string()))?;
    Ok(namespace_to_view(updated))
  }

  pub async fn delete_namespace(&self, namespace_id: i64) -> Result<(), ProjectSpaceError> {
    let namespace = self
      .get_namespace_model(namespace_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("namespace not found".to_string()))?;

    if self.has_child_namespaces(namespace.id).await? || self.has_projects(namespace.id).await? {
      return Err(ProjectSpaceError::BadRequest(
        "cannot delete namespace with child namespaces or projects".to_string(),
      ));
    }

    let mut db = self.db.clone();
    NamespaceMember::filter(NamespaceMember::fields().namespace_id().eq(namespace.id))
      .delete()
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;
    Namespace::filter(Namespace::fields().id().eq(namespace.id))
      .delete()
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;
    Ok(())
  }

  pub async fn list_namespaces(&self) -> Result<Vec<NamespaceView>, ProjectSpaceError> {
    let mut db = self.db.clone();
    let query = Namespace::all().order_by(Namespace::fields().id().asc());
    let items = query.exec(&mut db).await.map_err(map_db_error)?;
    Ok(items.into_iter().map(namespace_to_view).collect())
  }

  pub async fn list_group_namespaces_for_user(
    &self,
    user_id: &str,
    is_super_admin: bool,
  ) -> Result<Vec<(NamespaceView, String)>, ProjectSpaceError> {
    let items = self.list_namespaces().await?;
    let mut output = Vec::new();

    for namespace in items
      .into_iter()
      .filter(|item| item.kind == namespace_kind::GROUP)
    {
      if is_super_admin {
        output.push((namespace, "super_admin".to_string()));
        continue;
      }

      if namespace.owner_user_id.as_deref() == Some(user_id) {
        output.push((namespace, member_role::OWNER.to_string()));
        continue;
      }

      if let Some(role) = self.get_namespace_role(namespace.id, user_id).await? {
        output.push((namespace, role));
      }
    }

    Ok(output)
  }

  pub async fn get_namespace(
    &self,
    namespace_id: i64,
  ) -> Result<Option<NamespaceView>, ProjectSpaceError> {
    Ok(
      self
        .get_namespace_model(namespace_id)
        .await?
        .map(namespace_to_view),
    )
  }

  pub async fn get_namespace_by_full_path(
    &self,
    full_path: &str,
  ) -> Result<Option<NamespaceView>, ProjectSpaceError> {
    Ok(
      self
        .find_namespace_by_full_path(full_path)
        .await?
        .map(namespace_to_view),
    )
  }

  pub async fn add_namespace_member(
    &self,
    namespace_id: i64,
    user_id: &str,
    role: &str,
    actor_user_id: Option<&str>,
  ) -> Result<NamespaceMemberView, ProjectSpaceError> {
    let namespace = self
      .get_namespace_model(namespace_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("namespace not found".to_string()))?;
    let user_id = normalize_non_empty(user_id, "user_id")?;
    let role = normalize_member_role(role)?;

    if let Some(actor_user_id) = actor_user_id {
      if !self
        .user_has_namespace_role(
          namespace.id,
          actor_user_id,
          &[member_role::OWNER, member_role::MAINTAINER],
        )
        .await?
      {
        return Err(ProjectSpaceError::Forbidden(
          "namespace owner or maintainer permission is required".to_string(),
        ));
      }
    }

    if let Some(mut existing) = self
      .find_namespace_member(namespace.id, user_id.as_str())
      .await?
    {
      let mut db = self.db.clone();
      existing
        .update()
        .role(role)
        .state(record_state::ACTIVE.to_string())
        .exec(&mut db)
        .await
        .map_err(map_db_error)?;
      let updated = self
        .find_namespace_member(namespace.id, user_id.as_str())
        .await?
        .ok_or_else(|| ProjectSpaceError::NotFound("namespace member not found".to_string()))?;
      return Ok(namespace_member_to_view(updated));
    }

    let mut db = self.db.clone();
    let member = toasty::create!(NamespaceMember {
      namespace_id: namespace.id,
      user_id: user_id,
      role: role,
      state: record_state::ACTIVE,
    })
    .exec(&mut db)
    .await
    .map_err(map_db_error)?;

    Ok(namespace_member_to_view(member))
  }

  pub async fn list_namespace_members(
    &self,
    namespace_id: i64,
  ) -> Result<Vec<NamespaceMemberView>, ProjectSpaceError> {
    let mut db = self.db.clone();
    let query = NamespaceMember::filter(NamespaceMember::fields().namespace_id().eq(namespace_id))
      .order_by(NamespaceMember::fields().id().asc());
    let items = query.exec(&mut db).await.map_err(map_db_error)?;
    Ok(items.into_iter().map(namespace_member_to_view).collect())
  }

  pub async fn create_namespace_invitation(
    &self,
    namespace_id: i64,
    email: &str,
    role: &str,
    expires_in_hours: Option<i64>,
    invited_by_user_id: &str,
  ) -> Result<NamespaceInvitationView, ProjectSpaceError> {
    let namespace = self
      .get_namespace_model(namespace_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("namespace not found".to_string()))?;
    let email = normalize_email(email)?;
    let role = normalize_member_role(role)?;
    let invited_by_user_id = normalize_non_empty(invited_by_user_id, "invited_by_user_id")?;

    if !self
      .user_has_namespace_role(
        namespace.id,
        invited_by_user_id.as_str(),
        &[member_role::OWNER, member_role::MAINTAINER],
      )
      .await?
    {
      return Err(ProjectSpaceError::Forbidden(
        "namespace owner or maintainer permission is required".to_string(),
      ));
    }

    if self
      .find_pending_namespace_invitation(namespace.id, email.as_str())
      .await?
      .is_some()
    {
      return Err(ProjectSpaceError::Conflict(
        "pending invitation already exists for this email".to_string(),
      ));
    }

    let expires_in_hours = expires_in_hours.unwrap_or(72);
    if !(1..=24 * 30).contains(&expires_in_hours) {
      return Err(ProjectSpaceError::BadRequest(
        "expires_in_hours must be between 1 and 720".to_string(),
      ));
    }
    let expires_at_unix = Some((Utc::now() + Duration::hours(expires_in_hours)).timestamp());

    let mut db = self.db.clone();
    let invitation = toasty::create!(NamespaceInvitation {
      namespace_id: namespace.id,
      email: email,
      role: role,
      state: invitation_state::PENDING,
      invited_by_user_id: invited_by_user_id,
      accepted_by_user_id: None,
      expires_at_unix: expires_at_unix,
    })
    .exec(&mut db)
    .await
    .map_err(map_db_error)?;

    Ok(namespace_invitation_to_view(invitation))
  }

  pub async fn accept_namespace_invitation(
    &self,
    invitation_id: i64,
    current_user_id: &str,
    current_user_email: &str,
    expected_email: Option<&str>,
    expected_namespace_id: Option<i64>,
  ) -> Result<NamespaceMemberView, ProjectSpaceError> {
    let current_user_id = normalize_non_empty(current_user_id, "current_user_id")?;
    let current_user_email = normalize_email(current_user_email)?;
    let invitation = self
      .find_namespace_invitation_by_id(invitation_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("invitation not found".to_string()))?;

    if invitation.state != invitation_state::PENDING {
      return Err(ProjectSpaceError::Conflict(
        "invitation is not pending".to_string(),
      ));
    }

    if let Some(expected_namespace_id) = expected_namespace_id
      && invitation.namespace_id != expected_namespace_id
    {
      return Err(ProjectSpaceError::Forbidden(
        "invitation token organization mismatch".to_string(),
      ));
    }

    if let Some(expected_email) = expected_email {
      let expected_email = normalize_email(expected_email)?;
      if invitation.email != expected_email {
        return Err(ProjectSpaceError::Forbidden(
          "invitation token email mismatch".to_string(),
        ));
      }
    }

    if invitation.email != current_user_email {
      return Err(ProjectSpaceError::Forbidden(
        "invitation email does not match current user".to_string(),
      ));
    }

    if invitation
      .expires_at_unix
      .is_some_and(|expires_at_unix| expires_at_unix < Utc::now().timestamp())
    {
      let mut expired_invitation = invitation;
      let mut db = self.db.clone();
      expired_invitation
        .update()
        .state(invitation_state::EXPIRED.to_string())
        .exec(&mut db)
        .await
        .map_err(map_db_error)?;
      return Err(ProjectSpaceError::Conflict(
        "invitation has expired".to_string(),
      ));
    }

    let member = self
      .add_namespace_member(
        invitation.namespace_id,
        current_user_id.as_str(),
        invitation.role.as_str(),
        None,
      )
      .await?;

    let mut accepted_invitation = self
      .find_namespace_invitation_by_id(invitation_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("invitation not found".to_string()))?;
    let mut db = self.db.clone();
    accepted_invitation
      .update()
      .state(invitation_state::ACCEPTED.to_string())
      .accepted_by_user_id(Some(current_user_id))
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;

    Ok(member)
  }

  pub async fn revoke_namespace_invitation(
    &self,
    namespace_id: i64,
    invitation_id: i64,
    actor_user_id: &str,
  ) -> Result<(), ProjectSpaceError> {
    let actor_user_id = normalize_non_empty(actor_user_id, "actor_user_id")?;
    if !self
      .user_has_namespace_role(namespace_id, actor_user_id.as_str(), &[member_role::OWNER])
      .await?
    {
      return Err(ProjectSpaceError::Forbidden(
        "namespace owner permission is required".to_string(),
      ));
    }

    let mut invitation = self
      .find_namespace_invitation_by_id(invitation_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("invitation not found".to_string()))?;

    if invitation.namespace_id != namespace_id {
      return Err(ProjectSpaceError::NotFound(
        "invitation not found".to_string(),
      ));
    }

    if invitation.state != invitation_state::PENDING {
      return Err(ProjectSpaceError::Conflict(
        "only pending invitation can be revoked".to_string(),
      ));
    }

    let mut db = self.db.clone();
    invitation
      .update()
      .state(invitation_state::REVOKED.to_string())
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;

    Ok(())
  }

  pub async fn get_namespace_role(
    &self,
    namespace_id: i64,
    user_id: &str,
  ) -> Result<Option<String>, ProjectSpaceError> {
    let namespace = self.get_namespace_model(namespace_id).await?;
    let Some(namespace) = namespace else {
      return Ok(None);
    };

    if namespace.owner_user_id.as_deref() == Some(user_id) {
      return Ok(Some(member_role::OWNER.to_string()));
    }

    Ok(
      self
        .find_namespace_member(namespace.id, user_id)
        .await?
        .map(|item| item.role),
    )
  }

  pub async fn user_has_namespace_role(
    &self,
    namespace_id: i64,
    user_id: &str,
    accepted_roles: &[&str],
  ) -> Result<bool, ProjectSpaceError> {
    let Some(role) = self.get_namespace_role(namespace_id, user_id).await? else {
      return Ok(false);
    };
    Ok(accepted_roles.iter().any(|item| *item == role))
  }

  pub async fn create_project(
    &self,
    mut command: CreateProjectCommand,
  ) -> Result<ProjectView, ProjectSpaceError> {
    command.path_key = normalize_path_key(command.path_key.as_str())?;
    command.name = normalize_non_empty(command.name.as_str(), "name")?;
    command.visibility = normalize_visibility(command.visibility.as_str())?;
    command.description = normalize_optional(command.description);
    command.actor_user_id = normalize_non_empty(command.actor_user_id.as_str(), "actor_user_id")?;
    let default_branch = command
      .default_branch
      .take()
      .map(|value| normalize_non_empty(value.as_str(), "default_branch"))
      .transpose()?
      .unwrap_or_else(|| "main".to_string());

    let namespace = self
      .get_namespace_model(command.namespace_id)
      .await?
      .ok_or_else(|| ProjectSpaceError::NotFound("namespace not found".to_string()))?;
    let full_path = build_full_path(
      Some(namespace.full_path.as_str()),
      command.path_key.as_str(),
    );

    if self
      .find_project_by_full_path(full_path.as_str())
      .await?
      .is_some()
    {
      return Err(ProjectSpaceError::Conflict(
        "project path already exists".to_string(),
      ));
    }

    let mut db = self.db.clone();
    let project = toasty::create!(Project {
      namespace_id: command.namespace_id,
      full_path: full_path,
      path_key: command.path_key,
      name: command.name,
      description: command.description,
      visibility: command.visibility,
      default_branch: default_branch,
      archived: false,
      created_by_user_id: command.actor_user_id,
    })
    .exec(&mut db)
    .await
    .map_err(map_db_error)?;

    if let Err(err) = self.initialize_project_storage(&namespace, &project).await {
      self.delete_project_record(project.id).await;
      return Err(map_git_backend_error(err));
    }

    Ok(project_to_view(project))
  }

  pub async fn list_projects(&self) -> Result<Vec<ProjectView>, ProjectSpaceError> {
    let mut db = self.db.clone();
    let query = Project::all().order_by(Project::fields().id().asc());
    let items = query.exec(&mut db).await.map_err(map_db_error)?;
    Ok(items.into_iter().map(project_to_view).collect())
  }

  pub async fn get_project(
    &self,
    project_id: i64,
  ) -> Result<Option<ProjectView>, ProjectSpaceError> {
    Ok(
      self
        .get_project_model(project_id)
        .await?
        .map(project_to_view),
    )
  }

  pub async fn get_project_by_full_path(
    &self,
    full_path: &str,
  ) -> Result<Option<ProjectView>, ProjectSpaceError> {
    Ok(
      self
        .find_project_by_full_path(full_path)
        .await?
        .map(project_to_view),
    )
  }

  async fn get_namespace_model(
    &self,
    namespace_id: i64,
  ) -> Result<Option<Namespace>, ProjectSpaceError> {
    let mut db = self.db.clone();
    Namespace::filter(Namespace::fields().id().eq(namespace_id))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)
  }

  async fn find_namespace_by_full_path(
    &self,
    full_path: &str,
  ) -> Result<Option<Namespace>, ProjectSpaceError> {
    let mut db = self.db.clone();
    Namespace::filter(Namespace::fields().full_path().eq(full_path))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)
  }

  async fn has_child_namespaces(&self, namespace_id: i64) -> Result<bool, ProjectSpaceError> {
    let mut db = self.db.clone();
    let item = Namespace::filter(Namespace::fields().parent_namespace_id().eq(namespace_id))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;
    Ok(item.is_some())
  }

  async fn has_projects(&self, namespace_id: i64) -> Result<bool, ProjectSpaceError> {
    let mut db = self.db.clone();
    let item = Project::filter(Project::fields().namespace_id().eq(namespace_id))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)?;
    Ok(item.is_some())
  }

  async fn find_namespace_member(
    &self,
    namespace_id: i64,
    user_id: &str,
  ) -> Result<Option<NamespaceMember>, ProjectSpaceError> {
    let mut db = self.db.clone();
    NamespaceMember::filter(NamespaceMember::fields().namespace_id().eq(namespace_id))
      .filter(NamespaceMember::fields().user_id().eq(user_id))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)
  }

  async fn find_pending_namespace_invitation(
    &self,
    namespace_id: i64,
    email: &str,
  ) -> Result<Option<NamespaceInvitation>, ProjectSpaceError> {
    let mut db = self.db.clone();
    NamespaceInvitation::filter(
      NamespaceInvitation::fields()
        .namespace_id()
        .eq(namespace_id),
    )
    .filter(NamespaceInvitation::fields().email().eq(email))
    .filter(
      NamespaceInvitation::fields()
        .state()
        .eq(invitation_state::PENDING),
    )
    .first()
    .exec(&mut db)
    .await
    .map_err(map_db_error)
  }

  async fn find_namespace_invitation_by_id(
    &self,
    invitation_id: i64,
  ) -> Result<Option<NamespaceInvitation>, ProjectSpaceError> {
    let mut db = self.db.clone();
    NamespaceInvitation::filter(NamespaceInvitation::fields().id().eq(invitation_id))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)
  }

  async fn get_project_model(&self, project_id: i64) -> Result<Option<Project>, ProjectSpaceError> {
    let mut db = self.db.clone();
    Project::filter(Project::fields().id().eq(project_id))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)
  }

  async fn find_project_by_full_path(
    &self,
    full_path: &str,
  ) -> Result<Option<Project>, ProjectSpaceError> {
    let mut db = self.db.clone();
    Project::filter(Project::fields().full_path().eq(full_path))
      .first()
      .exec(&mut db)
      .await
      .map_err(map_db_error)
  }

  async fn initialize_project_storage(
    &self,
    namespace: &Namespace,
    project: &Project,
  ) -> Result<(), GitBackendError> {
    match self
      .git_backend
      .init_bare_repository_storage(
        namespace.full_path.as_str(),
        project.path_key.as_str(),
        project.default_branch.as_str(),
      )
      .await
    {
      Ok(()) => {}
      Err(GitBackendError::StorageNotConfigured) => return Ok(()),
      Err(err) => return Err(err),
    }

    self
      .git_backend
      .seed_initial_commit(
        namespace.full_path.as_str(),
        project.path_key.as_str(),
        project.default_branch.as_str(),
        vec![(
          "README.md".to_string(),
          format!("# {}\n\nInitialized by Gity.\n", project.name),
        )],
        "Initialize project repository",
      )
      .await?;

    Ok(())
  }

  async fn delete_project_record(&self, project_id: i64) {
    let mut db = self.db.clone();
    let _ = Project::filter(Project::fields().id().eq(project_id))
      .delete()
      .exec(&mut db)
      .await;
  }
}

fn normalize_path_key(value: &str) -> Result<String, ProjectSpaceError> {
  let normalized = value.trim().to_ascii_lowercase();
  if normalized.is_empty() {
    return Err(ProjectSpaceError::BadRequest(
      "path_key is required".to_string(),
    ));
  }
  if !normalized
    .chars()
    .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.'))
  {
    return Err(ProjectSpaceError::BadRequest(
      "path_key contains unsupported characters".to_string(),
    ));
  }
  Ok(normalized)
}

fn normalize_non_empty(value: &str, field: &str) -> Result<String, ProjectSpaceError> {
  let normalized = value.trim().to_string();
  if normalized.is_empty() {
    return Err(ProjectSpaceError::BadRequest(format!(
      "{field} is required"
    )));
  }
  Ok(normalized)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
  value.and_then(normalize_optional_value)
}

fn normalize_optional_value(value: String) -> Option<String> {
  let trimmed = value.trim().to_string();
  if trimmed.is_empty() {
    None
  } else {
    Some(trimmed)
  }
}

fn normalize_namespace_kind(value: &str) -> Result<String, ProjectSpaceError> {
  let normalized = value.trim().to_ascii_lowercase();
  match normalized.as_str() {
    namespace_kind::USER | namespace_kind::GROUP => Ok(normalized),
    _ => Err(ProjectSpaceError::BadRequest(
      "kind must be user or group".to_string(),
    )),
  }
}

fn normalize_visibility(value: &str) -> Result<String, ProjectSpaceError> {
  let normalized = value.trim().to_ascii_lowercase();
  match normalized.as_str() {
    visibility_level::PRIVATE | visibility_level::INTERNAL | visibility_level::PUBLIC => {
      Ok(normalized)
    }
    _ => Err(ProjectSpaceError::BadRequest(
      "visibility must be private, internal, or public".to_string(),
    )),
  }
}

fn normalize_email(value: &str) -> Result<String, ProjectSpaceError> {
  let normalized = value.trim().to_ascii_lowercase();
  if normalized.is_empty() || !normalized.contains('@') {
    return Err(ProjectSpaceError::BadRequest(
      "email is required".to_string(),
    ));
  }
  Ok(normalized)
}

fn normalize_member_role(value: &str) -> Result<String, ProjectSpaceError> {
  let normalized = value.trim().to_ascii_lowercase();
  match normalized.as_str() {
    member_role::GUEST
    | member_role::REPORTER
    | member_role::DEVELOPER
    | member_role::MAINTAINER
    | member_role::OWNER => Ok(normalized),
    _ => Err(ProjectSpaceError::BadRequest(
      "role must be guest, reporter, developer, maintainer, or owner".to_string(),
    )),
  }
}

fn build_full_path(parent_full_path: Option<&str>, path_key: &str) -> String {
  match parent_full_path {
    Some(parent) if !parent.trim().is_empty() => format!("{parent}/{path_key}"),
    _ => path_key.to_string(),
  }
}

fn namespace_to_view(value: Namespace) -> NamespaceView {
  NamespaceView {
    id: value.id,
    full_path: value.full_path,
    parent_namespace_id: value.parent_namespace_id,
    owner_user_id: value.owner_user_id,
    path_key: value.path_key,
    name: value.name,
    description: value.description,
    kind: value.kind,
    visibility: value.visibility,
  }
}

fn namespace_member_to_view(value: NamespaceMember) -> NamespaceMemberView {
  NamespaceMemberView {
    id: value.id,
    namespace_id: value.namespace_id,
    user_id: value.user_id,
    role: value.role,
    state: value.state,
  }
}

fn namespace_invitation_to_view(value: NamespaceInvitation) -> NamespaceInvitationView {
  NamespaceInvitationView {
    id: value.id,
    namespace_id: value.namespace_id,
    email: value.email,
    role: value.role,
    state: value.state,
    invited_by_user_id: value.invited_by_user_id,
    accepted_by_user_id: value.accepted_by_user_id,
    expires_at_unix: value.expires_at_unix,
  }
}

fn project_to_view(value: Project) -> ProjectView {
  ProjectView {
    id: value.id,
    namespace_id: value.namespace_id,
    full_path: value.full_path,
    path_key: value.path_key,
    name: value.name,
    description: value.description,
    visibility: value.visibility,
    default_branch: value.default_branch,
    archived: value.archived,
    created_by_user_id: value.created_by_user_id,
  }
}

fn map_db_error(err: toasty::Error) -> ProjectSpaceError {
  let message = err.to_string();
  let normalized = message.to_ascii_lowercase();
  if normalized.contains("unique")
    || normalized.contains("duplicate")
    || normalized.contains("already exists")
  {
    ProjectSpaceError::Conflict(message)
  } else {
    ProjectSpaceError::BadRequest(message)
  }
}

fn map_git_backend_error(err: GitBackendError) -> ProjectSpaceError {
  match err {
    GitBackendError::AlreadyExists(path) => ProjectSpaceError::Conflict(format!(
      "project repository storage already exists at {}",
      path.to_string_lossy()
    )),
    GitBackendError::InvalidComponent(message) => ProjectSpaceError::BadRequest(message),
    GitBackendError::StorageNotConfigured => {
      ProjectSpaceError::Internal("storage.repo_root is not configured".to_string())
    }
    GitBackendError::InvalidRepositoryPath
    | GitBackendError::RepositoryNotFound
    | GitBackendError::Io(_)
    | GitBackendError::Git(_)
    | GitBackendError::Db(_)
    | GitBackendError::Utf8(_) => ProjectSpaceError::Internal(err.to_string()),
  }
}
