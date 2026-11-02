//! SeaORM entity mappings generated from the versioned initial schema.
//! Public API models remain in the HTTP/OpenAPI layer.

mod application_id;
pub(crate) use application_id::application_ids;

pub mod artifact_storage_deletions;
pub mod audit_revisions;
pub mod auth_sessions;
pub mod bootstrap_claim;
pub mod organization_members;
pub mod organizations;
pub mod project_access_tokens;
pub mod project_audit_events;
pub mod project_branch_protection_audits;
pub mod project_branch_protections;
pub mod project_ci_variable_audits;
pub mod project_ci_variables;
pub mod project_deploy_key_audits;
pub mod project_deploy_keys;
pub mod project_iid_counters;
pub mod project_issue_assignees;
pub mod project_issue_attachments;
pub mod project_issue_comments;
pub mod project_issues;
pub mod project_job_artifacts;
pub mod project_job_logs;
pub mod project_jobs;
pub mod project_label_links;
pub mod project_labels;
pub mod project_lfs_locks;
pub mod project_lfs_objects;
pub mod project_member_audits;
pub mod project_members;
pub mod project_merge_request_approval_rule_audits;
pub mod project_merge_request_approval_rules;
pub mod project_merge_request_approvals;
pub mod project_merge_request_comments;
pub mod project_merge_request_participants;
pub mod project_merge_requests;
pub mod project_milestones;
pub mod project_npm_dist_tags;
pub mod project_package_files;
pub mod project_package_versions;
pub mod project_packages;
pub mod project_pipeline_jobs;
pub mod project_pipelines;
pub mod project_release_links;
pub mod project_releases;
pub mod project_runners;
pub mod project_todos;
pub mod project_wiki_pages;
pub mod projects;
pub mod user_access_tokens;
pub mod user_passwords;
pub mod users;

pub mod git_repository_placements;
pub mod project_registry_settings;

application_ids!(
  artifact_storage_deletions,
  audit_revisions,
  organization_members,
  organizations,
  project_access_tokens,
  project_audit_events,
  project_branch_protections,
  project_ci_variables,
  project_deploy_keys,
  project_issue_assignees,
  project_issue_attachments,
  project_issue_comments,
  project_issues,
  project_job_artifacts,
  project_job_logs,
  project_jobs,
  project_label_links,
  project_labels,
  project_lfs_locks,
  project_lfs_objects,
  project_members,
  project_merge_request_approval_rules,
  project_merge_request_approvals,
  project_merge_request_comments,
  project_merge_request_participants,
  project_merge_requests,
  project_milestones,
  project_npm_dist_tags,
  project_package_files,
  project_package_versions,
  project_packages,
  project_pipeline_jobs,
  project_pipelines,
  project_release_links,
  project_releases,
  project_runners,
  project_todos,
  project_wiki_pages,
  projects,
  user_access_tokens,
  users
);
