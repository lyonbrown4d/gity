use gity_core::AppError;
use gity_entities::{
  audit_revisions, project_branch_protection_audits, project_branch_protections,
  project_ci_variable_audits, project_ci_variables, project_deploy_key_audits, project_deploy_keys,
  project_member_audits, project_members, project_merge_request_approval_rule_audits,
  project_merge_request_approval_rules,
};
use sea_orm::{ActiveModelTrait, ConnectionTrait, Set};

async fn revision<C: ConnectionTrait>(
  connection: &C,
  project: i64,
  operation: &str,
  actor: Option<i64>,
) -> Result<i64, AppError> {
  let id = gity_core::new_id();
  audit_revisions::ActiveModel {
    id: Set(id),
    created_at: Set(chrono::Utc::now().naive_utc()),
    actor: Set(
      actor
        .or_else(gity_core::audit_actor)
        .map(|id| id.to_string()),
    ),
    tenant: Set(Some(project.to_string())),
    reason: Set(Some(operation.to_owned())),
    metadata: Set(Some("{}".to_owned())),
  }
  .insert(connection)
  .await?;
  Ok(id)
}

// Each invocation is an explicit, reviewed allowlist. Secret fields are never
// included; callers pass the same transaction used for the entity mutation.
macro_rules! audit_snapshot {
  ($function:ident, $entity:ident, $snapshot:ident, $entity_id:ident, [$($field:ident),+ $(,)?]) => {
    pub async fn $function<C: ConnectionTrait>(
      connection: &C,
      model: &$entity::Model,
      operation: &str,
      actor: Option<i64>,
    ) -> Result<(), AppError> {
      let revision = revision(connection, model.project_id, operation, actor).await?;
      $snapshot::ActiveModel {
        revision_id: Set(revision),
        operation: Set(operation.to_owned()),
        $entity_id: Set(model.id),
        $($field: Set(model.$field.to_owned()),)+
      }.insert(connection).await?;
      Ok(())
    }
  };
}

audit_snapshot!(
  audit_member,
  project_members,
  project_member_audits,
  project_member_id,
  [project_id, user_id, role, created_at, updated_at]
);
audit_snapshot!(
  audit_protection,
  project_branch_protections,
  project_branch_protection_audits,
  project_branch_protection_id,
  [
    project_id,
    branch_name,
    rule_type,
    push_access_level,
    merge_access_level,
    require_merge_request,
    require_pipeline_success,
    allow_force_push,
    allow_delete,
    created_at,
    updated_at
  ]
);
audit_snapshot!(
  audit_rule,
  project_merge_request_approval_rules,
  project_merge_request_approval_rule_audits,
  project_merge_request_approval_rule_id,
  [
    project_id,
    name,
    target_branch,
    approvals_required,
    eligible_user_ids,
    code_owner,
    created_at,
    updated_at
  ]
);
audit_snapshot!(
  audit_variable,
  project_ci_variables,
  project_ci_variable_audits,
  project_ci_variable_id,
  [project_id, key, masked, protected, created_at, updated_at]
);
audit_snapshot!(
  audit_deploy_key,
  project_deploy_keys,
  project_deploy_key_audits,
  project_deploy_key_id,
  [
    project_id,
    title,
    fingerprint,
    can_push,
    created_by_user_id,
    last_used_at,
    created_at,
    updated_at
  ]
);
