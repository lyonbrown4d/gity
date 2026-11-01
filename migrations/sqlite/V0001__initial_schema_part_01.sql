-- Current Gity initial schema. Apply all initial parts to an empty database.

CREATE TABLE "audit_revisions" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "created_at" TIMESTAMP NOT NULL,
    "actor" TEXT NULL,
    "tenant" TEXT NULL,
    "reason" TEXT NULL,
    "metadata" TEXT NULL
);

CREATE INDEX "ix_audit_revisions_actor" ON "audit_revisions" ("actor");

CREATE INDEX "ix_audit_revisions_created_at" ON "audit_revisions" ("created_at");

CREATE TABLE auth_sessions (
    token_hash VARCHAR(64) PRIMARY KEY,
    user_id BIGINT NOT NULL,
    kind VARCHAR(16) NOT NULL,
    expires_at TIMESTAMP NOT NULL,
    family VARCHAR(64) NOT NULL
);

CREATE TABLE bootstrap_claim (
    id BIGINT PRIMARY KEY
);

CREATE TABLE "organizations" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "path_key" TEXT NOT NULL UNIQUE,
    "full_path" TEXT NOT NULL UNIQUE,
    "description" TEXT,
    "visibility" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL
);

CREATE TABLE user_passwords (
    user_id BIGINT PRIMARY KEY,
    password_hash TEXT NOT NULL
);

CREATE TABLE "users" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "username" TEXT NOT NULL UNIQUE,
    "display_name" TEXT NOT NULL,
    "email" TEXT UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    "is_super_admin" INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'active'
);

CREATE TABLE "organization_members" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "organization_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_organization_members_user" ON "organization_members" ("user_id");

CREATE UNIQUE INDEX "ux_organization_members_org_user" ON "organization_members" ("organization_id", "user_id");

CREATE TABLE "project_branch_protection_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_branch_protection_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "branch_name" TEXT NOT NULL,
    "rule_type" TEXT NOT NULL,
    "push_access_level" TEXT NOT NULL,
    "merge_access_level" TEXT NOT NULL,
    "require_merge_request" INTEGER NOT NULL,
    "require_pipeline_success" INTEGER NOT NULL,
    "allow_force_push" INTEGER NOT NULL,
    "allow_delete" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX "ix_project_branch_protection_audits_entity" ON "project_branch_protection_audits" ("project_branch_protection_id");

CREATE INDEX "ix_project_branch_protection_audits_project" ON "project_branch_protection_audits" ("project_id");

CREATE INDEX "ix_project_branch_protection_audits_revision" ON "project_branch_protection_audits" ("revision_id");

CREATE TABLE "project_ci_variable_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_ci_variable_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "key" TEXT NOT NULL,
    "masked" INTEGER NOT NULL,
    "protected" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX "ix_project_ci_variable_audits_entity" ON "project_ci_variable_audits" ("project_ci_variable_id");

CREATE INDEX "ix_project_ci_variable_audits_key" ON "project_ci_variable_audits" ("key");

CREATE INDEX "ix_project_ci_variable_audits_project" ON "project_ci_variable_audits" ("project_id");

CREATE INDEX "ix_project_ci_variable_audits_revision" ON "project_ci_variable_audits" ("revision_id");

CREATE TABLE "project_deploy_key_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_deploy_key_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "title" TEXT NOT NULL,
    "fingerprint" TEXT NOT NULL,
    "can_push" INTEGER NOT NULL,
    "created_by_user_id" INTEGER NOT NULL,
    "last_used_at" TIMESTAMP NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX "ix_project_deploy_key_audits_entity" ON "project_deploy_key_audits" ("project_deploy_key_id");

CREATE INDEX "ix_project_deploy_key_audits_fingerprint" ON "project_deploy_key_audits" ("fingerprint");

CREATE INDEX "ix_project_deploy_key_audits_project" ON "project_deploy_key_audits" ("project_id");

CREATE INDEX "ix_project_deploy_key_audits_revision" ON "project_deploy_key_audits" ("revision_id");

CREATE TABLE "project_member_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_member_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "user_id" INTEGER NOT NULL,
    "role" TEXT NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX "ix_project_member_audits_entity" ON "project_member_audits" ("project_member_id");

CREATE INDEX "ix_project_member_audits_project" ON "project_member_audits" ("project_id");

CREATE INDEX "ix_project_member_audits_revision" ON "project_member_audits" ("revision_id");

CREATE INDEX "ix_project_member_audits_user" ON "project_member_audits" ("user_id");

CREATE TABLE "project_merge_request_approval_rule_audits" (
    "revision_id" INTEGER NOT NULL,
    "operation" TEXT NOT NULL,
    "project_merge_request_approval_rule_id" INTEGER NOT NULL,
    "project_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "target_branch" TEXT NOT NULL,
    "approvals_required" INTEGER NOT NULL,
    "eligible_user_ids" TEXT NOT NULL,
    "code_owner" INTEGER NOT NULL,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("revision_id") REFERENCES "audit_revisions" ("id")
);

CREATE INDEX "ix_project_mr_approval_rule_audits_entity" ON "project_merge_request_approval_rule_audits" ("project_merge_request_approval_rule_id");

CREATE INDEX "ix_project_mr_approval_rule_audits_project" ON "project_merge_request_approval_rule_audits" ("project_id");

CREATE INDEX "ix_project_mr_approval_rule_audits_revision" ON "project_merge_request_approval_rule_audits" ("revision_id");

CREATE TABLE "projects" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "organization_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "path_key" TEXT NOT NULL,
    "full_path" TEXT NOT NULL UNIQUE,
    "visibility" TEXT NOT NULL,
    "description" TEXT,
    "default_branch" TEXT NOT NULL,
    "status" TEXT NOT NULL,
    "deleted_at" TIMESTAMP,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("organization_id") REFERENCES "organizations" ("id") ON DELETE CASCADE
);

CREATE INDEX "ix_projects_organization_status" ON "projects" ("organization_id", "status");

CREATE INDEX "ix_projects_visibility" ON "projects" ("visibility");

CREATE TABLE "user_access_tokens" (
    "id" INTEGER NOT NULL PRIMARY KEY,
    "user_id" INTEGER NOT NULL,
    "name" TEXT NOT NULL,
    "token" TEXT NOT NULL UNIQUE,
    "created_at" TIMESTAMP NOT NULL,
    "updated_at" TIMESTAMP NOT NULL,
    FOREIGN KEY ("user_id") REFERENCES "users" ("id") ON DELETE CASCADE
);
