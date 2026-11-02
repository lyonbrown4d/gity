//! Immutable typed snapshot of the latest Rust business schema.

use sea_orm_migration::prelude::*;

mod columns;
mod metadata;

// One dependency-ordered list declares modules and runs their typed definitions.
macro_rules! schema_modules {
  ($($module:ident),+ $(,)?) => {
    $(mod $module;)+

    pub(super) async fn create(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
      $($module::create(manager).await?;)+
      Ok(())
    }
  };
}

schema_modules! {
  audit_revisions,
  auth_sessions,
  bootstrap_claim,
  organizations,
  user_passwords,
  users,
  artifact_storage_deletions,
  git_repository_placements,
  project_registry_settings,
  organization_members,
  project_branch_protection_audits,
  project_ci_variable_audits,
  project_deploy_key_audits,
  project_member_audits,
  project_merge_request_approval_rule_audits,
  projects,
  user_access_tokens,
  project_access_tokens,
  project_audit_events,
  project_branch_protections,
  project_ci_variables,
  project_deploy_keys,
  project_iid_counters,
  project_issues,
  project_jobs,
  project_lfs_locks,
  project_lfs_objects,
  project_members,
  project_merge_request_approval_rules,
  project_merge_requests,
  project_packages,
  project_pipelines,
  project_releases,
  project_runners,
  project_todos,
  project_wiki_pages,
  project_milestones,
  project_labels,
  project_issue_assignees,
  project_issue_attachments,
  project_issue_comments,
  project_job_artifacts,
  project_job_logs,
  project_merge_request_approvals,
  project_merge_request_comments,
  project_merge_request_participants,
  project_package_versions,
  project_pipeline_jobs,
  project_release_links,
  project_npm_dist_tags,
  project_label_links,
  project_package_files,
}
