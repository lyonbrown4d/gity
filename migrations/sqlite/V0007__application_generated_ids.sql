-- SQLite INTEGER PRIMARY KEY otherwise silently allocates ROWIDs when omitted.

CREATE TRIGGER trg_artifact_storage_deletions_id_required
BEFORE INSERT ON artifact_storage_deletions
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'artifact_storage_deletions.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_audit_revisions_id_required
BEFORE INSERT ON audit_revisions
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'audit_revisions.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_bootstrap_claim_id_required
BEFORE INSERT ON bootstrap_claim
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'bootstrap_claim.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_organization_members_id_required
BEFORE INSERT ON organization_members
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'organization_members.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_organizations_id_required
BEFORE INSERT ON organizations
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'organizations.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_access_tokens_id_required
BEFORE INSERT ON project_access_tokens
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_access_tokens.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_audit_events_id_required
BEFORE INSERT ON project_audit_events
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_audit_events.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_branch_protections_id_required
BEFORE INSERT ON project_branch_protections
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_branch_protections.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_ci_variables_id_required
BEFORE INSERT ON project_ci_variables
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_ci_variables.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_deploy_keys_id_required
BEFORE INSERT ON project_deploy_keys
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_deploy_keys.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_issue_assignees_id_required
BEFORE INSERT ON project_issue_assignees
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_issue_assignees.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_issue_attachments_id_required
BEFORE INSERT ON project_issue_attachments
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_issue_attachments.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_issue_comments_id_required
BEFORE INSERT ON project_issue_comments
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_issue_comments.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_issue_labels_id_required
BEFORE INSERT ON project_issue_labels
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_issue_labels.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_issues_id_required
BEFORE INSERT ON project_issues
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_issues.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_job_artifacts_id_required
BEFORE INSERT ON project_job_artifacts
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_job_artifacts.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_job_logs_id_required
BEFORE INSERT ON project_job_logs
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_job_logs.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_jobs_id_required
BEFORE INSERT ON project_jobs
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_jobs.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_lfs_locks_id_required
BEFORE INSERT ON project_lfs_locks
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_lfs_locks.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_lfs_objects_id_required
BEFORE INSERT ON project_lfs_objects
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_lfs_objects.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_members_id_required
BEFORE INSERT ON project_members
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_members.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_merge_request_approval_rules_id_required
BEFORE INSERT ON project_merge_request_approval_rules
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_merge_request_approval_rules.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_merge_request_approvals_id_required
BEFORE INSERT ON project_merge_request_approvals
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_merge_request_approvals.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_merge_request_comments_id_required
BEFORE INSERT ON project_merge_request_comments
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_merge_request_comments.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_merge_request_participants_id_required
BEFORE INSERT ON project_merge_request_participants
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_merge_request_participants.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_merge_requests_id_required
BEFORE INSERT ON project_merge_requests
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_merge_requests.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_npm_dist_tags_id_required
BEFORE INSERT ON project_npm_dist_tags
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_npm_dist_tags.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_package_files_id_required
BEFORE INSERT ON project_package_files
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_package_files.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_package_versions_id_required
BEFORE INSERT ON project_package_versions
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_package_versions.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_packages_id_required
BEFORE INSERT ON project_packages
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_packages.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_pipeline_jobs_id_required
BEFORE INSERT ON project_pipeline_jobs
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_pipeline_jobs.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_pipelines_id_required
BEFORE INSERT ON project_pipelines
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_pipelines.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_release_links_id_required
BEFORE INSERT ON project_release_links
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_release_links.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_releases_id_required
BEFORE INSERT ON project_releases
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_releases.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_runners_id_required
BEFORE INSERT ON project_runners
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_runners.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_todos_id_required
BEFORE INSERT ON project_todos
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_todos.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_project_wiki_pages_id_required
BEFORE INSERT ON project_wiki_pages
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'project_wiki_pages.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_projects_id_required
BEFORE INSERT ON projects
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'projects.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_user_access_tokens_id_required
BEFORE INSERT ON user_access_tokens
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'user_access_tokens.id must be an application-generated positive ID');
END;

CREATE TRIGGER trg_users_id_required
BEFORE INSERT ON users
WHEN NEW.id IS NULL OR NEW.id <= 0
BEGIN
    SELECT RAISE(ABORT, 'users.id must be an application-generated positive ID');
END;
